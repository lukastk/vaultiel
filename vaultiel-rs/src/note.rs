//! Note representation and operations.

use crate::error::Result;
use crate::parser::{
    self, parse_block_ids, parse_headings, parse_inline_attrs, parse_all_links, parse_tags,
    split_frontmatter,
};
use crate::types::{BlockId, Heading, InlineAttr, Link, Tag};
use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;
use std::path::{Path, PathBuf};

/// Represents a note in the vault.
#[derive(Debug, Clone)]
pub struct Note {
    /// Path relative to vault root (e.g., "proj/My Project.md").
    pub path: PathBuf,

    /// Raw content of the note.
    pub content: String,
}

impl Note {
    /// Create a new note from path and content.
    pub fn new(path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
        }
    }

    /// Load a note from disk.
    pub fn load(vault_root: &Path, relative_path: &Path) -> Result<Self> {
        let full_path = vault_root.join(relative_path);
        let content = std::fs::read_to_string(&full_path)?;
        Ok(Self {
            path: relative_path.to_path_buf(),
            content,
        })
    }

    /// Save the note to disk.
    pub fn save(&self, vault_root: &Path) -> Result<()> {
        let full_path = vault_root.join(&self.path);

        // Ensure parent directory exists
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&full_path, &self.content)?;
        Ok(())
    }

    /// Get the note name (filename without .md extension).
    pub fn name(&self) -> &str {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    }

    /// Get the parent folder path.
    pub fn folder(&self) -> Option<&Path> {
        self.path.parent()
    }

    /// Check if the note has frontmatter.
    pub fn has_frontmatter(&self) -> bool {
        parser::extract_frontmatter(&self.content).is_some()
    }

    /// Get raw frontmatter YAML string.
    pub fn frontmatter_raw(&self) -> Option<&str> {
        parser::extract_frontmatter(&self.content)
    }

    /// Parse frontmatter as YAML value.
    pub fn frontmatter(&self) -> Result<Option<YamlValue>> {
        parser::parse_frontmatter_with_path(&self.content, &self.path)
    }

    /// Get content without frontmatter.
    pub fn body(&self) -> &str {
        split_frontmatter(&self.content).content
    }

    /// Get content including frontmatter.
    pub fn full_content(&self) -> &str {
        &self.content
    }

    /// Get the line number where the body starts.
    pub fn body_start_line(&self) -> usize {
        split_frontmatter(&self.content).content_start_line
    }

    /// Parse all links in the note.
    pub fn links(&self) -> Vec<Link> {
        parse_all_links(&self.content)
    }

    /// Parse all tags in the note.
    pub fn tags(&self) -> Vec<Tag> {
        parse_tags(&self.content)
    }

    /// Parse all block IDs in the note.
    pub fn block_ids(&self) -> Vec<BlockId> {
        parse_block_ids(&self.content)
    }

    /// Parse all headings in the note.
    pub fn headings(&self) -> Vec<Heading> {
        parse_headings(&self.content)
    }

    /// Parse all inline attributes in the note.
    pub fn inline_attrs(&self) -> Vec<InlineAttr> {
        parse_inline_attrs(&self.content)
    }

    /// Update the note's frontmatter.
    pub fn with_frontmatter(&self, new_frontmatter: &YamlValue) -> Result<Self> {
        let new_content = parser::update_frontmatter(&self.content, new_frontmatter)?;
        Ok(Self {
            path: self.path.clone(),
            content: new_content,
        })
    }

    /// Update the note's body (content below frontmatter).
    pub fn with_body(&self, new_body: &str) -> Self {
        let split = split_frontmatter(&self.content);

        let new_content = if let Some(yaml) = split.yaml {
            format!("---\n{}\n---\n{}", yaml, new_body)
        } else {
            new_body.to_string()
        };

        Self {
            path: self.path.clone(),
            content: new_content,
        }
    }

    /// Append content to the note.
    pub fn append(&self, content: &str) -> Self {
        Self {
            path: self.path.clone(),
            content: format!("{}{}", self.content, content),
        }
    }

    /// Prepend content after frontmatter.
    pub fn prepend(&self, content: &str) -> Self {
        let split = split_frontmatter(&self.content);

        let new_content = if let Some(yaml) = split.yaml {
            format!("---\n{}\n---\n{}{}", yaml, content, split.content)
        } else {
            format!("{}{}", content, self.content)
        };

        Self {
            path: self.path.clone(),
            content: new_content,
        }
    }

    /// Replace content with the new full content.
    pub fn with_content(&self, new_content: impl Into<String>) -> Self {
        Self {
            path: self.path.clone(),
            content: new_content.into(),
        }
    }

    /// Change the task checkbox symbol on a specific line.
    ///
    /// `line` is 1-indexed (consistent with `Task.location.line`).
    /// The target line must match the pattern `- [.] ...` (with optional leading whitespace).
    /// Returns a new `Note` with the modified content.
    pub fn set_task_symbol(&self, line: usize, new_symbol: char) -> Result<Self> {
        if line == 0 {
            return Err(crate::error::VaultError::Other(
                "Line number must be 1-indexed (got 0)".to_string(),
            ));
        }

        let lines: Vec<&str> = self.content.lines().collect();

        if line > lines.len() {
            return Err(crate::error::VaultError::Other(format!(
                "Line {} is out of range (note has {} lines)",
                line,
                lines.len()
            )));
        }

        let target_line = lines[line - 1];

        // Match: optional whitespace, list marker, ` [`, any single char, `]`, rest
        let re = regex::Regex::new(r"^(\s*(?:[-*+]|\d+\.) \[).\](.*)$").unwrap();
        if !re.is_match(target_line) {
            return Err(crate::error::VaultError::Other(format!(
                "Line {} is not a task: {:?}",
                line, target_line
            )));
        }

        let new_line = re.replace(target_line, |caps: &regex::Captures| {
            format!("{}{}]{}", &caps[1], new_symbol, &caps[2])
        });

        let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        new_lines[line - 1] = new_line.to_string();

        // Preserve trailing newline if original content had one
        let mut new_content = new_lines.join("\n");
        if self.content.ends_with('\n') {
            new_content.push('\n');
        }

        Ok(Self {
            path: self.path.clone(),
            content: new_content,
        })
    }
}

/// Output representation of a note for CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteInfo {
    pub path: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

impl NoteInfo {
    pub fn from_path(vault_root: &Path, relative_path: &Path) -> Result<Self> {
        let full_path = vault_root.join(relative_path);
        let metadata = std::fs::metadata(&full_path)?;

        let modified = metadata
            .modified()
            .ok()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());

        let created = metadata
            .created()
            .ok()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());

        let name = relative_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        Ok(Self {
            path: relative_path.to_string_lossy().to_string(),
            name,
            modified,
            created,
            size_bytes: Some(metadata.len()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_name() {
        let note = Note::new("proj/My Project.md", "content");
        assert_eq!(note.name(), "My Project");
    }

    #[test]
    fn test_note_folder() {
        let note = Note::new("proj/sub/Note.md", "content");
        assert_eq!(note.folder(), Some(Path::new("proj/sub")));
    }

    #[test]
    fn test_note_body_no_frontmatter() {
        let note = Note::new("note.md", "Just content");
        assert_eq!(note.body(), "Just content");
        assert!(!note.has_frontmatter());
    }

    #[test]
    fn test_note_body_with_frontmatter() {
        let content = "---\ntitle: Test\n---\n\nBody content";
        let note = Note::new("note.md", content);
        assert!(note.has_frontmatter());
        assert_eq!(note.body(), "\nBody content");
    }

    #[test]
    fn test_note_frontmatter() {
        let content = "---\ntitle: Test\ntags:\n  - rust\n---\n\nBody";
        let note = Note::new("note.md", content);
        let fm = note.frontmatter().unwrap().unwrap();
        assert_eq!(fm["title"].as_str(), Some("Test"));
    }

    #[test]
    fn test_note_append() {
        let note = Note::new("note.md", "Hello");
        let updated = note.append(" World");
        assert_eq!(updated.content, "Hello World");
    }

    #[test]
    fn test_note_prepend_no_frontmatter() {
        let note = Note::new("note.md", "World");
        let updated = note.prepend("Hello ");
        assert_eq!(updated.content, "Hello World");
    }

    #[test]
    fn test_note_prepend_with_frontmatter() {
        let content = "---\ntitle: Test\n---\nWorld";
        let note = Note::new("note.md", content);
        let updated = note.prepend("Hello ");
        assert!(updated.content.contains("---\ntitle: Test\n---\nHello World"));
    }

    #[test]
    fn test_note_with_body() {
        let content = "---\ntitle: Test\n---\n\nOld body";
        let note = Note::new("note.md", content);
        let updated = note.with_body("New body");
        assert!(updated.content.contains("title: Test"));
        assert!(updated.content.contains("New body"));
        assert!(!updated.content.contains("Old body"));
    }

    #[test]
    fn test_note_links() {
        let note = Note::new("note.md", "See [[Other Note]] and [[Another|alias]].");
        let links = note.links();
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_note_tags() {
        let note = Note::new("note.md", "Tags: #rust #cli");
        let tags = note.tags();
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_set_task_symbol_check() {
        let content = "---\ntitle: Test\n---\n\n- [ ] My task\n- [ ] Another task\n";
        let note = Note::new("note.md", content);
        let updated = note.set_task_symbol(5, 'x').unwrap();
        assert!(updated.content.contains("- [x] My task"));
        assert!(updated.content.contains("- [ ] Another task"));
    }

    #[test]
    fn test_set_task_symbol_uncheck() {
        let content = "- [x] Done task\n- [ ] Open task\n";
        let note = Note::new("note.md", content);
        let updated = note.set_task_symbol(1, ' ').unwrap();
        assert!(updated.content.contains("- [ ] Done task"));
        assert!(updated.content.contains("- [ ] Open task"));
    }

    #[test]
    fn test_set_task_symbol_preserves_indentation() {
        let content = "- [ ] Top\n  - [ ] Indented task\n    - [x] Deeply indented\n";
        let note = Note::new("note.md", content);
        let updated = note.set_task_symbol(2, 'x').unwrap();
        assert!(updated.content.contains("  - [x] Indented task"));
        // Others unchanged
        assert!(updated.content.contains("- [ ] Top"));
        assert!(updated.content.contains("    - [x] Deeply indented"));
    }

    #[test]
    fn test_set_task_symbol_error_non_task_line() {
        let content = "# Heading\n- [ ] Task\n";
        let note = Note::new("note.md", content);
        let result = note.set_task_symbol(1, 'x');
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a task"));
    }

    #[test]
    fn test_set_task_symbol_error_out_of_range() {
        let content = "- [ ] Only task\n";
        let note = Note::new("note.md", content);
        let result = note.set_task_symbol(5, 'x');
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of range"));
    }

    #[test]
    fn test_set_task_symbol_error_zero_line() {
        let note = Note::new("note.md", "- [ ] Task\n");
        let result = note.set_task_symbol(0, 'x');
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("1-indexed"));
    }
}
