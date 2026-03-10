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
    /// Replace a provider in the state.
    ReplaceProvider {
        /// Source provider address.
        from: String,
        /// Destination provider address.
        to: String,
    },
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
    auto_approve: bool,
    dry_run: bool,
    lock: Option<bool>,
    lock_timeout: Option<String>,
    raw_args: Vec<String>,
}

impl StateCommand {
    /// List resources in the state.
    #[must_use]
    pub fn list() -> Self {
        Self {
            subcommand: StateSubcommand::List,
            auto_approve: false,
            dry_run: false,
            lock: None,
            lock_timeout: None,
            raw_args: Vec::new(),
        }
    }

    /// Show a single resource in the state.
    #[must_use]
    pub fn show(address: &str) -> Self {
        Self {
            subcommand: StateSubcommand::Show(address.to_string()),
            auto_approve: false,
            dry_run: false,
            lock: None,
            lock_timeout: None,
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
            auto_approve: false,
            dry_run: false,
            lock: None,
            lock_timeout: None,
            raw_args: Vec::new(),
        }
    }

    /// Remove resources from the state (without destroying them).
    #[must_use]
    pub fn rm(addresses: Vec<String>) -> Self {
        Self {
            subcommand: StateSubcommand::Rm(addresses),
            auto_approve: false,
            dry_run: false,
            lock: None,
            lock_timeout: None,
            raw_args: Vec::new(),
        }
    }

    /// Pull remote state and output to stdout.
    #[must_use]
    pub fn pull() -> Self {
        Self {
            subcommand: StateSubcommand::Pull,
            auto_approve: false,
            dry_run: false,
            lock: None,
            lock_timeout: None,
            raw_args: Vec::new(),
        }
    }

    /// Push local state to remote backend.
    #[must_use]
    pub fn push() -> Self {
        Self {
            subcommand: StateSubcommand::Push,
            auto_approve: false,
            dry_run: false,
            lock: None,
            lock_timeout: None,
            raw_args: Vec::new(),
        }
    }

    /// Replace a provider address in the state.
    #[must_use]
    pub fn replace_provider(from: &str, to: &str) -> Self {
        Self {
            subcommand: StateSubcommand::ReplaceProvider {
                from: from.to_string(),
                to: to.to_string(),
            },
            auto_approve: false,
            dry_run: false,
            lock: None,
            lock_timeout: None,
            raw_args: Vec::new(),
        }
    }

    /// Skip interactive approval (`-auto-approve`).
    ///
    /// Applies to `replace-provider` subcommand only; ignored for other subcommands.
    #[must_use]
    pub fn auto_approve(mut self) -> Self {
        self.auto_approve = true;
        self
    }

    /// Preview the operation without making changes (`-dry-run`).
    ///
    /// Applies to `mv` and `rm` subcommands only; ignored for other subcommands.
    #[must_use]
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Enable or disable state locking (`-lock`).
    ///
    /// Applies to `mv` and `rm` subcommands only; ignored for other subcommands.
    #[must_use]
    pub fn lock(mut self, enabled: bool) -> Self {
        self.lock = Some(enabled);
        self
    }

    /// Duration to wait for state lock (`-lock-timeout`).
    ///
    /// Applies to `mv` and `rm` subcommands only; ignored for other subcommands.
    #[must_use]
    pub fn lock_timeout(mut self, timeout: &str) -> Self {
        self.lock_timeout = Some(timeout.to_string());
        self
    }

    /// Add a raw argument (escape hatch for unsupported options).
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.raw_args.push(arg.into());
        self
    }

    /// Push mv/rm-specific flags into the args list.
    fn push_mv_rm_flags(&self, args: &mut Vec<String>) {
        if self.dry_run {
            args.push("-dry-run".to_string());
        }
        if let Some(lock) = self.lock {
            args.push(format!("-lock={lock}"));
        }
        if let Some(ref timeout) = self.lock_timeout {
            args.push(format!("-lock-timeout={timeout}"));
        }
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
                self.push_mv_rm_flags(&mut args);
                args.push(source.clone());
                args.push(destination.clone());
            }
            StateSubcommand::Rm(addresses) => {
                args.push("rm".to_string());
                self.push_mv_rm_flags(&mut args);
                args.extend(addresses.clone());
            }
            StateSubcommand::Pull => args.push("pull".to_string()),
            StateSubcommand::Push => args.push("push".to_string()),
            StateSubcommand::ReplaceProvider { from, to } => {
                args.push("replace-provider".to_string());
                if self.auto_approve {
                    args.push("-auto-approve".to_string());
                }
                if let Some(lock) = self.lock {
                    args.push(format!("-lock={lock}"));
                }
                if let Some(ref timeout) = self.lock_timeout {
                    args.push(format!("-lock-timeout={timeout}"));
                }
                args.push(from.clone());
                args.push(to.clone());
            }
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
    fn mv_dry_run_args() {
        let cmd = StateCommand::mv("null_resource.old", "null_resource.new").dry_run();
        assert_eq!(
            cmd.args(),
            vec![
                "state",
                "mv",
                "-dry-run",
                "null_resource.old",
                "null_resource.new"
            ]
        );
    }

    #[test]
    fn mv_lock_args() {
        let cmd = StateCommand::mv("null_resource.old", "null_resource.new")
            .lock(false)
            .lock_timeout("10s");
        assert_eq!(
            cmd.args(),
            vec![
                "state",
                "mv",
                "-lock=false",
                "-lock-timeout=10s",
                "null_resource.old",
                "null_resource.new"
            ]
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
    fn rm_dry_run_args() {
        let cmd = StateCommand::rm(vec!["null_resource.a".to_string()]).dry_run();
        assert_eq!(
            cmd.args(),
            vec!["state", "rm", "-dry-run", "null_resource.a"]
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

    #[test]
    fn replace_provider_args() {
        let cmd = StateCommand::replace_provider(
            "registry.terraform.io/hashicorp/aws",
            "registry.terraform.io/acme/aws",
        );
        assert_eq!(
            cmd.args(),
            vec![
                "state",
                "replace-provider",
                "registry.terraform.io/hashicorp/aws",
                "registry.terraform.io/acme/aws"
            ]
        );
    }

    #[test]
    fn replace_provider_auto_approve_args() {
        let cmd = StateCommand::replace_provider(
            "registry.terraform.io/hashicorp/aws",
            "registry.terraform.io/acme/aws",
        )
        .auto_approve()
        .lock(false);
        assert_eq!(
            cmd.args(),
            vec![
                "state",
                "replace-provider",
                "-auto-approve",
                "-lock=false",
                "registry.terraform.io/hashicorp/aws",
                "registry.terraform.io/acme/aws"
            ]
        );
    }

    #[test]
    fn list_ignores_mv_rm_flags() {
        let cmd = StateCommand::list()
            .dry_run()
            .lock(false)
            .lock_timeout("10s");
        assert_eq!(cmd.args(), vec!["state", "list"]);
    }
}
