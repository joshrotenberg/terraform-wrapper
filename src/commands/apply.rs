use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// Command for applying Terraform changes.
///
/// When applying a saved plan file, options like `-var`, `-var-file`, and
/// `-target` are not valid (Terraform will reject them).
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::apply::ApplyCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
/// ApplyCommand::new()
///     .auto_approve()
///     .var("region", "us-west-2")
///     .execute(&tf)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct ApplyCommand {
    plan_file: Option<String>,
    auto_approve: bool,
    vars: Vec<(String, String)>,
    var_files: Vec<String>,
    targets: Vec<String>,
    replace: Vec<String>,
    lock: Option<bool>,
    lock_timeout: Option<String>,
    parallelism: Option<u32>,
    json: bool,
    raw_args: Vec<String>,
}

impl ApplyCommand {
    /// Create a new apply command with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a previously saved plan file (positional argument).
    #[must_use]
    pub fn plan_file(mut self, path: &str) -> Self {
        self.plan_file = Some(path.to_string());
        self
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

    /// Mark a resource for replacement (`-replace`).
    #[must_use]
    pub fn replace(mut self, resource: &str) -> Self {
        self.replace.push(resource.to_string());
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

impl TerraformCommand for ApplyCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["apply".to_string()];
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
        for resource in &self.replace {
            args.push(format!("-replace={resource}"));
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
        // Plan file is a positional argument at the end
        if let Some(ref plan) = self.plan_file {
            args.push(plan.clone());
        }
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
        let cmd = ApplyCommand::new();
        assert_eq!(cmd.args(), vec!["apply"]);
    }

    #[test]
    fn auto_approve_with_vars() {
        let cmd = ApplyCommand::new()
            .auto_approve()
            .var("region", "us-west-2")
            .var_file("prod.tfvars");
        let args = cmd.args();
        assert_eq!(args[0], "apply");
        assert!(args.contains(&"-auto-approve".to_string()));
        assert!(args.contains(&"-var=region=us-west-2".to_string()));
        assert!(args.contains(&"-var-file=prod.tfvars".to_string()));
    }

    #[test]
    fn plan_file_at_end() {
        let cmd = ApplyCommand::new().auto_approve().plan_file("tfplan");
        let args = cmd.args();
        assert_eq!(args.last().unwrap(), "tfplan");
    }

    #[test]
    fn parallelism_and_json() {
        let cmd = ApplyCommand::new().parallelism(10).json();
        let args = cmd.args();
        assert!(args.contains(&"-parallelism=10".to_string()));
        assert!(args.contains(&"-json".to_string()));
    }
}
