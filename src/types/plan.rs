use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::output::OutputValue;
use crate::types::state::{Module, StateRepresentation};

/// Plan representation from `terraform show -json <planfile>`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlanRepresentation {
    /// Schema format version.
    pub format_version: String,
    /// Terraform version that created the plan.
    pub terraform_version: String,
    /// The planned values after apply.
    pub planned_values: PlannedValues,
    /// Changes to resources.
    #[serde(default)]
    pub resource_changes: Vec<ResourceChange>,
    /// Changes to outputs.
    #[serde(default)]
    pub output_changes: HashMap<String, OutputChange>,
    /// Prior state (if any).
    pub prior_state: Option<StateRepresentation>,
    /// Plan timestamp.
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Whether the plan can be applied.
    #[serde(default)]
    pub applyable: bool,
    /// Whether the plan is complete.
    #[serde(default)]
    pub complete: bool,
    /// Whether the plan errored.
    #[serde(default)]
    pub errored: bool,
}

/// Planned values after apply.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlannedValues {
    /// Planned output values.
    #[serde(default)]
    pub outputs: HashMap<String, OutputValue>,
    /// The planned root module.
    pub root_module: Module,
}

/// A change to a single resource.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceChange {
    /// Resource address (e.g., "null_resource.example").
    pub address: String,
    /// Mode: "managed" or "data".
    pub mode: String,
    /// Resource type.
    #[serde(rename = "type")]
    pub resource_type: String,
    /// Resource name.
    pub name: String,
    /// Provider name.
    pub provider_name: String,
    /// The change details.
    pub change: Change,
}

/// Details of a resource or output change.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Change {
    /// Actions: "no-op", "create", "read", "update", "delete", or combinations.
    pub actions: Vec<String>,
    /// Values before the change.
    #[serde(default)]
    pub before: Value,
    /// Values after the change.
    #[serde(default)]
    pub after: Value,
    /// Which values are unknown after apply.
    #[serde(default)]
    pub after_unknown: Value,
    /// Which before-values are sensitive.
    #[serde(default)]
    pub before_sensitive: Value,
    /// Which after-values are sensitive.
    #[serde(default)]
    pub after_sensitive: Value,
}

/// A change to a single output.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputChange {
    /// Actions: "no-op", "create", "update", "delete".
    pub actions: Vec<String>,
    /// Value before the change.
    #[serde(default)]
    pub before: Value,
    /// Value after the change.
    #[serde(default)]
    pub after: Value,
    /// Whether after value is unknown.
    #[serde(default)]
    pub after_unknown: Value,
    /// Whether before value is sensitive.
    #[serde(default)]
    pub before_sensitive: Value,
    /// Whether after value is sensitive.
    #[serde(default)]
    pub after_sensitive: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_plan() {
        let json = r#"{
            "format_version": "1.2",
            "terraform_version": "1.14.6",
            "planned_values": {
                "outputs": {
                    "id": {
                        "sensitive": false,
                        "type": "string",
                        "value": "123456"
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
                            "values": {"id": "123456"},
                            "sensitive_values": {}
                        }
                    ]
                }
            },
            "resource_changes": [
                {
                    "address": "null_resource.example",
                    "mode": "managed",
                    "type": "null_resource",
                    "name": "example",
                    "provider_name": "registry.terraform.io/hashicorp/null",
                    "change": {
                        "actions": ["no-op"],
                        "before": {"id": "123456"},
                        "after": {"id": "123456"},
                        "after_unknown": {},
                        "before_sensitive": {},
                        "after_sensitive": {}
                    }
                }
            ],
            "output_changes": {
                "id": {
                    "actions": ["no-op"],
                    "before": "123456",
                    "after": "123456",
                    "after_unknown": false,
                    "before_sensitive": false,
                    "after_sensitive": false
                }
            },
            "applyable": false,
            "complete": true,
            "errored": false
        }"#;
        let plan: PlanRepresentation = serde_json::from_str(json).unwrap();
        assert_eq!(plan.format_version, "1.2");
        assert_eq!(plan.resource_changes.len(), 1);
        assert_eq!(plan.resource_changes[0].change.actions, vec!["no-op"]);
        assert_eq!(plan.output_changes.len(), 1);
        assert!(!plan.applyable);
        assert!(plan.complete);
    }

    #[test]
    fn deserialize_plan_with_create() {
        let json = r#"{
            "format_version": "1.2",
            "terraform_version": "1.14.6",
            "planned_values": {
                "root_module": { "resources": [] }
            },
            "resource_changes": [
                {
                    "address": "null_resource.new",
                    "mode": "managed",
                    "type": "null_resource",
                    "name": "new",
                    "provider_name": "registry.terraform.io/hashicorp/null",
                    "change": {
                        "actions": ["create"],
                        "before": null,
                        "after": {"triggers": null},
                        "after_unknown": {"id": true},
                        "before_sensitive": false,
                        "after_sensitive": false
                    }
                }
            ],
            "applyable": true,
            "complete": true,
            "errored": false
        }"#;
        let plan: PlanRepresentation = serde_json::from_str(json).unwrap();
        assert_eq!(plan.resource_changes[0].change.actions, vec!["create"]);
        assert!(plan.applyable);
    }
}
