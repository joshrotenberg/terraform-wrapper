use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// Command for manually releasing a stuck state lock.
///
/// Requires a lock ID as a positional argument. Use `-force` to skip the
/// confirmation prompt.
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::force_unlock::ForceUnlockCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
/// ForceUnlockCommand::new("lock-id-here")
///     .force()
///     .execute(&tf)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ForceUnlockCommand {
    lock_id: String,
    force: bool,
    raw_args: Vec<String>,
}

impl ForceUnlockCommand {
    /// Create a new force-unlock command with the given lock ID.
    #[must_use]
    pub fn new(lock_id: &str) -> Self {
        Self {
            lock_id: lock_id.to_string(),
            force: false,
            raw_args: Vec::new(),
        }
    }

    /// Skip the confirmation prompt (`-force`).
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Add a raw argument (escape hatch for unsupported options).
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.raw_args.push(arg.into());
        self
    }
}

impl TerraformCommand for ForceUnlockCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["force-unlock".to_string()];
        if self.force {
            args.push("-force".to_string());
        }
        args.extend(self.raw_args.clone());
        args.push(self.lock_id.clone());
        args
    }

    async fn execute(&self, tf: &Terraform) -> Result<CommandOutput> {
        exec::run_terraform(tf, self.args()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_args() {
        let cmd = ForceUnlockCommand::new("abc-123");
        assert_eq!(cmd.args(), vec!["force-unlock", "abc-123"]);
    }

    #[test]
    fn with_force() {
        let cmd = ForceUnlockCommand::new("abc-123").force();
        assert_eq!(cmd.args(), vec!["force-unlock", "-force", "abc-123"]);
    }

    #[test]
    fn lock_id_at_end() {
        let cmd = ForceUnlockCommand::new("abc-123").force().arg("-no-color");
        let args = cmd.args();
        assert_eq!(args.last().unwrap(), "abc-123");
    }
}
