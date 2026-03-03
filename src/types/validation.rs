use serde::{Deserialize, Serialize};

/// Result from `terraform validate -json`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationResult {
    /// Schema format version.
    pub format_version: String,
    /// Whether the configuration is valid.
    pub valid: bool,
    /// Number of errors.
    pub error_count: u32,
    /// Number of warnings.
    pub warning_count: u32,
    /// Diagnostic messages.
    pub diagnostics: Vec<Diagnostic>,
}

/// A diagnostic message from Terraform.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Diagnostic {
    /// Severity: "error" or "warning".
    pub severity: String,
    /// Short summary of the issue.
    pub summary: String,
    /// Detailed description.
    #[serde(default)]
    pub detail: String,
    /// Source range where the issue was found.
    pub range: Option<DiagnosticRange>,
}

/// Source location for a diagnostic.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiagnosticRange {
    /// The filename containing the issue.
    pub filename: String,
    /// Start position.
    pub start: DiagnosticPos,
    /// End position.
    pub end: DiagnosticPos,
}

/// A position within a source file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiagnosticPos {
    /// Line number (1-indexed).
    pub line: u32,
    /// Column number (1-indexed).
    pub column: u32,
    /// Byte offset.
    pub byte: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_valid_result() {
        let json = r#"{
            "format_version": "1.0",
            "valid": true,
            "error_count": 0,
            "warning_count": 0,
            "diagnostics": []
        }"#;
        let result: ValidationResult = serde_json::from_str(json).unwrap();
        assert!(result.valid);
        assert_eq!(result.error_count, 0);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn deserialize_invalid_result() {
        let json = r#"{
            "format_version": "1.0",
            "valid": false,
            "error_count": 1,
            "warning_count": 0,
            "diagnostics": [
                {
                    "severity": "error",
                    "summary": "Missing required argument",
                    "detail": "The argument \"region\" is required.",
                    "range": {
                        "filename": "main.tf",
                        "start": {"line": 5, "column": 1, "byte": 42},
                        "end": {"line": 5, "column": 10, "byte": 51}
                    }
                }
            ]
        }"#;
        let result: ValidationResult = serde_json::from_str(json).unwrap();
        assert!(!result.valid);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, "error");
        assert_eq!(result.diagnostics[0].summary, "Missing required argument");
        let range = result.diagnostics[0].range.as_ref().unwrap();
        assert_eq!(range.filename, "main.tf");
        assert_eq!(range.start.line, 5);
    }

    #[test]
    fn deserialize_diagnostic_without_range() {
        let json = r#"{
            "severity": "warning",
            "summary": "Deprecated feature",
            "detail": "",
            "range": null
        }"#;
        let diag: Diagnostic = serde_json::from_str(json).unwrap();
        assert_eq!(diag.severity, "warning");
        assert!(diag.range.is_none());
    }
}
