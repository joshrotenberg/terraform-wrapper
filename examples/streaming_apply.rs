//! Streaming apply example: watch Terraform events in real-time.
//!
//! Uses null_resource so no cloud credentials are needed.
//!
//! Usage:
//!   cargo run --example streaming_apply

use terraform_wrapper::commands::apply::ApplyCommand;
use terraform_wrapper::commands::destroy::DestroyCommand;
use terraform_wrapper::commands::init::InitCommand;
use terraform_wrapper::streaming::{JsonLogLine, stream_terraform};
use terraform_wrapper::{Terraform, TerraformCommand};

const CONFIG: &str = r#"
terraform {
  required_providers {
    null = { source = "hashicorp/null", version = "~> 3.0" }
  }
}
resource "null_resource" "a" { triggers = { v = "1" } }
resource "null_resource" "b" { triggers = { v = "2" } }
resource "null_resource" "c" { triggers = { v = "3" } }
output "count" { value = 3 }
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dir = tmp.path();
    std::fs::write(dir.join("main.tf"), CONFIG)?;

    let tf = Terraform::builder().working_dir(dir).build()?;

    println!("--- Initializing ---");
    InitCommand::new().execute(&tf).await?;

    println!("\n--- Streaming Apply ---");
    let mut summary = String::new();
    let result = stream_terraform(
        &tf,
        ApplyCommand::new().auto_approve().json(),
        |line: JsonLogLine| match line.log_type.as_str() {
            "version" => println!("  Terraform {}", line.message),
            "planned_change" => println!("  PLAN: {}", line.message),
            "change_summary" => {
                println!("  SUMMARY: {}", line.message);
                summary = line.message.clone();
            }
            "apply_start" => println!("  START: {}", line.message),
            "apply_complete" => println!("  DONE:  {}", line.message),
            "apply_errored" => eprintln!("  ERROR: {}", line.message),
            "outputs" => println!("  OUTPUTS: {}", line.message),
            _ => println!("  [{}] {}", line.log_type, line.message),
        },
    )
    .await?;

    println!(
        "\nResult: exit_code={}, success={}",
        result.exit_code, result.success
    );
    if !summary.is_empty() {
        println!("Summary: {summary}");
    }

    println!("\n--- Destroying ---");
    DestroyCommand::new().auto_approve().execute(&tf).await?;
    println!("Done.");

    Ok(())
}
