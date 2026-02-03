//! Cache index management.

use super::types::*;
use super::{acquire_lock, atomic_write, ensure_cache_dir, get_cache_dir, CACHE_VERSION};
use crate::config::{CacheConfig, TaskConfig};
use crate::error::{Result, VaultError};
use crate::parser::{
    parse_all_links, parse_block_ids, parse_headings, parse_tags, parse_tasks,
};
use crate::types::LinkContext;
use crate::vault::Vault;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The vault cache.
pub struct VaultCache {
    /// Cache directory path.
    cache_dir: PathBuf,
    /// Cache metadata.
    pub meta: CacheMeta,
    /// Note index.
    pub notes: NoteIndex,
    /// Link index.
    pub links: LinkIndex,
    /// Tag index.
    pub tags: TagIndex,
    /// Block ID index.
    pub blocks: BlockIndex,
    /// Heading index.
    pub headings: HeadingIndex,
    /// Task index.
    pub tasks: TaskIndex,
    /// Whether the cache has been modified and needs saving.
    dirty: bool,
}

impl VaultCache {
    /// Load cache from disk or create a new empty cache.
    pub fn load(vault_root: &Path, config: &CacheConfig) -> Result<Self> {
        let cache_dir = get_cache_dir(vault_root, config);

        if cache_dir.exists() {
            Self::load_from_dir(&cache_dir, vault_root)
        } else {
            Ok(Self::new(cache_dir, vault_root.to_path_buf()))
        }
    }

    /// Create a new empty cache.
    pub fn new(cache_dir: PathBuf, vault_root: PathBuf) -> Self {
        Self {
            cache_dir,
            meta: CacheMeta::new(vault_root),
            notes: NoteIndex::default(),
            links: LinkIndex::default(),
            tags: TagIndex::default(),
            blocks: BlockIndex::default(),
            headings: HeadingIndex::default(),
            tasks: TaskIndex::default(),
            dirty: false,
        }
    }

    /// Load cache from a directory.
    fn load_from_dir(cache_dir: &Path, vault_root: &Path) -> Result<Self> {
        let meta_path = cache_dir.join("meta.json");

        // Load metadata first to check version
        let meta: CacheMeta = if meta_path.exists() {
            let content = fs::read_to_string(&meta_path).map_err(|e| {
                VaultError::CacheError(format!("Failed to read cache metadata: {}", e))
            })?;
            serde_json::from_str(&content).map_err(|e| {
                VaultError::CacheError(format!("Failed to parse cache metadata: {}", e))
            })?
        } else {
            return Ok(Self::new(cache_dir.to_path_buf(), vault_root.to_path_buf()));
        };

        // Check version compatibility
        if meta.version != CACHE_VERSION {
            // Cache is outdated, return empty cache
            return Ok(Self::new(cache_dir.to_path_buf(), vault_root.to_path_buf()));
        }

        // Load all indices
        let notes = Self::load_json(cache_dir, "index.json")?;
        let links = Self::load_json(cache_dir, "links.json")?;
        let tags = Self::load_json(cache_dir, "tags.json")?;
        let blocks = Self::load_json(cache_dir, "blocks.json")?;
        let headings = Self::load_json(cache_dir, "headings.json")?;
        let tasks = Self::load_json(cache_dir, "tasks.json")?;

        Ok(Self {
            cache_dir: cache_dir.to_path_buf(),
            meta,
            notes,
            links,
            tags,
            blocks,
            headings,
            tasks,
            dirty: false,
        })
    }

    /// Load a JSON file from the cache directory.
    fn load_json<T: for<'de> serde::Deserialize<'de> + Default>(
        cache_dir: &Path,
        filename: &str,
    ) -> Result<T> {
        let path = cache_dir.join(filename);
        if path.exists() {
            let content = fs::read_to_string(&path).map_err(|e| {
                VaultError::CacheError(format!("Failed to read {}: {}", filename, e))
            })?;
            serde_json::from_str(&content).map_err(|e| {
                VaultError::CacheError(format!("Failed to parse {}: {}", filename, e))
            })
        } else {
            Ok(T::default())
        }
    }

    /// Save the cache to disk.
    pub fn save(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        ensure_cache_dir(&self.cache_dir)?;
        let _lock = acquire_lock(&self.cache_dir)?;

        // Update metadata
        self.meta.indexed_notes = self.notes.len();
        self.meta.last_full_index = Some(chrono::Utc::now().to_rfc3339());

        // Save all files atomically
        self.save_json("meta.json", &self.meta)?;
        self.save_json("index.json", &self.notes)?;
        self.save_json("links.json", &self.links)?;
        self.save_json("tags.json", &self.tags)?;
        self.save_json("blocks.json", &self.blocks)?;
        self.save_json("headings.json", &self.headings)?;
        self.save_json("tasks.json", &self.tasks)?;

        self.dirty = false;
        Ok(())
    }

    /// Save a JSON file to the cache directory.
    fn save_json<T: serde::Serialize>(&self, filename: &str, data: &T) -> Result<()> {
        let path = self.cache_dir.join(filename);
        let content = serde_json::to_string_pretty(data).map_err(|e| {
            VaultError::CacheError(format!("Failed to serialize {}: {}", filename, e))
        })?;
        atomic_write(&path, content.as_bytes())
    }

    /// Get the cache directory path.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Check if a note is stale (needs re-indexing).
    pub fn is_note_stale(&self, path: &PathBuf, vault_root: &Path) -> bool {
        let full_path = vault_root.join(path);
        match fs::metadata(&full_path) {
            Ok(metadata) => {
                let mtime = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                match self.notes.get(path) {
                    Some(cached) => cached.is_stale(mtime),
                    None => true, // Not in cache
                }
            }
            Err(_) => true, // File doesn't exist or error
        }
    }

    /// Find all stale notes in the vault.
    pub fn find_stale_notes(&self, vault: &Vault) -> Result<Vec<PathBuf>> {
        let mut stale = Vec::new();
        let all_notes = vault.list_notes()?;

        for path in all_notes {
            if self.is_note_stale(&path, &vault.root) {
                stale.push(path);
            }
        }

        Ok(stale)
    }

    /// Find notes that were deleted from the vault but still in cache.
    pub fn find_deleted_notes(&self, vault: &Vault) -> Result<Vec<PathBuf>> {
        let all_notes: HashSet<_> = vault.list_notes()?.into_iter().collect();
        let deleted: Vec<PathBuf> = self
            .notes
            .notes
            .keys()
            .filter(|p| !all_notes.contains(*p))
            .cloned()
            .collect();
        Ok(deleted)
    }

    /// Re-index a single note.
    pub fn index_note(&mut self, vault: &Vault, path: &PathBuf, task_config: &TaskConfig) -> Result<()> {
        let full_path = vault.root.join(path);
        let metadata = fs::metadata(&full_path).map_err(|_| {
            VaultError::NoteNotFound(path.clone())
        })?;

        let mtime = metadata
            .modified()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let content = fs::read_to_string(&full_path)?;
        let note = vault.load_note(path)?;

        // Extract note name
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        // Parse frontmatter
        let frontmatter = note.frontmatter().ok().flatten().map(|fm| {
            serde_json::to_value(&fm).unwrap_or(serde_json::Value::Null)
        });

        // Extract aliases
        let aliases = note
            .frontmatter()
            .ok()
            .flatten()
            .and_then(|fm| {
                fm.get("aliases")
                    .and_then(|v| v.as_sequence())
                    .map(|seq| {
                        seq.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
            })
            .unwrap_or_default();

        // Create cached note
        let cached_note = CachedNote {
            path: path.clone(),
            mtime,
            size: metadata.len(),
            name,
            frontmatter,
            aliases,
        };

        // Index the note
        self.notes.insert(cached_note);

        // Index links
        let wikilinks = parse_all_links(&content);
        let cached_links: Vec<CachedLink> = wikilinks
            .into_iter()
            .map(|link| CachedLink {
                from: path.clone(),
                target: link.target,
                line: link.line,
                context: LinkContext::Body, // Default context for body links
                alias: link.alias,
                heading: link.heading,
                block_id: link.block_id,
                is_embed: link.embed,
            })
            .collect();
        self.links.set_outgoing(path.clone(), cached_links);

        // Index tags
        let tags = parse_tags(&content);
        let cached_tags: Vec<CachedTag> = tags
            .into_iter()
            .map(|tag| CachedTag {
                name: tag.name,
                note: path.clone(),
                line: tag.line,
            })
            .collect();
        self.tags.set_tags(path.clone(), cached_tags);

        // Index block IDs
        let blocks = parse_block_ids(&content);
        let cached_blocks: Vec<CachedBlockId> = blocks
            .into_iter()
            .map(|block| CachedBlockId {
                id: block.id,
                note: path.clone(),
                line: block.line,
                block_type: format!("{:?}", block.block_type).to_lowercase(),
            })
            .collect();
        self.blocks.set_blocks(path.clone(), cached_blocks);

        // Index headings
        let headings = parse_headings(&content);
        let cached_headings: Vec<CachedHeading> = headings
            .into_iter()
            .map(|h| CachedHeading {
                text: h.text,
                level: h.level,
                note: path.clone(),
                line: h.line,
                slug: h.slug,
            })
            .collect();
        self.headings.set_headings(path.clone(), cached_headings);

        // Index tasks
        let tasks = parse_tasks(&content, path, task_config);
        let cached_tasks: Vec<CachedTask> = tasks
            .into_iter()
            .map(|task| CachedTask {
                note: path.clone(),
                line: task.location.line,
                raw: task.raw,
                symbol: task.symbol,
                description: task.description,
                indent: task.indent,
                parent_line: task.parent_line,
                scheduled: task.scheduled,
                due: task.due,
                done: task.done,
                priority: task.priority.map(|p| format!("{:?}", p).to_lowercase()),
                tags: task.tags,
                block_id: task.block_id,
            })
            .collect();
        self.tasks.set_tasks(path.clone(), cached_tasks);

        self.dirty = true;
        Ok(())
    }

    /// Remove a note from the cache.
    pub fn remove_note(&mut self, path: &PathBuf) {
        self.notes.remove(path);
        self.links.remove(path);
        self.tags.remove(path);
        self.blocks.remove(path);
        self.headings.remove(path);
        self.tasks.remove(path);
        self.dirty = true;
    }

    /// Perform incremental update: re-index stale notes, remove deleted notes.
    pub fn update(&mut self, vault: &Vault, task_config: &TaskConfig, verbose: bool) -> Result<usize> {
        let stale = self.find_stale_notes(vault)?;
        let deleted = self.find_deleted_notes(vault)?;

        let total = stale.len() + deleted.len();

        // Remove deleted notes
        for path in &deleted {
            if verbose {
                eprintln!("Removing from cache: {}", path.display());
            }
            self.remove_note(path);
        }

        // Re-index stale notes
        for path in &stale {
            if verbose {
                eprintln!("Indexing: {}", path.display());
            }
            if let Err(e) = self.index_note(vault, path, task_config) {
                if verbose {
                    eprintln!("Warning: Failed to index {}: {}", path.display(), e);
                }
            }
        }

        if total > 0 {
            self.save()?;
        }

        Ok(total)
    }

    /// Full rebuild: clear and re-index everything.
    pub fn rebuild(&mut self, vault: &Vault, task_config: &TaskConfig, verbose: bool) -> Result<usize> {
        // Clear all indices
        self.notes = NoteIndex::default();
        self.links = LinkIndex::default();
        self.tags = TagIndex::default();
        self.blocks = BlockIndex::default();
        self.headings = HeadingIndex::default();
        self.tasks = TaskIndex::default();

        let all_notes = vault.list_notes()?;
        let total = all_notes.len();

        for (i, path) in all_notes.iter().enumerate() {
            if verbose {
                eprintln!("Indexing [{}/{}]: {}", i + 1, total, path.display());
            }
            if let Err(e) = self.index_note(vault, path, task_config) {
                if verbose {
                    eprintln!("Warning: Failed to index {}: {}", path.display(), e);
                }
            }
        }

        self.dirty = true;
        self.save()?;

        Ok(total)
    }

    /// Ensure the cache is up-to-date.
    /// Returns true if the cache was already valid and usable.
    /// Returns false if we had to build/update the cache.
    pub fn ensure_current(&mut self, vault: &Vault, task_config: &TaskConfig) -> Result<bool> {
        // If cache is empty, do a full rebuild
        if self.notes.is_empty() {
            self.rebuild(vault, task_config, false)?;
            return Ok(false);
        }

        // Otherwise do incremental update
        let updated = self.update(vault, task_config, false)?;
        Ok(updated == 0)
    }

    /// Check if the cache has any indexed notes.
    pub fn has_data(&self) -> bool {
        !self.notes.is_empty()
    }

    /// Get cache status.
    pub fn status(&self, vault: &Vault) -> Result<CacheStatus> {
        let stale_notes = self.find_stale_notes(vault)?.len();

        // Calculate cache size
        let cache_size_bytes = if self.cache_dir.exists() {
            fs::read_dir(&self.cache_dir)
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter_map(|e| e.metadata().ok())
                        .map(|m| m.len())
                        .sum()
                })
                .unwrap_or(0)
        } else {
            0
        };

        Ok(CacheStatus {
            vault: vault.root.clone(),
            cache_path: self.cache_dir.clone(),
            indexed_notes: self.notes.len(),
            last_full_index: self.meta.last_full_index.clone(),
            stale_notes,
            cache_size_bytes,
            cache_version: self.meta.version,
            vaultiel_version: self.meta.vaultiel_version.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_vault() -> (TempDir, Vault) {
        let temp_dir = TempDir::new().unwrap();

        // Create a test note
        let note_path = temp_dir.path().join("test.md");
        fs::write(
            &note_path,
            r#"---
title: Test Note
aliases:
  - test-alias
---

# Test Heading

Some content with a [[link]] and a #tag.

- [ ] A task 📅 2026-02-10

A paragraph with ^block-id
"#,
        )
        .unwrap();

        let config = crate::config::Config::default();
        let vault = Vault::new(temp_dir.path().to_path_buf(), config).unwrap();

        (temp_dir, vault)
    }

    #[test]
    fn test_cache_new() {
        let cache_dir = PathBuf::from("/tmp/test-cache");
        let vault_root = PathBuf::from("/tmp/test-vault");

        let cache = VaultCache::new(cache_dir.clone(), vault_root.clone());

        assert_eq!(cache.cache_dir(), cache_dir);
        assert_eq!(cache.meta.vault_path, vault_root);
        assert_eq!(cache.meta.version, CACHE_VERSION);
    }

    #[test]
    fn test_index_note() {
        let (_temp_dir, vault) = create_test_vault();
        let cache_dir = TempDir::new().unwrap();

        let mut cache = VaultCache::new(
            cache_dir.path().to_path_buf(),
            vault.root.clone(),
        );

        let task_config = TaskConfig::default();
        let path = PathBuf::from("test.md");

        cache.index_note(&vault, &path, &task_config).unwrap();

        // Check note was indexed
        assert!(cache.notes.get(&path).is_some());
        let cached_note = cache.notes.get(&path).unwrap();
        assert_eq!(cached_note.name, "test");
        assert!(cached_note.aliases.contains(&"test-alias".to_string()));

        // Check links
        let links = cache.links.get_outgoing(&path);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "link");

        // Check tags
        let tags = cache.tags.get_tags(&path);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "#tag");

        // Check blocks
        let blocks = cache.blocks.get_blocks(&path);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].id, "block-id");

        // Check headings
        let headings = cache.headings.get_headings(&path);
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text, "Test Heading");

        // Check tasks
        let tasks = cache.tasks.get_tasks(&path);
        assert_eq!(tasks.len(), 1);
        assert!(tasks[0].description.contains("A task"));
    }

    #[test]
    fn test_cache_save_and_load() {
        let (_temp_dir, vault) = create_test_vault();
        let cache_dir = TempDir::new().unwrap();

        let task_config = TaskConfig::default();
        let path = PathBuf::from("test.md");

        // Create and save cache
        {
            let mut cache = VaultCache::new(
                cache_dir.path().to_path_buf(),
                vault.root.clone(),
            );
            cache.index_note(&vault, &path, &task_config).unwrap();
            cache.save().unwrap();
        }

        // Load cache
        let cache = VaultCache::load_from_dir(cache_dir.path(), &vault.root).unwrap();

        assert!(cache.notes.get(&path).is_some());
        assert_eq!(cache.notes.len(), 1);
    }

    #[test]
    fn test_remove_note() {
        let (_temp_dir, vault) = create_test_vault();
        let cache_dir = TempDir::new().unwrap();

        let mut cache = VaultCache::new(
            cache_dir.path().to_path_buf(),
            vault.root.clone(),
        );

        let task_config = TaskConfig::default();
        let path = PathBuf::from("test.md");

        cache.index_note(&vault, &path, &task_config).unwrap();
        assert!(cache.notes.get(&path).is_some());

        cache.remove_note(&path);
        assert!(cache.notes.get(&path).is_none());
        assert!(cache.links.get_outgoing(&path).is_empty());
        assert!(cache.tags.get_tags(&path).is_empty());
    }
}
