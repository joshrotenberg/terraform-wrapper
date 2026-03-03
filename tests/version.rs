use terraform_wrapper::commands::version::VersionCommand;
use terraform_wrapper::{Terraform, TerraformCommand};

fn setup_terraform() -> Option<Terraform> {
    Terraform::builder().build().ok()
}

#[tokio::test]
async fn version_returns_valid_info() {
    let Some(tf) = setup_terraform() else {
        eprintln!("terraform not found, skipping test");
        return;
    };

    let info = VersionCommand::new().execute(&tf).await.unwrap();
    assert!(!info.terraform_version.is_empty());
    assert!(!info.platform.is_empty());
}

#[tokio::test]
async fn version_convenience_method() {
    let Some(tf) = setup_terraform() else {
        eprintln!("terraform not found, skipping test");
        return;
    };

    let info = tf.version().await.unwrap();
    assert!(!info.terraform_version.is_empty());
}
