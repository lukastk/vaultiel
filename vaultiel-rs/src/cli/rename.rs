//! Rename command with link propagation.

use crate::cli::output::Output;
use crate::error::{ExitCode, Result};
use crate::graph::LinkGraph;
use crate::parser::wikilink::format_wikilink;
use crate::types::Link;
use crate::vault::Vault;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Output for rename command.
#[derive(Debug, Serialize)]
pub struct RenameOutput {
    pub from: PathBuf,
    pub to: PathBuf,
    pub updated_files: Vec<UpdatedFile>,
    pub message: String,
}

/// A file that was updated during rename.
#[derive(Debug, Serialize)]
pub struct UpdatedFile {
    pub path: PathBuf,
    pub links_updated: usize,
}

/// Output for dry-run mode.
#[derive(Debug, Serialize)]
pub struct RenameDryRunOutput {
    pub action: String,
    pub from: PathBuf,
    pub to: PathBuf,
    pub would_update: Vec<WouldUpdate>,
}

/// A file that would be updated.
#[derive(Debug, Serialize)]
pub struct WouldUpdate {
    pub path: PathBuf,
    pub links: Vec<LinkChange>,
}

/// A link that would be changed.
#[derive(Debug, Serialize)]
pub struct LinkChange {
    pub line: usize,
    pub old: String,
    pub new: String,
}

/// Execute rename command.
pub fn rename(
    vault: &Vault,
    from: &str,
    to: &str,
    no_propagate: bool,
    dry_run: bool,
    output: &Output,
) -> Result<ExitCode> {
    let from_path = vault.resolve_note(from)?;
    let to_path = vault.normalize_note_path(to);

    // Check if target already exists
    if vault.note_exists(&to_path) {
        return Err(crate::error::VaultError::NoteAlreadyExists(to_path));
    }

    if no_propagate {
        // Simple rename without link propagation
        if dry_run {
            let result = RenameDryRunOutput {
                action: "rename".to_string(),
                from: from_path.clone(),
                to: to_path.clone(),
                would_update: vec![],
            };
            output.print(&result)?;
        } else {
            vault.rename_note(&from_path, &to_path)?;

            let result = RenameOutput {
                from: from_path,
                to: to_path,
                updated_files: vec![],
                message: "Note renamed successfully (no propagation)".to_string(),
            };
            output.print(&result)?;
        }
    } else {
        // Build link graph to find incoming links
        let graph = LinkGraph::build(vault)?;
        let incoming = graph.get_incoming(&from_path);

        // Group incoming links by source file
        let mut links_by_file: HashMap<PathBuf, Vec<&Link>> = HashMap::new();
        for incoming_link in &incoming {
            links_by_file
                .entry(incoming_link.from.clone())
                .or_default()
                .push(&incoming_link.link);
        }

        // Calculate the new link target
        let old_name = from_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let new_name = to_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if dry_run {
            // Collect what would change
            let mut would_update = Vec::new();

            for (source_path, links) in &links_by_file {
                let _note = vault.load_note(source_path)?;
                let mut link_changes = Vec::new();

                for link in links {
                    let old_link_str = format_wikilink(link);
                    let new_link = create_updated_link(link, old_name, new_name, &to_path);
                    let new_link_str = format_wikilink(&new_link);

                    if old_link_str != new_link_str {
                        link_changes.push(LinkChange {
                            line: link.line,
                            old: old_link_str,
                            new: new_link_str,
                        });
                    }
                }

                if !link_changes.is_empty() {
                    would_update.push(WouldUpdate {
                        path: source_path.clone(),
                        links: link_changes,
                    });
                }
            }

            let result = RenameDryRunOutput {
                action: "rename".to_string(),
                from: from_path,
                to: to_path,
                would_update,
            };
            output.print(&result)?;
        } else {
            // Actually perform the rename and update links
            let mut updated_files = Vec::new();

            // Update links in each source file
            for (source_path, links) in &links_by_file {
                let note = vault.load_note(source_path)?;
                let mut content = note.content.clone();
                let mut updates = 0;

                // Sort links by position (reverse order to update from end to start)
                let mut sorted_links: Vec<_> = links.iter().collect();
                sorted_links.sort_by(|a, b| {
                    (b.line, b.start_col).cmp(&(a.line, a.start_col))
                });

                for link in sorted_links {
                    let old_link_str = format_wikilink(link);
                    let new_link = create_updated_link(link, old_name, new_name, &to_path);
                    let new_link_str = format_wikilink(&new_link);

                    if old_link_str != new_link_str {
                        // Find and replace the link in content
                        if let Some(pos) = find_link_position(&content, link) {
                            let end_pos = pos + old_link_str.len();
                            content = format!(
                                "{}{}{}",
                                &content[..pos],
                                new_link_str,
                                &content[end_pos..]
                            );
                            updates += 1;
                        }
                    }
                }

                if updates > 0 {
                    // Save the updated note
                    let mut updated_note = note.clone();
                    updated_note.content = content;
                    vault.save_note(&updated_note)?;

                    updated_files.push(UpdatedFile {
                        path: source_path.clone(),
                        links_updated: updates,
                    });
                }
            }

            // Rename the actual file
            vault.rename_note(&from_path, &to_path)?;

            let result = RenameOutput {
                from: from_path,
                to: to_path,
                updated_files,
                message: "Note renamed successfully".to_string(),
            };
            output.print(&result)?;
        }
    }

    Ok(ExitCode::Success)
}

/// Create an updated link with the new target.
fn create_updated_link(old_link: &Link, old_name: &str, new_name: &str, new_path: &PathBuf) -> Link {
    let mut new_link = old_link.clone();

    // Update target
    if old_link.target.to_lowercase() == old_name.to_lowercase() {
        // Simple name reference
        new_link.target = new_name.to_string();
    } else if old_link.target.to_lowercase().ends_with(&format!("/{}", old_name.to_lowercase())) {
        // Path reference - update the filename part
        let new_target = new_path.to_string_lossy().trim_end_matches(".md").to_string();
        new_link.target = new_target;
    } else {
        // Full path match
        new_link.target = new_path.to_string_lossy().trim_end_matches(".md").to_string();
    }

    new_link
}

/// Find the byte position of a link in content.
fn find_link_position(content: &str, link: &Link) -> Option<usize> {
    let lines: Vec<&str> = content.lines().collect();
    if link.line == 0 || link.line > lines.len() {
        return None;
    }

    // Calculate byte offset of the line start
    let mut offset = 0;
    for line in lines.iter().take(link.line - 1) {
        offset += line.len() + 1; // +1 for newline
    }

    // Add column offset
    Some(offset + link.start_col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_updated_link() {
        let old_link = Link {
            target: "Old Note".to_string(),
            alias: None,
            heading: None,
            block_id: None,
            embed: false,
            line: 1,
            start_col: 0,
            end_col: 12,
        };

        let new_link = create_updated_link(
            &old_link,
            "Old Note",
            "New Note",
            &PathBuf::from("New Note.md"),
        );

        assert_eq!(new_link.target, "New Note");
    }

    #[test]
    fn test_create_updated_link_with_heading() {
        let old_link = Link {
            target: "Old Note".to_string(),
            alias: None,
            heading: Some("Section".to_string()),
            block_id: None,
            embed: false,
            line: 1,
            start_col: 0,
            end_col: 20,
        };

        let new_link = create_updated_link(
            &old_link,
            "Old Note",
            "New Note",
            &PathBuf::from("New Note.md"),
        );

        assert_eq!(new_link.target, "New Note");
        assert_eq!(new_link.heading, Some("Section".to_string()));
    }
}
