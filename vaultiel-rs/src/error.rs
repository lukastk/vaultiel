//! Error types for Vaultiel.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for Vaultiel operations.
#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Note not found: {0}")]
    NoteNotFound(PathBuf),

    #[error("Note already exists: {0}")]
    NoteAlreadyExists(PathBuf),

    #[error("Ambiguous resolution: {count} notes match '{query}'")]
    AmbiguousResolution {
        query: String,
        count: usize,
        matches: Vec<PathBuf>,
    },

    #[error("Invalid frontmatter in {path}: {message}")]
    InvalidFrontmatter { path: PathBuf, message: String },

    #[error("Vault not found at: {0}")]
    VaultNotFound(PathBuf),

    #[error("Invalid vault path: {0}")]
    InvalidVaultPath(PathBuf),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Glob pattern error: {0}")]
    GlobPattern(#[from] glob::PatternError),

    #[error("Section not found: {0}")]
    SectionNotFound(String),

    #[error("Heading not found in {note}: {heading}")]
    HeadingNotFound { note: PathBuf, heading: String },

    #[error("Block not found: {0}")]
    BlockNotFound(String),

    #[error("Invalid line range: {0}")]
    InvalidLineRange(String),

    #[error("No content provided")]
    NoContentProvided,

    #[error("{0}")]
    Other(String),
}

/// Result type alias for Vaultiel operations.
pub type Result<T> = std::result::Result<T, VaultError>;
