//! Inline property parsing ([key::value]).

use crate::parser::code_block::{find_code_block_ranges, is_in_code_block};
use crate::types::InlineProperty;
use regex::Regex;
use std::sync::LazyLock;

// Inline property pattern: [key::value]
// Key can contain word characters and hyphens
// Value can contain most characters except ], but can include ]] for wikilinks
static INLINE_PROPERTY: LazyLock<Regex> = LazyLock::new(|| {
    // Value pattern: [^\]]* matches non-] chars, then (?:\]\][^\]]*)* allows ]] followed by non-] chars
    // This handles wikilinks like [[Note]] inside the value
    Regex::new(r"\[([\w-]+)::([^\]]*(?:\]\][^\]]*)*)\]").unwrap()
});

/// Parse all inline properties from content.
pub fn parse_inline_properties(content: &str) -> Vec<InlineProperty> {
    let code_ranges = find_code_block_ranges(content);
    let mut props = Vec::new();

    for cap in INLINE_PROPERTY.captures_iter(content) {
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

        props.push(InlineProperty {
            key,
            value,
            line,
            start_col,
            end_col,
        });
    }

    props
}

/// Collect inline properties into a map.
pub fn collect_inline_properties(
    props: &[InlineProperty],
) -> std::collections::HashMap<String, Vec<&InlineProperty>> {
    let mut map: std::collections::HashMap<String, Vec<&InlineProperty>> =
        std::collections::HashMap::new();

    for prop in props {
        map.entry(prop.key.clone()).or_default().push(prop);
    }

    map
}

/// Format an inline property as a string.
pub fn format_inline_property(key: &str, value: &str) -> String {
    format!("[{}::{}]", key, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_inline_property() {
        let content = "Some text [status::active] here.";
        let props = parse_inline_properties(content);
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].key, "status");
        assert_eq!(props[0].value, "active");
    }

    #[test]
    fn test_inline_property_with_link() {
        let content = "[parent::[[Other Note]]]";
        let props = parse_inline_properties(content);
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].key, "parent");
        assert_eq!(props[0].value, "[[Other Note]]");
    }

    #[test]
    fn test_multiple_inline_properties() {
        let content = "[key1::value1] some text [key2::value2]";
        let props = parse_inline_properties(content);
        assert_eq!(props.len(), 2);
    }

    #[test]
    fn test_inline_property_with_hyphen_key() {
        let content = "[my-key::my value]";
        let props = parse_inline_properties(content);
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].key, "my-key");
        assert_eq!(props[0].value, "my value");
    }

    #[test]
    fn test_inline_property_value_with_spaces() {
        let content = "[description::This is a longer value with spaces]";
        let props = parse_inline_properties(content);
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].value, "This is a longer value with spaces");
    }

    #[test]
    fn test_inline_property_in_code_block_skipped() {
        let content = "[real::prop]\n\n```\n[fake::prop]\n```";
        let props = parse_inline_properties(content);
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].key, "real");
    }

    #[test]
    fn test_inline_property_in_inline_code_skipped() {
        let content = "[real::prop] and `[fake::prop]` here.";
        let props = parse_inline_properties(content);
        assert_eq!(props.len(), 1);
    }

    #[test]
    fn test_inline_property_line_numbers() {
        let content = "[prop1::val1]\nsome text\n[prop2::val2]";
        let props = parse_inline_properties(content);
        assert_eq!(props.len(), 2);
        assert_eq!(props[0].line, 1);
        assert_eq!(props[1].line, 3);
    }

    #[test]
    fn test_collect_inline_properties() {
        let content = "[tag::a] [tag::b] [other::c]";
        let props = parse_inline_properties(content);
        let collected = collect_inline_properties(&props);

        assert_eq!(collected.get("tag").map(|v| v.len()), Some(2));
        assert_eq!(collected.get("other").map(|v| v.len()), Some(1));
    }

    #[test]
    fn test_format_inline_property() {
        assert_eq!(format_inline_property("key", "value"), "[key::value]");
        assert_eq!(
            format_inline_property("parent", "[[Note]]"),
            "[parent::[[Note]]]"
        );
    }

    #[test]
    fn test_not_dataview_style() {
        // Dataview uses key:: without brackets - we should NOT match these
        let content = "status:: active";
        let props = parse_inline_properties(content);
        assert_eq!(props.len(), 0);
    }
}
