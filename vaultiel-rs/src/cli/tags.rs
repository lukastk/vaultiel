//! Tag-related CLI commands.

use crate::cli::output::Output;
use crate::error::{ExitCode, Result};
use crate::parser::parse_tags;
use crate::vault::Vault;
use serde::Serialize;
use std::collections::HashMap;

/// Output for get-tags from a specific note.
#[derive(Debug, Serialize)]
pub struct NoteTagsOutput {
    pub tags: Vec<NoteTagOutput>,
}

/// A tag in a specific note.
#[derive(Debug, Serialize)]
pub struct NoteTagOutput {
    pub tag: String,
    pub line: usize,
    pub context: String,
}

/// Output for get-tags vault-wide.
#[derive(Debug, Serialize)]
pub struct VaultTagsOutput {
    pub tags: Vec<VaultTagOutput>,
}

/// A tag with count.
#[derive(Debug, Serialize)]
pub struct VaultTagOutput {
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
}

/// Output for nested tags hierarchy.
#[derive(Debug, Serialize)]
pub struct NestedTagsOutput {
    pub tags: Vec<NestedTag>,
}

/// A tag in nested hierarchy format.
#[derive(Debug, Serialize)]
pub struct NestedTag {
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<NestedTag>,
}

/// Get tags from a specific note.
pub fn get_tags_from_note(vault: &Vault, path: &str, output: &Output) -> Result<ExitCode> {
    let note_path = vault.resolve_note(path)?;
    let note = vault.load_note(&note_path)?;

    let tags = parse_tags(&note.content);

    let tag_outputs: Vec<_> = tags
        .iter()
        .map(|t| NoteTagOutput {
            tag: t.name.clone(),
            line: t.line,
            context: "body".to_string(), // Tags are always in body for now
        })
        .collect();

    let result = NoteTagsOutput { tags: tag_outputs };
    output.print(&result)?;

    Ok(ExitCode::Success)
}

/// Get all tags in the vault.
pub fn get_tags_vault(
    vault: &Vault,
    with_counts: bool,
    nested: bool,
    glob_pattern: Option<&str>,
    output: &Output,
) -> Result<ExitCode> {
    let notes = if let Some(pattern) = glob_pattern {
        vault.list_notes_matching(pattern)?
    } else {
        vault.list_notes()?
    };

    // Collect all tags with counts
    let mut tag_counts: HashMap<String, usize> = HashMap::new();

    for note_path in notes {
        if let Ok(note) = vault.load_note(&note_path) {
            let tags = parse_tags(&note.content);
            for tag in tags {
                *tag_counts.entry(tag.name).or_insert(0) += 1;
            }
        }
    }

    if nested {
        let nested_tags = build_nested_tags(&tag_counts, with_counts);
        let result = NestedTagsOutput { tags: nested_tags };
        output.print(&result)?;
    } else {
        let mut tags: Vec<_> = tag_counts
            .into_iter()
            .map(|(tag, count)| VaultTagOutput {
                tag,
                count: if with_counts { Some(count) } else { None },
            })
            .collect();

        // Sort by tag name
        tags.sort_by(|a, b| a.tag.cmp(&b.tag));

        let result = VaultTagsOutput { tags };
        output.print(&result)?;
    }

    Ok(ExitCode::Success)
}

/// Build a nested tag hierarchy from flat tag counts.
fn build_nested_tags(tag_counts: &HashMap<String, usize>, with_counts: bool) -> Vec<NestedTag> {
    // Group tags by their root
    let mut roots: HashMap<String, Vec<(Vec<&str>, usize)>> = HashMap::new();

    for (tag, count) in tag_counts {
        let parts: Vec<&str> = tag.strip_prefix('#').unwrap_or(tag).split('/').collect();
        if let Some(root) = parts.first() {
            roots
                .entry(format!("#{}", root))
                .or_default()
                .push((parts, *count));
        }
    }

    // Build nested structure for each root
    let mut result: Vec<NestedTag> = Vec::new();

    for (root, tag_parts) in roots {
        let nested = build_nested_tag_recursive(&root, &tag_parts, 0, with_counts);
        result.push(nested);
    }

    // Sort by tag name
    result.sort_by(|a, b| a.tag.cmp(&b.tag));
    result
}

/// Recursively build nested tag structure.
fn build_nested_tag_recursive(
    current_tag: &str,
    all_parts: &[(Vec<&str>, usize)],
    depth: usize,
    with_counts: bool,
) -> NestedTag {
    // Find count for current tag
    let current_count = all_parts
        .iter()
        .find(|(parts, _)| parts.len() == depth + 1)
        .map(|(_, count)| *count);

    // Find children
    let mut children_map: HashMap<String, Vec<(Vec<&str>, usize)>> = HashMap::new();

    for (parts, count) in all_parts {
        if parts.len() > depth + 1 {
            let child_name = parts[depth + 1];
            let child_tag = format!("{}/{}", current_tag, child_name);
            children_map.entry(child_tag).or_default().push((parts.clone(), *count));
        }
    }

    let mut children: Vec<NestedTag> = Vec::new();
    for (child_tag, child_parts) in children_map {
        children.push(build_nested_tag_recursive(
            &child_tag,
            &child_parts,
            depth + 1,
            with_counts,
        ));
    }

    children.sort_by(|a, b| a.tag.cmp(&b.tag));

    NestedTag {
        tag: current_tag.to_string(),
        count: if with_counts { current_count } else { None },
        children,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_nested_tags() {
        let mut tag_counts = HashMap::new();
        tag_counts.insert("#rust".to_string(), 5);
        tag_counts.insert("#rust/cli".to_string(), 3);
        tag_counts.insert("#rust/web".to_string(), 2);
        tag_counts.insert("#python".to_string(), 4);

        let nested = build_nested_tags(&tag_counts, true);

        assert_eq!(nested.len(), 2); // #rust and #python

        let rust = nested.iter().find(|t| t.tag == "#rust").unwrap();
        assert_eq!(rust.count, Some(5));
        assert_eq!(rust.children.len(), 2);
    }
}
