use std::fmt;
use std::process::Stdio;

use tokio::process::Command as TokioCommand;
use tracing::{debug, trace, warn};

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

impl fmt::Display for CommandOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.stdout.trim())
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
///
/// Uses the client's default timeout if set.
pub async fn run_terraform(tf: &Terraform, command_args: Vec<String>) -> Result<CommandOutput> {
    run_terraform_inner(tf, command_args, &[0], tf.timeout).await
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
    run_terraform_inner(tf, command_args, allowed_codes, tf.timeout).await
}

/// Execute a Terraform command with a specific timeout override.
///
/// The provided timeout takes precedence over the client's default.
pub async fn run_terraform_with_timeout(
    tf: &Terraform,
    command_args: Vec<String>,
    timeout: std::time::Duration,
) -> Result<CommandOutput> {
    run_terraform_inner(tf, command_args, &[0], Some(timeout)).await
}

async fn run_terraform_inner(
    tf: &Terraform,
    command_args: Vec<String>,
    allowed_codes: &[i32],
    timeout: Option<std::time::Duration>,
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
    // This handles compound commands like "workspace show" and "state list".
    for arg in &tf.global_args {
        cmd.arg(arg);
    }

    // Environment variables
    for (key, value) in &tf.env {
        cmd.env(key, value);
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    trace!(binary = ?tf.binary, args = ?command_args, timeout_secs = ?timeout.map(|t| t.as_secs()), "executing terraform command");

    let io_result = if let Some(duration) = timeout {
        match tokio::time::timeout(duration, cmd.output()).await {
            Ok(result) => result,
            Err(_) => {
                warn!(
                    timeout_seconds = duration.as_secs(),
                    "terraform command timed out"
                );
                return Err(Error::Timeout {
                    timeout_seconds: duration.as_secs(),
                });
            }
        }
    } else {
        cmd.output().await
    };

    let output = io_result.map_err(|e| {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_command_output_trims_whitespace() {
        let output = CommandOutput {
            stdout: "  hello world  \n".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };
        assert_eq!(output.to_string(), "hello world");
    }

    #[test]
    fn display_command_output_empty() {
        let output = CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };
        assert_eq!(output.to_string(), "");
    }

    #[test]
    fn display_command_output_multiline() {
        let output = CommandOutput {
            stdout: "line1\nline2\nline3\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };
        assert_eq!(output.to_string(), "line1\nline2\nline3");
    }
}
