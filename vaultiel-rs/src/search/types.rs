//! Query AST and result types for vault search.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A search query AST node.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SearchQuery {
    /// A field-level predicate (content, path, tag, etc.).
    Field(FieldPredicate),
    /// All children must match (logical AND).
    And { children: Vec<SearchQuery> },
    /// At least one child must match (logical OR).
    Or { children: Vec<SearchQuery> },
    /// The child must NOT match.
    Not { child: Box<SearchQuery> },
}

/// A predicate on a specific field of a note.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "field", rename_all = "snake_case")]
pub enum FieldPredicate {
    /// Match against the note's file path.
    Path { matcher: StringMatcher },
    /// Match against the note's filename (without extension).
    Filename { matcher: StringMatcher },
    /// Match a tag (checks both frontmatter `tags` array and inline `#tag` in body).
    Tag { value: String },
    /// Match against the note's body content.
    Content { matcher: StringMatcher },
    /// Sub-query must match within a single heading section.
    Section { query: Box<SearchQuery> },
    /// Sub-query must match within a single line.
    Line { query: Box<SearchQuery> },
    /// Match a frontmatter or inline property.
    Property {
        key: String,
        op: PropertyOp,
        value: Option<String>,
    },
}

/// How to match a string value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StringMatcher {
    /// Case-insensitive substring match (default).
    Contains { value: String },
    /// Exact string match.
    Exact { value: String },
    /// Regular expression match.
    Regex { pattern: String },
}

/// Comparison operator for property predicates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyOp {
    /// Property exists (any value).
    Exists,
    /// Equal.
    Eq,
    /// Not equal.
    NotEq,
    /// Less than.
    Lt,
    /// Greater than.
    Gt,
    /// Less than or equal.
    Lte,
    /// Greater than or equal.
    Gte,
}

/// A search result for a single note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Path to the matching note (relative to vault root).
    pub path: PathBuf,
    /// Individual matches within the note.
    pub matches: Vec<SearchMatch>,
}

/// A single match location within a note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    /// Which field matched (e.g. "content", "tag", "property:status").
    pub field: String,
    /// Line number (1-indexed) if applicable.
    pub line: Option<usize>,
    /// The matched text or value.
    pub text: Option<String>,
}
