//! # terraform-wrapper
//!
//! A type-safe Terraform CLI wrapper for Rust.
//!
//! This crate provides an idiomatic Rust interface to the Terraform command-line tool.
//! All commands use a builder pattern and async execution via Tokio.
//!
//! # Quick Start
//!
//! ```no_run
//! use terraform_wrapper::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let tf = Terraform::builder()
//!         .working_dir("./infra")
//!         .build()?;
//!
//!     // Initialize, apply, read outputs, destroy
//!     InitCommand::new().execute(&tf).await?;
//!
//!     ApplyCommand::new()
//!         .auto_approve()
//!         .var("region", "us-west-2")
//!         .execute(&tf)
//!         .await?;
//!
//!     let result = OutputCommand::new()
//!         .name("endpoint")
//!         .raw()
//!         .execute(&tf)
//!         .await?;
//!
//!     if let OutputResult::Raw(value) = result {
//!         println!("Endpoint: {value}");
//!     }
//!
//!     DestroyCommand::new().auto_approve().execute(&tf).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Core Concepts
//!
//! ## The `TerraformCommand` Trait
//!
//! All commands implement [`TerraformCommand`], which provides the
//! [`execute()`](TerraformCommand::execute) method. You must import this trait
//! to call `.execute()`:
//!
//! ```rust
//! use terraform_wrapper::TerraformCommand; // Required for .execute()
//! ```
//!
//! ## Builder Pattern
//!
//! Commands are configured using method chaining:
//!
//! ```rust,no_run
//! # use terraform_wrapper::prelude::*;
//! # async fn example() -> terraform_wrapper::error::Result<()> {
//! # let tf = Terraform::builder().build()?;
//! ApplyCommand::new()
//!     .auto_approve()
//!     .var("region", "us-west-2")
//!     .var_file("prod.tfvars")
//!     .target("module.vpc")
//!     .parallelism(10)
//!     .execute(&tf)
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## The `Terraform` Client
//!
//! The [`Terraform`] struct holds shared configuration (binary path, working
//! directory, environment variables) passed to every command:
//!
//! ```rust,no_run
//! # use terraform_wrapper::prelude::*;
//! # fn example() -> terraform_wrapper::error::Result<()> {
//! let tf = Terraform::builder()
//!     .working_dir("./infra")
//!     .env("AWS_REGION", "us-west-2")
//!     .env_var("instance_type", "t3.medium")  // Sets TF_VAR_instance_type
//!     .timeout_secs(300)
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! Programmatic defaults: `-no-color` and `-input=false` are enabled by default.
//! Override with `.color(true)` and `.input(true)`.
//!
//! ## Error Handling
//!
//! All commands return `Result<T, terraform_wrapper::Error>`. The error type
//! implements `std::error::Error`, so it works with `anyhow` and other error
//! libraries via `?`:
//!
//! ```rust,no_run
//! # use terraform_wrapper::prelude::*;
//! # use terraform_wrapper::Error;
//! # async fn example() -> terraform_wrapper::error::Result<()> {
//! # let tf = Terraform::builder().build()?;
//! match InitCommand::new().execute(&tf).await {
//!     Ok(output) => println!("Initialized: {}", output.stdout),
//!     Err(Error::NotFound) => eprintln!("Terraform binary not found"),
//!     Err(Error::CommandFailed { stderr, .. }) => eprintln!("Failed: {stderr}"),
//!     Err(Error::Timeout { timeout_seconds }) => eprintln!("Timed out after {timeout_seconds}s"),
//!     Err(e) => eprintln!("Error: {e}"),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Command Categories
//!
//! ## Lifecycle
//!
//! ```rust
//! use terraform_wrapper::commands::{
//!     InitCommand,     // terraform init
//!     PlanCommand,     // terraform plan
//!     ApplyCommand,    // terraform apply
//!     DestroyCommand,  // terraform destroy
//! };
//! ```
//!
//! ## Inspection
//!
//! ```rust
//! use terraform_wrapper::commands::{
//!     ValidateCommand, // terraform validate
//!     ShowCommand,     // terraform show (state or plan)
//!     OutputCommand,   // terraform output
//!     FmtCommand,      // terraform fmt
//!     GraphCommand,    // terraform graph (DOT format)
//!     ModulesCommand,  // terraform modules
//!     VersionCommand,  // terraform version
//! };
//! ```
//!
//! ## State and Workspace Management
//!
//! ```rust
//! use terraform_wrapper::commands::{
//!     StateCommand,       // terraform state (list, show, mv, rm, pull, push)
//!     WorkspaceCommand,   // terraform workspace (list, show, new, select, delete)
//!     ImportCommand,      // terraform import
//!     ForceUnlockCommand, // terraform force-unlock
//!     GetCommand,         // terraform get (download modules)
//! };
//! ```
//!
//! # JSON Output Types
//!
//! With the `json` feature (enabled by default), commands return typed structs
//! instead of raw strings:
//!
//! ```rust,no_run
//! # use terraform_wrapper::prelude::*;
//! # async fn example() -> terraform_wrapper::error::Result<()> {
//! # let tf = Terraform::builder().build()?;
//! // Version info
//! let info = tf.version().await?;
//! println!("Terraform {} on {}", info.terraform_version, info.platform);
//!
//! // Validate with diagnostics
//! let result = ValidateCommand::new().execute(&tf).await?;
//! if !result.valid {
//!     for diag in &result.diagnostics {
//!         eprintln!("[{}] {}: {}", diag.severity, diag.summary, diag.detail);
//!     }
//! }
//!
//! // Show state with typed resources
//! let result = ShowCommand::new().execute(&tf).await?;
//! if let ShowResult::State(state) = result {
//!     for resource in &state.values.root_module.resources {
//!         println!("{} ({})", resource.address, resource.resource_type);
//!     }
//! }
//!
//! // Show plan with resource changes
//! let result = ShowCommand::new().plan_file("tfplan").execute(&tf).await?;
//! if let ShowResult::Plan(plan) = result {
//!     for change in &plan.resource_changes {
//!         println!("{}: {:?}", change.address, change.change.actions);
//!     }
//! }
//!
//! // Output values
//! let result = OutputCommand::new().json().execute(&tf).await?;
//! if let OutputResult::Json(outputs) = result {
//!     for (name, val) in &outputs {
//!         println!("{name} = {}", val.value);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Streaming Output
//!
//! Long-running commands like `apply` and `plan` with `-json` produce streaming
//! NDJSON (one JSON object per line) instead of a single blob. Use
//! [`streaming::stream_terraform`] to process events as they arrive -- useful
//! for progress reporting, logging, or UI updates:
//!
//! ```rust,no_run
//! # use terraform_wrapper::prelude::*;
//! use terraform_wrapper::streaming::{stream_terraform, JsonLogLine};
//!
//! # async fn example() -> terraform_wrapper::error::Result<()> {
//! # let tf = Terraform::builder().build()?;
//! let result = stream_terraform(
//!     &tf,
//!     ApplyCommand::new().auto_approve().json(),
//!     |line: JsonLogLine| {
//!         match line.log_type.as_str() {
//!             "apply_start" => println!("Creating: {}", line.message),
//!             "apply_progress" => println!("  {}", line.message),
//!             "apply_complete" => println!("Done: {}", line.message),
//!             "apply_errored" => eprintln!("Error: {}", line.message),
//!             "change_summary" => println!("Summary: {}", line.message),
//!             _ => {}
//!         }
//!     },
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! Common event types: `version`, `planned_change`, `change_summary`,
//! `apply_start`, `apply_progress`, `apply_complete`, `apply_errored`, `outputs`.
//!
//! # Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `json` | Yes | Typed JSON output parsing via `serde` / `serde_json` |
//!
//! Disable for raw command output only:
//!
//! ```toml
//! terraform-wrapper = { version = "0.1", default-features = false }
//! ```
//!
//! # OpenTofu Compatibility
//!
//! [OpenTofu](https://opentofu.org/) works out of the box by pointing the client
//! at the `tofu` binary:
//!
//! ```rust,no_run
//! # use terraform_wrapper::prelude::*;
//! # fn example() -> terraform_wrapper::error::Result<()> {
//! let tf = Terraform::builder()
//!     .binary("tofu")
//!     .working_dir("./infra")
//!     .build()?;
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

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub mod command;
pub mod commands;
pub mod error;
pub mod exec;
pub mod prelude;
#[cfg(feature = "json")]
pub mod streaming;
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

    /// Create a clone of this client with a different working directory.
    ///
    /// Useful for running a single command against a different directory
    /// without modifying the original client:
    ///
    /// ```rust,no_run
    /// # use terraform_wrapper::prelude::*;
    /// # async fn example() -> terraform_wrapper::error::Result<()> {
    /// let tf = Terraform::builder()
    ///     .working_dir("./infra/network")
    ///     .build()?;
    ///
    /// // Run one command against a different directory
    /// let compute = tf.with_working_dir("./infra/compute");
    /// InitCommand::new().execute(&compute).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn with_working_dir(&self, path: impl AsRef<Path>) -> Self {
        let mut clone = self.clone();
        clone.working_dir = Some(path.as_ref().to_path_buf());
        clone
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
