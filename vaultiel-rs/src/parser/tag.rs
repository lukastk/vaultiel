//! Tag parsing (#tag and #tag/subtag).

use crate::parser::code_block::{find_code_block_ranges, is_in_code_block};
use crate::types::Tag;
use regex::Regex;
use std::sync::LazyLock;

// Tag pattern: # followed by word characters and optional /subtags
// Must not be preceded by & (HTML entity) or another word character
// Cannot be just numbers (like #123)
// Note: Rust regex doesn't support lookahead/lookbehind, so we:
// 1. Match with preceding boundary and capture tag
// 2. Validate following char in post-processing
static TAG: LazyLock<Regex> = LazyLock::new(|| {
    // Match either start of string/line OR a non-word, non-& char before the #
    // The tag itself must start with a letter or underscore, not a digit
    Regex::new(r"(?:^|[^\w&])#([a-zA-Z_][\w/-]*)").unwrap()
});

/// Parse all tags from content.
pub fn parse_tags(content: &str) -> Vec<Tag> {
    let code_ranges = find_code_block_ranges(content);
    let mut tags = Vec::new();

    for cap in TAG.captures_iter(content) {
        // Group 1 is the tag name (without #)
        let tag_match = cap.get(1).unwrap();

        // The # is just before the tag name
        let start = tag_match.start() - 1;
        let end = tag_match.end();

        // Check that tag isn't followed by word char or / (simulate negative lookahead)
        if end < content.len() {
            let next_char = content[end..].chars().next().unwrap();
            if next_char.is_alphanumeric() || next_char == '_' || next_char == '/' {
                continue;
            }
        }

        // Skip if inside code block
        if is_in_code_block(start, &code_ranges) {
            continue;
        }

        // Skip if inside a wikilink (between [[ and ]])
        if is_in_wikilink(content, start) {
            continue;
        }

        let tag_name = format!("#{}", tag_match.as_str());

        // Calculate line number
        let line = content[..start].matches('\n').count() + 1;

        // Calculate column
        let line_start = content[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let start_col = start - line_start;
        let end_col = end - line_start;

        tags.push(Tag {
            name: tag_name,
            line,
            start_col,
            end_col,
        });
    }

    tags
}

/// Check if a position is inside a wikilink.
fn is_in_wikilink(content: &str, pos: usize) -> bool {
    let before = &content[..pos];
    let after = &content[pos..];

    // Find the last [[ before this position
    let last_open = before.rfind("[[");
    // Find the last ]] before this position
    let last_close = before.rfind("]]");

    match (last_open, last_close) {
        (Some(open), Some(close)) => {
            // We're inside if [[ is after ]] (or they're nested somehow)
            if open > close {
                // Check if there's a ]] after our position
                after.find("]]").is_some()
            } else {
                false
            }
        }
        (Some(_), None) => {
            // There's an open but no close before us, check for close after
            after.find("]]").is_some()
        }
        _ => false,
    }
}

/// Get unique tags from a list, preserving order of first occurrence.
pub fn unique_tags(tags: &[Tag]) -> Vec<&Tag> {
    let mut seen = std::collections::HashSet::new();
    tags.iter()
        .filter(|tag| seen.insert(&tag.name))
        .collect()
}

/// Group tags by their root (first segment).
pub fn group_tags_by_root(tags: &[Tag]) -> std::collections::HashMap<String, Vec<&Tag>> {
    let mut groups: std::collections::HashMap<String, Vec<&Tag>> = std::collections::HashMap::new();

    for tag in tags {
        let root = tag
            .without_hash()
            .split('/')
            .next()
            .unwrap_or(tag.without_hash());
        let root_tag = format!("#{}", root);
        groups.entry(root_tag).or_default().push(tag);
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tag() {
        let content = "Some text #rust here.";
        let tags = parse_tags(content);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "#rust");
    }

    #[test]
    fn test_nested_tag() {
        let content = "#tray/autonomy/urgent";
        let tags = parse_tags(content);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "#tray/autonomy/urgent");
    }

    #[test]
    fn test_multiple_tags() {
        let content = "Tags: #rust #cli #obsidian";
        let tags = parse_tags(content);
        assert_eq!(tags.len(), 3);
    }

    #[test]
    fn test_tag_with_hyphen() {
        let content = "#my-tag";
        let tags = parse_tags(content);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "#my-tag");
    }

    #[test]
    fn test_tag_with_underscore() {
        let content = "#my_tag";
        let tags = parse_tags(content);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "#my_tag");
    }

    #[test]
    fn test_tag_starting_with_underscore() {
        let content = "#_private";
        let tags = parse_tags(content);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "#_private");
    }

    #[test]
    fn test_numeric_not_a_tag() {
        let content = "Issue #123 is fixed.";
        let tags = parse_tags(content);
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_html_entity_not_a_tag() {
        let content = "Use &nbsp; for space.";
        let tags = parse_tags(content);
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_heading_not_a_tag() {
        let content = "# Heading\n## Subheading";
        let tags = parse_tags(content);
        // Headings start with # but have space after, so they're not tags
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_tag_in_code_block_skipped() {
        let content = "Real #tag\n\n```\n#fake-tag\n```";
        let tags = parse_tags(content);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "#tag");
    }

    #[test]
    fn test_tag_in_inline_code_skipped() {
        let content = "Real #tag and `#fake-tag` here.";
        let tags = parse_tags(content);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "#tag");
    }

    #[test]
    fn test_tag_line_numbers() {
        let content = "#tag1\nsome text\n#tag2";
        let tags = parse_tags(content);
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].line, 1);
        assert_eq!(tags[1].line, 3);
    }

    #[test]
    fn test_tag_in_wikilink_skipped() {
        let content = "Real #tag and [[Note#heading]] here.";
        let tags = parse_tags(content);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "#tag");
    }

    #[test]
    fn test_unique_tags() {
        let content = "#rust #cli #rust #obsidian #cli";
        let tags = parse_tags(content);
        let unique = unique_tags(&tags);
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn test_group_tags_by_root() {
        let content = "#tray #tray/work #tray/personal #rust";
        let tags = parse_tags(content);
        let groups = group_tags_by_root(&tags);

        assert_eq!(groups.get("#tray").map(|v| v.len()), Some(3));
        assert_eq!(groups.get("#rust").map(|v| v.len()), Some(1));
    }
}
