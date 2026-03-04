use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// Command for importing existing infrastructure into Terraform state.
///
/// Associates an existing resource with a Terraform resource address.
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::import::ImportCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
/// ImportCommand::new("aws_instance.web", "i-1234567890abcdef0")
///     .execute(&tf)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ImportCommand {
    address: String,
    id: String,
    vars: Vec<(String, String)>,
    var_files: Vec<String>,
    lock: Option<bool>,
    lock_timeout: Option<String>,
    raw_args: Vec<String>,
}

impl ImportCommand {
    /// Create a new import command.
    ///
    /// - `address`: The Terraform resource address (e.g., "aws_instance.web")
    /// - `id`: The provider-specific resource ID (e.g., "i-1234567890abcdef0")
    #[must_use]
    pub fn new(address: &str, id: &str) -> Self {
        Self {
            address: address.to_string(),
            id: id.to_string(),
            vars: Vec::new(),
            var_files: Vec::new(),
            lock: None,
            lock_timeout: None,
            raw_args: Vec::new(),
        }
    }

    /// Set a variable value (`-var="name=value"`).
    #[must_use]
    pub fn var(mut self, name: &str, value: &str) -> Self {
        self.vars.push((name.to_string(), value.to_string()));
        self
    }

    /// Add a variable definitions file (`-var-file`).
    #[must_use]
    pub fn var_file(mut self, path: &str) -> Self {
        self.var_files.push(path.to_string());
        self
    }

    /// Enable or disable state locking (`-lock`).
    #[must_use]
    pub fn lock(mut self, enabled: bool) -> Self {
        self.lock = Some(enabled);
        self
    }

    /// Duration to wait for state lock (`-lock-timeout`).
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
}

impl TerraformCommand for ImportCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["import".to_string()];
        for (name, value) in &self.vars {
            args.push(format!("-var={name}={value}"));
        }
        for file in &self.var_files {
            args.push(format!("-var-file={file}"));
        }
        if let Some(lock) = self.lock {
            args.push(format!("-lock={lock}"));
        }
        if let Some(ref timeout) = self.lock_timeout {
            args.push(format!("-lock-timeout={timeout}"));
        }
        args.extend(self.raw_args.clone());
        // Positional args at end: address, then id
        args.push(self.address.clone());
        args.push(self.id.clone());
        args
    }

    fn supports_input(&self) -> bool {
        true
    }

    async fn execute(&self, tf: &Terraform) -> Result<CommandOutput> {
        exec::run_terraform(tf, self.prepare_args(tf)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_args() {
        let cmd = ImportCommand::new("aws_instance.web", "i-123");
        assert_eq!(cmd.args(), vec!["import", "aws_instance.web", "i-123"]);
    }

    #[test]
    fn with_vars() {
        let cmd = ImportCommand::new("aws_instance.web", "i-123").var("region", "us-west-2");
        let args = cmd.args();
        assert!(args.contains(&"-var=region=us-west-2".to_string()));
        // Positional args still at end
        let len = args.len();
        assert_eq!(args[len - 2], "aws_instance.web");
        assert_eq!(args[len - 1], "i-123");
    }

    #[test]
    fn with_lock_options() {
        let cmd = ImportCommand::new("aws_instance.web", "i-123")
            .lock(false)
            .lock_timeout("10s");
        let args = cmd.args();
        assert!(args.contains(&"-lock=false".to_string()));
        assert!(args.contains(&"-lock-timeout=10s".to_string()));
    }
}
