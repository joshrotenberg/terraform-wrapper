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
    backend: Option<bool>,
    backend_configs: Vec<(String, String)>,
    backend_config_files: Vec<String>,
    from_module: Option<String>,
    get: Option<bool>,
    upgrade: bool,
    reconfigure: bool,
    migrate_state: bool,
    plugin_dir: Option<String>,
    lockfile: Option<String>,
    lock: Option<bool>,
    lock_timeout: Option<String>,
    json: bool,
    raw_args: Vec<String>,
}

impl InitCommand {
    /// Create a new init command with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable backend initialization (`-backend`).
    #[must_use]
    pub fn backend(mut self, enabled: bool) -> Self {
        self.backend = Some(enabled);
        self
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

    /// Copy the contents of the given module into the target directory (`-from-module=SOURCE`).
    #[must_use]
    pub fn from_module(mut self, source: &str) -> Self {
        self.from_module = Some(source.to_string());
        self
    }

    /// Enable or disable downloading modules for this configuration (`-get`).
    #[must_use]
    pub fn get(mut self, enabled: bool) -> Self {
        self.get = Some(enabled);
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

    /// Set the lockfile mode (`-lockfile=MODE`), e.g. `"readonly"`.
    #[must_use]
    pub fn lockfile(mut self, mode: &str) -> Self {
        self.lockfile = Some(mode.to_string());
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

    /// Produce output in a machine-readable JSON format (`-json`).
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

impl TerraformCommand for InitCommand {
    type Output = CommandOutput;

    fn args(&self) -> Vec<String> {
        let mut args = vec!["init".to_string()];

        if let Some(backend) = self.backend {
            args.push(format!("-backend={backend}"));
        }
        for (key, value) in &self.backend_configs {
            args.push(format!("-backend-config={key}={value}"));
        }
        for file in &self.backend_config_files {
            args.push(format!("-backend-config={file}"));
        }
        if let Some(ref source) = self.from_module {
            args.push(format!("-from-module={source}"));
        }
        if let Some(get) = self.get {
            args.push(format!("-get={get}"));
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
        if let Some(ref mode) = self.lockfile {
            args.push(format!("-lockfile={mode}"));
        }
        if let Some(lock) = self.lock {
            args.push(format!("-lock={lock}"));
        }
        if let Some(ref timeout) = self.lock_timeout {
            args.push(format!("-lock-timeout={timeout}"));
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
        let cmd = InitCommand::new();
        assert_eq!(cmd.args(), vec!["init"]);
    }

    #[test]
    fn all_options() {
        let cmd = InitCommand::new()
            .backend(false)
            .backend_config("key", "value")
            .backend_config_file("backend.hcl")
            .from_module("./staging")
            .get(false)
            .upgrade()
            .reconfigure()
            .plugin_dir("/plugins")
            .lockfile("readonly")
            .lock(false)
            .lock_timeout("10s")
            .json();
        let args = cmd.args();
        assert!(args.contains(&"-backend=false".to_string()));
        assert!(args.contains(&"-backend-config=key=value".to_string()));
        assert!(args.contains(&"-backend-config=backend.hcl".to_string()));
        assert!(args.contains(&"-from-module=./staging".to_string()));
        assert!(args.contains(&"-get=false".to_string()));
        assert!(args.contains(&"-upgrade".to_string()));
        assert!(args.contains(&"-reconfigure".to_string()));
        assert!(args.contains(&"-plugin-dir=/plugins".to_string()));
        assert!(args.contains(&"-lockfile=readonly".to_string()));
        assert!(args.contains(&"-lock=false".to_string()));
        assert!(args.contains(&"-lock-timeout=10s".to_string()));
        assert!(args.contains(&"-json".to_string()));
    }

    #[test]
    fn backend_disabled() {
        let cmd = InitCommand::new().backend(false);
        assert!(cmd.args().contains(&"-backend=false".to_string()));
    }

    #[test]
    fn from_module_source() {
        let cmd = InitCommand::new().from_module("./staging");
        assert!(cmd.args().contains(&"-from-module=./staging".to_string()));
    }

    #[test]
    fn get_disabled() {
        let cmd = InitCommand::new().get(false);
        assert!(cmd.args().contains(&"-get=false".to_string()));
    }

    #[test]
    fn lockfile_readonly() {
        let cmd = InitCommand::new().lockfile("readonly");
        assert!(cmd.args().contains(&"-lockfile=readonly".to_string()));
    }

    #[test]
    fn json_output() {
        let cmd = InitCommand::new().json();
        assert!(cmd.args().contains(&"-json".to_string()));
    }

    #[test]
    fn raw_arg_escape_hatch() {
        let cmd = InitCommand::new().arg("-no-color");
        let args = cmd.args();
        assert!(args.contains(&"-no-color".to_string()));
    }
}
