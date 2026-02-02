//! Shared types for Vaultiel.

use serde::{Deserialize, Serialize};

/// A wikilink or embed found in a note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    /// The target note path (without .md extension in user-facing output).
    pub target: String,

    /// Optional display alias (the part after |).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,

    /// Optional heading reference (the part after #, before ^).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,

    /// Optional block reference (the part after #^).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,

    /// Whether this is an embed (![[...]]) rather than a link.
    pub embed: bool,

    /// Line number where this link appears (1-indexed).
    pub line: usize,

    /// Start column in the line (0-indexed).
    pub start_col: usize,

    /// End column in the line (0-indexed, exclusive).
    pub end_col: usize,
}

impl Link {
    /// Returns the full link target including heading/block reference.
    pub fn full_target(&self) -> String {
        let mut result = self.target.clone();
        if let Some(ref heading) = self.heading {
            result.push('#');
            result.push_str(heading);
        }
        if let Some(ref block_id) = self.block_id {
            result.push_str("#^");
            result.push_str(block_id);
        }
        result
    }

    /// Returns the display text for this link.
    pub fn display_text(&self) -> &str {
        self.alias.as_deref().unwrap_or(&self.target)
    }
}

/// A tag found in a note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    /// The full tag including # (e.g., "#rust" or "#tray/autonomy").
    pub name: String,

    /// Line number where this tag appears (1-indexed).
    pub line: usize,

    /// Start column in the line (0-indexed).
    pub start_col: usize,

    /// End column in the line (0-indexed, exclusive).
    pub end_col: usize,
}

impl Tag {
    /// Returns the tag without the leading #.
    pub fn without_hash(&self) -> &str {
        self.name.strip_prefix('#').unwrap_or(&self.name)
    }

    /// Returns the parent tag if this is a nested tag.
    /// e.g., "#tray/autonomy" -> Some("#tray")
    pub fn parent(&self) -> Option<String> {
        let without_hash = self.without_hash();
        without_hash
            .rfind('/')
            .map(|idx| format!("#{}", &without_hash[..idx]))
    }

    /// Returns all ancestor tags.
    /// e.g., "#a/b/c" -> ["#a", "#a/b"]
    pub fn ancestors(&self) -> Vec<String> {
        let without_hash = self.without_hash();
        let mut ancestors = Vec::new();
        let mut current = String::new();

        for part in without_hash.split('/') {
            if !current.is_empty() {
                ancestors.push(format!("#{}", current));
                current.push('/');
            }
            current.push_str(part);
        }

        ancestors
    }
}

/// A block ID found in a note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockId {
    /// The block ID without the ^ prefix.
    pub id: String,

    /// Line number where this block ID appears (1-indexed).
    pub line: usize,

    /// The type of block this ID is attached to.
    pub block_type: BlockType,
}

/// Type of block that a block ID is attached to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BlockType {
    Paragraph,
    ListItem,
    Heading,
    Blockquote,
    CodeBlock,
    Table,
    Other,
}

/// A heading found in a note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heading {
    /// The heading text (without the # prefix).
    pub text: String,

    /// The heading level (1-6).
    pub level: u8,

    /// Line number where this heading appears (1-indexed).
    pub line: usize,

    /// The slug for linking (lowercase, hyphens for spaces).
    pub slug: String,
}

/// An inline attribute found in a note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InlineAttr {
    /// The attribute key.
    pub key: String,

    /// The attribute value.
    pub value: String,

    /// Line number where this attribute appears (1-indexed).
    pub line: usize,

    /// Start column in the line (0-indexed).
    pub start_col: usize,

    /// End column in the line (0-indexed, exclusive).
    pub end_col: usize,
}

/// Context where a link appears in a note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkContext {
    /// In the note body (regular markdown content).
    Body,

    /// In a frontmatter field (scalar value).
    FrontmatterScalar { key: String },

    /// In a frontmatter field (list item).
    FrontmatterList { key: String, index: usize },

    /// In an inline attribute.
    Inline { key: String },

    /// Inside a task item.
    Task,
}

impl LinkContext {
    /// Returns a string representation for output.
    pub fn as_string(&self) -> String {
        match self {
            LinkContext::Body => "body".to_string(),
            LinkContext::FrontmatterScalar { key } => format!("frontmatter:{}", key),
            LinkContext::FrontmatterList { key, index } => {
                format!("frontmatter:{}[{}]", key, index)
            }
            LinkContext::Inline { key } => format!("inline:{}", key),
            LinkContext::Task => "task".to_string(),
        }
    }
}

/// Output format for CLI commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Json,
    Yaml,
    Toml,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_without_hash() {
        let tag = Tag {
            name: "#rust".to_string(),
            line: 1,
            start_col: 0,
            end_col: 5,
        };
        assert_eq!(tag.without_hash(), "rust");
    }

    #[test]
    fn test_tag_parent() {
        let tag = Tag {
            name: "#tray/autonomy".to_string(),
            line: 1,
            start_col: 0,
            end_col: 14,
        };
        assert_eq!(tag.parent(), Some("#tray".to_string()));

        let root_tag = Tag {
            name: "#rust".to_string(),
            line: 1,
            start_col: 0,
            end_col: 5,
        };
        assert_eq!(root_tag.parent(), None);
    }

    #[test]
    fn test_tag_ancestors() {
        let tag = Tag {
            name: "#a/b/c".to_string(),
            line: 1,
            start_col: 0,
            end_col: 6,
        };
        assert_eq!(tag.ancestors(), vec!["#a", "#a/b"]);
    }

    #[test]
    fn test_link_full_target() {
        let link = Link {
            target: "note".to_string(),
            alias: None,
            heading: Some("section".to_string()),
            block_id: None,
            embed: false,
            line: 1,
            start_col: 0,
            end_col: 10,
        };
        assert_eq!(link.full_target(), "note#section");

        let link_with_block = Link {
            target: "note".to_string(),
            alias: None,
            heading: None,
            block_id: Some("abc123".to_string()),
            embed: false,
            line: 1,
            start_col: 0,
            end_col: 10,
        };
        assert_eq!(link_with_block.full_target(), "note#^abc123");
    }
}
