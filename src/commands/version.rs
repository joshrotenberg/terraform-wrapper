use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec;

#[cfg(feature = "json")]
use crate::types::version::VersionInfo;

/// Command for retrieving Terraform version information.
///
/// By default, requests JSON output for programmatic parsing.
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::version::VersionCommand;
///
/// let tf = Terraform::builder().build()?;
/// let info = VersionCommand::new().execute(&tf).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct VersionCommand {
    json: bool,
}

impl Default for VersionCommand {
    fn default() -> Self {
        Self { json: true }
    }
}

impl VersionCommand {
    /// Create a new version command (JSON output enabled by default).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Disable JSON output.
    #[must_use]
    pub fn no_json(mut self) -> Self {
        self.json = false;
        self
    }
}

#[cfg(feature = "json")]
impl TerraformCommand for VersionCommand {
    type Output = VersionInfo;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["version".to_string()];
        if self.json {
            args.push("-json".to_string());
        }
        args
    }

    async fn execute(&self, tf: &Terraform) -> Result<VersionInfo> {
        let output = exec::run_terraform(tf, self.args()).await?;
        serde_json::from_str(&output.stdout).map_err(|e| crate::error::Error::ParseError {
            message: format!("failed to parse version json: {e}"),
        })
    }
}

#[cfg(not(feature = "json"))]
impl TerraformCommand for VersionCommand {
    type Output = exec::CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["version".to_string()];
        if self.json {
            args.push("-json".to_string());
        }
        args
    }

    async fn execute(&self, tf: &Terraform) -> Result<exec::CommandOutput> {
        exec::run_terraform(tf, self.args()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_args_include_json() {
        let cmd = VersionCommand::new();
        assert_eq!(cmd.args(), vec!["version", "-json"]);
    }

    #[test]
    fn no_json_args() {
        let cmd = VersionCommand::new().no_json();
        assert_eq!(cmd.args(), vec!["version"]);
    }
}
