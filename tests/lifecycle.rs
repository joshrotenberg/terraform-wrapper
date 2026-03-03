use terraform_wrapper::commands::apply::ApplyCommand;
use terraform_wrapper::commands::destroy::DestroyCommand;
use terraform_wrapper::commands::init::InitCommand;
use terraform_wrapper::commands::output::{OutputCommand, OutputResult};
use terraform_wrapper::commands::plan::PlanCommand;
use terraform_wrapper::commands::validate::ValidateCommand;
use terraform_wrapper::{Terraform, TerraformCommand};

fn setup_terraform(dir: &std::path::Path) -> Option<Terraform> {
    Terraform::builder().working_dir(dir).build().ok()
}

/// Write a minimal Terraform config using null_resource (no cloud provider needed).
fn write_null_config(dir: &std::path::Path) {
    let main_tf = r#"
terraform {
  required_providers {
    null = {
      source  = "hashicorp/null"
      version = "~> 3.0"
    }
  }
}

resource "null_resource" "example" {
  triggers = {
    value = var.trigger_value
  }
}

variable "trigger_value" {
  default = "hello"
}

output "trigger" {
  value = var.trigger_value
}

output "id" {
  value = null_resource.example.id
}
"#;
    std::fs::write(dir.join("main.tf"), main_tf).unwrap();
}

#[tokio::test]
async fn init_plan_apply_output_destroy() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let Some(tf) = setup_terraform(dir) else {
        eprintln!("terraform not found, skipping test");
        return;
    };

    write_null_config(dir);

    // Init
    let init_output = InitCommand::new().execute(&tf).await.unwrap();
    assert!(init_output.success);

    // Plan with detailed exit code (exit code 2 = changes present)
    let plan_output = PlanCommand::new()
        .out("tfplan")
        .detailed_exitcode()
        .execute(&tf)
        .await
        .unwrap();
    assert_eq!(plan_output.exit_code, 2);

    // Apply saved plan
    let apply_output = ApplyCommand::new()
        .plan_file("tfplan")
        .execute(&tf)
        .await
        .unwrap();
    assert!(apply_output.success);

    // Output - JSON all
    let result = OutputCommand::new().json().execute(&tf).await.unwrap();
    match result {
        OutputResult::Json(ref outputs) => {
            assert!(outputs.contains_key("trigger"));
            assert!(outputs.contains_key("id"));
            assert_eq!(
                outputs["trigger"].value,
                serde_json::Value::String("hello".into())
            );
        }
        _ => panic!("expected Json variant"),
    }

    // Output - raw single value
    let result = OutputCommand::new()
        .name("trigger")
        .raw()
        .execute(&tf)
        .await
        .unwrap();
    match result {
        OutputResult::Raw(ref value) => {
            assert_eq!(value, "hello");
        }
        _ => panic!("expected Raw variant"),
    }

    // Plan again (should show no changes)
    let plan_output = PlanCommand::new().execute(&tf).await.unwrap();
    assert_eq!(plan_output.exit_code, 0);

    // Destroy
    let destroy_output = DestroyCommand::new()
        .auto_approve()
        .execute(&tf)
        .await
        .unwrap();
    assert!(destroy_output.success);
}

#[tokio::test]
async fn init_with_upgrade() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let Some(tf) = setup_terraform(dir) else {
        eprintln!("terraform not found, skipping test");
        return;
    };

    write_null_config(dir);

    // First init
    InitCommand::new().execute(&tf).await.unwrap();

    // Init with upgrade
    let output = InitCommand::new().upgrade().execute(&tf).await.unwrap();
    assert!(output.success);
}

#[tokio::test]
async fn apply_with_var_override() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let Some(tf) = setup_terraform(dir) else {
        eprintln!("terraform not found, skipping test");
        return;
    };

    write_null_config(dir);
    InitCommand::new().execute(&tf).await.unwrap();

    ApplyCommand::new()
        .auto_approve()
        .var("trigger_value", "custom")
        .execute(&tf)
        .await
        .unwrap();

    let result = OutputCommand::new()
        .name("trigger")
        .raw()
        .execute(&tf)
        .await
        .unwrap();
    match result {
        OutputResult::Raw(ref value) => assert_eq!(value, "custom"),
        _ => panic!("expected Raw variant"),
    }

    DestroyCommand::new()
        .auto_approve()
        .execute(&tf)
        .await
        .unwrap();
}

#[tokio::test]
async fn validate_valid_config() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let Some(tf) = setup_terraform(dir) else {
        eprintln!("terraform not found, skipping test");
        return;
    };

    write_null_config(dir);
    InitCommand::new().execute(&tf).await.unwrap();

    let result = ValidateCommand::new().execute(&tf).await.unwrap();
    assert!(result.valid);
    assert_eq!(result.error_count, 0);
}

#[tokio::test]
async fn validate_invalid_config() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let Some(tf) = setup_terraform(dir) else {
        eprintln!("terraform not found, skipping test");
        return;
    };

    // Write invalid config (reference to nonexistent resource)
    let bad_tf = r#"
output "bad" {
  value = nonexistent_resource.foo.id
}
"#;
    std::fs::write(dir.join("main.tf"), bad_tf).unwrap();

    let result = ValidateCommand::new().execute(&tf).await.unwrap();
    assert!(!result.valid);
    assert!(result.error_count > 0);
    assert!(!result.diagnostics.is_empty());
}
