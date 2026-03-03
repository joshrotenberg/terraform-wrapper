//! Convenience re-exports for common usage.
//!
//! ```rust
//! use terraform_wrapper::prelude::*;
//! ```
//!
//! This imports the [`Terraform`] client, [`TerraformCommand`] trait (required
//! for `.execute()`), all command types, result enums, and [`CommandOutput`].

pub use crate::Terraform;
pub use crate::command::TerraformCommand;
pub use crate::commands::{
    ApplyCommand, DestroyCommand, FmtCommand, ImportCommand, InitCommand, OutputCommand,
    OutputResult, PlanCommand, ShowCommand, ShowResult, StateCommand, ValidateCommand,
    VersionCommand, WorkspaceCommand,
};
pub use crate::exec::CommandOutput;
