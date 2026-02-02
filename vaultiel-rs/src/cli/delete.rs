//! Delete command implementation.

use crate::cli::args::DeleteArgs;
use crate::cli::output::{DryRunResponse, Output};
use crate::error::Result;
use crate::vault::Vault;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub path: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

pub fn run(vault: &Vault, args: &DeleteArgs, output: &Output) -> Result<()> {
    let path = vault.normalize_note_path(&args.path);

    // Verify note exists
    let _note = vault.load_note(&path)?;

    // Check for incoming links (unless no_propagate)
    let mut incoming_links = Vec::new();
    if !args.no_propagate {
        // Scan all notes for links to this one
        // This is Phase 1 - we just warn, don't remove links yet
        let note_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        for other_path in vault.list_notes()? {
            if other_path == path {
                continue;
            }

            if let Ok(other_note) = vault.load_note(&other_path) {
                for link in other_note.links() {
                    // Check if link target matches our note
                    let link_target = link.target.trim();
                    if link_target == note_name
                        || link_target == path.to_string_lossy().trim_end_matches(".md")
                        || link_target == path.to_string_lossy().as_ref()
                    {
                        incoming_links.push((other_path.clone(), link.line));
                    }
                }
            }
        }
    }

    // Build warning message if there are incoming links
    let warning = if !incoming_links.is_empty() && !args.remove_links {
        Some(format!(
            "{} notes link to this note. Links will be broken: {}",
            incoming_links.len(),
            incoming_links
                .iter()
                .take(5)
                .map(|(p, l)| format!("{}:{}", p.display(), l))
                .collect::<Vec<_>>()
                .join(", ")
        ))
    } else {
        None
    };

    if args.dry_run {
        let mut changes = vec![format!("Delete: {}", path.display())];

        if args.remove_links && !incoming_links.is_empty() {
            for (link_path, line) in &incoming_links {
                changes.push(format!("Remove link in {}:{}", link_path.display(), line));
            }
        }

        let response = DryRunResponse {
            action: "delete".to_string(),
            path: path.to_string_lossy().to_string(),
            content: None,
            changes: Some(changes),
        };
        output.print(&response)?;
        return Ok(());
    }

    // TODO: Actually remove links if --remove-links is set
    // This requires editing other notes, which is more complex
    // For Phase 1, we just warn about broken links

    if !incoming_links.is_empty() && !args.no_propagate && !args.force {
        if let Some(ref warn) = warning {
            output.warn(warn);
        }
        // In non-force mode, we still delete but warn
        // A future enhancement could prompt for confirmation
    }

    // Delete the note
    vault.delete_note(&path)?;

    let response = DeleteResponse {
        path: path.to_string_lossy().to_string(),
        message: "Note deleted successfully".to_string(),
        warning,
    };
    output.print(&response)?;

    Ok(())
}
