//! Heading-related CLI commands.

use crate::cli::output::Output;
use crate::error::{ExitCode, Result, VaultError};
use crate::parser::{find_heading_by_slug, find_heading_by_text, parse_headings};
use crate::types::Heading;
use crate::vault::Vault;
use serde::Serialize;

/// Output for get-headings command (flat).
#[derive(Debug, Serialize)]
pub struct HeadingsOutput {
    pub headings: Vec<HeadingOutput>,
}

/// A heading in flat format.
#[derive(Debug, Serialize)]
pub struct HeadingOutput {
    pub text: String,
    pub level: u8,
    pub line: usize,
    pub slug: String,
}

/// Output for get-headings --nested command.
#[derive(Debug, Serialize)]
pub struct NestedHeadingsOutput {
    pub headings: Vec<NestedHeading>,
}

/// A heading in nested format.
#[derive(Debug, Serialize)]
pub struct NestedHeading {
    pub text: String,
    pub level: u8,
    pub line: usize,
    pub slug: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<NestedHeading>,
}

/// Get headings from a note.
pub fn get_headings(
    vault: &Vault,
    path: &str,
    min_level: Option<u8>,
    max_level: Option<u8>,
    nested: bool,
    output: &Output,
) -> Result<ExitCode> {
    let note_path = vault.resolve_note(path)?;
    let note = vault.load_note(&note_path)?;

    let headings = parse_headings(&note.content);

    // Filter by level
    let filtered: Vec<_> = headings
        .into_iter()
        .filter(|h| {
            let min_ok = min_level.map(|m| h.level >= m).unwrap_or(true);
            let max_ok = max_level.map(|m| h.level <= m).unwrap_or(true);
            min_ok && max_ok
        })
        .collect();

    if nested {
        let nested_headings = build_nested_headings(&filtered);
        let result = NestedHeadingsOutput {
            headings: nested_headings,
        };
        output.print(&result)?;
    } else {
        let heading_outputs: Vec<_> = filtered
            .iter()
            .map(|h| HeadingOutput {
                text: h.text.clone(),
                level: h.level,
                line: h.line,
                slug: h.slug.clone(),
            })
            .collect();

        let result = HeadingsOutput {
            headings: heading_outputs,
        };
        output.print(&result)?;
    }

    Ok(ExitCode::Success)
}

/// Build nested heading hierarchy.
fn build_nested_headings(headings: &[Heading]) -> Vec<NestedHeading> {
    if headings.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut stack: Vec<(u8, usize)> = Vec::new(); // (level, index in result/children)

    for heading in headings {
        let nested = NestedHeading {
            text: heading.text.clone(),
            level: heading.level,
            line: heading.line,
            slug: heading.slug.clone(),
            children: Vec::new(),
        };

        // Pop stack until we find a heading with lower level
        while !stack.is_empty() && stack.last().unwrap().0 >= heading.level {
            stack.pop();
        }

        if stack.is_empty() {
            // Top-level heading
            result.push(nested);
            stack.push((heading.level, result.len() - 1));
        } else {
            // Find parent and add as child
            let parent_path: Vec<usize> = stack.iter().map(|(_, idx)| *idx).collect();
            add_child_heading(&mut result, &parent_path, nested);

            // Push current heading to stack
            let child_idx = get_children_count(&result, &parent_path) - 1;
            stack.push((heading.level, child_idx));
        }
    }

    result
}

/// Add a child heading to the tree at the specified path.
fn add_child_heading(
    result: &mut Vec<NestedHeading>,
    path: &[usize],
    child: NestedHeading,
) {
    if path.is_empty() {
        return;
    }

    let mut current = &mut result[path[0]];
    for &idx in &path[1..] {
        current = &mut current.children[idx];
    }
    current.children.push(child);
}

/// Get the number of children at the specified path.
fn get_children_count(result: &[NestedHeading], path: &[usize]) -> usize {
    if path.is_empty() {
        return 0;
    }

    let mut current = &result[path[0]];
    for &idx in &path[1..] {
        current = &current.children[idx];
    }
    current.children.len()
}

/// Get the content of a section.
pub fn get_section(
    vault: &Vault,
    path: &str,
    heading_query: &str,
    by_slug: bool,
    include_subheadings: bool,
    content_only: bool,
    output: &Output,
) -> Result<ExitCode> {
    let note_path = vault.resolve_note(path)?;
    let note = vault.load_note(&note_path)?;

    let headings = parse_headings(&note.content);

    // Find the target heading
    let target_heading = if by_slug {
        find_heading_by_slug(&headings, heading_query)
    } else {
        // Parse heading query - could be "## Heading" or just "Heading"
        let query = heading_query
            .trim_start_matches('#')
            .trim();
        find_heading_by_text(&headings, query)
    };

    let target = target_heading.ok_or_else(|| VaultError::HeadingNotFound {
        note: note_path.clone(),
        heading: heading_query.to_string(),
    })?;

    // Find the end of the section
    let target_level = target.level;
    let lines: Vec<&str> = note.content.lines().collect();

    let start_line = if content_only {
        target.line // Skip the heading line itself
    } else {
        target.line - 1 // Include heading (convert 1-indexed to 0-indexed)
    };

    // Find end line - next heading of same or higher level
    let end_line = if include_subheadings {
        // Stop at next heading of same or lower level (higher level number)
        headings
            .iter()
            .find(|h| h.line > target.line && h.level <= target_level)
            .map(|h| h.line - 1) // Convert to 0-indexed and get line before heading
            .unwrap_or(lines.len())
    } else {
        // Stop at any heading
        headings
            .iter()
            .find(|h| h.line > target.line)
            .map(|h| h.line - 1)
            .unwrap_or(lines.len())
    };

    // Extract content
    let section_content = if start_line < lines.len() {
        let end = end_line.min(lines.len());
        let start = if content_only { start_line } else { start_line };
        lines[start..end].join("\n")
    } else {
        String::new()
    };

    // Trim trailing whitespace
    let section_content = section_content.trim_end().to_string();

    #[derive(Serialize)]
    struct SectionOutput {
        heading: HeadingOutput,
        content: String,
    }

    let result = SectionOutput {
        heading: HeadingOutput {
            text: target.text.clone(),
            level: target.level,
            line: target.line,
            slug: target.slug.clone(),
        },
        content: section_content,
    };

    output.print(&result)?;

    Ok(ExitCode::Success)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_nested_headings() {
        let headings = vec![
            Heading {
                text: "Top".to_string(),
                level: 1,
                line: 1,
                slug: "top".to_string(),
            },
            Heading {
                text: "Section A".to_string(),
                level: 2,
                line: 3,
                slug: "section-a".to_string(),
            },
            Heading {
                text: "Subsection".to_string(),
                level: 3,
                line: 5,
                slug: "subsection".to_string(),
            },
            Heading {
                text: "Section B".to_string(),
                level: 2,
                line: 7,
                slug: "section-b".to_string(),
            },
        ];

        let nested = build_nested_headings(&headings);

        assert_eq!(nested.len(), 1); // One top-level heading
        assert_eq!(nested[0].text, "Top");
        assert_eq!(nested[0].children.len(), 2); // Section A and Section B
        assert_eq!(nested[0].children[0].children.len(), 1); // Subsection under Section A
    }
}
