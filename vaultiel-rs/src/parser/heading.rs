//! Heading parsing and slug generation.

use crate::parser::code_block::{find_code_block_ranges, is_line_in_fenced_code_block};
use crate::types::Heading;
use regex::Regex;
use std::sync::LazyLock;
use unicode_normalization::UnicodeNormalization;

// ATX-style heading: # Heading, ## Heading, etc.
static HEADING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(#{1,6})\s+(.+?)(?:\s+\^[a-zA-Z0-9_-]+)?\s*$").unwrap()
});

/// Parse all headings from content.
pub fn parse_headings(content: &str) -> Vec<Heading> {
    let code_ranges = find_code_block_ranges(content);
    let mut headings = Vec::new();
    let mut slug_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1; // 1-indexed

        // Skip lines inside fenced code blocks
        if is_line_in_fenced_code_block(line_num, &code_ranges) {
            continue;
        }

        if let Some(cap) = HEADING.captures(line) {
            let level = cap.get(1).unwrap().as_str().len() as u8;
            let text = cap.get(2).unwrap().as_str().trim().to_string();

            // Generate slug
            let base_slug = slugify(&text);
            let slug = make_unique_slug(&base_slug, &mut slug_counts);

            headings.push(Heading {
                text,
                level,
                line: line_num,
                slug,
            });
        }
    }

    headings
}

/// Generate a URL-safe slug from heading text.
///
/// Follows Obsidian's algorithm:
/// - Normalize unicode
/// - Convert to lowercase
/// - Replace spaces with hyphens
/// - Remove special characters (keep alphanumeric, hyphens, underscores)
/// - Collapse multiple hyphens
pub fn slugify(text: &str) -> String {
    let normalized: String = text.nfc().collect();

    let mut slug = String::new();
    let mut last_was_hyphen = false;

    for c in normalized.chars() {
        if c.is_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            last_was_hyphen = false;
        } else if c == '-' || c == '_' {
            if !last_was_hyphen && !slug.is_empty() {
                slug.push(c);
                last_was_hyphen = c == '-';
            }
        } else if c.is_whitespace() {
            if !last_was_hyphen && !slug.is_empty() {
                slug.push('-');
                last_was_hyphen = true;
            }
        }
        // Other characters are stripped
    }

    // Remove trailing hyphen
    while slug.ends_with('-') {
        slug.pop();
    }

    slug
}

/// Make a slug unique by appending a number if necessary.
fn make_unique_slug(
    base_slug: &str,
    counts: &mut std::collections::HashMap<String, usize>,
) -> String {
    let count = counts.entry(base_slug.to_string()).or_insert(0);
    *count += 1;

    if *count == 1 {
        base_slug.to_string()
    } else {
        format!("{}-{}", base_slug, *count - 1)
    }
}

/// Find a heading by its slug.
pub fn find_heading_by_slug<'a>(headings: &'a [Heading], slug: &str) -> Option<&'a Heading> {
    headings.iter().find(|h| h.slug == slug)
}

/// Find a heading by its text (case-insensitive).
pub fn find_heading_by_text<'a>(headings: &'a [Heading], text: &str) -> Option<&'a Heading> {
    let lower_text = text.to_lowercase();
    headings.iter().find(|h| h.text.to_lowercase() == lower_text)
}

/// Get headings at or below a certain level.
pub fn filter_headings_by_level(headings: &[Heading], min: u8, max: u8) -> Vec<&Heading> {
    headings
        .iter()
        .filter(|h| h.level >= min && h.level <= max)
        .collect()
}

/// Build a nested heading structure.
#[derive(Debug, Clone)]
pub struct HeadingNode {
    pub heading: Heading,
    pub children: Vec<HeadingNode>,
}

/// Build a nested tree structure from flat headings.
pub fn build_heading_tree(headings: &[Heading]) -> Vec<HeadingNode> {
    let mut root: Vec<HeadingNode> = Vec::new();
    let mut stack: Vec<(u8, usize)> = Vec::new(); // (level, index in parent's children)

    for heading in headings {
        let node = HeadingNode {
            heading: heading.clone(),
            children: Vec::new(),
        };

        // Pop stack until we find a parent with a lower level
        while let Some(&(level, _)) = stack.last() {
            if level >= heading.level {
                stack.pop();
            } else {
                break;
            }
        }

        if stack.is_empty() {
            // This is a root-level heading
            root.push(node);
            stack.push((heading.level, root.len() - 1));
        } else {
            // Find the parent and add as child
            let mut current = &mut root;
            for (i, &(_, idx)) in stack.iter().enumerate() {
                if i == stack.len() - 1 {
                    current[idx].children.push(node.clone());
                    let new_idx = current[idx].children.len() - 1;
                    stack.push((heading.level, new_idx));
                    break;
                } else {
                    current = &mut current[idx].children;
                }
            }
        }
    }

    root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_heading() {
        let content = "# Heading 1\n\nSome text\n\n## Heading 2";
        let headings = parse_headings(content);
        assert_eq!(headings.len(), 2);
        assert_eq!(headings[0].text, "Heading 1");
        assert_eq!(headings[0].level, 1);
        assert_eq!(headings[0].line, 1);
        assert_eq!(headings[1].text, "Heading 2");
        assert_eq!(headings[1].level, 2);
        assert_eq!(headings[1].line, 5);
    }

    #[test]
    fn test_heading_with_block_id() {
        let content = "# Heading ^block-id";
        let headings = parse_headings(content);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text, "Heading");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("API Design"), "api-design");
        assert_eq!(slugify("What's New?"), "whats-new");
        assert_eq!(slugify("C++ Programming"), "c-programming");
        assert_eq!(slugify("  Spaced  "), "spaced");
        assert_eq!(slugify("Under_score"), "under_score");
        assert_eq!(slugify("Multiple   Spaces"), "multiple-spaces");
    }

    #[test]
    fn test_duplicate_headings_unique_slugs() {
        let content = "# Test\n\n## Test\n\n### Test";
        let headings = parse_headings(content);
        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0].slug, "test");
        assert_eq!(headings[1].slug, "test-1");
        assert_eq!(headings[2].slug, "test-2");
    }

    #[test]
    fn test_heading_in_code_block_skipped() {
        let content = "# Real Heading\n\n```\n# Not a heading\n```";
        let headings = parse_headings(content);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text, "Real Heading");
    }

    #[test]
    fn test_all_heading_levels() {
        let content = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6";
        let headings = parse_headings(content);
        assert_eq!(headings.len(), 6);
        for (i, h) in headings.iter().enumerate() {
            assert_eq!(h.level, (i + 1) as u8);
        }
    }

    #[test]
    fn test_find_heading_by_slug() {
        let content = "# First\n## Second\n### Third";
        let headings = parse_headings(content);
        let found = find_heading_by_slug(&headings, "second");
        assert!(found.is_some());
        assert_eq!(found.unwrap().text, "Second");
    }

    #[test]
    fn test_find_heading_by_text() {
        let content = "# First\n## Second\n### Third";
        let headings = parse_headings(content);
        let found = find_heading_by_text(&headings, "SECOND");
        assert!(found.is_some());
        assert_eq!(found.unwrap().text, "Second");
    }

    #[test]
    fn test_filter_headings_by_level() {
        let content = "# H1\n## H2\n### H3\n## H2b";
        let headings = parse_headings(content);
        let filtered = filter_headings_by_level(&headings, 2, 2);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_heading_not_at_line_start() {
        let content = "text # not a heading\n# Real heading";
        let headings = parse_headings(content);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text, "Real heading");
    }
}
