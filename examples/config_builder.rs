//! Config builder example: define infrastructure entirely in Rust.
//!
//! No .tf files needed -- generates .tf.json, runs init/apply, reads outputs,
//! then destroys. Uses null_resource so no cloud credentials are needed.
//!
//! Usage:
//!   cargo run --example config_builder --features config

use serde_json::json;
use terraform_wrapper::commands::apply::ApplyCommand;
use terraform_wrapper::commands::destroy::DestroyCommand;
use terraform_wrapper::commands::init::InitCommand;
use terraform_wrapper::commands::output::{OutputCommand, OutputResult};
use terraform_wrapper::config::TerraformConfig;
use terraform_wrapper::{Terraform, TerraformCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define infrastructure in code
    let config = TerraformConfig::new()
        .required_provider("null", "hashicorp/null", "~> 3.0")
        .resource(
            "null_resource",
            "greeting",
            json!({ "triggers": { "message": "${var.greeting}" } }),
        )
        .variable(
            "greeting",
            json!({ "type": "string", "default": "Hello from terraform-wrapper!" }),
        )
        .output("message", json!({ "value": "${var.greeting}" }))
        .output(
            "resource_id",
            json!({ "value": "${null_resource.greeting.id}" }),
        );

    // Write to a temp directory
    let dir = config.write_to_tempdir()?;
    println!("Generated config at: {}", dir.path().display());
    println!(
        "{}",
        std::fs::read_to_string(dir.path().join("main.tf.json"))?
    );

    // Drive with terraform-wrapper
    let tf = Terraform::builder().working_dir(dir.path()).build()?;

    println!("\n--- Init ---");
    InitCommand::new().execute(&tf).await?;

    println!("--- Apply ---");
    ApplyCommand::new().auto_approve().execute(&tf).await?;

    println!("--- Outputs ---");
    let result = OutputCommand::new().json().execute(&tf).await?;
    if let OutputResult::Json(outputs) = result {
        for (name, val) in &outputs {
            println!("  {name} = {}", val.value);
        }
    }

    println!("\n--- Destroy ---");
    DestroyCommand::new().auto_approve().execute(&tf).await?;

    println!("Done.");
    Ok(())
}
