//! YAML frontmatter parsing.

use crate::error::{Result, VaultError};
use serde_yaml::Value;
use std::path::Path;

/// Frontmatter extraction result.
#[derive(Debug, Clone)]
pub struct FrontmatterSplit<'a> {
    /// The raw YAML string (without delimiters).
    pub yaml: Option<&'a str>,
    /// The content after the frontmatter.
    pub content: &'a str,
    /// Line number where content starts (1-indexed).
    pub content_start_line: usize,
}

/// Split content into frontmatter and body.
///
/// Returns the raw YAML string (if present) and the remaining content.
pub fn split_frontmatter(content: &str) -> FrontmatterSplit<'_> {
    // Frontmatter must start at the very beginning with ---
    if !content.starts_with("---") {
        return FrontmatterSplit {
            yaml: None,
            content,
            content_start_line: 1,
        };
    }

    // Find the closing ---
    let after_first_delimiter = &content[3..];

    // Skip the newline after the opening ---
    let yaml_start = if after_first_delimiter.starts_with('\n') {
        4
    } else if after_first_delimiter.starts_with("\r\n") {
        5
    } else {
        // No newline after ---, not valid frontmatter
        return FrontmatterSplit {
            yaml: None,
            content,
            content_start_line: 1,
        };
    };

    // Find the closing delimiter
    // It must be on its own line: \n---\n or \n--- (at end of file)
    let search_start = yaml_start;
    let remaining = &content[search_start..];

    let closing_pos = remaining
        .find("\n---\n")
        .or_else(|| remaining.find("\n---\r\n"))
        .or_else(|| {
            // Handle --- at end of file
            if remaining.ends_with("\n---") {
                Some(remaining.len() - 4) // position of the \n, not the ---
            } else {
                None
            }
        });

    match closing_pos {
        Some(pos) => {
            let yaml_end = search_start + pos;
            let yaml = &content[yaml_start..yaml_end];

            // Calculate where content starts (after the closing --- and newline)
            let content_start = yaml_end + 4; // \n---
            let content_after = if content_start < content.len() {
                // Skip the newline after ---
                let rest = &content[content_start..];
                if rest.starts_with('\n') {
                    &content[content_start + 1..]
                } else if rest.starts_with("\r\n") {
                    &content[content_start + 2..]
                } else {
                    rest
                }
            } else {
                ""
            };

            // Count lines in frontmatter to determine content start line
            let frontmatter_lines = content[..yaml_end + 4].matches('\n').count();
            let content_start_line = frontmatter_lines + 2; // +1 for 1-indexing, +1 for line after ---

            FrontmatterSplit {
                yaml: Some(yaml),
                content: content_after,
                content_start_line,
            }
        }
        None => {
            // No closing delimiter found
            FrontmatterSplit {
                yaml: None,
                content,
                content_start_line: 1,
            }
        }
    }
}

/// Extract frontmatter as a raw YAML string.
pub fn extract_frontmatter(content: &str) -> Option<&str> {
    split_frontmatter(content).yaml
}

/// Parse frontmatter into a serde_yaml::Value.
pub fn parse_frontmatter(content: &str) -> Result<Option<Value>> {
    match extract_frontmatter(content) {
        Some(yaml) => {
            let value: Value = serde_yaml::from_str(yaml).map_err(|e| {
                VaultError::InvalidFrontmatter {
                    path: Path::new("<unknown>").to_path_buf(),
                    message: e.to_string(),
                }
            })?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

/// Parse frontmatter with path context for error messages.
pub fn parse_frontmatter_with_path(content: &str, path: &Path) -> Result<Option<Value>> {
    match extract_frontmatter(content) {
        Some(yaml) => {
            let value: Value = serde_yaml::from_str(yaml).map_err(|e| {
                VaultError::InvalidFrontmatter {
                    path: path.to_path_buf(),
                    message: e.to_string(),
                }
            })?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

/// Serialize a Value back to YAML frontmatter format (with delimiters).
pub fn serialize_frontmatter(value: &Value) -> Result<String> {
    let yaml = serde_yaml::to_string(value)?;
    Ok(format!("---\n{}---\n", yaml))
}

/// Update content with new frontmatter.
pub fn update_frontmatter(content: &str, new_frontmatter: &Value) -> Result<String> {
    let split = split_frontmatter(content);
    let fm_str = serialize_frontmatter(new_frontmatter)?;

    Ok(format!("{}{}", fm_str, split.content))
}

/// Remove frontmatter from content.
pub fn remove_all_frontmatter(content: &str) -> &str {
    split_frontmatter(content).content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_no_frontmatter() {
        let content = "Just some content";
        let split = split_frontmatter(content);
        assert!(split.yaml.is_none());
        assert_eq!(split.content, "Just some content");
        assert_eq!(split.content_start_line, 1);
    }

    #[test]
    fn test_split_with_frontmatter() {
        let content = "---\ntitle: Test\ntags: [a, b]\n---\n\nContent here";
        let split = split_frontmatter(content);
        assert_eq!(split.yaml, Some("title: Test\ntags: [a, b]"));
        assert_eq!(split.content, "\nContent here");
        assert_eq!(split.content_start_line, 5);
    }

    #[test]
    fn test_split_frontmatter_at_eof() {
        let content = "---\ntitle: Test\n---";
        let split = split_frontmatter(content);
        assert_eq!(split.yaml, Some("title: Test"));
        assert_eq!(split.content, "");
    }

    #[test]
    fn test_parse_frontmatter() {
        let content = "---\ntitle: My Note\ntags:\n  - rust\n  - cli\n---\n\nContent";
        let value = parse_frontmatter(content).unwrap().unwrap();

        assert_eq!(value["title"].as_str(), Some("My Note"));
        let tags = value["tags"].as_sequence().unwrap();
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_invalid_frontmatter() {
        let content = "---\ninvalid: yaml: syntax:\n---\nContent";
        let result = parse_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_closing_delimiter() {
        let content = "---\ntitle: Test\n\nContent without closing";
        let split = split_frontmatter(content);
        // Should treat entire thing as content since no valid closing
        assert!(split.yaml.is_none());
    }

    #[test]
    fn test_triple_dash_in_content() {
        let content = "---\ntitle: Test\n---\n\n---\n\nThis has triple dashes in content";
        let split = split_frontmatter(content);
        assert_eq!(split.yaml, Some("title: Test"));
        assert!(split.content.contains("---"));
    }

    #[test]
    fn test_update_frontmatter() {
        let content = "---\ntitle: Old\n---\n\nContent";
        let mut value = serde_yaml::from_str::<Value>("title: New").unwrap();
        value["tags"] = Value::Sequence(vec![Value::String("rust".to_string())]);

        let updated = update_frontmatter(content, &value).unwrap();
        assert!(updated.contains("title: New"));
        assert!(updated.contains("tags:"));
        assert!(updated.contains("Content"));
    }
}
