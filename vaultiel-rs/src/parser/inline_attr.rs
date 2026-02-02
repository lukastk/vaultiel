//! Inline attribute parsing ([key::value]).

use crate::parser::code_block::{find_code_block_ranges, is_in_code_block};
use crate::types::InlineAttr;
use regex::Regex;
use std::sync::LazyLock;

// Inline attribute pattern: [key::value]
// Key can contain word characters and hyphens
// Value can contain most characters except ], but can include ]] for wikilinks
static INLINE_ATTR: LazyLock<Regex> = LazyLock::new(|| {
    // Value pattern: [^\]]* matches non-] chars, then (?:\]\][^\]]*)* allows ]] followed by non-] chars
    // This handles wikilinks like [[Note]] inside the value
    Regex::new(r"\[([\w-]+)::([^\]]*(?:\]\][^\]]*)*)\]").unwrap()
});

/// Parse all inline attributes from content.
pub fn parse_inline_attrs(content: &str) -> Vec<InlineAttr> {
    let code_ranges = find_code_block_ranges(content);
    let mut attrs = Vec::new();

    for cap in INLINE_ATTR.captures_iter(content) {
        let full_match = cap.get(0).unwrap();
        let start = full_match.start();
        let end = full_match.end();

        // Skip if inside code block
        if is_in_code_block(start, &code_ranges) {
            continue;
        }

        let key = cap.get(1).unwrap().as_str().to_string();
        let value = cap.get(2).unwrap().as_str().trim().to_string();

        // Calculate line number
        let line = content[..start].matches('\n').count() + 1;

        // Calculate column
        let line_start = content[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let start_col = start - line_start;
        let end_col = end - line_start;

        attrs.push(InlineAttr {
            key,
            value,
            line,
            start_col,
            end_col,
        });
    }

    attrs
}

/// Collect inline attributes into a map.
pub fn collect_inline_attrs(
    attrs: &[InlineAttr],
) -> std::collections::HashMap<String, Vec<&InlineAttr>> {
    let mut map: std::collections::HashMap<String, Vec<&InlineAttr>> =
        std::collections::HashMap::new();

    for attr in attrs {
        map.entry(attr.key.clone()).or_default().push(attr);
    }

    map
}

/// Format an inline attribute as a string.
pub fn format_inline_attr(key: &str, value: &str) -> String {
    format!("[{}::{}]", key, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_inline_attr() {
        let content = "Some text [status::active] here.";
        let attrs = parse_inline_attrs(content);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].key, "status");
        assert_eq!(attrs[0].value, "active");
    }

    #[test]
    fn test_inline_attr_with_link() {
        let content = "[parent::[[Other Note]]]";
        let attrs = parse_inline_attrs(content);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].key, "parent");
        assert_eq!(attrs[0].value, "[[Other Note]]");
    }

    #[test]
    fn test_multiple_inline_attrs() {
        let content = "[key1::value1] some text [key2::value2]";
        let attrs = parse_inline_attrs(content);
        assert_eq!(attrs.len(), 2);
    }

    #[test]
    fn test_inline_attr_with_hyphen_key() {
        let content = "[my-key::my value]";
        let attrs = parse_inline_attrs(content);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].key, "my-key");
        assert_eq!(attrs[0].value, "my value");
    }

    #[test]
    fn test_inline_attr_value_with_spaces() {
        let content = "[description::This is a longer value with spaces]";
        let attrs = parse_inline_attrs(content);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].value, "This is a longer value with spaces");
    }

    #[test]
    fn test_inline_attr_in_code_block_skipped() {
        let content = "[real::attr]\n\n```\n[fake::attr]\n```";
        let attrs = parse_inline_attrs(content);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].key, "real");
    }

    #[test]
    fn test_inline_attr_in_inline_code_skipped() {
        let content = "[real::attr] and `[fake::attr]` here.";
        let attrs = parse_inline_attrs(content);
        assert_eq!(attrs.len(), 1);
    }

    #[test]
    fn test_inline_attr_line_numbers() {
        let content = "[attr1::val1]\nsome text\n[attr2::val2]";
        let attrs = parse_inline_attrs(content);
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].line, 1);
        assert_eq!(attrs[1].line, 3);
    }

    #[test]
    fn test_collect_inline_attrs() {
        let content = "[tag::a] [tag::b] [other::c]";
        let attrs = parse_inline_attrs(content);
        let collected = collect_inline_attrs(&attrs);

        assert_eq!(collected.get("tag").map(|v| v.len()), Some(2));
        assert_eq!(collected.get("other").map(|v| v.len()), Some(1));
    }

    #[test]
    fn test_format_inline_attr() {
        assert_eq!(format_inline_attr("key", "value"), "[key::value]");
        assert_eq!(
            format_inline_attr("parent", "[[Note]]"),
            "[parent::[[Note]]]"
        );
    }

    #[test]
    fn test_not_dataview_style() {
        // Dataview uses key:: without brackets - we should NOT match these
        let content = "status:: active";
        let attrs = parse_inline_attrs(content);
        assert_eq!(attrs.len(), 0);
    }
}
