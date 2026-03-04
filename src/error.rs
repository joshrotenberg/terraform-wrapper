use thiserror::Error;

/// Result type for terraform-wrapper operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for terraform-wrapper operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Terraform binary not found.
    #[error("terraform binary not found")]
    NotFound,

    /// Command exited with non-zero status.
    #[error("terraform command failed: {command} (exit code {exit_code})")]
    CommandFailed {
        /// The subcommand that failed (e.g. "init", "plan").
        command: String,
        /// Process exit code.
        exit_code: i32,
        /// Captured stdout.
        stdout: String,
        /// Captured stderr.
        stderr: String,
    },

    /// IO error during subprocess execution.
    #[error("io error: {message}")]
    Io {
        /// Description of the IO operation.
        message: String,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Command execution timed out.
    #[error("terraform command timed out after {timeout_seconds}s")]
    Timeout {
        /// The timeout duration in seconds.
        timeout_seconds: u64,
    },

    /// JSON deserialization error.
    #[cfg(feature = "json")]
    #[error("json parse error: {message}")]
    Json {
        /// Description of what was being parsed.
        message: String,
        /// Underlying serde_json error.
        #[source]
        source: serde_json::Error,
    },
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::NotFound
        } else {
            Error::Io {
                message: e.to_string(),
                source: e,
            }
        }
    }
}
