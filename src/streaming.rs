//! Streaming JSON output from `terraform plan` and `terraform apply`.
//!
//! When run with `-json`, Terraform produces one JSON object per line (NDJSON),
//! each representing an event like resource creation, progress, or completion.
//!
//! This module provides [`stream_terraform`] which yields [`JsonLogLine`] events
//! as they arrive, useful for progress reporting in orchestration tools.
//!
//! # Example
//!
//! ```rust,no_run
//! use terraform_wrapper::prelude::*;
//! use terraform_wrapper::streaming::{stream_terraform, JsonLogLine};
//!
//! # async fn example() -> terraform_wrapper::error::Result<()> {
//! let tf = Terraform::builder().working_dir("./infra").build()?;
//!
//! let result = stream_terraform(&tf, ApplyCommand::new().auto_approve().json(), &[0], |line| {
//!     println!("[{}] {}", line.log_type, line.message);
//! }).await?;
//!
//! assert!(result.success);
//! # Ok(())
//! # }
//! ```

use std::process::Stdio;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tracing::{debug, trace, warn};

use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::{Error, Result};
use crate::exec::CommandOutput;

/// A single JSON log line from Terraform's streaming output.
///
/// Terraform emits these as NDJSON (one per line) when commands are run
/// with the `-json` flag.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonLogLine {
    /// Log level: "info", "warn", "error".
    #[serde(rename = "@level")]
    pub level: String,
    /// Human-readable message.
    #[serde(rename = "@message")]
    pub message: String,
    /// Module that emitted the message (e.g., "terraform.ui").
    #[serde(rename = "@module", default)]
    pub module: String,
    /// ISO 8601 timestamp.
    #[serde(rename = "@timestamp", default)]
    pub timestamp: String,
    /// Event type: "version", "planned_change", "change_summary",
    /// "apply_start", "apply_progress", "apply_complete",
    /// "apply_errored", "outputs", etc.
    #[serde(rename = "type", default)]
    pub log_type: String,
    /// Change details (for planned_change events).
    #[serde(default)]
    pub change: serde_json::Value,
    /// Hook details (for apply_start/apply_complete/apply_progress events).
    #[serde(default)]
    pub hook: serde_json::Value,
    /// Change summary (for change_summary events).
    #[serde(default)]
    pub changes: serde_json::Value,
    /// Output values (for outputs events).
    #[serde(default)]
    pub outputs: serde_json::Value,
}

/// Execute a Terraform command with streaming JSON output.
///
/// Spawns the terraform process and calls `handler` with each [`JsonLogLine`]
/// as it arrives on stdout. Lines that fail to parse as JSON are logged and
/// skipped.
///
/// `allowed_exit_codes` controls which exit codes are treated as success.
/// Pass `&[0]` for most commands, or `&[0, 2]` for `plan -detailed-exitcode`
/// where exit code 2 means "changes present".
///
/// Respects the client's timeout setting. If the command exceeds the timeout,
/// the child process is killed and [`Error::Timeout`] is returned.
///
/// Returns a [`CommandOutput`] with the complete stderr and exit code after
/// the process finishes. Stdout is empty since lines were consumed by the
/// handler.
///
/// The command should have `.json()` enabled. If not, lines won't parse and
/// will be skipped.
pub async fn stream_terraform<C, F>(
    tf: &Terraform,
    command: C,
    allowed_exit_codes: &[i32],
    mut handler: F,
) -> Result<CommandOutput>
where
    C: TerraformCommand,
    F: FnMut(JsonLogLine),
{
    let args = command.prepare_args(tf);

    let mut cmd = TokioCommand::new(&tf.binary);

    if let Some(ref working_dir) = tf.working_dir {
        cmd.arg(format!("-chdir={}", working_dir.display()));
    }

    for arg in &args {
        cmd.arg(arg);
    }

    // Global args at end (same ordering as exec.rs)
    for arg in &tf.global_args {
        cmd.arg(arg);
    }

    for (key, value) in &tf.env {
        cmd.env(key, value);
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    trace!(binary = ?tf.binary, args = ?args, timeout_secs = ?tf.timeout.map(|t| t.as_secs()), "streaming terraform command");

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::NotFound
        } else {
            Error::Io {
                message: format!("failed to spawn terraform: {e}"),
                source: e,
            }
        }
    })?;

    let stdout = child.stdout.take().ok_or_else(|| Error::Io {
        message: "failed to capture stdout".to_string(),
        source: std::io::Error::other("no stdout"),
    })?;

    let stream_and_wait = async {
        let mut reader = BufReader::new(stdout).lines();

        while let Some(line) = reader.next_line().await.map_err(|e| Error::Io {
            message: format!("failed to read stdout line: {e}"),
            source: e,
        })? {
            trace!(%line, "stream line");
            match serde_json::from_str::<JsonLogLine>(&line) {
                Ok(log_line) => handler(log_line),
                Err(e) => {
                    warn!(%line, error = %e, "failed to parse streaming json line, skipping");
                }
            }
        }

        child.wait_with_output().await.map_err(|e| Error::Io {
            message: format!("failed to wait for terraform: {e}"),
            source: e,
        })
    };

    let output = if let Some(duration) = tf.timeout {
        match tokio::time::timeout(duration, stream_and_wait).await {
            Ok(result) => result?,
            Err(_) => {
                warn!(
                    timeout_seconds = duration.as_secs(),
                    "streaming terraform command timed out"
                );
                return Err(Error::Timeout {
                    timeout_seconds: duration.as_secs(),
                });
            }
        }
    } else {
        stream_and_wait.await?
    };

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    let success = allowed_exit_codes.contains(&exit_code);

    debug!(exit_code, success, "streaming terraform command completed");

    if !success {
        return Err(Error::CommandFailed {
            command: args.first().cloned().unwrap_or_default(),
            exit_code,
            stdout: String::new(),
            stderr,
        });
    }

    Ok(CommandOutput {
        stdout: String::new(), // consumed by handler
        stderr,
        exit_code,
        success,
    })
}
