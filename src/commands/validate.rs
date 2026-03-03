use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec;

#[cfg(feature = "json")]
use crate::types::validation::ValidationResult;

/// Command for validating Terraform configuration.
///
/// Checks that the configuration is syntactically valid and internally
/// consistent. Does not access any remote services (state, providers).
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::validate::ValidateCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
/// let result = ValidateCommand::new().execute(&tf).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ValidateCommand {
    json: bool,
    raw_args: Vec<String>,
}

impl Default for ValidateCommand {
    fn default() -> Self {
        Self {
            json: true,
            raw_args: Vec::new(),
        }
    }
}

impl ValidateCommand {
    /// Create a new validate command (JSON output enabled by default).
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

    /// Add a raw argument (escape hatch for unsupported options).
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.raw_args.push(arg.into());
        self
    }
}

#[cfg(feature = "json")]
impl TerraformCommand for ValidateCommand {
    type Output = ValidationResult;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["validate".to_string()];
        if self.json {
            args.push("-json".to_string());
        }
        args.extend(self.raw_args.clone());
        args
    }

    async fn execute(&self, tf: &Terraform) -> Result<ValidationResult> {
        // validate returns exit code 1 for invalid config, but with -json
        // it still writes valid JSON to stdout. Accept both exit codes.
        let output = exec::run_terraform_allow_exit_codes(tf, self.args(), &[0, 1]).await?;
        serde_json::from_str(&output.stdout).map_err(|e| crate::error::Error::ParseError {
            message: format!("failed to parse validate json: {e}"),
        })
    }
}

#[cfg(not(feature = "json"))]
impl TerraformCommand for ValidateCommand {
    type Output = exec::CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["validate".to_string()];
        if self.json {
            args.push("-json".to_string());
        }
        args.extend(self.raw_args.clone());
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
        let cmd = ValidateCommand::new();
        assert_eq!(cmd.args(), vec!["validate", "-json"]);
    }

    #[test]
    fn no_json_args() {
        let cmd = ValidateCommand::new().no_json();
        assert_eq!(cmd.args(), vec!["validate"]);
    }

    #[test]
    fn raw_arg_escape_hatch() {
        let cmd = ValidateCommand::new().arg("-test-directory=tests");
        let args = cmd.args();
        assert!(args.contains(&"-test-directory=tests".to_string()));
    }
}
