//! Workspace management example: create, switch, list, and delete workspaces.
//!
//! Uses null_resource so no cloud credentials are needed.
//!
//! Usage:
//!   cargo run --example workspace_management

use terraform_wrapper::commands::apply::ApplyCommand;
use terraform_wrapper::commands::destroy::DestroyCommand;
use terraform_wrapper::commands::init::InitCommand;
use terraform_wrapper::commands::output::{OutputCommand, OutputResult};
use terraform_wrapper::commands::workspace::WorkspaceCommand;
use terraform_wrapper::{Terraform, TerraformCommand};

const CONFIG: &str = r#"
terraform {
  required_providers {
    null = { source = "hashicorp/null", version = "~> 3.0" }
  }
}

variable "env" {
  default = "default"
}

resource "null_resource" "example" {
  triggers = { env = var.env }
}

output "workspace_env" {
  value = var.env
}
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dir = tmp.path();
    std::fs::write(dir.join("main.tf"), CONFIG)?;

    let tf = Terraform::builder().working_dir(dir).build()?;

    println!("--- Initializing ---");
    InitCommand::new().execute(&tf).await?;

    // Show current workspace (starts as "default")
    let output = WorkspaceCommand::show().execute(&tf).await?;
    println!("Current workspace: {}", output.stdout.trim());

    // Create and switch to "staging"
    println!("\n--- Creating 'staging' workspace ---");
    WorkspaceCommand::new_workspace("staging")
        .execute(&tf)
        .await?;

    let output = WorkspaceCommand::show().execute(&tf).await?;
    println!("Current workspace: {}", output.stdout.trim());

    // Apply in the staging workspace
    ApplyCommand::new()
        .auto_approve()
        .var("env", "staging")
        .execute(&tf)
        .await?;

    let result = OutputCommand::new()
        .name("workspace_env")
        .raw()
        .execute(&tf)
        .await?;
    if let OutputResult::Raw(value) = result {
        println!("Output in staging: {value}");
    }

    // List all workspaces
    println!("\n--- All workspaces ---");
    let output = WorkspaceCommand::list().execute(&tf).await?;
    print!("{}", output.stdout);

    // Clean up: destroy, switch back, delete
    println!("\n--- Cleaning up ---");
    DestroyCommand::new().auto_approve().execute(&tf).await?;
    WorkspaceCommand::select("default").execute(&tf).await?;
    WorkspaceCommand::delete("staging").execute(&tf).await?;
    println!("Done.");

    Ok(())
}
