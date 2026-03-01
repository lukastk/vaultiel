//! Vault representation and operations.

use crate::error::{Result, VaultError};
use crate::note::{Note, NoteInfo};
use glob::glob;
use std::path::{Path, PathBuf};

/// Represents an Obsidian vault.
#[derive(Debug, Clone)]
pub struct Vault {
    /// Root path of the vault.
    pub root: PathBuf,
}

impl Vault {
    /// Create a new vault instance.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();

        if !root.is_dir() {
            return Err(VaultError::VaultNotFound(root));
        }

        Ok(Self { root })
    }

    /// Get the full path to a note.
    pub fn note_path(&self, relative_path: &Path) -> PathBuf {
        self.root.join(relative_path)
    }

    /// Normalize a note path (add .md extension if needed).
    pub fn normalize_note_path(&self, path: &str) -> PathBuf {
        let path = path.trim();
        if path.ends_with(".md") {
            PathBuf::from(path)
        } else {
            PathBuf::from(format!("{}.md", path))
        }
    }

    /// Check if a note exists.
    pub fn note_exists(&self, relative_path: &Path) -> bool {
        self.note_path(relative_path).is_file()
    }

    /// Load a note from the vault.
    pub fn load_note(&self, relative_path: &Path) -> Result<Note> {
        if !self.note_exists(relative_path) {
            return Err(VaultError::NoteNotFound(relative_path.to_path_buf()));
        }
        Note::load(&self.root, relative_path)
    }

    /// Save a note to the vault.
    pub fn save_note(&self, note: &Note) -> Result<()> {
        note.save(&self.root)
    }

    /// Create a new note.
    pub fn create_note(&self, relative_path: &Path, content: &str) -> Result<Note> {
        if self.note_exists(relative_path) {
            return Err(VaultError::NoteAlreadyExists(relative_path.to_path_buf()));
        }

        let note = Note::new(relative_path, content);
        self.save_note(&note)?;
        Ok(note)
    }

    /// Set the raw content of an existing note (replaces everything including frontmatter).
    pub fn set_raw_content(&self, relative_path: &Path, content: &str) -> Result<()> {
        if !self.note_exists(relative_path) {
            return Err(VaultError::NoteNotFound(relative_path.to_path_buf()));
        }

        let note = Note::new(relative_path, content);
        self.save_note(&note)
    }

    /// Delete a note.
    pub fn delete_note(&self, relative_path: &Path) -> Result<()> {
        if !self.note_exists(relative_path) {
            return Err(VaultError::NoteNotFound(relative_path.to_path_buf()));
        }

        let full_path = self.note_path(relative_path);
        std::fs::remove_file(full_path)?;
        Ok(())
    }

    /// Rename a note.
    pub fn rename_note(&self, from: &Path, to: &Path) -> Result<()> {
        if !self.note_exists(from) {
            return Err(VaultError::NoteNotFound(from.to_path_buf()));
        }

        if self.note_exists(to) {
            return Err(VaultError::NoteAlreadyExists(to.to_path_buf()));
        }

        let from_full = self.note_path(from);
        let to_full = self.note_path(to);

        // Ensure target directory exists
        if let Some(parent) = to_full.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::rename(from_full, to_full)?;
        Ok(())
    }

    /// List all markdown files in the vault.
    pub fn list_notes(&self) -> Result<Vec<PathBuf>> {
        let pattern = self.root.join("**/*.md");
        let pattern_str = pattern.to_string_lossy();

        let mut notes = Vec::new();

        for entry in glob(&pattern_str)? {
            match entry {
                Ok(path) => {
                    // Get relative path
                    if let Ok(relative) = path.strip_prefix(&self.root) {
                        // Skip hidden files and directories
                        if !relative
                            .components()
                            .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
                        {
                            notes.push(relative.to_path_buf());
                        }
                    }
                }
                Err(e) => {
                    // Log but continue on glob errors
                    eprintln!("Warning: glob error: {}", e);
                }
            }
        }

        // Sort by path
        notes.sort();

        Ok(notes)
    }

    /// List notes matching a glob pattern.
    pub fn list_notes_matching(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let full_pattern = self.root.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();

        let mut notes = Vec::new();

        for entry in glob(&pattern_str)? {
            match entry {
                Ok(path) => {
                    if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                        if let Ok(relative) = path.strip_prefix(&self.root) {
                            notes.push(relative.to_path_buf());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: glob error: {}", e);
                }
            }
        }

        notes.sort();
        Ok(notes)
    }

    /// Get note info for a path.
    pub fn note_info(&self, relative_path: &Path) -> Result<NoteInfo> {
        NoteInfo::from_path(&self.root, relative_path)
    }

    /// Resolve a note name to a path.
    ///
    /// Handles:
    /// - Exact path matches
    /// - Note name without extension
    /// - Aliases (requires loading and parsing notes)
    pub fn resolve_note(&self, query: &str) -> Result<PathBuf> {
        let normalized = self.normalize_note_path(query);

        // First try exact match
        if self.note_exists(&normalized) {
            return Ok(normalized);
        }

        // Try without .md in case query already had it
        let query_path = PathBuf::from(query);
        if self.note_exists(&query_path) {
            return Ok(query_path);
        }

        // Search for notes matching the name
        let notes = self.list_notes()?;
        let query_lower = query.to_lowercase();

        let mut matches: Vec<PathBuf> = Vec::new();

        for note_path in notes {
            let name = note_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            if name.to_lowercase() == query_lower {
                matches.push(note_path.clone());
                continue;
            }

            // Check aliases (this is expensive - loads and parses the note)
            if let Ok(note) = self.load_note(&note_path) {
                if let Ok(Some(fm)) = note.frontmatter() {
                    if let Some(aliases) = fm.get("aliases") {
                        if let Some(arr) = aliases.as_sequence() {
                            for alias in arr {
                                if let Some(alias_str) = alias.as_str() {
                                    if alias_str.to_lowercase() == query_lower {
                                        matches.push(note_path.clone());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        match matches.len() {
            0 => Err(VaultError::NoteNotFound(PathBuf::from(query))),
            1 => Ok(matches.into_iter().next().unwrap()),
            _ => Err(VaultError::AmbiguousResolution {
                query: query.to_string(),
                count: matches.len(),
                matches,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_vault() -> (TempDir, Vault) {
        let dir = TempDir::new().unwrap();
        let vault = Vault::new(dir.path()).unwrap();
        (dir, vault)
    }

    #[test]
    fn test_create_and_load_note() {
        let (_dir, vault) = setup_test_vault();

        let path = PathBuf::from("test.md");
        vault.create_note(&path, "Hello, world!").unwrap();

        assert!(vault.note_exists(&path));

        let note = vault.load_note(&path).unwrap();
        assert_eq!(note.content, "Hello, world!");
    }

    #[test]
    fn test_create_note_in_subdirectory() {
        let (_dir, vault) = setup_test_vault();

        let path = PathBuf::from("sub/folder/note.md");
        vault.create_note(&path, "Nested note").unwrap();

        assert!(vault.note_exists(&path));
    }

    #[test]
    fn test_create_existing_note_fails() {
        let (_dir, vault) = setup_test_vault();

        let path = PathBuf::from("test.md");
        vault.create_note(&path, "First").unwrap();

        let result = vault.create_note(&path, "Second");
        assert!(matches!(result, Err(VaultError::NoteAlreadyExists(_))));
    }

    #[test]
    fn test_set_raw_content() {
        let (_dir, vault) = setup_test_vault();

        let path = PathBuf::from("test.md");
        vault.create_note(&path, "---\ntitle: Old\n---\nOld body").unwrap();

        let new_content = "---\ntitle: New\n---\nNew body";
        vault.set_raw_content(&path, new_content).unwrap();

        let note = vault.load_note(&path).unwrap();
        assert_eq!(note.content, new_content);
    }

    #[test]
    fn test_set_raw_content_nonexistent_fails() {
        let (_dir, vault) = setup_test_vault();

        let path = PathBuf::from("nonexistent.md");
        let result = vault.set_raw_content(&path, "content");
        assert!(matches!(result, Err(VaultError::NoteNotFound(_))));
    }

    #[test]
    fn test_delete_note() {
        let (_dir, vault) = setup_test_vault();

        let path = PathBuf::from("test.md");
        vault.create_note(&path, "Content").unwrap();
        assert!(vault.note_exists(&path));

        vault.delete_note(&path).unwrap();
        assert!(!vault.note_exists(&path));
    }

    #[test]
    fn test_delete_nonexistent_note_fails() {
        let (_dir, vault) = setup_test_vault();

        let path = PathBuf::from("nonexistent.md");
        let result = vault.delete_note(&path);
        assert!(matches!(result, Err(VaultError::NoteNotFound(_))));
    }

    #[test]
    fn test_rename_note() {
        let (_dir, vault) = setup_test_vault();

        let from = PathBuf::from("old.md");
        let to = PathBuf::from("new.md");

        vault.create_note(&from, "Content").unwrap();
        vault.rename_note(&from, &to).unwrap();

        assert!(!vault.note_exists(&from));
        assert!(vault.note_exists(&to));
    }

    #[test]
    fn test_list_notes() {
        let (_dir, vault) = setup_test_vault();

        vault.create_note(&PathBuf::from("a.md"), "A").unwrap();
        vault.create_note(&PathBuf::from("b.md"), "B").unwrap();
        vault
            .create_note(&PathBuf::from("sub/c.md"), "C")
            .unwrap();

        let notes = vault.list_notes().unwrap();
        assert_eq!(notes.len(), 3);
    }

    #[test]
    fn test_normalize_note_path() {
        let (_dir, vault) = setup_test_vault();

        assert_eq!(
            vault.normalize_note_path("note"),
            PathBuf::from("note.md")
        );
        assert_eq!(
            vault.normalize_note_path("note.md"),
            PathBuf::from("note.md")
        );
        assert_eq!(
            vault.normalize_note_path("folder/note"),
            PathBuf::from("folder/note.md")
        );
    }

    #[test]
    fn test_resolve_note_exact() {
        let (_dir, vault) = setup_test_vault();

        vault
            .create_note(&PathBuf::from("test.md"), "Content")
            .unwrap();

        let resolved = vault.resolve_note("test").unwrap();
        assert_eq!(resolved, PathBuf::from("test.md"));
    }

    #[test]
    fn test_resolve_note_by_alias() {
        let (_dir, vault) = setup_test_vault();

        let content = "---\naliases:\n  - myalias\n---\nContent";
        vault
            .create_note(&PathBuf::from("actual-name.md"), content)
            .unwrap();

        let resolved = vault.resolve_note("myalias").unwrap();
        assert_eq!(resolved, PathBuf::from("actual-name.md"));
    }
}
