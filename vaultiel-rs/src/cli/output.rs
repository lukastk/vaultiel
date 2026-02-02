//! Output formatting for CLI commands.

use crate::cli::args::OutputFormat;
use crate::error::Result;
use serde::Serialize;

/// Helper for formatting and printing output.
pub struct Output {
    format: OutputFormat,
    quiet: bool,
}

impl Output {
    pub fn new(format: OutputFormat, quiet: bool) -> Self {
        Self { format, quiet }
    }

    /// Print a serializable value in the configured format.
    pub fn print<T: Serialize>(&self, value: &T) -> Result<()> {
        let output = match self.format {
            OutputFormat::Json => serde_json::to_string_pretty(value)?,
            OutputFormat::Yaml => serde_yaml::to_string(value)?,
            OutputFormat::Toml => toml::to_string_pretty(value)?,
        };
        println!("{}", output);
        Ok(())
    }

    /// Print a serializable value without pretty-printing (compact JSON).
    pub fn print_compact<T: Serialize>(&self, value: &T) -> Result<()> {
        let output = match self.format {
            OutputFormat::Json => serde_json::to_string(value)?,
            OutputFormat::Yaml => serde_yaml::to_string(value)?,
            OutputFormat::Toml => toml::to_string(value)?,
        };
        println!("{}", output);
        Ok(())
    }

    /// Print raw text (not serialized).
    pub fn print_raw(&self, text: &str) {
        println!("{}", text);
    }

    /// Print a message if not in quiet mode.
    pub fn info(&self, message: &str) {
        if !self.quiet {
            eprintln!("{}", message);
        }
    }

    /// Print a warning message.
    pub fn warn(&self, message: &str) {
        eprintln!("Warning: {}", message);
    }

    /// Print an error message.
    pub fn error(&self, message: &str) {
        eprintln!("Error: {}", message);
    }

    /// Check if quiet mode is enabled.
    pub fn is_quiet(&self) -> bool {
        self.quiet
    }
}

/// Standard response structure for commands.
#[derive(Debug, Serialize)]
pub struct CommandResponse<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

impl<T> CommandResponse<T> {
    pub fn data(data: T) -> Self {
        Self {
            data: Some(data),
            message: None,
            warning: None,
        }
    }

    pub fn message(msg: impl Into<String>) -> Self {
        Self {
            data: None,
            message: Some(msg.into()),
            warning: None,
        }
    }

    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warning = Some(warning.into());
        self
    }
}

/// Dry-run response showing what would change.
#[derive(Debug, Serialize)]
pub struct DryRunResponse {
    pub action: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changes: Option<Vec<String>>,
}
