use serde::{Deserialize, Serialize};

/// A single Terraform output value from `terraform output -json`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputValue {
    /// Whether the output is marked as sensitive.
    pub sensitive: bool,
    /// The Terraform type expression (as a JSON value).
    ///
    /// May be absent in plan output for outputs whose type is not yet known.
    #[serde(rename = "type", default)]
    pub output_type: serde_json::Value,
    /// The output value.
    ///
    /// May be absent in plan output for outputs whose value is not yet known.
    #[serde(default)]
    pub value: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn deserialize_single_output() {
        let json = r#"{
            "sensitive": false,
            "type": "string",
            "value": "10.0.1.5"
        }"#;
        let output: OutputValue = serde_json::from_str(json).unwrap();
        assert!(!output.sensitive);
        assert_eq!(output.value, serde_json::Value::String("10.0.1.5".into()));
    }

    #[test]
    fn deserialize_sensitive_output() {
        let json = r#"{
            "sensitive": true,
            "type": "string",
            "value": "s3cr3t"
        }"#;
        let output: OutputValue = serde_json::from_str(json).unwrap();
        assert!(output.sensitive);
    }

    #[test]
    fn deserialize_all_outputs() {
        let json = r#"{
            "instance_id": {
                "sensitive": false,
                "type": "string",
                "value": "i-abc123"
            },
            "public_ip": {
                "sensitive": false,
                "type": "string",
                "value": "1.2.3.4"
            }
        }"#;
        let outputs: HashMap<String, OutputValue> = serde_json::from_str(json).unwrap();
        assert_eq!(outputs.len(), 2);
        assert_eq!(
            outputs["instance_id"].value,
            serde_json::Value::String("i-abc123".into())
        );
    }
}
