use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::output::OutputValue;

/// State representation from `terraform show -json` (current state).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StateRepresentation {
    /// Schema format version.
    pub format_version: String,
    /// Terraform version that wrote the state.
    pub terraform_version: String,
    /// The state values.
    pub values: StateValues,
}

/// Top-level state values.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StateValues {
    /// Output values.
    #[serde(default)]
    pub outputs: HashMap<String, OutputValue>,
    /// The root module.
    pub root_module: Module,
}

/// A module in the state or plan. Contains resources and child modules.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Module {
    /// Resources in this module.
    #[serde(default)]
    pub resources: Vec<Resource>,
    /// Child modules.
    #[serde(default)]
    pub child_modules: Vec<ChildModule>,
}

/// A child module reference.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChildModule {
    /// Module address (e.g., "module.vpc").
    pub address: String,
    /// Resources in this child module.
    #[serde(default)]
    pub resources: Vec<Resource>,
    /// Nested child modules.
    #[serde(default)]
    pub child_modules: Vec<ChildModule>,
}

/// A resource in the state or plan.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Resource {
    /// Full address (e.g., "null_resource.example").
    pub address: String,
    /// Mode: "managed" or "data".
    pub mode: String,
    /// Resource type (e.g., "null_resource").
    #[serde(rename = "type")]
    pub resource_type: String,
    /// Resource name (e.g., "example").
    pub name: String,
    /// Provider name (e.g., "registry.terraform.io/hashicorp/null").
    pub provider_name: String,
    /// Schema version.
    pub schema_version: u32,
    /// Resource attribute values.
    #[serde(default)]
    pub values: Value,
    /// Which values are sensitive.
    #[serde(default)]
    pub sensitive_values: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_state() {
        let json = r#"{
            "format_version": "1.0",
            "terraform_version": "1.14.6",
            "values": {
                "outputs": {
                    "id": {
                        "sensitive": false,
                        "value": "123456",
                        "type": "string"
                    }
                },
                "root_module": {
                    "resources": [
                        {
                            "address": "null_resource.example",
                            "mode": "managed",
                            "type": "null_resource",
                            "name": "example",
                            "provider_name": "registry.terraform.io/hashicorp/null",
                            "schema_version": 0,
                            "values": {"id": "123456", "triggers": {"value": "hello"}},
                            "sensitive_values": {"triggers": {}}
                        }
                    ]
                }
            }
        }"#;
        let state: StateRepresentation = serde_json::from_str(json).unwrap();
        assert_eq!(state.format_version, "1.0");
        assert_eq!(state.values.outputs.len(), 1);
        assert_eq!(state.values.root_module.resources.len(), 1);
        assert_eq!(
            state.values.root_module.resources[0].address,
            "null_resource.example"
        );
        assert_eq!(state.values.root_module.resources[0].mode, "managed");
    }

    #[test]
    fn deserialize_state_no_outputs() {
        let json = r#"{
            "format_version": "1.0",
            "terraform_version": "1.14.6",
            "values": {
                "root_module": {
                    "resources": []
                }
            }
        }"#;
        let state: StateRepresentation = serde_json::from_str(json).unwrap();
        assert!(state.values.outputs.is_empty());
        assert!(state.values.root_module.resources.is_empty());
    }
}
