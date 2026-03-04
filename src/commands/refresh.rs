use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// Command for updating Terraform state to match remote systems.
///
/// **Deprecated:** `terraform refresh` is deprecated in favor of
/// `terraform apply -refresh-only`. Consider using [`ApplyCommand`] with
/// `-refresh-only` instead. This command is provided for compatibility
/// with existing workflows.
///
/// [`ApplyCommand`]: super::ApplyCommand
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::refresh::RefreshCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
/// RefreshCommand::new()
///     .var("region", "us-west-2")
///     .execute(&tf)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct RefreshCommand {
    vars: Vec<(String, String)>,
    var_files: Vec<String>,
    targets: Vec<String>,
    lock: Option<bool>,
    lock_timeout: Option<String>,
    parallelism: Option<u32>,
    json: bool,
    raw_args: Vec<String>,
}

impl RefreshCommand {
    /// Create a new refresh command with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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

    /// Target a specific resource or module (`-target`).
    #[must_use]
    pub fn target(mut self, resource: &str) -> Self {
        self.targets.push(resource.to_string());
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

    /// Limit the number of concurrent operations (`-parallelism`).
    #[must_use]
    pub fn parallelism(mut self, n: u32) -> Self {
        self.parallelism = Some(n);
        self
    }

    /// Enable machine-readable JSON output (`-json`).
    #[must_use]
    pub fn json(mut self) -> Self {
        self.json = true;
        self
    }

    /// Add a raw argument (escape hatch for unsupported options).
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.raw_args.push(arg.into());
        self
    }
}

impl TerraformCommand for RefreshCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["refresh".to_string()];
        for (name, value) in &self.vars {
            args.push(format!("-var={name}={value}"));
        }
        for file in &self.var_files {
            args.push(format!("-var-file={file}"));
        }
        for target in &self.targets {
            args.push(format!("-target={target}"));
        }
        if let Some(lock) = self.lock {
            args.push(format!("-lock={lock}"));
        }
        if let Some(ref timeout) = self.lock_timeout {
            args.push(format!("-lock-timeout={timeout}"));
        }
        if let Some(n) = self.parallelism {
            args.push(format!("-parallelism={n}"));
        }
        if self.json {
            args.push("-json".to_string());
        }
        args.extend(self.raw_args.clone());
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
    fn default_args() {
        let cmd = RefreshCommand::new();
        assert_eq!(cmd.args(), vec!["refresh"]);
    }

    #[test]
    fn with_vars_and_targets() {
        let cmd = RefreshCommand::new()
            .var("region", "us-west-2")
            .target("module.vpc")
            .json();
        let args = cmd.args();
        assert_eq!(args[0], "refresh");
        assert!(args.contains(&"-var=region=us-west-2".to_string()));
        assert!(args.contains(&"-target=module.vpc".to_string()));
        assert!(args.contains(&"-json".to_string()));
    }

    #[test]
    fn with_lock_options() {
        let cmd = RefreshCommand::new()
            .lock(false)
            .lock_timeout("10s")
            .parallelism(5);
        let args = cmd.args();
        assert!(args.contains(&"-lock=false".to_string()));
        assert!(args.contains(&"-lock-timeout=10s".to_string()));
        assert!(args.contains(&"-parallelism=5".to_string()));
    }
}
