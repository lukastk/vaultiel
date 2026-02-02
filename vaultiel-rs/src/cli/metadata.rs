//! Metadata CLI commands.

use crate::cli::output::Output;
use crate::error::{ExitCode, Result, VaultError};
use crate::metadata::{find_by_id, get_metadata, init_metadata, VaultielMetadata};
use crate::vault::Vault;
use serde::Serialize;
use std::path::PathBuf;

/// Initialize metadata for a single note.
pub fn init_metadata_note(
    vault: &Vault,
    path: &str,
    force: bool,
    dry_run: bool,
    output: &Output,
) -> Result<ExitCode> {
    let note_path = vault.normalize_note_path(path);

    if !vault.note_exists(&note_path) {
        return Err(VaultError::NoteNotFound(note_path));
    }

    if dry_run {
        // Check if note already has metadata
        let existing = get_metadata(vault, &note_path)?;
        let result = InitMetadataResult {
            path: note_path,
            action: if existing.is_some() && !force {
                "skipped".to_string()
            } else {
                "would_initialize".to_string()
            },
            metadata: if existing.is_some() && !force {
                existing
            } else {
                Some(VaultielMetadata::new())
            },
        };
        output.print(&result)?;
        return Ok(ExitCode::Success);
    }

    let result = init_metadata(vault, &note_path, force)?;

    let init_result = InitMetadataResult {
        path: note_path,
        action: if result.is_some() {
            "initialized".to_string()
        } else {
            "skipped".to_string()
        },
        metadata: result.or_else(|| get_metadata(vault, &vault.normalize_note_path(path)).ok().flatten()),
    };

    output.print(&init_result)?;
    Ok(ExitCode::Success)
}

/// Initialize metadata for notes matching a glob pattern.
pub fn init_metadata_glob(
    vault: &Vault,
    pattern: &str,
    force: bool,
    dry_run: bool,
    output: &Output,
) -> Result<ExitCode> {
    let notes = vault.list_notes_matching(pattern)?;

    let mut results: Vec<InitMetadataResult> = Vec::new();
    let mut initialized = 0;
    let mut skipped = 0;

    for note_path in notes {
        if dry_run {
            let existing = get_metadata(vault, &note_path)?;
            let action = if existing.is_some() && !force {
                skipped += 1;
                "skipped"
            } else {
                initialized += 1;
                "would_initialize"
            };
            results.push(InitMetadataResult {
                path: note_path,
                action: action.to_string(),
                metadata: if existing.is_some() && !force {
                    existing
                } else {
                    Some(VaultielMetadata::new())
                },
            });
        } else {
            let result = init_metadata(vault, &note_path, force)?;
            if result.is_some() {
                initialized += 1;
                results.push(InitMetadataResult {
                    path: note_path,
                    action: "initialized".to_string(),
                    metadata: result,
                });
            } else {
                skipped += 1;
                results.push(InitMetadataResult {
                    path: note_path.clone(),
                    action: "skipped".to_string(),
                    metadata: get_metadata(vault, &note_path).ok().flatten(),
                });
            }
        }
    }

    let batch_result = BatchInitResult {
        results,
        initialized,
        skipped,
        total: initialized + skipped,
    };

    output.print(&batch_result)?;
    Ok(ExitCode::Success)
}

/// Find a note by its vaultiel ID.
pub fn get_by_id(vault: &Vault, id: &str, output: &Output) -> Result<ExitCode> {
    let result = find_by_id(vault, id)?;

    match result {
        Some(path) => {
            let found = GetByIdResult {
                id: id.to_string(),
                found: true,
                path: Some(path),
            };
            output.print(&found)?;
            Ok(ExitCode::Success)
        }
        None => {
            let not_found = GetByIdResult {
                id: id.to_string(),
                found: false,
                path: None,
            };
            output.print(&not_found)?;
            // Return success but with found=false in output
            Ok(ExitCode::Success)
        }
    }
}

/// Get metadata from a specific note.
pub fn get_note_metadata(vault: &Vault, path: &str, output: &Output) -> Result<ExitCode> {
    let note_path = vault.normalize_note_path(path);

    if !vault.note_exists(&note_path) {
        return Err(VaultError::NoteNotFound(note_path));
    }

    let metadata = get_metadata(vault, &note_path)?;

    let result = NoteMetadataResult {
        path: note_path,
        has_metadata: metadata.is_some(),
        metadata,
    };

    output.print(&result)?;
    Ok(ExitCode::Success)
}

#[derive(Debug, Serialize)]
struct InitMetadataResult {
    path: PathBuf,
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<VaultielMetadata>,
}

#[derive(Debug, Serialize)]
struct BatchInitResult {
    results: Vec<InitMetadataResult>,
    initialized: usize,
    skipped: usize,
    total: usize,
}

#[derive(Debug, Serialize)]
struct GetByIdResult {
    id: String,
    found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct NoteMetadataResult {
    path: PathBuf,
    has_metadata: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<VaultielMetadata>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args::OutputFormat;
    use crate::config::Config;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_vault() -> (TempDir, Vault) {
        let temp_dir = TempDir::new().unwrap();

        let note_content = r#"---
title: Test Note
---

# Test

Content.
"#;
        fs::write(temp_dir.path().join("test.md"), note_content).unwrap();

        let note_with_meta = r#"---
title: With Meta
vaultiel:
  id: "test-uuid-123"
  created: "2026-01-01T00:00:00Z"
---

# With Meta

Content.
"#;
        fs::write(temp_dir.path().join("with-meta.md"), note_with_meta).unwrap();

        let config = Config::default();
        let vault = Vault::new(temp_dir.path().to_path_buf(), config).unwrap();

        (temp_dir, vault)
    }

    #[test]
    fn test_init_metadata_note() {
        let (_temp_dir, vault) = create_test_vault();
        let output = Output::new(OutputFormat::Json, true);

        let result = init_metadata_note(&vault, "test", false, false, &output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_by_id() {
        let (_temp_dir, vault) = create_test_vault();
        let output = Output::new(OutputFormat::Json, true);

        let result = get_by_id(&vault, "test-uuid-123", &output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_note_metadata() {
        let (_temp_dir, vault) = create_test_vault();
        let output = Output::new(OutputFormat::Json, true);

        let result = get_note_metadata(&vault, "with-meta", &output);
        assert!(result.is_ok());
    }
}
