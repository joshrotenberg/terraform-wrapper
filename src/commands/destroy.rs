use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// Command for destroying Terraform-managed infrastructure.
///
/// This is equivalent to `terraform apply -destroy` but provided as a
/// separate command for clarity and discoverability.
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::destroy::DestroyCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
/// DestroyCommand::new()
///     .auto_approve()
///     .execute(&tf)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct DestroyCommand {
    auto_approve: bool,
    vars: Vec<(String, String)>,
    var_files: Vec<String>,
    targets: Vec<String>,
    lock: Option<bool>,
    lock_timeout: Option<String>,
    parallelism: Option<u32>,
    json: bool,
    raw_args: Vec<String>,
}

impl DestroyCommand {
    /// Create a new destroy command with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Skip interactive approval (`-auto-approve`).
    #[must_use]
    pub fn auto_approve(mut self) -> Self {
        self.auto_approve = true;
        self
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

impl TerraformCommand for DestroyCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["destroy".to_string()];
        if self.auto_approve {
            args.push("-auto-approve".to_string());
        }
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
        let cmd = DestroyCommand::new();
        assert_eq!(cmd.args(), vec!["destroy"]);
    }

    #[test]
    fn auto_approve_with_targets() {
        let cmd = DestroyCommand::new()
            .auto_approve()
            .target("module.vpc")
            .var("region", "us-west-2")
            .json();
        let args = cmd.args();
        assert_eq!(args[0], "destroy");
        assert!(args.contains(&"-auto-approve".to_string()));
        assert!(args.contains(&"-target=module.vpc".to_string()));
        assert!(args.contains(&"-var=region=us-west-2".to_string()));
        assert!(args.contains(&"-json".to_string()));
    }
}
