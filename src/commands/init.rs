use crate::Terraform;
use crate::command::TerraformCommand;
use crate::error::Result;
use crate::exec::{self, CommandOutput};

/// Command for initializing a Terraform working directory.
///
/// Downloads providers, initializes backend, and prepares the directory
/// for other commands.
///
/// ```no_run
/// # async fn example() -> terraform_wrapper::error::Result<()> {
/// use terraform_wrapper::{Terraform, TerraformCommand};
/// use terraform_wrapper::commands::init::InitCommand;
///
/// let tf = Terraform::builder().working_dir("/tmp/infra").build()?;
/// InitCommand::new().execute(&tf).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct InitCommand {
    backend_configs: Vec<(String, String)>,
    backend_config_files: Vec<String>,
    upgrade: bool,
    reconfigure: bool,
    migrate_state: bool,
    plugin_dir: Option<String>,
    lock: Option<bool>,
    lock_timeout: Option<String>,
    raw_args: Vec<String>,
}

impl InitCommand {
    /// Create a new init command with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a backend configuration key-value pair (`-backend-config=key=value`).
    #[must_use]
    pub fn backend_config(mut self, key: &str, value: &str) -> Self {
        self.backend_configs
            .push((key.to_string(), value.to_string()));
        self
    }

    /// Add a backend configuration file (`-backend-config=<path>`).
    #[must_use]
    pub fn backend_config_file(mut self, path: &str) -> Self {
        self.backend_config_files.push(path.to_string());
        self
    }

    /// Update modules and plugins to the latest allowed versions (`-upgrade`).
    #[must_use]
    pub fn upgrade(mut self) -> Self {
        self.upgrade = true;
        self
    }

    /// Reconfigure backend, ignoring any saved configuration (`-reconfigure`).
    #[must_use]
    pub fn reconfigure(mut self) -> Self {
        self.reconfigure = true;
        self
    }

    /// Reconfigure backend and attempt to migrate state (`-migrate-state`).
    #[must_use]
    pub fn migrate_state(mut self) -> Self {
        self.migrate_state = true;
        self
    }

    /// Directory to search for provider plugins (`-plugin-dir`).
    #[must_use]
    pub fn plugin_dir(mut self, path: &str) -> Self {
        self.plugin_dir = Some(path.to_string());
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

impl TerraformCommand for InitCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["init".to_string()];

        for (key, value) in &self.backend_configs {
            args.push(format!("-backend-config={key}={value}"));
        }
        for file in &self.backend_config_files {
            args.push(format!("-backend-config={file}"));
        }
        if self.upgrade {
            args.push("-upgrade".to_string());
        }
        if self.reconfigure {
            args.push("-reconfigure".to_string());
        }
        if self.migrate_state {
            args.push("-migrate-state".to_string());
        }
        if let Some(ref dir) = self.plugin_dir {
            args.push(format!("-plugin-dir={dir}"));
        }
        if let Some(lock) = self.lock {
            args.push(format!("-lock={lock}"));
        }
        if let Some(ref timeout) = self.lock_timeout {
            args.push(format!("-lock-timeout={timeout}"));
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
        let cmd = InitCommand::new();
        assert_eq!(cmd.args(), vec!["init"]);
    }

    #[test]
    fn all_options() {
        let cmd = InitCommand::new()
            .backend_config("key", "value")
            .backend_config_file("backend.hcl")
            .upgrade()
            .reconfigure()
            .plugin_dir("/plugins")
            .lock(false)
            .lock_timeout("10s");
        let args = cmd.args();
        assert!(args.contains(&"-backend-config=key=value".to_string()));
        assert!(args.contains(&"-backend-config=backend.hcl".to_string()));
        assert!(args.contains(&"-upgrade".to_string()));
        assert!(args.contains(&"-reconfigure".to_string()));
        assert!(args.contains(&"-plugin-dir=/plugins".to_string()));
        assert!(args.contains(&"-lock=false".to_string()));
        assert!(args.contains(&"-lock-timeout=10s".to_string()));
    }

    #[test]
    fn raw_arg_escape_hatch() {
        let cmd = InitCommand::new().arg("-from-module=./staging");
        let args = cmd.args();
        assert!(args.contains(&"-from-module=./staging".to_string()));
    }
}
