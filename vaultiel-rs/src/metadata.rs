//! Vaultiel metadata field handling.
//!
//! Notes can have a `vaultiel` field in frontmatter for stable identification:
//! ```yaml
//! vaultiel:
//!   id: "550e8400-e29b-41d4-a716-446655440000"
//!   created: "2026-02-02T18:30:00Z"
//! ```

use crate::error::Result;
use crate::vault::Vault;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Vaultiel metadata stored in note frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultielMetadata {
    /// Unique identifier for the note (UUID v4).
    pub id: String,
    /// Creation timestamp (ISO 8601).
    pub created: String,
}

impl VaultielMetadata {
    /// Create new metadata with a fresh UUID and current timestamp.
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created: Utc::now().to_rfc3339(),
        }
    }

    /// Create metadata with a specific ID (for testing).
    pub fn with_id(id: &str) -> Self {
        Self {
            id: id.to_string(),
            created: Utc::now().to_rfc3339(),
        }
    }

    /// Parse metadata from a YAML value.
    pub fn from_yaml(value: &serde_yaml::Value) -> Option<Self> {
        if let serde_yaml::Value::Mapping(map) = value {
            let id = map
                .get(&serde_yaml::Value::String("id".to_string()))?
                .as_str()?
                .to_string();
            let created = map
                .get(&serde_yaml::Value::String("created".to_string()))?
                .as_str()?
                .to_string();
            Some(Self { id, created })
        } else {
            None
        }
    }

    /// Convert to YAML value.
    pub fn to_yaml(&self) -> serde_yaml::Value {
        let mut map = serde_yaml::Mapping::new();
        map.insert(
            serde_yaml::Value::String("id".to_string()),
            serde_yaml::Value::String(self.id.clone()),
        );
        map.insert(
            serde_yaml::Value::String("created".to_string()),
            serde_yaml::Value::String(self.created.clone()),
        );
        serde_yaml::Value::Mapping(map)
    }
}

impl Default for VaultielMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize vaultiel metadata for a note.
/// If the note already has metadata, it is preserved unless `force` is true.
pub fn init_metadata(vault: &Vault, path: &Path, force: bool) -> Result<Option<VaultielMetadata>> {
    let mut note = vault.load_note(path)?;

    // Check if metadata already exists
    if let Ok(Some(fm)) = note.frontmatter() {
        if fm.get("vaultiel").is_some() && !force {
            // Already has metadata, return None to indicate no change
            return Ok(None);
        }
    }

    // Create new metadata
    let metadata = VaultielMetadata::new();

    // Get current frontmatter or create empty one
    let mut frontmatter = note.frontmatter().ok().flatten().unwrap_or_else(|| {
        serde_yaml::Mapping::new().into()
    });

    // Add vaultiel field
    if let serde_yaml::Value::Mapping(ref mut map) = frontmatter {
        map.insert(
            serde_yaml::Value::String("vaultiel".to_string()),
            metadata.to_yaml(),
        );
    }

    // Update the note with new frontmatter
    let new_content = crate::parser::update_frontmatter(&note.content, &frontmatter)?;
    note.content = new_content;
    vault.save_note(&note)?;

    Ok(Some(metadata))
}

/// Get vaultiel metadata from a note.
pub fn get_metadata(vault: &Vault, path: &Path) -> Result<Option<VaultielMetadata>> {
    let note = vault.load_note(path)?;

    if let Ok(Some(fm)) = note.frontmatter() {
        if let Some(vaultiel) = fm.get("vaultiel") {
            return Ok(VaultielMetadata::from_yaml(vaultiel));
        }
    }

    Ok(None)
}

/// Find a note by its vaultiel ID.
pub fn find_by_id(vault: &Vault, id: &str) -> Result<Option<PathBuf>> {
    let notes = vault.list_notes()?;

    for note_path in notes {
        if let Ok(Some(metadata)) = get_metadata(vault, &note_path) {
            if metadata.id == id {
                return Ok(Some(note_path));
            }
        }
    }

    Ok(None)
}

/// Extract the vaultiel ID from a note's frontmatter, if present.
pub fn extract_id(vault: &Vault, path: &Path) -> Result<Option<String>> {
    get_metadata(vault, path).map(|m| m.map(|meta| meta.id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_vault() -> (TempDir, Vault) {
        let temp_dir = TempDir::new().unwrap();

        // Create a test note without metadata
        let note_content = r#"---
title: Test Note
tags:
  - rust
---

# Test

Some content.
"#;
        fs::write(temp_dir.path().join("test.md"), note_content).unwrap();

        // Create a note with existing metadata
        let note_with_meta = r#"---
title: With Meta
vaultiel:
  id: "existing-uuid"
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
    fn test_metadata_new() {
        let meta = VaultielMetadata::new();
        assert!(!meta.id.is_empty());
        assert!(!meta.created.is_empty());
        // Verify it's a valid UUID
        assert!(Uuid::parse_str(&meta.id).is_ok());
    }

    #[test]
    fn test_metadata_to_yaml() {
        let meta = VaultielMetadata::with_id("test-id");
        let yaml = meta.to_yaml();

        if let serde_yaml::Value::Mapping(map) = yaml {
            assert_eq!(
                map.get(&serde_yaml::Value::String("id".to_string())),
                Some(&serde_yaml::Value::String("test-id".to_string()))
            );
        } else {
            panic!("Expected mapping");
        }
    }

    #[test]
    fn test_init_metadata() {
        let (_temp_dir, vault) = create_test_vault();
        let path = PathBuf::from("test.md");

        let result = init_metadata(&vault, &path, false).unwrap();
        assert!(result.is_some());

        // Verify metadata was added
        let meta = get_metadata(&vault, &path).unwrap();
        assert!(meta.is_some());
    }

    #[test]
    fn test_init_metadata_preserves_existing() {
        let (_temp_dir, vault) = create_test_vault();
        let path = PathBuf::from("with-meta.md");

        // Should return None when metadata exists
        let result = init_metadata(&vault, &path, false).unwrap();
        assert!(result.is_none());

        // Verify original metadata preserved
        let meta = get_metadata(&vault, &path).unwrap().unwrap();
        assert_eq!(meta.id, "existing-uuid");
    }

    #[test]
    fn test_init_metadata_force() {
        let (_temp_dir, vault) = create_test_vault();
        let path = PathBuf::from("with-meta.md");

        // Force should replace existing metadata
        let result = init_metadata(&vault, &path, true).unwrap();
        assert!(result.is_some());

        // Verify new metadata
        let meta = get_metadata(&vault, &path).unwrap().unwrap();
        assert_ne!(meta.id, "existing-uuid");
    }

    #[test]
    fn test_find_by_id() {
        let (_temp_dir, vault) = create_test_vault();

        let result = find_by_id(&vault, "existing-uuid").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), PathBuf::from("with-meta.md"));

        let not_found = find_by_id(&vault, "nonexistent").unwrap();
        assert!(not_found.is_none());
    }
}
