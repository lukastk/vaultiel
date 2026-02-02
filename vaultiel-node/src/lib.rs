//! Node.js bindings for Vaultiel.
//!
//! Provides access to Vaultiel's vault operations from Node.js/TypeScript.

#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use vaultiel::config::Config;
use vaultiel::graph::LinkGraph;
use vaultiel::metadata::{find_by_id, get_metadata, init_metadata};
use vaultiel::parser::{parse_all_links, parse_block_ids, parse_headings, parse_tags, parse_tasks};
use vaultiel::Vault;

// ============================================================================
// Types for JavaScript
// ============================================================================

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsNoteInfo {
    pub path: String,
    pub name: String,
    pub modified: Option<String>,
    pub created: Option<String>,
    pub size_bytes: u32,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsLink {
    pub target: String,
    pub alias: Option<String>,
    pub heading: Option<String>,
    pub block_id: Option<String>,
    pub embed: bool,
    pub line: u32,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsTag {
    pub name: String,
    pub line: u32,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsHeading {
    pub text: String,
    pub level: u32,
    pub line: u32,
    pub slug: String,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsBlockId {
    pub id: String,
    pub line: u32,
    pub block_type: String,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsTask {
    pub file: String,
    pub line: u32,
    pub raw: String,
    pub symbol: String,
    pub description: String,
    pub indent: u32,
    pub scheduled: Option<String>,
    pub due: Option<String>,
    pub done: Option<String>,
    pub priority: Option<String>,
    pub tags: Vec<String>,
    pub block_id: Option<String>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsVaultielMetadata {
    pub id: String,
    pub created: String,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsLinkRef {
    pub from: String,
    pub line: u32,
    pub context: String,
    pub alias: Option<String>,
    pub heading: Option<String>,
    pub block_id: Option<String>,
    pub embed: bool,
}

// ============================================================================
// Vault Class
// ============================================================================

#[napi]
pub struct JsVault {
    vault: Vault,
}

#[napi]
impl JsVault {
    /// Open a vault at the specified path.
    #[napi(constructor)]
    pub fn new(path: String) -> Result<Self> {
        let config = Config::default();
        let vault = Vault::new(PathBuf::from(path), config)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { vault })
    }

    /// Get the vault root path.
    #[napi(getter)]
    pub fn root(&self) -> String {
        self.vault.root.to_string_lossy().to_string()
    }

    /// List all notes in the vault.
    #[napi]
    pub fn list_notes(&self) -> Result<Vec<String>> {
        self.vault
            .list_notes()
            .map(|notes| notes.into_iter().map(|p| p.to_string_lossy().to_string()).collect())
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// List notes matching a glob pattern.
    #[napi]
    pub fn list_notes_matching(&self, pattern: String) -> Result<Vec<String>> {
        self.vault
            .list_notes_matching(&pattern)
            .map(|notes| notes.into_iter().map(|p| p.to_string_lossy().to_string()).collect())
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Check if a note exists.
    #[napi]
    pub fn note_exists(&self, path: String) -> bool {
        let note_path = self.vault.normalize_note_path(&path);
        self.vault.note_exists(&note_path)
    }

    /// Get note content.
    #[napi]
    pub fn get_content(&self, path: String) -> Result<String> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(note.content)
    }

    /// Get note body (content without frontmatter).
    #[napi]
    pub fn get_body(&self, path: String) -> Result<String> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(note.body().to_string())
    }

    /// Get note frontmatter as JSON.
    #[napi]
    pub fn get_frontmatter(&self, path: String) -> Result<Option<String>> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        match note.frontmatter() {
            Ok(Some(fm)) => {
                let json = serde_json::to_string(&fm)
                    .map_err(|e| Error::from_reason(e.to_string()))?;
                Ok(Some(json))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    /// Create a new note.
    #[napi]
    pub fn create_note(&self, path: String, content: String) -> Result<()> {
        let note_path = self.vault.normalize_note_path(&path);
        self.vault.create_note(&note_path, &content)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(())
    }

    /// Delete a note.
    #[napi]
    pub fn delete_note(&self, path: String) -> Result<()> {
        let note_path = self.vault.normalize_note_path(&path);
        self.vault.delete_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Rename a note (without link propagation).
    #[napi]
    pub fn rename_note(&self, from: String, to: String) -> Result<()> {
        let from_path = self.vault.normalize_note_path(&from);
        let to_path = self.vault.normalize_note_path(&to);
        self.vault.rename_note(&from_path, &to_path)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Resolve a note name or alias to a path.
    #[napi]
    pub fn resolve_note(&self, query: String) -> Result<String> {
        self.vault.resolve_note(&query)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    // ========================================================================
    // Parsing
    // ========================================================================

    /// Parse links from a note.
    #[napi]
    pub fn get_links(&self, path: String) -> Result<Vec<JsLink>> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let links = parse_all_links(&note.content);
        Ok(links
            .into_iter()
            .map(|l| JsLink {
                target: l.target,
                alias: l.alias,
                heading: l.heading,
                block_id: l.block_id,
                embed: l.embed,
                line: l.line as u32,
            })
            .collect())
    }

    /// Parse tags from a note.
    #[napi]
    pub fn get_tags(&self, path: String) -> Result<Vec<JsTag>> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let tags = parse_tags(&note.content);
        Ok(tags
            .into_iter()
            .map(|t| JsTag {
                name: t.name,
                line: t.line as u32,
            })
            .collect())
    }

    /// Parse headings from a note.
    #[napi]
    pub fn get_headings(&self, path: String) -> Result<Vec<JsHeading>> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let headings = parse_headings(&note.content);
        Ok(headings
            .into_iter()
            .map(|h| JsHeading {
                text: h.text,
                level: h.level as u32,
                line: h.line as u32,
                slug: h.slug,
            })
            .collect())
    }

    /// Parse block IDs from a note.
    #[napi]
    pub fn get_block_ids(&self, path: String) -> Result<Vec<JsBlockId>> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let blocks = parse_block_ids(&note.content);
        Ok(blocks
            .into_iter()
            .map(|b| JsBlockId {
                id: b.id,
                line: b.line as u32,
                block_type: format!("{:?}", b.block_type).to_lowercase(),
            })
            .collect())
    }

    /// Parse tasks from a note.
    #[napi]
    pub fn get_tasks(&self, path: String) -> Result<Vec<JsTask>> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let task_config = vaultiel::config::TaskConfig::default();
        let tasks = parse_tasks(&note.content, &note_path, &task_config);

        Ok(tasks
            .into_iter()
            .map(|t| JsTask {
                file: t.location.file.to_string_lossy().to_string(),
                line: t.location.line as u32,
                raw: t.raw,
                symbol: t.symbol,
                description: t.description,
                indent: t.indent as u32,
                scheduled: t.scheduled,
                due: t.due,
                done: t.done,
                priority: t.priority.map(|p| format!("{:?}", p).to_lowercase()),
                tags: t.tags,
                block_id: t.block_id,
            })
            .collect())
    }

    // ========================================================================
    // Link Graph
    // ========================================================================

    /// Get incoming links to a note.
    #[napi]
    pub fn get_incoming_links(&self, path: String) -> Result<Vec<JsLinkRef>> {
        let note_path = self.vault.normalize_note_path(&path);
        let graph = LinkGraph::build(&self.vault)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let incoming = graph.get_incoming(&note_path);
        Ok(incoming
            .iter()
            .map(|l| JsLinkRef {
                from: l.from.to_string_lossy().to_string(),
                line: l.link.line as u32,
                context: l.context.as_string(),
                alias: l.link.alias.clone(),
                heading: l.link.heading.clone(),
                block_id: l.link.block_id.clone(),
                embed: l.link.embed,
            })
            .collect())
    }

    /// Get outgoing links from a note.
    #[napi]
    pub fn get_outgoing_links(&self, path: String) -> Result<Vec<JsLinkRef>> {
        let note_path = self.vault.normalize_note_path(&path);
        let graph = LinkGraph::build(&self.vault)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let outgoing = graph.get_outgoing(&note_path);
        Ok(outgoing
            .iter()
            .map(|l| JsLinkRef {
                from: note_path.to_string_lossy().to_string(),
                line: l.link.line as u32,
                context: l.context.as_string(),
                alias: l.link.alias.clone(),
                heading: l.link.heading.clone(),
                block_id: l.link.block_id.clone(),
                embed: l.link.embed,
            })
            .collect())
    }

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Initialize vaultiel metadata for a note.
    #[napi]
    pub fn init_metadata(&self, path: String, force: bool) -> Result<Option<JsVaultielMetadata>> {
        let note_path = self.vault.normalize_note_path(&path);
        let result = init_metadata(&self.vault, &note_path, force)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(result.map(|m| JsVaultielMetadata {
            id: m.id,
            created: m.created,
        }))
    }

    /// Get vaultiel metadata from a note.
    #[napi]
    pub fn get_vaultiel_metadata(&self, path: String) -> Result<Option<JsVaultielMetadata>> {
        let note_path = self.vault.normalize_note_path(&path);
        let result = get_metadata(&self.vault, &note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(result.map(|m| JsVaultielMetadata {
            id: m.id,
            created: m.created,
        }))
    }

    /// Find a note by its vaultiel ID.
    #[napi]
    pub fn find_by_id(&self, id: String) -> Result<Option<String>> {
        let result = find_by_id(&self.vault, &id)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(result.map(|p| p.to_string_lossy().to_string()))
    }
}

// ============================================================================
// Standalone Functions
// ============================================================================

/// Parse links from markdown content.
#[napi]
pub fn parse_links(content: String) -> Vec<JsLink> {
    let links = parse_all_links(&content);
    links
        .into_iter()
        .map(|l| JsLink {
            target: l.target,
            alias: l.alias,
            heading: l.heading,
            block_id: l.block_id,
            embed: l.embed,
            line: l.line as u32,
        })
        .collect()
}

/// Parse tags from markdown content.
#[napi]
pub fn parse_content_tags(content: String) -> Vec<JsTag> {
    let tags = parse_tags(&content);
    tags.into_iter()
        .map(|t| JsTag {
            name: t.name,
            line: t.line as u32,
        })
        .collect()
}

/// Parse headings from markdown content.
#[napi]
pub fn parse_content_headings(content: String) -> Vec<JsHeading> {
    let headings = parse_headings(&content);
    headings
        .into_iter()
        .map(|h| JsHeading {
            text: h.text,
            level: h.level as u32,
            line: h.line as u32,
            slug: h.slug,
        })
        .collect()
}

/// Parse block IDs from markdown content.
#[napi]
pub fn parse_content_block_ids(content: String) -> Vec<JsBlockId> {
    let blocks = parse_block_ids(&content);
    blocks
        .into_iter()
        .map(|b| JsBlockId {
            id: b.id,
            line: b.line as u32,
            block_type: format!("{:?}", b.block_type).to_lowercase(),
        })
        .collect()
}
