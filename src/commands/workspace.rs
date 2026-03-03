use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// The workspace subcommand to execute.
#[derive(Debug, Clone)]
pub enum WorkspaceSubcommand {
    /// List all workspaces.
    List,
    /// Show the current workspace name.
    Show,
    /// Create a new workspace.
    New(String),
    /// Switch to an existing workspace.
    Select(String),
    /// Delete a workspace.
    Delete(String),
}

/// Command for managing Terraform workspaces.
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::workspace::WorkspaceCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
///
/// // List workspaces
/// let output = WorkspaceCommand::list().execute(&tf).await?;
///
/// // Create and switch to a new workspace
/// WorkspaceCommand::new_workspace("staging").execute(&tf).await?;
///
/// // Switch back
/// WorkspaceCommand::select("default").execute(&tf).await?;
///
/// // Delete
/// WorkspaceCommand::delete("staging").execute(&tf).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct WorkspaceCommand {
    subcommand: WorkspaceSubcommand,
    force: bool,
    raw_args: Vec<String>,
}

impl WorkspaceCommand {
    /// List all workspaces.
    #[must_use]
    pub fn list() -> Self {
        Self {
            subcommand: WorkspaceSubcommand::List,
            force: false,
            raw_args: Vec::new(),
        }
    }

    /// Show the current workspace name.
    #[must_use]
    pub fn show() -> Self {
        Self {
            subcommand: WorkspaceSubcommand::Show,
            force: false,
            raw_args: Vec::new(),
        }
    }

    /// Create a new workspace.
    #[must_use]
    pub fn new_workspace(name: &str) -> Self {
        Self {
            subcommand: WorkspaceSubcommand::New(name.to_string()),
            force: false,
            raw_args: Vec::new(),
        }
    }

    /// Select (switch to) an existing workspace.
    #[must_use]
    pub fn select(name: &str) -> Self {
        Self {
            subcommand: WorkspaceSubcommand::Select(name.to_string()),
            force: false,
            raw_args: Vec::new(),
        }
    }

    /// Delete a workspace.
    #[must_use]
    pub fn delete(name: &str) -> Self {
        Self {
            subcommand: WorkspaceSubcommand::Delete(name.to_string()),
            force: false,
            raw_args: Vec::new(),
        }
    }

    /// Force deletion of a non-empty workspace (`-force`).
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

impl TerraformCommand for WorkspaceCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["workspace".to_string()];
        match &self.subcommand {
            WorkspaceSubcommand::List => args.push("list".to_string()),
            WorkspaceSubcommand::Show => args.push("show".to_string()),
            WorkspaceSubcommand::New(name) => {
                args.push("new".to_string());
                args.push(name.clone());
            }
            WorkspaceSubcommand::Select(name) => {
                args.push("select".to_string());
                args.push(name.clone());
            }
            WorkspaceSubcommand::Delete(name) => {
                args.push("delete".to_string());
                if self.force {
                    args.push("-force".to_string());
                }
                args.push(name.clone());
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
        let cmd = WorkspaceCommand::list();
        assert_eq!(cmd.args(), vec!["workspace", "list"]);
    }

    #[test]
    fn show_args() {
        let cmd = WorkspaceCommand::show();
        assert_eq!(cmd.args(), vec!["workspace", "show"]);
    }

    #[test]
    fn new_args() {
        let cmd = WorkspaceCommand::new_workspace("staging");
        assert_eq!(cmd.args(), vec!["workspace", "new", "staging"]);
    }

    #[test]
    fn select_args() {
        let cmd = WorkspaceCommand::select("production");
        assert_eq!(cmd.args(), vec!["workspace", "select", "production"]);
    }

    #[test]
    fn delete_args() {
        let cmd = WorkspaceCommand::delete("staging");
        assert_eq!(cmd.args(), vec!["workspace", "delete", "staging"]);
    }

    #[test]
    fn delete_force_args() {
        let cmd = WorkspaceCommand::delete("staging").force();
        assert_eq!(cmd.args(), vec!["workspace", "delete", "-force", "staging"]);
    }
}
