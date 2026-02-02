//! List command implementation.

use crate::cli::args::{ListArgs, SortField};
use crate::cli::output::Output;
use crate::error::Result;
use crate::note::NoteInfo;
use crate::vault::Vault;
use serde::Serialize;
use std::cmp::Ordering;

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub notes: Vec<NoteInfo>,
    pub total: usize,
}

pub fn run(vault: &Vault, args: &ListArgs, output: &Output) -> Result<()> {
    // Get base list of notes
    let paths = if let Some(ref pattern) = args.glob {
        vault.list_notes_matching(pattern)?
    } else {
        vault.list_notes()?
    };

    // Convert to NoteInfo and collect
    let mut notes: Vec<NoteInfo> = paths
        .iter()
        .filter_map(|path| vault.note_info(path).ok())
        .collect();

    // Apply tag filter if specified
    if !args.tag.is_empty() {
        notes = notes
            .into_iter()
            .filter(|info| {
                // Load note and check tags
                let path = std::path::PathBuf::from(&info.path);
                if let Ok(note) = vault.load_note(&path) {
                    let note_tags: Vec<String> =
                        note.tags().iter().map(|t| t.name.clone()).collect();
                    // All specified tags must be present (AND logic)
                    args.tag.iter().all(|required_tag| {
                        let required = if required_tag.starts_with('#') {
                            required_tag.clone()
                        } else {
                            format!("#{}", required_tag)
                        };
                        note_tags.iter().any(|t| t == &required || t.starts_with(&format!("{}/", required)))
                    })
                } else {
                    false
                }
            })
            .collect();
    }

    // Apply frontmatter filter if specified
    if !args.frontmatter.is_empty() {
        notes = notes
            .into_iter()
            .filter(|info| {
                let path = std::path::PathBuf::from(&info.path);
                if let Ok(note) = vault.load_note(&path) {
                    if let Ok(Some(fm)) = note.frontmatter() {
                        args.frontmatter.iter().all(|filter| {
                            if let Some((key, value)) = filter.split_once('=') {
                                if let Some(fm_value) = fm.get(key) {
                                    match fm_value {
                                        serde_yaml::Value::String(s) => s == value,
                                        serde_yaml::Value::Bool(b) => {
                                            (value == "true" && *b) || (value == "false" && !*b)
                                        }
                                        serde_yaml::Value::Number(n) => {
                                            n.to_string() == value
                                        }
                                        _ => false,
                                    }
                                } else {
                                    false
                                }
                            } else {
                                // Just check key existence
                                fm.get(filter).is_some()
                            }
                        })
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .collect();
    }

    // Apply has_links filter
    if args.has_links {
        notes = notes
            .into_iter()
            .filter(|info| {
                let path = std::path::PathBuf::from(&info.path);
                if let Ok(note) = vault.load_note(&path) {
                    !note.links().is_empty()
                } else {
                    false
                }
            })
            .collect();
    }

    // TODO: has_backlinks and orphans require building the full link graph
    // These will be implemented in Phase 2

    // Sort
    notes.sort_by(|a, b| {
        let cmp = match args.sort {
            SortField::Path => a.path.cmp(&b.path),
            SortField::Name => a.name.cmp(&b.name),
            SortField::Modified => compare_optional_strings(&a.modified, &b.modified),
            SortField::Created => compare_optional_strings(&a.created, &b.created),
        };

        if args.reverse {
            cmp.reverse()
        } else {
            cmp
        }
    });

    // Apply limit
    let total = notes.len();
    if let Some(limit) = args.limit {
        notes.truncate(limit);
    }

    let response = ListResponse { notes, total };
    output.print(&response)?;

    Ok(())
}

fn compare_optional_strings(a: &Option<String>, b: &Option<String>) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => a.cmp(b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}
