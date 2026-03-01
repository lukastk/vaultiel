//! Predicate evaluation against a Note.

use crate::error::VaultError;
use crate::note::Note;
use crate::parser::{parse_headings, parse_inline_properties, parse_tags, split_frontmatter};
use crate::search::types::*;
use regex::RegexBuilder;

/// Evaluate a search query against a note, returning all matches.
///
/// Returns an empty vec if the note doesn't match.
pub fn evaluate_note(note: &Note, query: &SearchQuery) -> Vec<SearchMatch> {
    match evaluate_inner(note, query) {
        Ok(matches) => matches,
        Err(_) => Vec::new(), // Skip notes that cause errors (e.g. invalid regex)
    }
}

fn evaluate_inner(
    note: &Note,
    query: &SearchQuery,
) -> Result<Vec<SearchMatch>, VaultError> {
    match query {
        SearchQuery::Field(predicate) => evaluate_field(note, predicate),
        SearchQuery::And { children } => {
            let mut all_matches = Vec::new();
            for child in children {
                let matches = evaluate_inner(note, child)?;
                if matches.is_empty() {
                    return Ok(Vec::new()); // short-circuit
                }
                all_matches.extend(matches);
            }
            Ok(all_matches)
        }
        SearchQuery::Or { children } => {
            for child in children {
                let matches = evaluate_inner(note, child)?;
                if !matches.is_empty() {
                    return Ok(matches); // first match wins
                }
            }
            Ok(Vec::new())
        }
        SearchQuery::Not { child } => {
            let matches = evaluate_inner(note, child)?;
            if matches.is_empty() {
                // Child did NOT match → NOT succeeds
                Ok(vec![SearchMatch {
                    field: "not".to_string(),
                    line: None,
                    text: None,
                }])
            } else {
                // Child matched → NOT fails
                Ok(Vec::new())
            }
        }
    }
}

fn evaluate_field(
    note: &Note,
    predicate: &FieldPredicate,
) -> Result<Vec<SearchMatch>, VaultError> {
    match predicate {
        FieldPredicate::Path { matcher } => {
            let path_str = note.path.to_string_lossy();
            if matches_string(&path_str, matcher, true)? {
                Ok(vec![SearchMatch {
                    field: "path".to_string(),
                    line: None,
                    text: Some(path_str.to_string()),
                }])
            } else {
                Ok(Vec::new())
            }
        }
        FieldPredicate::Filename { matcher } => {
            let name = note.name();
            if matches_string(name, matcher, true)? {
                Ok(vec![SearchMatch {
                    field: "filename".to_string(),
                    line: None,
                    text: Some(name.to_string()),
                }])
            } else {
                Ok(Vec::new())
            }
        }
        FieldPredicate::Tag { value } => evaluate_tag(note, value),
        FieldPredicate::Content { matcher } => evaluate_content(note, matcher),
        FieldPredicate::Section { query } => evaluate_section(note, query),
        FieldPredicate::Line { query } => evaluate_line(note, query),
        FieldPredicate::Property { key, op, value } => {
            evaluate_property(note, key, op, value.as_deref())
        }
    }
}

// ============================================================================
// String matching
// ============================================================================

fn matches_string(
    haystack: &str,
    matcher: &StringMatcher,
    case_sensitive: bool,
) -> Result<bool, VaultError> {
    match matcher {
        StringMatcher::Contains { value } => {
            if case_sensitive {
                Ok(haystack.contains(value.as_str()))
            } else {
                Ok(haystack.to_lowercase().contains(&value.to_lowercase()))
            }
        }
        StringMatcher::Exact { value } => {
            if case_sensitive {
                Ok(haystack == value)
            } else {
                Ok(haystack.eq_ignore_ascii_case(value))
            }
        }
        StringMatcher::Regex { pattern } => {
            let re = RegexBuilder::new(pattern)
                .case_insensitive(!case_sensitive)
                .build()
                .map_err(|e| VaultError::SearchError(format!("Invalid regex: {}", e)))?;
            Ok(re.is_match(haystack))
        }
    }
}

// ============================================================================
// Tag matching
// ============================================================================

fn evaluate_tag(note: &Note, value: &str) -> Result<Vec<SearchMatch>, VaultError> {
    let value_lower = value.to_lowercase();
    let value_stripped = value_lower.trim_start_matches('#');

    // Check inline tags in body
    let tags = parse_tags(&note.content);
    for tag in &tags {
        let tag_stripped = tag.name.trim_start_matches('#').to_lowercase();
        if tag_stripped == value_stripped {
            return Ok(vec![SearchMatch {
                field: "tag".to_string(),
                line: Some(tag.line),
                text: Some(tag.name.clone()),
            }]);
        }
    }

    // Check frontmatter tags
    if let Ok(Some(fm)) = note.frontmatter() {
        if let Some(tags_value) = fm.get("tags") {
            if let Some(arr) = tags_value.as_sequence() {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        let s_stripped = s.trim_start_matches('#').to_lowercase();
                        if s_stripped == value_stripped {
                            return Ok(vec![SearchMatch {
                                field: "tag".to_string(),
                                line: Some(1), // frontmatter
                                text: Some(s.to_string()),
                            }]);
                        }
                    }
                }
            }
        }
    }

    Ok(Vec::new())
}

// ============================================================================
// Content matching
// ============================================================================

fn evaluate_content(note: &Note, matcher: &StringMatcher) -> Result<Vec<SearchMatch>, VaultError> {
    let body = note.body();
    let split = split_frontmatter(&note.content);
    let body_start = split.content_start_line;

    let mut matches = Vec::new();

    for (i, line) in body.lines().enumerate() {
        let line_num = body_start + i;
        if matches_string(line, matcher, false)? {
            matches.push(SearchMatch {
                field: "content".to_string(),
                line: Some(line_num),
                text: Some(line.to_string()),
            });
        }
    }

    Ok(matches)
}

// ============================================================================
// Section matching
// ============================================================================

fn evaluate_section(
    note: &Note,
    sub_query: &SearchQuery,
) -> Result<Vec<SearchMatch>, VaultError> {
    let body = note.body();
    let headings = parse_headings(&note.content);
    let split = split_frontmatter(&note.content);
    let body_start = split.content_start_line;

    // Build sections: each section is the text under a heading (or before the first heading)
    let body_lines: Vec<&str> = body.lines().collect();

    // Heading line numbers are 1-indexed and relative to full content
    // Convert to 0-indexed body line indices
    let heading_body_indices: Vec<usize> = headings
        .iter()
        .filter_map(|h| {
            if h.line >= body_start {
                Some(h.line - body_start)
            } else {
                None
            }
        })
        .collect();

    // Build section ranges
    let mut sections: Vec<(usize, usize)> = Vec::new(); // (start_body_idx, end_body_idx exclusive)

    if heading_body_indices.is_empty() {
        // No headings: entire body is one section
        sections.push((0, body_lines.len()));
    } else {
        // Before first heading
        if heading_body_indices[0] > 0 {
            sections.push((0, heading_body_indices[0]));
        }
        // Each heading section
        for (i, &start) in heading_body_indices.iter().enumerate() {
            let end = if i + 1 < heading_body_indices.len() {
                heading_body_indices[i + 1]
            } else {
                body_lines.len()
            };
            sections.push((start, end));
        }
    }

    // Evaluate sub_query against each section
    for &(start, end) in &sections {
        let section_text: String = body_lines[start..end].join("\n");
        let section_note = Note::new(note.path.clone(), section_text);
        let matches = evaluate_inner(&section_note, sub_query)?;
        if !matches.is_empty() {
            // Re-map line numbers back to original note
            let remapped: Vec<SearchMatch> = matches
                .into_iter()
                .map(|mut m| {
                    if let Some(ref mut line) = m.line {
                        *line += body_start + start - 1;
                    }
                    m.field = format!("section:{}", m.field);
                    m
                })
                .collect();
            return Ok(remapped);
        }
    }

    Ok(Vec::new())
}

// ============================================================================
// Line matching
// ============================================================================

fn evaluate_line(
    note: &Note,
    sub_query: &SearchQuery,
) -> Result<Vec<SearchMatch>, VaultError> {
    let body = note.body();
    let split = split_frontmatter(&note.content);
    let body_start = split.content_start_line;

    for (i, line) in body.lines().enumerate() {
        let line_num = body_start + i;
        let line_note = Note::new(note.path.clone(), line.to_string());
        let matches = evaluate_inner(&line_note, sub_query)?;
        if !matches.is_empty() {
            let remapped: Vec<SearchMatch> = matches
                .into_iter()
                .map(|mut m| {
                    m.line = Some(line_num);
                    m.field = format!("line:{}", m.field);
                    m
                })
                .collect();
            return Ok(remapped);
        }
    }

    Ok(Vec::new())
}

// ============================================================================
// Property matching
// ============================================================================

fn evaluate_property(
    note: &Note,
    key: &str,
    op: &PropertyOp,
    expected_value: Option<&str>,
) -> Result<Vec<SearchMatch>, VaultError> {
    // Collect all property values for this key
    let mut found_values: Vec<(String, Option<usize>)> = Vec::new();

    // Check frontmatter
    if let Ok(Some(fm)) = note.frontmatter() {
        if let Some(fm_val) = fm.get(key) {
            let val_str = yaml_value_to_string(fm_val);
            found_values.push((val_str, Some(1))); // line 1 for frontmatter
        }
    }

    // Check inline properties
    let inline_props = parse_inline_properties(&note.content);
    for prop in &inline_props {
        if prop.key == key {
            found_values.push((prop.value.clone(), Some(prop.line)));
        }
    }

    if found_values.is_empty() {
        return match op {
            PropertyOp::Exists => Ok(Vec::new()), // doesn't exist → no match
            _ => Ok(Vec::new()),
        };
    }

    match op {
        PropertyOp::Exists => Ok(vec![SearchMatch {
            field: format!("property:{}", key),
            line: found_values[0].1,
            text: Some(found_values[0].0.clone()),
        }]),
        _ => {
            let expected = expected_value.unwrap_or("");
            for (val, line) in &found_values {
                if compare_values(val, expected, op) {
                    return Ok(vec![SearchMatch {
                        field: format!("property:{}", key),
                        line: *line,
                        text: Some(val.clone()),
                    }]);
                }
            }
            Ok(Vec::new())
        }
    }
}

fn yaml_value_to_string(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(f) = n.as_f64() {
                f.to_string()
            } else {
                n.to_string()
            }
        }
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Null => "null".to_string(),
        _ => serde_yaml::to_string(value).unwrap_or_default().trim().to_string(),
    }
}

/// Compare two values using the given operator.
/// Tries numeric → date (ISO 8601) → string fallback.
fn compare_values(actual: &str, expected: &str, op: &PropertyOp) -> bool {
    // Try numeric comparison
    if let (Ok(a), Ok(b)) = (actual.parse::<f64>(), expected.parse::<f64>()) {
        return match op {
            PropertyOp::Eq => (a - b).abs() < f64::EPSILON,
            PropertyOp::NotEq => (a - b).abs() >= f64::EPSILON,
            PropertyOp::Lt => a < b,
            PropertyOp::Gt => a > b,
            PropertyOp::Lte => a <= b,
            PropertyOp::Gte => a >= b,
            PropertyOp::Exists => true,
        };
    }

    // Date and string comparison: both use lexicographic ordering
    // ISO 8601 dates sort correctly with string comparison
    match op {
        PropertyOp::Eq => actual.eq_ignore_ascii_case(expected),
        PropertyOp::NotEq => !actual.eq_ignore_ascii_case(expected),
        PropertyOp::Lt => actual < expected,
        PropertyOp::Gt => actual > expected,
        PropertyOp::Lte => actual <= expected,
        PropertyOp::Gte => actual >= expected,
        PropertyOp::Exists => true,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_note(content: &str) -> Note {
        Note::new(PathBuf::from("test/note.md"), content)
    }

    fn make_note_at(path: &str, content: &str) -> Note {
        Note::new(PathBuf::from(path), content)
    }

    // -- Path matching --

    #[test]
    fn test_path_contains() {
        let note = make_note_at("daily/2024-01-15.md", "content");
        let q = SearchQuery::Field(FieldPredicate::Path {
            matcher: StringMatcher::Contains {
                value: "daily/".to_string(),
            },
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].field, "path");
    }

    #[test]
    fn test_path_no_match() {
        let note = make_note_at("projects/note.md", "content");
        let q = SearchQuery::Field(FieldPredicate::Path {
            matcher: StringMatcher::Contains {
                value: "daily/".to_string(),
            },
        });
        let matches = evaluate_note(&note, &q);
        assert!(matches.is_empty());
    }

    // -- Filename matching --

    #[test]
    fn test_filename_contains() {
        let note = make_note_at("folder/Meeting Notes.md", "content");
        let q = SearchQuery::Field(FieldPredicate::Filename {
            matcher: StringMatcher::Contains {
                value: "Meeting".to_string(),
            },
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 1);
    }

    // -- Tag matching --

    #[test]
    fn test_tag_inline() {
        let note = make_note("Some text #project more text");
        let q = SearchQuery::Field(FieldPredicate::Tag {
            value: "project".to_string(),
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].field, "tag");
    }

    #[test]
    fn test_tag_with_hash() {
        let note = make_note("Some text #project more text");
        let q = SearchQuery::Field(FieldPredicate::Tag {
            value: "#project".to_string(),
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_tag_case_insensitive() {
        let note = make_note("Some text #Project more text");
        let q = SearchQuery::Field(FieldPredicate::Tag {
            value: "project".to_string(),
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_tag_frontmatter() {
        let note = make_note("---\ntags:\n  - project\n  - draft\n---\nBody text");
        let q = SearchQuery::Field(FieldPredicate::Tag {
            value: "project".to_string(),
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_tag_no_match() {
        let note = make_note("Some text without tags");
        let q = SearchQuery::Field(FieldPredicate::Tag {
            value: "project".to_string(),
        });
        let matches = evaluate_note(&note, &q);
        assert!(matches.is_empty());
    }

    // -- Content matching --

    #[test]
    fn test_content_contains() {
        let note = make_note("First line\nSecond line with meeting\nThird line");
        let q = SearchQuery::Field(FieldPredicate::Content {
            matcher: StringMatcher::Contains {
                value: "meeting".to_string(),
            },
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line, Some(2));
    }

    #[test]
    fn test_content_case_insensitive() {
        let note = make_note("Hello WORLD");
        let q = SearchQuery::Field(FieldPredicate::Content {
            matcher: StringMatcher::Contains {
                value: "hello".to_string(),
            },
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_content_exact() {
        let note = make_note("First line\nexact match line\nThird");
        let q = SearchQuery::Field(FieldPredicate::Content {
            matcher: StringMatcher::Exact {
                value: "exact match line".to_string(),
            },
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_content_regex() {
        let note = make_note("error123 happened\nerror456 also");
        let q = SearchQuery::Field(FieldPredicate::Content {
            matcher: StringMatcher::Regex {
                pattern: "error\\d+".to_string(),
            },
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_content_with_frontmatter() {
        let note = make_note("---\ntitle: Test\n---\nBody with keyword");
        let q = SearchQuery::Field(FieldPredicate::Content {
            matcher: StringMatcher::Contains {
                value: "keyword".to_string(),
            },
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line, Some(4));
    }

    // -- Section matching --

    #[test]
    fn test_section_match() {
        let note = make_note("# Introduction\nGeneral text\n# Error Handling\nFix error123 here\nMore error info");
        let sub_query = SearchQuery::Field(FieldPredicate::Content {
            matcher: StringMatcher::Contains {
                value: "error".to_string(),
            },
        });
        let q = SearchQuery::Field(FieldPredicate::Section {
            query: Box::new(sub_query),
        });
        let matches = evaluate_note(&note, &q);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_section_no_headings() {
        let note = make_note("Just a note\nwith some content\nand keywords");
        let sub_query = SearchQuery::Field(FieldPredicate::Content {
            matcher: StringMatcher::Contains {
                value: "keywords".to_string(),
            },
        });
        let q = SearchQuery::Field(FieldPredicate::Section {
            query: Box::new(sub_query),
        });
        let matches = evaluate_note(&note, &q);
        assert!(!matches.is_empty());
    }

    // -- Line matching --

    #[test]
    fn test_line_co_occurrence() {
        let note = make_note("TODO: fix bug\nDone: fix other thing\nTODO deadline: tomorrow");
        let sub_query = SearchQuery::And {
            children: vec![
                SearchQuery::Field(FieldPredicate::Content {
                    matcher: StringMatcher::Contains {
                        value: "TODO".to_string(),
                    },
                }),
                SearchQuery::Field(FieldPredicate::Content {
                    matcher: StringMatcher::Contains {
                        value: "deadline".to_string(),
                    },
                }),
            ],
        };
        let q = SearchQuery::Field(FieldPredicate::Line {
            query: Box::new(sub_query),
        });
        let matches = evaluate_note(&note, &q);
        assert!(!matches.is_empty());
        // Should match line 3 (the one with both TODO and deadline)
        assert!(matches.iter().any(|m| m.line == Some(3)));
    }

    #[test]
    fn test_line_no_co_occurrence() {
        let note = make_note("TODO: fix bug\ndeadline: tomorrow");
        let sub_query = SearchQuery::And {
            children: vec![
                SearchQuery::Field(FieldPredicate::Content {
                    matcher: StringMatcher::Contains {
                        value: "TODO".to_string(),
                    },
                }),
                SearchQuery::Field(FieldPredicate::Content {
                    matcher: StringMatcher::Contains {
                        value: "deadline".to_string(),
                    },
                }),
            ],
        };
        let q = SearchQuery::Field(FieldPredicate::Line {
            query: Box::new(sub_query),
        });
        let matches = evaluate_note(&note, &q);
        assert!(matches.is_empty());
    }

    // -- Property matching --

    #[test]
    fn test_property_exists() {
        let note = make_note("---\nstatus: active\n---\nBody");
        let q = SearchQuery::Field(FieldPredicate::Property {
            key: "status".to_string(),
            op: PropertyOp::Exists,
            value: None,
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].field, "property:status");
    }

    #[test]
    fn test_property_eq() {
        let note = make_note("---\nstatus: active\n---\nBody");
        let q = SearchQuery::Field(FieldPredicate::Property {
            key: "status".to_string(),
            op: PropertyOp::Eq,
            value: Some("active".to_string()),
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_property_eq_no_match() {
        let note = make_note("---\nstatus: draft\n---\nBody");
        let q = SearchQuery::Field(FieldPredicate::Property {
            key: "status".to_string(),
            op: PropertyOp::Eq,
            value: Some("active".to_string()),
        });
        let matches = evaluate_note(&note, &q);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_property_numeric_comparison() {
        let note = make_note("---\npriority: 5\n---\nBody");
        let q = SearchQuery::Field(FieldPredicate::Property {
            key: "priority".to_string(),
            op: PropertyOp::Gte,
            value: Some("3".to_string()),
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_property_date_comparison() {
        let note = make_note("---\ndue: 2024-02-15\n---\nBody");
        let q = SearchQuery::Field(FieldPredicate::Property {
            key: "due".to_string(),
            op: PropertyOp::Lt,
            value: Some("2024-03-01".to_string()),
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_property_inline() {
        let note = make_note("Body with [status::active] inline property");
        let q = SearchQuery::Field(FieldPredicate::Property {
            key: "status".to_string(),
            op: PropertyOp::Eq,
            value: Some("active".to_string()),
        });
        let matches = evaluate_note(&note, &q);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_property_not_exists() {
        let note = make_note("---\ntitle: Test\n---\nBody");
        let q = SearchQuery::Field(FieldPredicate::Property {
            key: "status".to_string(),
            op: PropertyOp::Exists,
            value: None,
        });
        let matches = evaluate_note(&note, &q);
        assert!(matches.is_empty());
    }

    // -- Boolean logic --

    #[test]
    fn test_and_both_match() {
        let note = make_note("Some #project text with meeting notes");
        let q = SearchQuery::And {
            children: vec![
                SearchQuery::Field(FieldPredicate::Tag {
                    value: "project".to_string(),
                }),
                SearchQuery::Field(FieldPredicate::Content {
                    matcher: StringMatcher::Contains {
                        value: "meeting".to_string(),
                    },
                }),
            ],
        };
        let matches = evaluate_note(&note, &q);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_and_one_fails() {
        let note = make_note("Some #project text");
        let q = SearchQuery::And {
            children: vec![
                SearchQuery::Field(FieldPredicate::Tag {
                    value: "project".to_string(),
                }),
                SearchQuery::Field(FieldPredicate::Content {
                    matcher: StringMatcher::Contains {
                        value: "meeting".to_string(),
                    },
                }),
            ],
        };
        let matches = evaluate_note(&note, &q);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_or_first_matches() {
        let note = make_note("Some #project text");
        let q = SearchQuery::Or {
            children: vec![
                SearchQuery::Field(FieldPredicate::Tag {
                    value: "project".to_string(),
                }),
                SearchQuery::Field(FieldPredicate::Tag {
                    value: "log".to_string(),
                }),
            ],
        };
        let matches = evaluate_note(&note, &q);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_not_match() {
        let note = make_note("No tags here");
        let q = SearchQuery::Not {
            child: Box::new(SearchQuery::Field(FieldPredicate::Tag {
                value: "archived".to_string(),
            })),
        };
        let matches = evaluate_note(&note, &q);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_not_no_match() {
        let note = make_note("Has #archived tag");
        let q = SearchQuery::Not {
            child: Box::new(SearchQuery::Field(FieldPredicate::Tag {
                value: "archived".to_string(),
            })),
        };
        let matches = evaluate_note(&note, &q);
        assert!(matches.is_empty());
    }

    // -- Compare values --

    #[test]
    fn test_compare_numeric() {
        assert!(compare_values("10", "5", &PropertyOp::Gt));
        assert!(!compare_values("3", "5", &PropertyOp::Gt));
        assert!(compare_values("5", "5", &PropertyOp::Eq));
    }

    #[test]
    fn test_compare_dates() {
        assert!(compare_values("2024-01-15", "2024-03-01", &PropertyOp::Lt));
        assert!(compare_values("2024-03-01", "2024-01-15", &PropertyOp::Gt));
    }

    #[test]
    fn test_compare_strings() {
        assert!(compare_values("active", "active", &PropertyOp::Eq));
        assert!(!compare_values("active", "done", &PropertyOp::Eq));
    }
}
