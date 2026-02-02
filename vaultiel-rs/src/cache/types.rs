//! Cache data structures.

use crate::types::LinkContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Cache metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMeta {
    /// Cache format version.
    pub version: u32,
    /// Vault root path.
    pub vault_path: PathBuf,
    /// Last full index timestamp.
    pub last_full_index: Option<String>,
    /// Number of indexed notes.
    pub indexed_notes: usize,
    /// Vaultiel version that created this cache.
    pub vaultiel_version: String,
}

impl CacheMeta {
    pub fn new(vault_path: PathBuf) -> Self {
        Self {
            version: super::CACHE_VERSION,
            vault_path,
            last_full_index: None,
            indexed_notes: 0,
            vaultiel_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Cached note entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedNote {
    /// Relative path from vault root.
    pub path: PathBuf,
    /// File modification time (as unix timestamp for serialization).
    pub mtime: u64,
    /// File size in bytes.
    pub size: u64,
    /// Note name (without .md extension).
    pub name: String,
    /// Parsed frontmatter.
    pub frontmatter: Option<serde_json::Value>,
    /// Aliases from frontmatter.
    pub aliases: Vec<String>,
}

impl CachedNote {
    /// Check if the cached note is stale compared to the actual file.
    pub fn is_stale(&self, actual_mtime: SystemTime) -> bool {
        let actual_secs = actual_mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.mtime != actual_secs
    }
}

/// Cached link entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedLink {
    /// Source note path.
    pub from: PathBuf,
    /// Target (raw link text, may need resolution).
    pub target: String,
    /// Line number in source file.
    pub line: usize,
    /// Link context.
    pub context: LinkContext,
    /// Display alias if any.
    pub alias: Option<String>,
    /// Heading reference if any.
    pub heading: Option<String>,
    /// Block ID reference if any.
    pub block_id: Option<String>,
    /// Whether this is an embed.
    pub is_embed: bool,
}

/// Cached tag entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTag {
    /// Tag name (including #).
    pub name: String,
    /// Note path where this tag appears.
    pub note: PathBuf,
    /// Line number.
    pub line: usize,
}

/// Cached block ID entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedBlockId {
    /// Block ID (without ^).
    pub id: String,
    /// Note path.
    pub note: PathBuf,
    /// Line number.
    pub line: usize,
    /// Block type (paragraph, list-item, etc.).
    pub block_type: String,
}

/// Cached heading entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedHeading {
    /// Heading text.
    pub text: String,
    /// Heading level (1-6).
    pub level: u8,
    /// Note path.
    pub note: PathBuf,
    /// Line number.
    pub line: usize,
    /// Computed slug.
    pub slug: String,
}

/// The main note index.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NoteIndex {
    /// All notes keyed by relative path.
    pub notes: HashMap<PathBuf, CachedNote>,
}

impl NoteIndex {
    pub fn get(&self, path: &PathBuf) -> Option<&CachedNote> {
        self.notes.get(path)
    }

    pub fn insert(&mut self, note: CachedNote) {
        self.notes.insert(note.path.clone(), note);
    }

    pub fn remove(&mut self, path: &PathBuf) -> Option<CachedNote> {
        self.notes.remove(path)
    }

    pub fn len(&self) -> usize {
        self.notes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.notes.is_empty()
    }
}

/// The link index (outgoing links from each note).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinkIndex {
    /// Links grouped by source note path.
    pub by_source: HashMap<PathBuf, Vec<CachedLink>>,
}

impl LinkIndex {
    pub fn get_outgoing(&self, path: &PathBuf) -> &[CachedLink] {
        self.by_source.get(path).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn set_outgoing(&mut self, path: PathBuf, links: Vec<CachedLink>) {
        self.by_source.insert(path, links);
    }

    pub fn remove(&mut self, path: &PathBuf) {
        self.by_source.remove(path);
    }
}

/// The tag index.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TagIndex {
    /// Tags grouped by note path.
    pub by_note: HashMap<PathBuf, Vec<CachedTag>>,
    /// Tag counts for quick lookup.
    pub counts: HashMap<String, usize>,
}

impl TagIndex {
    pub fn set_tags(&mut self, path: PathBuf, tags: Vec<CachedTag>) {
        // Remove old counts
        if let Some(old_tags) = self.by_note.get(&path) {
            for tag in old_tags {
                if let Some(count) = self.counts.get_mut(&tag.name) {
                    *count = count.saturating_sub(1);
                }
            }
        }

        // Add new counts
        for tag in &tags {
            *self.counts.entry(tag.name.clone()).or_insert(0) += 1;
        }

        self.by_note.insert(path, tags);
    }

    pub fn remove(&mut self, path: &PathBuf) {
        if let Some(tags) = self.by_note.remove(path) {
            for tag in tags {
                if let Some(count) = self.counts.get_mut(&tag.name) {
                    *count = count.saturating_sub(1);
                }
            }
        }
    }

    pub fn get_tags(&self, path: &PathBuf) -> &[CachedTag] {
        self.by_note.get(path).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

/// The block ID index.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlockIndex {
    /// Blocks grouped by note path.
    pub by_note: HashMap<PathBuf, Vec<CachedBlockId>>,
}

impl BlockIndex {
    pub fn set_blocks(&mut self, path: PathBuf, blocks: Vec<CachedBlockId>) {
        self.by_note.insert(path, blocks);
    }

    pub fn remove(&mut self, path: &PathBuf) {
        self.by_note.remove(path);
    }

    pub fn get_blocks(&self, path: &PathBuf) -> &[CachedBlockId] {
        self.by_note.get(path).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

/// The heading index.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeadingIndex {
    /// Headings grouped by note path.
    pub by_note: HashMap<PathBuf, Vec<CachedHeading>>,
}

impl HeadingIndex {
    pub fn set_headings(&mut self, path: PathBuf, headings: Vec<CachedHeading>) {
        self.by_note.insert(path, headings);
    }

    pub fn remove(&mut self, path: &PathBuf) {
        self.by_note.remove(path);
    }

    pub fn get_headings(&self, path: &PathBuf) -> &[CachedHeading] {
        self.by_note.get(path).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

/// Cached task (simplified for storage).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTask {
    /// Note path.
    pub note: PathBuf,
    /// Line number.
    pub line: usize,
    /// Raw task line.
    pub raw: String,
    /// Task symbol.
    pub symbol: String,
    /// Task description.
    pub description: String,
    /// Indentation level.
    pub indent: usize,
    /// Parent task line (if nested).
    pub parent_line: Option<usize>,
    /// Scheduled date.
    pub scheduled: Option<String>,
    /// Due date.
    pub due: Option<String>,
    /// Done date.
    pub done: Option<String>,
    /// Priority.
    pub priority: Option<String>,
    /// Tags in the task.
    pub tags: Vec<String>,
    /// Block ID on the task.
    pub block_id: Option<String>,
}

/// The task index.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskIndex {
    /// Tasks grouped by note path.
    pub by_note: HashMap<PathBuf, Vec<CachedTask>>,
}

impl TaskIndex {
    pub fn set_tasks(&mut self, path: PathBuf, tasks: Vec<CachedTask>) {
        self.by_note.insert(path, tasks);
    }

    pub fn remove(&mut self, path: &PathBuf) {
        self.by_note.remove(path);
    }

    pub fn get_tasks(&self, path: &PathBuf) -> &[CachedTask] {
        self.by_note.get(path).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

/// Cache status information.
#[derive(Debug, Serialize)]
pub struct CacheStatus {
    pub vault: PathBuf,
    pub cache_path: PathBuf,
    pub indexed_notes: usize,
    pub last_full_index: Option<String>,
    pub stale_notes: usize,
    pub cache_size_bytes: u64,
    pub cache_version: u32,
    pub vaultiel_version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_note_is_stale() {
        let note = CachedNote {
            path: PathBuf::from("test.md"),
            mtime: 1000,
            size: 100,
            name: "test".to_string(),
            frontmatter: None,
            aliases: vec![],
        };

        let old_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000);
        let new_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(2000);

        assert!(!note.is_stale(old_time));
        assert!(note.is_stale(new_time));
    }

    #[test]
    fn test_note_index() {
        let mut index = NoteIndex::default();

        let note = CachedNote {
            path: PathBuf::from("test.md"),
            mtime: 1000,
            size: 100,
            name: "test".to_string(),
            frontmatter: None,
            aliases: vec![],
        };

        index.insert(note.clone());
        assert_eq!(index.len(), 1);
        assert!(index.get(&PathBuf::from("test.md")).is_some());

        index.remove(&PathBuf::from("test.md"));
        assert!(index.is_empty());
    }

    #[test]
    fn test_tag_index_counts() {
        let mut index = TagIndex::default();
        let path = PathBuf::from("test.md");

        let tags = vec![
            CachedTag { name: "#rust".to_string(), note: path.clone(), line: 1 },
            CachedTag { name: "#rust".to_string(), note: path.clone(), line: 5 },
            CachedTag { name: "#cli".to_string(), note: path.clone(), line: 10 },
        ];

        index.set_tags(path.clone(), tags);

        assert_eq!(index.counts.get("#rust"), Some(&2));
        assert_eq!(index.counts.get("#cli"), Some(&1));

        index.remove(&path);
        assert_eq!(index.counts.get("#rust"), Some(&0));
    }
}
