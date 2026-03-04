use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// The providers subcommand to execute.
#[derive(Debug, Clone)]
pub enum ProvidersSubcommand {
    /// List required providers for the configuration.
    Default,
    /// Write provider dependency lock file.
    Lock,
    /// Mirror providers to a local directory.
    Mirror(String),
    /// Output provider schemas as JSON.
    Schema,
}

/// Command for listing and managing Terraform providers.
///
/// Supports several subcommands for inspecting and managing provider
/// dependencies:
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::providers::ProvidersCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
///
/// // List required providers
/// let output = ProvidersCommand::new().execute(&tf).await?;
///
/// // Output provider schemas as JSON (useful for tooling)
/// let output = ProvidersCommand::schema().execute(&tf).await?;
///
/// // Mirror providers to a local directory
/// let output = ProvidersCommand::mirror("/tmp/providers")
///     .platform("linux_amd64")
///     .execute(&tf)
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ProvidersCommand {
    subcommand: ProvidersSubcommand,
    platforms: Vec<String>,
    raw_args: Vec<String>,
}

impl ProvidersCommand {
    /// List required providers (default subcommand).
    #[must_use]
    pub fn new() -> Self {
        Self {
            subcommand: ProvidersSubcommand::Default,
            platforms: Vec::new(),
            raw_args: Vec::new(),
        }
    }

    /// Write the provider dependency lock file (`providers lock`).
    #[must_use]
    pub fn lock() -> Self {
        Self {
            subcommand: ProvidersSubcommand::Lock,
            platforms: Vec::new(),
            raw_args: Vec::new(),
        }
    }

    /// Mirror providers to a local directory (`providers mirror <target_dir>`).
    #[must_use]
    pub fn mirror(target_dir: &str) -> Self {
        Self {
            subcommand: ProvidersSubcommand::Mirror(target_dir.to_string()),
            platforms: Vec::new(),
            raw_args: Vec::new(),
        }
    }

    /// Output provider schemas as JSON (`providers schema -json`).
    #[must_use]
    pub fn schema() -> Self {
        Self {
            subcommand: ProvidersSubcommand::Schema,
            platforms: Vec::new(),
            raw_args: Vec::new(),
        }
    }

    /// Target a specific platform (`-platform`). Can be called multiple times.
    ///
    /// Applies to `lock` and `mirror` subcommands.
    #[must_use]
    pub fn platform(mut self, platform: &str) -> Self {
        self.platforms.push(platform.to_string());
        self
    }

    /// Add a raw argument (escape hatch for unsupported options).
    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.raw_args.push(arg.into());
        self
    }
}

impl Default for ProvidersCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl TerraformCommand for ProvidersCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["providers".to_string()];
        match &self.subcommand {
            ProvidersSubcommand::Default => {}
            ProvidersSubcommand::Lock => {
                args.push("lock".to_string());
                for platform in &self.platforms {
                    args.push(format!("-platform={platform}"));
                }
            }
            ProvidersSubcommand::Mirror(target_dir) => {
                args.push("mirror".to_string());
                for platform in &self.platforms {
                    args.push(format!("-platform={platform}"));
                }
                args.push(target_dir.clone());
            }
            ProvidersSubcommand::Schema => {
                args.push("schema".to_string());
                args.push("-json".to_string());
            }
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
        let cmd = ProvidersCommand::new();
        assert_eq!(cmd.args(), vec!["providers"]);
    }

    #[test]
    fn lock_args() {
        let cmd = ProvidersCommand::lock();
        assert_eq!(cmd.args(), vec!["providers", "lock"]);
    }

    #[test]
    fn lock_with_platforms() {
        let cmd = ProvidersCommand::lock()
            .platform("linux_amd64")
            .platform("darwin_arm64");
        let args = cmd.args();
        assert_eq!(args[0], "providers");
        assert_eq!(args[1], "lock");
        assert!(args.contains(&"-platform=linux_amd64".to_string()));
        assert!(args.contains(&"-platform=darwin_arm64".to_string()));
    }

    #[test]
    fn mirror_args() {
        let cmd = ProvidersCommand::mirror("/tmp/providers");
        assert_eq!(cmd.args(), vec!["providers", "mirror", "/tmp/providers"]);
    }

    #[test]
    fn mirror_with_platform() {
        let cmd = ProvidersCommand::mirror("/tmp/providers").platform("linux_amd64");
        assert_eq!(
            cmd.args(),
            vec![
                "providers",
                "mirror",
                "-platform=linux_amd64",
                "/tmp/providers"
            ]
        );
    }

    #[test]
    fn schema_args() {
        let cmd = ProvidersCommand::schema();
        assert_eq!(cmd.args(), vec!["providers", "schema", "-json"]);
    }
}
