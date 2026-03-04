use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// Escape-hatch command for running any Terraform subcommand.
///
/// Use this when you need a subcommand that doesn't have a dedicated type,
/// while still benefiting from the [`Terraform`] client's binary resolution,
/// working directory, environment variables, and global arguments.
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::raw::RawCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
///
/// // Run any subcommand with arbitrary flags
/// let output = RawCommand::new("console")
///     .arg("-var=region=us-west-2")
///     .execute(&tf)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct RawCommand {
    subcommand: String,
    extra_args: Vec<String>,
}

impl RawCommand {
    /// Create a new raw command for the given subcommand.
    #[must_use]
    pub fn new(subcommand: &str) -> Self {
        Self {
            subcommand: subcommand.to_string(),
            extra_args: Vec::new(),
        }
    }

    /// Add an argument to the command.
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.extra_args.push(arg.into());
        self
    }

    /// Add multiple arguments to the command.
    #[must_use]
    pub fn with_args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.extra_args.extend(args.into_iter().map(Into::into));
        self
    }
}

impl TerraformCommand for RawCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec![self.subcommand.clone()];
        args.extend(self.extra_args.clone());
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
    fn subcommand_only() {
        let cmd = RawCommand::new("console");
        assert_eq!(cmd.args(), vec!["console"]);
    }

    #[test]
    fn with_single_arg() {
        let cmd = RawCommand::new("taint").arg("aws_instance.example");
        assert_eq!(cmd.args(), vec!["taint", "aws_instance.example"]);
    }

    #[test]
    fn with_multiple_args() {
        let cmd = RawCommand::new("untaint").with_args(["-lock=false", "aws_instance.example"]);
        assert_eq!(
            cmd.args(),
            vec!["untaint", "-lock=false", "aws_instance.example"]
        );
    }
}
