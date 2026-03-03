//! Basic GCE instance lifecycle: init, plan, apply, read outputs, destroy.
//!
//! Requires:
//! - GCP Application Default Credentials (`gcloud auth application-default login`)
//! - A GCP project with Compute Engine API enabled
//!
//! Usage:
//!   cargo run --example gce_instance -- --project=YOUR_PROJECT_ID
//!
//! To tear down only:
//!   cargo run --example gce_instance -- --project=YOUR_PROJECT_ID --destroy

use terraform_wrapper::commands::apply::ApplyCommand;
use terraform_wrapper::commands::destroy::DestroyCommand;
use terraform_wrapper::commands::init::InitCommand;
use terraform_wrapper::commands::output::{OutputCommand, OutputResult};
use terraform_wrapper::commands::plan::PlanCommand;
use terraform_wrapper::{Terraform, TerraformCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let project = args
        .iter()
        .find(|a| a.starts_with("--project="))
        .map(|a| a.trim_start_matches("--project=").to_string())
        .expect("Usage: cargo run --example gce_instance -- --project=YOUR_PROJECT_ID");

    let destroy_only = args.iter().any(|a| a == "--destroy");

    let tf = Terraform::builder()
        .working_dir("examples/gce-instance")
        .build()?;

    let version = tf.version().await?;
    println!("Terraform {}", version.terraform_version);

    if destroy_only {
        println!("\n--- Destroying infrastructure ---");
        DestroyCommand::new()
            .auto_approve()
            .var("project", &project)
            .execute(&tf)
            .await?;
        println!("Destroyed.");
        return Ok(());
    }

    // Init
    println!("\n--- Initializing ---");
    InitCommand::new().execute(&tf).await?;
    println!("Initialized.");

    // Plan
    println!("\n--- Planning ---");
    let plan_output = PlanCommand::new()
        .var("project", &project)
        .out("tfplan")
        .detailed_exitcode()
        .execute(&tf)
        .await?;

    if plan_output.exit_code == 0 {
        println!("No changes needed.");
        return Ok(());
    }
    println!("Changes detected.");

    // Apply
    println!("\n--- Applying ---");
    ApplyCommand::new().plan_file("tfplan").execute(&tf).await?;
    println!("Applied.");

    // Read outputs
    println!("\n--- Outputs ---");
    let result = OutputCommand::new().json().execute(&tf).await?;
    if let OutputResult::Json(outputs) = result {
        for (name, val) in &outputs {
            println!("  {name} = {}", val.value);
        }
    }

    println!("\nTo destroy: cargo run --example gce_instance -- --project={project} --destroy");

    Ok(())
}
