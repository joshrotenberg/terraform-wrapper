use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// Command for listing declared modules in a configuration.
///
/// Lists all modules declared in the current working directory.
/// Supports `-json` for machine-readable output.
///
/// Requires Terraform v1.10.0 or later.
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::modules::ModulesCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
/// let output = ModulesCommand::new()
///     .json()
///     .execute(&tf)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct ModulesCommand {
    json: bool,
    raw_args: Vec<String>,
}

impl ModulesCommand {
    /// Create a new modules command with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable machine-readable JSON output (`-json`).
    #[must_use]
    pub fn json(mut self) -> Self {
        self.json = true;
        self
    }

    /// Add a raw argument (escape hatch for unsupported options).
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.raw_args.push(arg.into());
        self
    }
}

impl TerraformCommand for ModulesCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["modules".to_string()];
        if self.json {
            args.push("-json".to_string());
        }
        args.extend(self.raw_args.clone());
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
        let cmd = ModulesCommand::new();
        assert_eq!(cmd.args(), vec!["modules"]);
    }

    #[test]
    fn with_json() {
        let cmd = ModulesCommand::new().json();
        assert_eq!(cmd.args(), vec!["modules", "-json"]);
    }
}
