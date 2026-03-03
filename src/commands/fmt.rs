use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// Command for formatting Terraform configuration files.
///
/// By default, rewrites files in-place. Use `.check()` to only verify
/// formatting without modifying files, or `.diff()` to display differences.
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::fmt::FmtCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
///
/// // Check formatting without modifying files
/// let output = FmtCommand::new().check().execute(&tf).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct FmtCommand {
    check: bool,
    diff: bool,
    recursive: bool,
    write: Option<bool>,
    raw_args: Vec<String>,
}

impl FmtCommand {
    /// Create a new fmt command with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if files are formatted without modifying them (`-check`).
    ///
    /// Returns exit code 0 if all files are formatted, exit code 3 if not.
    #[must_use]
    pub fn check(mut self) -> Self {
        self.check = true;
        self
    }

    /// Display diffs of formatting changes (`-diff`).
    #[must_use]
    pub fn diff(mut self) -> Self {
        self.diff = true;
        self
    }

    /// Process files in subdirectories recursively (`-recursive`).
    #[must_use]
    pub fn recursive(mut self) -> Self {
        self.recursive = true;
        self
    }

    /// Control whether to write changes to files (`-write`).
    #[must_use]
    pub fn write(mut self, enabled: bool) -> Self {
        self.write = Some(enabled);
        self
    }

    /// Add a raw argument (escape hatch for unsupported options).
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.raw_args.push(arg.into());
        self
    }
}

impl TerraformCommand for FmtCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["fmt".to_string()];
        if self.check {
            args.push("-check".to_string());
        }
        if self.diff {
            args.push("-diff".to_string());
        }
        if self.recursive {
            args.push("-recursive".to_string());
        }
        if let Some(write) = self.write {
            args.push(format!("-write={write}"));
        }
        args.extend(self.raw_args.clone());
        args
    }

    async fn execute(&self, tf: &Terraform) -> Result<CommandOutput> {
        // fmt -check returns exit code 3 when files need formatting
        exec::run_terraform_allow_exit_codes(tf, self.args(), &[0, 3]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_args() {
        let cmd = FmtCommand::new();
        assert_eq!(cmd.args(), vec!["fmt"]);
    }

    #[test]
    fn check_and_diff() {
        let cmd = FmtCommand::new().check().diff();
        let args = cmd.args();
        assert!(args.contains(&"-check".to_string()));
        assert!(args.contains(&"-diff".to_string()));
    }

    #[test]
    fn recursive() {
        let cmd = FmtCommand::new().recursive();
        assert!(cmd.args().contains(&"-recursive".to_string()));
    }

    #[test]
    fn write_false() {
        let cmd = FmtCommand::new().write(false);
        assert!(cmd.args().contains(&"-write=false".to_string()));
    }
}
