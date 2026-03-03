use std::process::Stdio;

use tokio::process::Command as TokioCommand;
use tracing::{debug, trace};

use crate::Terraform;
use crate::error::{Error, Result};

/// Raw output from a Terraform command execution.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Process exit code.
    pub exit_code: i32,
    /// Whether the command exited successfully.
    pub success: bool,
}

impl CommandOutput {
    /// Split stdout into lines.
    #[must_use]
    pub fn stdout_lines(&self) -> Vec<&str> {
        self.stdout.lines().collect()
    }
}

/// Execute a Terraform command.
///
/// Builds the full invocation:
/// `<binary> [-chdir=<dir>] <subcommand> [global_args...] [command_args...]`
///
/// Note: `-chdir` is the only true global option (before the subcommand).
/// Options like `-no-color` and `-input=false` are per-subcommand flags
/// and are placed after the subcommand name.
pub async fn run_terraform(tf: &Terraform, command_args: Vec<String>) -> Result<CommandOutput> {
    run_terraform_inner(tf, command_args, &[0]).await
}

/// Execute a Terraform command, accepting additional exit codes as success.
///
/// Terraform `plan` uses exit code 2 to indicate "changes present" which is
/// not an error. Pass `&[0, 2]` to accept both.
pub async fn run_terraform_allow_exit_codes(
    tf: &Terraform,
    command_args: Vec<String>,
    allowed_codes: &[i32],
) -> Result<CommandOutput> {
    run_terraform_inner(tf, command_args, allowed_codes).await
}

async fn run_terraform_inner(
    tf: &Terraform,
    command_args: Vec<String>,
    allowed_codes: &[i32],
) -> Result<CommandOutput> {
    let mut cmd = TokioCommand::new(&tf.binary);

    // -chdir is the only true global option (must come before the subcommand)
    if let Some(ref working_dir) = tf.working_dir {
        cmd.arg(format!("-chdir={}", working_dir.display()));
    }

    // Command args (subcommand name + flags)
    for arg in &command_args {
        cmd.arg(arg);
    }

    // Global args (-no-color) at the end, after all command args.
    // This handles compound commands like "workspace show" correctly.
    for arg in &tf.global_args {
        cmd.arg(arg);
    }

    // Environment variables
    for (key, value) in &tf.env {
        cmd.env(key, value);
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    trace!(binary = ?tf.binary, args = ?command_args, "executing terraform command");

    let output = cmd.output().await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::NotFound
        } else {
            Error::Io {
                message: format!("failed to execute terraform: {e}"),
                source: e,
            }
        }
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    let success = allowed_codes.contains(&exit_code);

    debug!(exit_code, success, "terraform command completed");
    trace!(%stdout, "stdout");
    if !stderr.is_empty() {
        trace!(%stderr, "stderr");
    }

    if !success {
        return Err(Error::CommandFailed {
            command: command_args.first().cloned().unwrap_or_default(),
            exit_code,
            stdout,
            stderr,
        });
    }

    Ok(CommandOutput {
        stdout,
        stderr,
        exit_code,
        success,
    })
}
