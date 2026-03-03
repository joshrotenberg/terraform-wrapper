use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec;

/// Result from a show command.
///
/// The variant depends on whether a plan file was specified:
/// - No plan file: [`ShowResult::State`] with current state
/// - With plan file: [`ShowResult::Plan`] with plan details
/// - Without JSON: [`ShowResult::Plain`] with raw output
#[derive(Debug, Clone)]
pub enum ShowResult {
    /// Current state from `terraform show -json`.
    #[cfg(feature = "json")]
    State(crate::types::state::StateRepresentation),
    /// Saved plan from `terraform show -json <planfile>`.
    #[cfg(feature = "json")]
    Plan(Box<crate::types::plan::PlanRepresentation>),
    /// Plain command output (no `-json`).
    Plain(exec::CommandOutput),
}

/// Command for inspecting current state or a saved plan.
///
/// With `-json`, returns structured data about the current state or a
/// saved plan file.
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::show::{ShowCommand, ShowResult};
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
///
/// // Show current state
/// let result = ShowCommand::new().execute(&tf).await?;
///
/// // Show a saved plan
/// let result = ShowCommand::new()
///     .plan_file("tfplan")
///     .execute(&tf)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct ShowCommand {
    plan_file: Option<String>,
    json: bool,
    raw_args: Vec<String>,
}

impl ShowCommand {
    /// Create a new show command.
    #[must_use]
    pub fn new() -> Self {
        Self {
            json: true,
            ..Self::default()
        }
    }

    /// Show a saved plan file instead of current state.
    #[must_use]
    pub fn plan_file(mut self, path: &str) -> Self {
        self.plan_file = Some(path.to_string());
        self
    }

    /// Disable JSON output.
    #[must_use]
    pub fn no_json(mut self) -> Self {
        self.json = false;
        self
    }

    /// Add a raw argument (escape hatch for unsupported options).
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.raw_args.push(arg.into());
        self
    }
}

impl TerraformCommand for ShowCommand {
    type Output = ShowResult;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["show".to_string()];
        if self.json {
            args.push("-json".to_string());
        }
        args.extend(self.raw_args.clone());
        // Plan file is a positional argument at the end
        if let Some(ref plan) = self.plan_file {
            args.push(plan.clone());
        }
        args
    }

    async fn execute(&self, tf: &Terraform) -> Result<ShowResult> {
        let output = exec::run_terraform(tf, self.args()).await?;

        if !self.json {
            return Ok(ShowResult::Plain(output));
        }

        #[cfg(feature = "json")]
        if self.plan_file.is_some() {
            let plan: crate::types::plan::PlanRepresentation = serde_json::from_str(&output.stdout)
                .map_err(|e| crate::error::Error::ParseError {
                    message: format!("failed to parse plan json: {e}"),
                })?;
            return Ok(ShowResult::Plan(Box::new(plan)));
        }

        #[cfg(feature = "json")]
        {
            let state: crate::types::state::StateRepresentation =
                serde_json::from_str(&output.stdout).map_err(|e| {
                    crate::error::Error::ParseError {
                        message: format!("failed to parse state json: {e}"),
                    }
                })?;
            Ok(ShowResult::State(state))
        }

        #[cfg(not(feature = "json"))]
        Ok(ShowResult::Plain(output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_args_include_json() {
        let cmd = ShowCommand::new();
        assert_eq!(cmd.args(), vec!["show", "-json"]);
    }

    #[test]
    fn plan_file_at_end() {
        let cmd = ShowCommand::new().plan_file("tfplan");
        assert_eq!(cmd.args(), vec!["show", "-json", "tfplan"]);
    }

    #[test]
    fn no_json_args() {
        let cmd = ShowCommand::new().no_json();
        assert_eq!(cmd.args(), vec!["show"]);
    }

    #[test]
    fn no_json_with_plan_file() {
        let cmd = ShowCommand::new().no_json().plan_file("tfplan");
        assert_eq!(cmd.args(), vec!["show", "tfplan"]);
    }
}
