use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// The state subcommand to execute.
#[derive(Debug, Clone)]
pub enum StateSubcommand {
    /// List resources in the state.
    List,
    /// Show a single resource in the state.
    Show(String),
    /// Move a resource to a different address.
    Mv {
        /// Source address.
        source: String,
        /// Destination address.
        destination: String,
    },
    /// Remove a resource from the state (without destroying it).
    Rm(Vec<String>),
    /// Pull remote state and output to stdout.
    Pull,
    /// Push local state to remote backend.
    Push,
}

/// Command for managing Terraform state.
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::state::StateCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
///
/// // List resources in state
/// let output = StateCommand::list().execute(&tf).await?;
///
/// // Show a specific resource
/// let output = StateCommand::show("null_resource.example")
///     .execute(&tf)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct StateCommand {
    subcommand: StateSubcommand,
    raw_args: Vec<String>,
}

impl StateCommand {
    /// List resources in the state.
    #[must_use]
    pub fn list() -> Self {
        Self {
            subcommand: StateSubcommand::List,
            raw_args: Vec::new(),
        }
    }

    /// Show a single resource in the state.
    #[must_use]
    pub fn show(address: &str) -> Self {
        Self {
            subcommand: StateSubcommand::Show(address.to_string()),
            raw_args: Vec::new(),
        }
    }

    /// Move a resource to a different address.
    #[must_use]
    pub fn mv(source: &str, destination: &str) -> Self {
        Self {
            subcommand: StateSubcommand::Mv {
                source: source.to_string(),
                destination: destination.to_string(),
            },
            raw_args: Vec::new(),
        }
    }

    /// Remove resources from the state (without destroying them).
    #[must_use]
    pub fn rm(addresses: Vec<String>) -> Self {
        Self {
            subcommand: StateSubcommand::Rm(addresses),
            raw_args: Vec::new(),
        }
    }

    /// Pull remote state and output to stdout.
    #[must_use]
    pub fn pull() -> Self {
        Self {
            subcommand: StateSubcommand::Pull,
            raw_args: Vec::new(),
        }
    }

    /// Push local state to remote backend.
    #[must_use]
    pub fn push() -> Self {
        Self {
            subcommand: StateSubcommand::Push,
            raw_args: Vec::new(),
        }
    }

    /// Add a raw argument (escape hatch for unsupported options).
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.raw_args.push(arg.into());
        self
    }
}

impl TerraformCommand for StateCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["state".to_string()];
        match &self.subcommand {
            StateSubcommand::List => args.push("list".to_string()),
            StateSubcommand::Show(address) => {
                args.push("show".to_string());
                args.push(address.clone());
            }
            StateSubcommand::Mv {
                source,
                destination,
            } => {
                args.push("mv".to_string());
                args.push(source.clone());
                args.push(destination.clone());
            }
            StateSubcommand::Rm(addresses) => {
                args.push("rm".to_string());
                args.extend(addresses.clone());
            }
            StateSubcommand::Pull => args.push("pull".to_string()),
            StateSubcommand::Push => args.push("push".to_string()),
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
    fn list_args() {
        let cmd = StateCommand::list();
        assert_eq!(cmd.args(), vec!["state", "list"]);
    }

    #[test]
    fn show_args() {
        let cmd = StateCommand::show("null_resource.example");
        assert_eq!(cmd.args(), vec!["state", "show", "null_resource.example"]);
    }

    #[test]
    fn mv_args() {
        let cmd = StateCommand::mv("null_resource.old", "null_resource.new");
        assert_eq!(
            cmd.args(),
            vec!["state", "mv", "null_resource.old", "null_resource.new"]
        );
    }

    #[test]
    fn rm_args() {
        let cmd = StateCommand::rm(vec![
            "null_resource.a".to_string(),
            "null_resource.b".to_string(),
        ]);
        assert_eq!(
            cmd.args(),
            vec!["state", "rm", "null_resource.a", "null_resource.b"]
        );
    }

    #[test]
    fn pull_args() {
        let cmd = StateCommand::pull();
        assert_eq!(cmd.args(), vec!["state", "pull"]);
    }

    #[test]
    fn push_args() {
        let cmd = StateCommand::push();
        assert_eq!(cmd.args(), vec!["state", "push"]);
    }
}
