//! Validate and format example: check configuration and fix formatting.
//!
//! Demonstrates using ValidateCommand and FmtCommand together as a
//! pre-commit or CI check workflow.
//!
//! Uses null_resource so no cloud credentials are needed.
//!
//! Usage:
//!   cargo run --example validate_and_fmt

use terraform_wrapper::commands::fmt::FmtCommand;
use terraform_wrapper::commands::init::InitCommand;
use terraform_wrapper::commands::validate::ValidateCommand;
use terraform_wrapper::{Terraform, TerraformCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let dir = tmp.path();

    let tf = Terraform::builder().working_dir(dir).build()?;

    // Write intentionally ugly (but valid) config
    let ugly_tf = r#"
terraform {
required_providers {
null = {
source  = "hashicorp/null"
version = "~> 3.0"
}
}
}

resource "null_resource"   "example" {
  triggers={
    value   =   "hello"
  }
}

output "id" {
value=null_resource.example.id
}
"#;
    std::fs::write(dir.join("main.tf"), ugly_tf)?;

    // Step 1: Check formatting (will fail -- config is ugly)
    println!("--- Step 1: Check formatting ---");
    let output = FmtCommand::new().check().execute(&tf).await?;
    if output.exit_code != 0 {
        println!("Formatting issues found (exit code {})", output.exit_code);
    } else {
        println!("All files formatted correctly");
    }

    // Step 2: Auto-fix formatting
    println!("\n--- Step 2: Auto-format ---");
    FmtCommand::new().execute(&tf).await?;
    println!("Formatting applied");

    // Step 3: Verify formatting is clean now
    let output = FmtCommand::new().check().execute(&tf).await?;
    println!(
        "Format check: {}",
        if output.exit_code == 0 {
            "PASS"
        } else {
            "FAIL"
        }
    );

    // Step 4: Initialize and validate
    println!("\n--- Step 3: Init and validate ---");
    InitCommand::new().execute(&tf).await?;
    let result = ValidateCommand::new().execute(&tf).await?;

    if result.valid {
        println!("Configuration is valid ({} warnings)", result.warning_count);
    } else {
        println!("Configuration is INVALID ({} errors):", result.error_count);
        for diag in &result.diagnostics {
            println!("  [{}] {}: {}", diag.severity, diag.summary, diag.detail);
        }
    }

    // Step 5: Validate something broken
    println!("\n--- Step 4: Validate broken config ---");
    let broken_tf = r#"
output "bad" {
  value = nonexistent_resource.foo.id
}
"#;
    std::fs::write(dir.join("broken.tf"), broken_tf)?;

    let result = ValidateCommand::new().execute(&tf).await?;
    if !result.valid {
        println!(
            "Found {} error(s), {} warning(s):",
            result.error_count, result.warning_count
        );
        for diag in &result.diagnostics {
            println!("  [{}] {}", diag.severity, diag.summary);
        }
    }

    println!("\nDone.");
    Ok(())
}
