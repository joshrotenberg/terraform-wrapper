use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// Command for downloading and updating remote modules.
///
/// Downloads modules referenced in the configuration. This is also done
/// automatically by `terraform init`, but `get` is useful for updating
/// modules independently.
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::get::GetCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
/// GetCommand::new()
///     .update()
///     .execute(&tf)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct GetCommand {
    update: bool,
    no_color: bool,
    raw_args: Vec<String>,
}

impl GetCommand {
    /// Create a new get command with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check for and download updates to already-installed modules (`-update`).
    #[must_use]
    pub fn update(mut self) -> Self {
        self.update = true;
        self
    }

    /// Disable color output (`-no-color`).
    #[must_use]
    pub fn no_color(mut self) -> Self {
        self.no_color = true;
        self
    }

    /// Add a raw argument (escape hatch for unsupported options).
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.raw_args.push(arg.into());
        self
    }
}

impl TerraformCommand for GetCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["get".to_string()];
        if self.update {
            args.push("-update".to_string());
        }
        if self.no_color {
            args.push("-no-color".to_string());
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
        let cmd = GetCommand::new();
        assert_eq!(cmd.args(), vec!["get"]);
    }

    #[test]
    fn with_update() {
        let cmd = GetCommand::new().update();
        assert_eq!(cmd.args(), vec!["get", "-update"]);
    }

    #[test]
    fn with_no_color() {
        let cmd = GetCommand::new().no_color();
        assert_eq!(cmd.args(), vec!["get", "-no-color"]);
    }

    #[test]
    fn all_options() {
        let cmd = GetCommand::new().update().no_color();
        let args = cmd.args();
        assert_eq!(args[0], "get");
        assert!(args.contains(&"-update".to_string()));
        assert!(args.contains(&"-no-color".to_string()));
    }
}
