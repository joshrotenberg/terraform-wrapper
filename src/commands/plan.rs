use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// Command for creating a Terraform execution plan.
///
/// Terraform `plan` uses exit code 2 to indicate "changes present" which
/// is treated as a success by this wrapper (not an error).
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::plan::PlanCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
/// let output = PlanCommand::new()
///     .var("region", "us-west-2")
///     .out("tfplan")
///     .execute(&tf)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct PlanCommand {
    vars: Vec<(String, String)>,
    var_files: Vec<String>,
    out: Option<String>,
    targets: Vec<String>,
    replace: Vec<String>,
    destroy: bool,
    refresh_only: bool,
    lock: Option<bool>,
    lock_timeout: Option<String>,
    parallelism: Option<u32>,
    detailed_exitcode: bool,
    json: bool,
    raw_args: Vec<String>,
}

impl PlanCommand {
    /// Create a new plan command with default options.
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

    /// Save the plan to a file (`-out`).
    #[must_use]
    pub fn out(mut self, path: &str) -> Self {
        self.out = Some(path.to_string());
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

    /// Create a destroy plan (`-destroy`).
    #[must_use]
    pub fn destroy(mut self) -> Self {
        self.destroy = true;
        self
    }

    /// Create a plan that only refreshes state (`-refresh-only`).
    #[must_use]
    pub fn refresh_only(mut self) -> Self {
        self.refresh_only = true;
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

    /// Return a detailed exit code (`-detailed-exitcode`).
    ///
    /// When enabled, exit code 0 means no changes, exit code 2 means changes
    /// are present. Without this flag, exit code 0 means success regardless
    /// of whether changes are needed.
    #[must_use]
    pub fn detailed_exitcode(mut self) -> Self {
        self.detailed_exitcode = true;
        self
    }

    /// Enable machine-readable JSON output (`-json`).
    ///
    /// When enabled, stdout contains streaming JSON log lines (one per event),
    /// not a single JSON blob.
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

impl TerraformCommand for PlanCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["plan".to_string()];
        for (name, value) in &self.vars {
            args.push(format!("-var={name}={value}"));
        }
        for file in &self.var_files {
            args.push(format!("-var-file={file}"));
        }
        if let Some(ref out) = self.out {
            args.push(format!("-out={out}"));
        }
        for target in &self.targets {
            args.push(format!("-target={target}"));
        }
        for resource in &self.replace {
            args.push(format!("-replace={resource}"));
        }
        if self.destroy {
            args.push("-destroy".to_string());
        }
        if self.refresh_only {
            args.push("-refresh-only".to_string());
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
        if self.detailed_exitcode {
            args.push("-detailed-exitcode".to_string());
        }
        if self.json {
            args.push("-json".to_string());
        }
        args.extend(self.raw_args.clone());
        args
    }

    async fn execute(&self, tf: &Terraform) -> Result<CommandOutput> {
        let mut args = self.args();
        if tf.no_input {
            args.insert(1, "-input=false".to_string());
        }
        exec::run_terraform_allow_exit_codes(tf, args, &[0, 2]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_args() {
        let cmd = PlanCommand::new();
        assert_eq!(cmd.args(), vec!["plan"]);
    }

    #[test]
    fn full_options() {
        let cmd = PlanCommand::new()
            .var("region", "us-west-2")
            .var_file("prod.tfvars")
            .out("tfplan")
            .target("module.vpc")
            .replace("aws_instance.web")
            .destroy()
            .parallelism(10)
            .json();
        let args = cmd.args();
        assert_eq!(args[0], "plan");
        assert!(args.contains(&"-var=region=us-west-2".to_string()));
        assert!(args.contains(&"-var-file=prod.tfvars".to_string()));
        assert!(args.contains(&"-out=tfplan".to_string()));
        assert!(args.contains(&"-target=module.vpc".to_string()));
        assert!(args.contains(&"-replace=aws_instance.web".to_string()));
        assert!(args.contains(&"-destroy".to_string()));
        assert!(args.contains(&"-parallelism=10".to_string()));
        assert!(args.contains(&"-json".to_string()));
    }

    #[test]
    fn multiple_targets() {
        let cmd = PlanCommand::new().target("module.vpc").target("module.rds");
        let args = cmd.args();
        assert!(args.contains(&"-target=module.vpc".to_string()));
        assert!(args.contains(&"-target=module.rds".to_string()));
    }
}
