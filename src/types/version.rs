use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Terraform version information from `terraform version -json`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionInfo {
    /// Terraform version string (e.g., "1.14.6").
    pub terraform_version: String,
    /// Platform identifier (e.g., "darwin_arm64").
    pub platform: String,
    /// Map of provider name to selected version.
    pub provider_selections: HashMap<String, String>,
    /// Whether a newer version of Terraform is available.
    pub terraform_outdated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_version_json() {
        let json = r#"{
            "terraform_version": "1.14.6",
            "platform": "darwin_arm64",
            "provider_selections": {},
            "terraform_outdated": false
        }"#;
        let info: VersionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.terraform_version, "1.14.6");
        assert_eq!(info.platform, "darwin_arm64");
        assert!(info.provider_selections.is_empty());
        assert!(!info.terraform_outdated);
    }

    #[test]
    fn deserialize_with_providers() {
        let json = r#"{
            "terraform_version": "1.14.6",
            "platform": "linux_amd64",
            "provider_selections": {
                "registry.terraform.io/hashicorp/aws": "5.0.0",
                "registry.terraform.io/hashicorp/null": "3.2.1"
            },
            "terraform_outdated": true
        }"#;
        let info: VersionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.provider_selections.len(), 2);
        assert_eq!(
            info.provider_selections["registry.terraform.io/hashicorp/aws"],
            "5.0.0"
        );
        assert!(info.terraform_outdated);
    }
}
