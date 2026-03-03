use std::collections::HashMap;

use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec;

/// Result from an output command.
///
/// The variant returned depends on which flags were set on the command:
/// - `.json()` returns [`OutputResult::Json`] or [`OutputResult::Single`]
/// - `.raw()` returns [`OutputResult::Raw`]
/// - Neither returns [`OutputResult::Plain`]
#[derive(Debug, Clone)]
pub enum OutputResult {
    /// Raw string value from `-raw` flag.
    Raw(String),
    /// All output values as JSON (when `.json()` and no `.name()`).
    #[cfg(feature = "json")]
    Json(HashMap<String, crate::types::output::OutputValue>),
    /// Single output value as JSON (when `.json()` and `.name()`).
    #[cfg(feature = "json")]
    Single(crate::types::output::OutputValue),
    /// Plain command output (no `-json` or `-raw`).
    Plain(exec::CommandOutput),
}

/// Command for reading Terraform output values.
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::output::{OutputCommand, OutputResult};
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
///
/// // Get all outputs as JSON
/// let result = OutputCommand::new().json().execute(&tf).await?;
///
/// // Get a single raw output value
/// let result = OutputCommand::new()
///     .name("public_ip")
///     .raw()
///     .execute(&tf)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct OutputCommand {
    name: Option<String>,
    json: bool,
    raw: bool,
    raw_args: Vec<String>,
}

impl OutputCommand {
    /// Create a new output command.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Request a specific named output (positional argument).
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Request JSON-formatted output (`-json`).
    #[must_use]
    pub fn json(mut self) -> Self {
        self.json = true;
        self
    }

    /// Request raw output value (`-raw`). Requires `.name()` to be set.
    #[must_use]
    pub fn raw(mut self) -> Self {
        self.raw = true;
        self
    }

    /// Add a raw argument (escape hatch for unsupported options).
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.raw_args.push(arg.into());
        self
    }
}

impl TerraformCommand for OutputCommand {
    type Output = OutputResult;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["output".to_string()];
        if self.json {
            args.push("-json".to_string());
        }
        if self.raw {
            args.push("-raw".to_string());
        }
        args.extend(self.raw_args.clone());
        if let Some(ref name) = self.name {
            args.push(name.clone());
        }
        args
    }

    async fn execute(&self, tf: &Terraform) -> Result<OutputResult> {
        let output = exec::run_terraform(tf, self.args()).await?;

        if self.raw {
            return Ok(OutputResult::Raw(output.stdout.trim_end().to_string()));
        }

        #[cfg(feature = "json")]
        if self.json {
            if self.name.is_some() {
                let value: crate::types::output::OutputValue = serde_json::from_str(&output.stdout)
                    .map_err(|e| crate::error::Error::ParseError {
                        message: format!("failed to parse output json: {e}"),
                    })?;
                return Ok(OutputResult::Single(value));
            }
            let values: HashMap<String, crate::types::output::OutputValue> =
                serde_json::from_str(&output.stdout).map_err(|e| {
                    crate::error::Error::ParseError {
                        message: format!("failed to parse output json: {e}"),
                    }
                })?;
            return Ok(OutputResult::Json(values));
        }

        Ok(OutputResult::Plain(output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_args() {
        let cmd = OutputCommand::new();
        assert_eq!(cmd.args(), vec!["output"]);
    }

    #[test]
    fn json_all_outputs() {
        let cmd = OutputCommand::new().json();
        assert_eq!(cmd.args(), vec!["output", "-json"]);
    }

    #[test]
    fn raw_named_output() {
        let cmd = OutputCommand::new().name("public_ip").raw();
        let args = cmd.args();
        assert_eq!(args, vec!["output", "-raw", "public_ip"]);
    }

    #[test]
    fn json_named_output() {
        let cmd = OutputCommand::new().name("vpc_id").json();
        let args = cmd.args();
        assert_eq!(args, vec!["output", "-json", "vpc_id"]);
    }

    #[test]
    fn name_at_end() {
        let cmd = OutputCommand::new().name("endpoint").arg("-no-color");
        let args = cmd.args();
        // Name should be the last positional argument
        assert_eq!(args.last().unwrap(), "endpoint");
    }
}
