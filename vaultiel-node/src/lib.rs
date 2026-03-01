//! Node.js bindings for Vaultiel.
//!
//! Provides access to Vaultiel's vault operations from Node.js/TypeScript.

#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use vaultiel::config::{EmojiFieldDef, EmojiValueType, TaskConfig};
use vaultiel::graph::LinkGraph;
use vaultiel::metadata::{find_by_id, get_metadata, init_metadata};
use vaultiel::parser::{parse_all_links, parse_block_ids, parse_headings, parse_tags, parse_task_trees, parse_tasks};
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
pub struct JsTaskLink {
    pub to: String,
    pub alias: Option<String>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsTask {
    pub file: String,
    pub line: u32,
    pub raw: String,
    pub marker: String,
    pub symbol: String,
    pub description: String,
    pub indent: u32,
    pub metadata: HashMap<String, String>,
    pub links: Vec<JsTaskLink>,
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
// Task Config JS types
// ============================================================================

#[napi(object)]
#[derive(Debug, Clone)]
pub struct JsEmojiFieldDef {
    pub emoji: String,
    pub field_name: String,
    /// One of: "date", "string", "text", "number", "flag", "enum"
    pub value_type: String,
    /// For "flag" and "enum" types, the predefined value
    pub value: Option<String>,
    pub order: u32,
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct JsTaskConfig {
    pub fields: Vec<JsEmojiFieldDef>,
}

/// Convert JS task config to Rust TaskConfig
fn js_config_to_rust(js_config: &JsTaskConfig) -> TaskConfig {
    TaskConfig {
        fields: js_config
            .fields
            .iter()
            .map(|f| EmojiFieldDef {
                emoji: f.emoji.clone(),
                field_name: f.field_name.clone(),
                value_type: match f.value_type.as_str() {
                    "date" => EmojiValueType::Date,
                    "string" => EmojiValueType::String,
                    "text" => EmojiValueType::Text,
                    "number" => EmojiValueType::Number,
                    "flag" => EmojiValueType::Flag {
                        value: f.value.clone().unwrap_or_default(),
                    },
                    "enum" => EmojiValueType::Enum {
                        value: f.value.clone().unwrap_or_default(),
                    },
                    _ => EmojiValueType::String,
                },
                order: f.order,
            })
            .collect(),
    }
}

// ============================================================================
// Task tree JSON serialization (camelCase, flattened location)
// ============================================================================

fn task_children_to_json(children: &[vaultiel::TaskChild]) -> serde_json::Value {
    serde_json::Value::Array(children.iter().map(task_child_to_json).collect())
}

fn task_child_to_json(child: &vaultiel::TaskChild) -> serde_json::Value {
    match child {
        vaultiel::TaskChild::Task(task) => {
            let mut map = serde_json::Map::new();
            map.insert("type".into(), "task".into());
            map.insert("file".into(), task.location.file.to_string_lossy().to_string().into());
            map.insert("line".into(), (task.location.line as u64).into());
            map.insert("raw".into(), task.raw.clone().into());
            map.insert("marker".into(), task.marker.clone().into());
            map.insert("symbol".into(), task.symbol.clone().into());
            map.insert("description".into(), task.description.clone().into());
            map.insert("indent".into(), (task.indent as u64).into());
            map.insert("metadata".into(), serde_json::to_value(&task.metadata).unwrap_or_default());
            map.insert("links".into(), serde_json::Value::Array(
                task.links.iter().map(|l| {
                    let mut lm = serde_json::Map::new();
                    lm.insert("to".into(), l.to.clone().into());
                    if let Some(ref alias) = l.alias {
                        lm.insert("alias".into(), alias.clone().into());
                    }
                    serde_json::Value::Object(lm)
                }).collect()
            ));
            map.insert("tags".into(), serde_json::to_value(&task.tags).unwrap_or_default());
            if let Some(ref block_id) = task.block_id {
                map.insert("blockId".into(), block_id.clone().into());
            }
            map.insert("children".into(), task_children_to_json(&task.children));
            serde_json::Value::Object(map)
        }
        vaultiel::TaskChild::Text(text) => {
            let mut map = serde_json::Map::new();
            map.insert("type".into(), "text".into());
            map.insert("file".into(), text.location.file.to_string_lossy().to_string().into());
            map.insert("line".into(), (text.location.line as u64).into());
            map.insert("raw".into(), text.raw.clone().into());
            map.insert("content".into(), text.content.clone().into());
            map.insert("marker".into(), text.marker.clone().into());
            map.insert("indent".into(), (text.indent as u64).into());
            if let Some(ref block_id) = text.block_id {
                map.insert("blockId".into(), block_id.clone().into());
            }
            map.insert("children".into(), task_children_to_json(&text.children));
            serde_json::Value::Object(map)
        }
    }
}

// ============================================================================
// Vault Class
// ============================================================================

#[napi]
pub struct JsVault {
    vault: Vault,
    task_config: TaskConfig,
}

#[napi]
impl JsVault {
    /// Open a vault at the specified path.
    #[napi(constructor)]
    pub fn new(path: String, task_config: Option<JsTaskConfig>) -> Result<Self> {
        let vault = Vault::new(PathBuf::from(path))
            .map_err(|e| Error::from_reason(e.to_string()))?;
        let task_config = task_config
            .as_ref()
            .map(js_config_to_rust)
            .unwrap_or_else(TaskConfig::empty);
        Ok(Self { vault, task_config })
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

    /// Set the raw content of a note (replaces everything including frontmatter).
    #[napi]
    pub fn set_raw_content(&self, path: String, content: String) -> Result<()> {
        let note_path = self.vault.normalize_note_path(&path);
        self.vault.set_raw_content(&note_path, &content)
            .map_err(|e| Error::from_reason(e.to_string()))
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

    /// Parse tasks from a note, optionally filtering to tasks linking to a specific note.
    #[napi]
    pub fn get_tasks(&self, path: String, links_to: Option<String>) -> Result<Vec<JsTask>> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let tasks = parse_tasks(&note.content, &note_path, &self.task_config);

        let filtered: Vec<_> = if let Some(ref target) = links_to {
            let target_normalized = target.trim_end_matches(".md").to_lowercase();
            tasks.into_iter().filter(|t| {
                t.links.iter().any(|link| {
                    link.to.trim_end_matches(".md").to_lowercase() == target_normalized
                })
            }).collect()
        } else {
            tasks
        };

        Ok(filtered
            .into_iter()
            .map(|t| JsTask {
                file: t.location.file.to_string_lossy().to_string(),
                line: t.location.line as u32,
                raw: t.raw,
                marker: t.marker,
                symbol: t.symbol,
                description: t.description,
                indent: t.indent as u32,
                metadata: t.metadata,
                links: t.links.into_iter().map(|l| JsTaskLink {
                    to: l.to,
                    alias: l.alias,
                }).collect(),
                tags: t.tags,
                block_id: t.block_id,
            })
            .collect())
    }

    /// Parse task trees from a note, returning a JSON string with the hierarchical structure.
    ///
    /// Returns a JSON array of TaskChild nodes (discriminated union with "type": "task" | "text").
    /// Uses JSON serialization since napi-rs cannot represent Rust tagged enums directly.
    #[napi]
    pub fn get_task_trees(&self, path: String) -> Result<String> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let trees = parse_task_trees(&note.content, &note_path, &self.task_config);

        // Convert to camelCase JSON using a custom serialization
        let json_value = task_children_to_json(&trees);
        serde_json::to_string(&json_value)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    // ========================================================================
    // Write Operations
    // ========================================================================

    /// Set the content of a note (replaces body, preserves frontmatter).
    #[napi]
    pub fn set_content(&self, path: String, content: String) -> Result<()> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        let updated = note.with_body(&content);
        updated.save(&self.vault.root)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Modify a frontmatter field.
    #[napi]
    pub fn modify_frontmatter(&self, path: String, key: String, value: String) -> Result<()> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let mut fm = note.frontmatter()
            .map_err(|e| Error::from_reason(e.to_string()))?
            .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

        // Parse the value as YAML to handle booleans, numbers, etc.
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&value)
            .unwrap_or(serde_yaml::Value::String(value));

        if let serde_yaml::Value::Mapping(ref mut map) = fm {
            map.insert(serde_yaml::Value::String(key), yaml_value);
        }

        let updated = note.with_frontmatter(&fm)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        updated.save(&self.vault.root)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Append content to a note.
    #[napi]
    pub fn append_content(&self, path: String, content: String) -> Result<()> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        let updated = note.append(&content);
        updated.save(&self.vault.root)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Replace first occurrence of pattern in note content.
    #[napi]
    pub fn replace_content(&self, path: String, pattern: String, replacement: String) -> Result<()> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        let new_content = note.content.replacen(&pattern, &replacement, 1);
        let updated = vaultiel::Note { path: note.path, content: new_content };
        updated.save(&self.vault.root)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Change the task checkbox symbol on a specific line of a note.
    /// `line` is 1-indexed. `new_symbol` must be a single character.
    #[napi]
    pub fn set_task_symbol(&self, path: String, line: u32, new_symbol: String) -> Result<()> {
        let chars: Vec<char> = new_symbol.chars().collect();
        if chars.len() != 1 {
            return Err(Error::from_reason(format!(
                "new_symbol must be a single character, got {} characters",
                chars.len()
            )));
        }

        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        let updated = note.set_task_symbol(line as usize, chars[0])
            .map_err(|e| Error::from_reason(e.to_string()))?;
        updated.save(&self.vault.root)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Inspect a note — returns full JSON representation.
    #[napi]
    pub fn inspect(&self, path: String) -> Result<String> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        let info = self.vault.note_info(&note_path)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        let frontmatter: Option<serde_json::Value> = match note.frontmatter() {
            Ok(Some(yaml)) => {
                let json_str = serde_json::to_string(&yaml).unwrap_or_default();
                serde_json::from_str(&json_str).ok()
            }
            _ => None,
        };

        let tasks = parse_tasks(&note.content, &note_path, &self.task_config);
        let links = note.links();
        let tags = note.tags();
        let headings = parse_headings(&note.content);
        let block_ids = vaultiel::parser::parse_block_ids(&note.content);
        let inline_attrs = vaultiel::parser::parse_inline_attrs(&note.content);

        let result = serde_json::json!({
            "path": note_path.to_string_lossy(),
            "name": note.name(),
            "frontmatter": frontmatter,
            "inline_attrs": inline_attrs,
            "headings": headings,
            "tasks": tasks,
            "links": {
                "outgoing": links,
            },
            "tags": tags,
            "block_ids": block_ids,
            "stats": {
                "lines": note.content.lines().count(),
                "words": note.content.split_whitespace().count(),
                "size_bytes": info.size_bytes.unwrap_or(0),
            }
        });

        serde_json::to_string(&result)
            .map_err(|e| Error::from_reason(e.to_string()))
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
