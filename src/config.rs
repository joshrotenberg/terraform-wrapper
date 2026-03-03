//! Terraform configuration builder for generating `.tf.json` files.
//!
//! Terraform natively accepts [JSON configuration syntax](https://developer.hashicorp.com/terraform/language/syntax/json)
//! alongside HCL. This module provides a builder API to construct configs
//! entirely in Rust and serialize them to `.tf.json`.
//!
//! # Example
//!
//! ```rust
//! use terraform_wrapper::config::TerraformConfig;
//! use serde_json::json;
//!
//! let config = TerraformConfig::new()
//!     .required_provider("null", "hashicorp/null", "~> 3.0")
//!     .resource("null_resource", "example", json!({
//!         "triggers": { "value": "hello" }
//!     }))
//!     .variable("name", json!({ "type": "string", "default": "world" }))
//!     .output("id", json!({ "value": "${null_resource.example.id}" }));
//!
//! let json = config.to_json_pretty().unwrap();
//! assert!(json.contains("null_resource"));
//! ```

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

/// Builder for constructing Terraform JSON configuration.
///
/// Produces a `.tf.json` file that Terraform can process identically
/// to an HCL `.tf` file.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TerraformConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    terraform: Option<TerraformBlock>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    provider: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    resource: BTreeMap<String, BTreeMap<String, Value>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    data: BTreeMap<String, BTreeMap<String, Value>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    variable: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    output: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    locals: BTreeMap<String, Value>,
}

/// The `terraform` block (required_providers, backend, etc.).
#[derive(Debug, Clone, Default, Serialize)]
struct TerraformBlock {
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    required_providers: BTreeMap<String, ProviderRequirement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    backend: Option<BTreeMap<String, Value>>,
}

/// A provider requirement in the `required_providers` block.
#[derive(Debug, Clone, Serialize)]
struct ProviderRequirement {
    source: String,
    version: String,
}

impl TerraformConfig {
    /// Create a new empty configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a required provider.
    ///
    /// ```rust
    /// # use terraform_wrapper::config::TerraformConfig;
    /// let config = TerraformConfig::new()
    ///     .required_provider("aws", "hashicorp/aws", "~> 5.0")
    ///     .required_provider("null", "hashicorp/null", "~> 3.0");
    /// ```
    #[must_use]
    pub fn required_provider(mut self, name: &str, source: &str, version: &str) -> Self {
        let block = self.terraform.get_or_insert_with(TerraformBlock::default);
        block.required_providers.insert(
            name.to_string(),
            ProviderRequirement {
                source: source.to_string(),
                version: version.to_string(),
            },
        );
        self
    }

    /// Configure a backend for remote state storage.
    ///
    /// ```rust
    /// # use terraform_wrapper::config::TerraformConfig;
    /// # use serde_json::json;
    /// let config = TerraformConfig::new()
    ///     .backend("s3", json!({
    ///         "bucket": "my-tf-state",
    ///         "key": "terraform.tfstate",
    ///         "region": "us-west-2"
    ///     }));
    /// ```
    #[must_use]
    pub fn backend(mut self, backend_type: &str, config: Value) -> Self {
        let block = self.terraform.get_or_insert_with(TerraformBlock::default);
        let mut backend = BTreeMap::new();
        backend.insert(backend_type.to_string(), config);
        block.backend = Some(backend);
        self
    }

    /// Configure a provider.
    ///
    /// ```rust
    /// # use terraform_wrapper::config::TerraformConfig;
    /// # use serde_json::json;
    /// let config = TerraformConfig::new()
    ///     .provider("aws", json!({ "region": "us-west-2" }));
    /// ```
    #[must_use]
    pub fn provider(mut self, name: &str, config: Value) -> Self {
        self.provider.insert(name.to_string(), config);
        self
    }

    /// Add a managed resource.
    ///
    /// ```rust
    /// # use terraform_wrapper::config::TerraformConfig;
    /// # use serde_json::json;
    /// let config = TerraformConfig::new()
    ///     .resource("aws_instance", "web", json!({
    ///         "ami": "ami-0c55b159",
    ///         "instance_type": "t3.micro"
    ///     }));
    /// ```
    #[must_use]
    pub fn resource(mut self, resource_type: &str, name: &str, config: Value) -> Self {
        self.resource
            .entry(resource_type.to_string())
            .or_default()
            .insert(name.to_string(), config);
        self
    }

    /// Add a data source.
    ///
    /// ```rust
    /// # use terraform_wrapper::config::TerraformConfig;
    /// # use serde_json::json;
    /// let config = TerraformConfig::new()
    ///     .data("aws_ami", "latest", json!({
    ///         "most_recent": true,
    ///         "owners": ["amazon"]
    ///     }));
    /// ```
    #[must_use]
    pub fn data(mut self, data_type: &str, name: &str, config: Value) -> Self {
        self.data
            .entry(data_type.to_string())
            .or_default()
            .insert(name.to_string(), config);
        self
    }

    /// Add a variable.
    ///
    /// ```rust
    /// # use terraform_wrapper::config::TerraformConfig;
    /// # use serde_json::json;
    /// let config = TerraformConfig::new()
    ///     .variable("region", json!({
    ///         "type": "string",
    ///         "default": "us-west-2",
    ///         "description": "AWS region"
    ///     }));
    /// ```
    #[must_use]
    pub fn variable(mut self, name: &str, config: Value) -> Self {
        self.variable.insert(name.to_string(), config);
        self
    }

    /// Add an output.
    ///
    /// ```rust
    /// # use terraform_wrapper::config::TerraformConfig;
    /// # use serde_json::json;
    /// let config = TerraformConfig::new()
    ///     .output("instance_id", json!({
    ///         "value": "${aws_instance.web.id}",
    ///         "description": "The instance ID"
    ///     }));
    /// ```
    #[must_use]
    pub fn output(mut self, name: &str, config: Value) -> Self {
        self.output.insert(name.to_string(), config);
        self
    }

    /// Add a local value.
    ///
    /// ```rust
    /// # use terraform_wrapper::config::TerraformConfig;
    /// # use serde_json::json;
    /// let config = TerraformConfig::new()
    ///     .local("common_tags", json!({
    ///         "Environment": "production",
    ///         "ManagedBy": "terraform-wrapper"
    ///     }));
    /// ```
    #[must_use]
    pub fn local(mut self, name: &str, value: Value) -> Self {
        self.locals.insert(name.to_string(), value);
        self
    }

    /// Serialize to a JSON string.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// Serialize to a pretty-printed JSON string.
    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Write the configuration to a file as `main.tf.json`.
    ///
    /// Creates parent directories if they don't exist.
    pub fn write_to(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    /// Write the configuration to a temporary directory as `main.tf.json`.
    ///
    /// Returns the `TempDir` which will be cleaned up when dropped.
    /// Use `.path()` to get the directory path for `Terraform::builder().working_dir()`.
    pub fn write_to_tempdir(&self) -> std::io::Result<tempfile::TempDir> {
        let dir = tempfile::tempdir()?;
        self.write_to(dir.path().join("main.tf.json"))?;
        Ok(dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_config() {
        let config = TerraformConfig::new();
        let json = config.to_json().unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn required_provider() {
        let config = TerraformConfig::new().required_provider("aws", "hashicorp/aws", "~> 5.0");
        let val: Value = serde_json::from_str(&config.to_json().unwrap()).unwrap();
        assert_eq!(
            val["terraform"]["required_providers"]["aws"]["source"],
            "hashicorp/aws"
        );
        assert_eq!(
            val["terraform"]["required_providers"]["aws"]["version"],
            "~> 5.0"
        );
    }

    #[test]
    fn full_config() {
        let config = TerraformConfig::new()
            .required_provider("null", "hashicorp/null", "~> 3.0")
            .provider("null", json!({}))
            .resource(
                "null_resource",
                "example",
                json!({
                    "triggers": { "value": "hello" }
                }),
            )
            .variable("name", json!({ "type": "string", "default": "world" }))
            .output("id", json!({ "value": "${null_resource.example.id}" }))
            .local("tag", json!("test"));

        let val: Value = serde_json::from_str(&config.to_json().unwrap()).unwrap();
        assert!(val["resource"]["null_resource"]["example"].is_object());
        assert_eq!(val["variable"]["name"]["default"], "world");
        assert_eq!(val["output"]["id"]["value"], "${null_resource.example.id}");
        assert_eq!(val["locals"]["tag"], "test");
    }

    #[test]
    fn multiple_resources_same_type() {
        let config = TerraformConfig::new()
            .resource("null_resource", "a", json!({}))
            .resource("null_resource", "b", json!({}));
        let val: Value = serde_json::from_str(&config.to_json().unwrap()).unwrap();
        assert!(val["resource"]["null_resource"]["a"].is_object());
        assert!(val["resource"]["null_resource"]["b"].is_object());
    }

    #[test]
    fn data_source() {
        let config =
            TerraformConfig::new().data("aws_ami", "latest", json!({ "most_recent": true }));
        let val: Value = serde_json::from_str(&config.to_json().unwrap()).unwrap();
        assert_eq!(val["data"]["aws_ami"]["latest"]["most_recent"], true);
    }

    #[test]
    fn backend() {
        let config = TerraformConfig::new().backend("s3", json!({ "bucket": "my-state" }));
        let val: Value = serde_json::from_str(&config.to_json().unwrap()).unwrap();
        assert_eq!(val["terraform"]["backend"]["s3"]["bucket"], "my-state");
    }

    #[test]
    fn write_to_tempdir() {
        let config = TerraformConfig::new()
            .required_provider("null", "hashicorp/null", "~> 3.0")
            .resource("null_resource", "test", json!({}));

        let dir = config.write_to_tempdir().unwrap();
        let path = dir.path().join("main.tf.json");
        assert!(path.exists());

        let contents = std::fs::read_to_string(&path).unwrap();
        let val: Value = serde_json::from_str(&contents).unwrap();
        assert!(val["resource"]["null_resource"]["test"].is_object());
    }
}
