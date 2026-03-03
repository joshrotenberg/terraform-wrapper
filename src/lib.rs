//! A type-safe Terraform CLI wrapper for Rust.
//!
//! `terraform-wrapper` provides builder-pattern command structs for driving the
//! Terraform CLI programmatically. Each command produces typed output and runs
//! asynchronously via tokio.
//!
//! # Quick Start
//!
//! ```no_run
//! use terraform_wrapper::prelude::*;
//!
//! # async fn example() -> terraform_wrapper::error::Result<()> {
//! let tf = Terraform::builder()
//!     .working_dir("/tmp/my-infra")
//!     .build()?;
//!
//! InitCommand::new().execute(&tf).await?;
//!
//! ApplyCommand::new()
//!     .auto_approve()
//!     .var("region", "us-west-2")
//!     .execute(&tf)
//!     .await?;
//!
//! let result = OutputCommand::new()
//!     .name("public_ip")
//!     .raw()
//!     .execute(&tf)
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Imports
//!
//! The [`prelude`] module re-exports everything you need:
//!
//! ```rust
//! use terraform_wrapper::prelude::*;
//! ```
//!
//! Or import selectively from [`commands`]:
//!
//! ```rust
//! use terraform_wrapper::{Terraform, TerraformCommand};
//! use terraform_wrapper::commands::{InitCommand, ApplyCommand, OutputCommand, OutputResult};
//! ```
//!
//! Note: You must import [`TerraformCommand`] (via prelude or directly) to call
//! `.execute()` on any command.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub mod command;
pub mod commands;
pub mod error;
pub mod exec;
pub mod prelude;
#[cfg(feature = "json")]
pub mod types;

pub use command::TerraformCommand;
pub use error::{Error, Result};
pub use exec::CommandOutput;

/// Terraform client configuration.
///
/// Holds the binary path, working directory, environment variables, and global
/// arguments shared across all command executions. Construct via
/// [`Terraform::builder()`].
#[derive(Debug, Clone)]
pub struct Terraform {
    pub(crate) binary: PathBuf,
    pub(crate) working_dir: Option<PathBuf>,
    pub(crate) env: HashMap<String, String>,
    /// Args applied to every subcommand (e.g., `-no-color`).
    pub(crate) global_args: Vec<String>,
    /// Whether to add `-input=false` to commands that support it.
    pub(crate) no_input: bool,
    /// Default timeout for command execution.
    pub(crate) timeout: Option<Duration>,
}

impl Terraform {
    /// Create a new [`TerraformBuilder`].
    #[must_use]
    pub fn builder() -> TerraformBuilder {
        TerraformBuilder::new()
    }

    /// Verify terraform is installed and return version info.
    #[cfg(feature = "json")]
    pub async fn version(&self) -> Result<types::version::VersionInfo> {
        commands::version::VersionCommand::new().execute(self).await
    }
}

/// Builder for constructing a [`Terraform`] client.
///
/// Defaults:
/// - Binary: resolved via `TERRAFORM_PATH` env var, or `terraform` on `PATH`
/// - `-no-color` enabled (disable with `.color(true)`)
/// - `-input=false` enabled (disable with `.input(true)`)
#[derive(Debug)]
pub struct TerraformBuilder {
    binary: Option<PathBuf>,
    working_dir: Option<PathBuf>,
    env: HashMap<String, String>,
    no_color: bool,
    input: bool,
    timeout: Option<Duration>,
}

impl TerraformBuilder {
    fn new() -> Self {
        Self {
            binary: None,
            working_dir: None,
            env: HashMap::new(),
            no_color: true,
            input: false,
            timeout: None,
        }
    }

    /// Set an explicit path to the terraform binary.
    #[must_use]
    pub fn binary(mut self, path: impl Into<PathBuf>) -> Self {
        self.binary = Some(path.into());
        self
    }

    /// Set the default working directory for all commands.
    ///
    /// This is passed as `-chdir=<path>` to terraform.
    #[must_use]
    pub fn working_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.working_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set an environment variable for all terraform subprocesses.
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set a Terraform variable via environment (`TF_VAR_<name>`).
    #[must_use]
    pub fn env_var(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.env
            .insert(format!("TF_VAR_{}", name.into()), value.into());
        self
    }

    /// Enable or disable color output (default: disabled for programmatic use).
    #[must_use]
    pub fn color(mut self, enable: bool) -> Self {
        self.no_color = !enable;
        self
    }

    /// Enable or disable interactive input prompts (default: disabled).
    #[must_use]
    pub fn input(mut self, enable: bool) -> Self {
        self.input = enable;
        self
    }

    /// Set a default timeout for all command executions.
    ///
    /// Commands that exceed this duration will be terminated and return
    /// [`Error::Timeout`]. No timeout is set by default.
    #[must_use]
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Set a default timeout in seconds for all command executions.
    #[must_use]
    pub fn timeout_secs(mut self, seconds: u64) -> Self {
        self.timeout = Some(Duration::from_secs(seconds));
        self
    }

    /// Build the [`Terraform`] client.
    ///
    /// Resolves the terraform binary in this order:
    /// 1. Explicit path set via [`.binary()`](TerraformBuilder::binary)
    /// 2. `TERRAFORM_PATH` environment variable
    /// 3. `terraform` found on `PATH`
    ///
    /// Returns [`Error::NotFound`] if the binary cannot be located.
    pub fn build(self) -> Result<Terraform> {
        let binary = if let Some(path) = self.binary {
            path
        } else if let Ok(path) = std::env::var("TERRAFORM_PATH") {
            PathBuf::from(path)
        } else {
            which::which("terraform").map_err(|_| Error::NotFound)?
        };

        let mut global_args = Vec::new();
        if self.no_color {
            global_args.push("-no-color".to_string());
        }

        Ok(Terraform {
            binary,
            working_dir: self.working_dir,
            env: self.env,
            global_args,
            no_input: !self.input,
            timeout: self.timeout,
        })
    }
}
