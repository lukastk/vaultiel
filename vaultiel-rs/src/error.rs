//! Error types and exit codes for Vaultiel.

use std::path::PathBuf;
use thiserror::Error;

/// Exit codes as specified in PROJECT_SPEC.md
pub mod exit_code {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const NOTE_NOT_FOUND: i32 = 2;
    pub const NOTE_ALREADY_EXISTS: i32 = 3;
    pub const AMBIGUOUS_RESOLUTION: i32 = 4;
    pub const INVALID_FRONTMATTER: i32 = 5;
    pub const LINT_ISSUES_FOUND: i32 = 10;
}

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

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

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

    #[error("Cache error: {0}")]
    CacheError(String),
}

impl VaultError {
    /// Returns the appropriate exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            VaultError::NoteNotFound(_) => exit_code::NOTE_NOT_FOUND,
            VaultError::NoteAlreadyExists(_) => exit_code::NOTE_ALREADY_EXISTS,
            VaultError::AmbiguousResolution { .. } => exit_code::AMBIGUOUS_RESOLUTION,
            VaultError::InvalidFrontmatter { .. } => exit_code::INVALID_FRONTMATTER,
            _ => exit_code::GENERAL_ERROR,
        }
    }
}

/// Result type alias for Vaultiel operations.
pub type Result<T> = std::result::Result<T, VaultError>;

/// Exit code for CLI operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success,
    GeneralError,
    NoteNotFound,
    NoteAlreadyExists,
    AmbiguousResolution,
    InvalidFrontmatter,
    LintIssuesFound,
}

impl ExitCode {
    /// Convert to exit code integer.
    pub fn code(self) -> i32 {
        match self {
            ExitCode::Success => exit_code::SUCCESS,
            ExitCode::GeneralError => exit_code::GENERAL_ERROR,
            ExitCode::NoteNotFound => exit_code::NOTE_NOT_FOUND,
            ExitCode::NoteAlreadyExists => exit_code::NOTE_ALREADY_EXISTS,
            ExitCode::AmbiguousResolution => exit_code::AMBIGUOUS_RESOLUTION,
            ExitCode::InvalidFrontmatter => exit_code::INVALID_FRONTMATTER,
            ExitCode::LintIssuesFound => exit_code::LINT_ISSUES_FOUND,
        }
    }
}
