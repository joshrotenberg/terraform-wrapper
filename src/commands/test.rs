use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// Command for running Terraform module integration tests.
///
/// Runs `.tftest.hcl` test files. Available in Terraform 1.6+.
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::test::TestCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
/// let output = TestCommand::new()
///     .filter("my_test")
///     .verbose()
///     .execute(&tf)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct TestCommand {
    filter: Option<String>,
    json: bool,
    test_directory: Option<String>,
    verbose: bool,
    vars: Vec<(String, String)>,
    var_files: Vec<String>,
    parallelism: Option<u32>,
    junit_xml: Option<String>,
    raw_args: Vec<String>,
}

impl TestCommand {
    /// Create a new test command with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter to a specific test (`-filter`).
    #[must_use]
    pub fn filter(mut self, name: &str) -> Self {
        self.filter = Some(name.to_string());
        self
    }

    /// Enable machine-readable JSON output (`-json`).
    #[must_use]
    pub fn json(mut self) -> Self {
        self.json = true;
        self
    }

    /// Set the directory containing test files (`-test-directory`).
    ///
    /// Defaults to `tests` if not specified.
    #[must_use]
    pub fn test_directory(mut self, path: &str) -> Self {
        self.test_directory = Some(path.to_string());
        self
    }

    /// Enable verbose output (`-verbose`).
    #[must_use]
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
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

    /// Limit the number of concurrent operations (`-parallelism`).
    #[must_use]
    pub fn parallelism(mut self, n: u32) -> Self {
        self.parallelism = Some(n);
        self
    }

    /// Write test results to a JUnit XML file (`-junit-xml`).
    #[must_use]
    pub fn junit_xml(mut self, path: &str) -> Self {
        self.junit_xml = Some(path.to_string());
        self
    }

    /// Add a raw argument (escape hatch for unsupported options).
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.raw_args.push(arg.into());
        self
    }
}

impl TerraformCommand for TestCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["test".to_string()];
        if let Some(ref filter) = self.filter {
            args.push(format!("-filter={filter}"));
        }
        if self.json {
            args.push("-json".to_string());
        }
        if let Some(ref dir) = self.test_directory {
            args.push(format!("-test-directory={dir}"));
        }
        if self.verbose {
            args.push("-verbose".to_string());
        }
        for (name, value) in &self.vars {
            args.push(format!("-var={name}={value}"));
        }
        for file in &self.var_files {
            args.push(format!("-var-file={file}"));
        }
        if let Some(n) = self.parallelism {
            args.push(format!("-parallelism={n}"));
        }
        if let Some(ref path) = self.junit_xml {
            args.push(format!("-junit-xml={path}"));
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
    fn default_args() {
        let cmd = TestCommand::new();
        assert_eq!(cmd.args(), vec!["test"]);
    }

    #[test]
    fn with_filter() {
        let cmd = TestCommand::new().filter("my_test");
        assert_eq!(cmd.args(), vec!["test", "-filter=my_test"]);
    }

    #[test]
    fn with_json_and_verbose() {
        let cmd = TestCommand::new().json().verbose();
        let args = cmd.args();
        assert_eq!(args[0], "test");
        assert!(args.contains(&"-json".to_string()));
        assert!(args.contains(&"-verbose".to_string()));
    }

    #[test]
    fn with_test_directory() {
        let cmd = TestCommand::new().test_directory("integration");
        assert_eq!(cmd.args(), vec!["test", "-test-directory=integration"]);
    }

    #[test]
    fn with_vars() {
        let cmd = TestCommand::new()
            .var("region", "us-west-2")
            .var("env", "staging");
        let args = cmd.args();
        assert!(args.contains(&"-var=region=us-west-2".to_string()));
        assert!(args.contains(&"-var=env=staging".to_string()));
    }

    #[test]
    fn with_var_files() {
        let cmd = TestCommand::new()
            .var_file("prod.tfvars")
            .var_file("overrides.tfvars");
        let args = cmd.args();
        assert!(args.contains(&"-var-file=prod.tfvars".to_string()));
        assert!(args.contains(&"-var-file=overrides.tfvars".to_string()));
    }

    #[test]
    fn with_parallelism() {
        let cmd = TestCommand::new().parallelism(4);
        let args = cmd.args();
        assert!(args.contains(&"-parallelism=4".to_string()));
    }

    #[test]
    fn with_junit_xml() {
        let cmd = TestCommand::new().junit_xml("results.xml");
        let args = cmd.args();
        assert!(args.contains(&"-junit-xml=results.xml".to_string()));
    }

    #[test]
    fn all_options() {
        let cmd = TestCommand::new()
            .filter("vpc_test")
            .json()
            .test_directory("e2e")
            .verbose()
            .var("region", "us-west-2")
            .var_file("prod.tfvars")
            .parallelism(8)
            .junit_xml("results.xml");
        let args = cmd.args();
        assert_eq!(args[0], "test");
        assert!(args.contains(&"-filter=vpc_test".to_string()));
        assert!(args.contains(&"-json".to_string()));
        assert!(args.contains(&"-test-directory=e2e".to_string()));
        assert!(args.contains(&"-verbose".to_string()));
        assert!(args.contains(&"-var=region=us-west-2".to_string()));
        assert!(args.contains(&"-var-file=prod.tfvars".to_string()));
        assert!(args.contains(&"-parallelism=8".to_string()));
        assert!(args.contains(&"-junit-xml=results.xml".to_string()));
    }
}
