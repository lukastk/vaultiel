//! Python bindings for Vaultiel.
//!
//! Provides access to Vaultiel's vault operations from Python.

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use pyo3::types::PyAny;
use std::path::PathBuf;

use ::vaultiel::config::Config;
use ::vaultiel::graph::LinkGraph;
use ::vaultiel::metadata::{find_by_id, get_metadata, init_metadata};
use ::vaultiel::parser::{parse_all_links, parse_block_ids, parse_headings, parse_tags, parse_tasks};
use ::vaultiel::Vault;

// ============================================================================
// Types for Python
// ============================================================================

/// Information about a note in the vault.
#[pyclass]
#[derive(Debug, Clone)]
pub struct NoteInfo {
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub modified: Option<String>,
    #[pyo3(get)]
    pub created: Option<String>,
    #[pyo3(get)]
    pub size_bytes: u64,
}

/// A link found in a note.
#[pyclass]
#[derive(Debug, Clone)]
pub struct Link {
    #[pyo3(get)]
    pub target: String,
    #[pyo3(get)]
    pub alias: Option<String>,
    #[pyo3(get)]
    pub heading: Option<String>,
    #[pyo3(get)]
    pub block_id: Option<String>,
    #[pyo3(get)]
    pub embed: bool,
    #[pyo3(get)]
    pub line: usize,
}

#[pymethods]
impl Link {
    fn __repr__(&self) -> String {
        format!("Link(target='{}', line={})", self.target, self.line)
    }
}

/// A tag found in a note.
#[pyclass]
#[derive(Debug, Clone)]
pub struct Tag {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub line: usize,
}

#[pymethods]
impl Tag {
    fn __repr__(&self) -> String {
        format!("Tag(name='{}', line={})", self.name, self.line)
    }
}

/// A heading found in a note.
#[pyclass]
#[derive(Debug, Clone)]
pub struct Heading {
    #[pyo3(get)]
    pub text: String,
    #[pyo3(get)]
    pub level: usize,
    #[pyo3(get)]
    pub line: usize,
    #[pyo3(get)]
    pub slug: String,
}

#[pymethods]
impl Heading {
    fn __repr__(&self) -> String {
        format!("Heading(text='{}', level={}, line={})", self.text, self.level, self.line)
    }
}

/// A block ID found in a note.
#[pyclass]
#[derive(Debug, Clone)]
pub struct BlockId {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub line: usize,
    #[pyo3(get)]
    pub block_type: String,
}

#[pymethods]
impl BlockId {
    fn __repr__(&self) -> String {
        format!("BlockId(id='{}', line={})", self.id, self.line)
    }
}

/// A task found in a note.
#[pyclass]
#[derive(Debug, Clone)]
pub struct Task {
    #[pyo3(get)]
    pub file: String,
    #[pyo3(get)]
    pub line: usize,
    #[pyo3(get)]
    pub raw: String,
    #[pyo3(get)]
    pub symbol: String,
    #[pyo3(get)]
    pub description: String,
    #[pyo3(get)]
    pub indent: usize,
    #[pyo3(get)]
    pub scheduled: Option<String>,
    #[pyo3(get)]
    pub due: Option<String>,
    #[pyo3(get)]
    pub done: Option<String>,
    #[pyo3(get)]
    pub start: Option<String>,
    #[pyo3(get)]
    pub created: Option<String>,
    #[pyo3(get)]
    pub cancelled: Option<String>,
    #[pyo3(get)]
    pub recurrence: Option<String>,
    #[pyo3(get)]
    pub on_completion: Option<String>,
    #[pyo3(get)]
    pub id: Option<String>,
    #[pyo3(get)]
    pub depends_on: Vec<String>,
    #[pyo3(get)]
    pub priority: Option<String>,
    #[pyo3(get)]
    pub tags: Vec<String>,
    #[pyo3(get)]
    pub block_id: Option<String>,
}

#[pymethods]
impl Task {
    fn __repr__(&self) -> String {
        format!("Task(description='{}', symbol='{}', line={})", self.description, self.symbol, self.line)
    }
}

/// Vaultiel metadata for a note.
#[pyclass]
#[derive(Debug, Clone)]
pub struct VaultielMetadata {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub created: String,
}

#[pymethods]
impl VaultielMetadata {
    fn __repr__(&self) -> String {
        format!("VaultielMetadata(id='{}', created='{}')", self.id, self.created)
    }
}

/// A reference to a link (incoming or outgoing).
#[pyclass]
#[derive(Debug, Clone)]
pub struct LinkRef {
    #[pyo3(get)]
    pub from_note: String,
    #[pyo3(get)]
    pub line: usize,
    #[pyo3(get)]
    pub context: String,
    #[pyo3(get)]
    pub alias: Option<String>,
    #[pyo3(get)]
    pub heading: Option<String>,
    #[pyo3(get)]
    pub block_id: Option<String>,
    #[pyo3(get)]
    pub embed: bool,
}

#[pymethods]
impl LinkRef {
    fn __repr__(&self) -> String {
        format!("LinkRef(from_note='{}', line={})", self.from_note, self.line)
    }
}

// ============================================================================
// Vault Class
// ============================================================================

/// A Vaultiel vault instance.
///
/// Example:
///     >>> vault = PyVault("/path/to/vault")
///     >>> notes = vault.list_notes()
///     >>> content = vault.get_content("my-note.md")
#[pyclass(name = "Vault")]
pub struct PyVault {
    vault: Vault,
}

#[pymethods]
impl PyVault {
    /// Create a new Vault instance.
    ///
    /// Args:
    ///     path: Path to the vault directory.
    ///
    /// Returns:
    ///     A new Vault instance.
    ///
    /// Raises:
    ///     RuntimeError: If the vault cannot be opened.
    #[new]
    pub fn new(path: String) -> PyResult<Self> {
        let config = Config::default();
        let vault = Vault::new(PathBuf::from(path), config)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self { vault })
    }

    /// Get the vault root path.
    #[getter]
    pub fn root(&self) -> String {
        self.vault.root.to_string_lossy().to_string()
    }

    /// List all notes in the vault.
    ///
    /// Returns:
    ///     List of note paths relative to the vault root.
    pub fn list_notes(&self) -> PyResult<Vec<String>> {
        self.vault
            .list_notes()
            .map(|notes| notes.into_iter().map(|p| p.to_string_lossy().to_string()).collect())
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// List notes matching a glob pattern.
    ///
    /// Args:
    ///     pattern: Glob pattern to match (e.g., "daily/*.md").
    ///
    /// Returns:
    ///     List of matching note paths.
    pub fn list_notes_matching(&self, pattern: String) -> PyResult<Vec<String>> {
        self.vault
            .list_notes_matching(&pattern)
            .map(|notes| notes.into_iter().map(|p| p.to_string_lossy().to_string()).collect())
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Check if a note exists.
    ///
    /// Args:
    ///     path: Path to the note (with or without .md extension).
    ///
    /// Returns:
    ///     True if the note exists, False otherwise.
    pub fn note_exists(&self, path: String) -> bool {
        let note_path = self.vault.normalize_note_path(&path);
        self.vault.note_exists(&note_path)
    }

    /// Get the full content of a note (including frontmatter).
    ///
    /// Args:
    ///     path: Path to the note.
    ///
    /// Returns:
    ///     The note's full content as a string.
    pub fn get_content(&self, path: String) -> PyResult<String> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(note.content)
    }

    /// Get the body of a note (content without frontmatter).
    ///
    /// Args:
    ///     path: Path to the note.
    ///
    /// Returns:
    ///     The note's body content as a string.
    pub fn get_body(&self, path: String) -> PyResult<String> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(note.body().to_string())
    }

    /// Get the frontmatter of a note as a JSON string.
    ///
    /// Args:
    ///     path: Path to the note.
    ///
    /// Returns:
    ///     JSON string of the frontmatter, or None if no frontmatter.
    pub fn get_frontmatter(&self, path: String) -> PyResult<Option<String>> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        match note.frontmatter() {
            Ok(Some(fm)) => {
                let json = serde_json::to_string(&fm)
                    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
                Ok(Some(json))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(PyRuntimeError::new_err(e.to_string())),
        }
    }

    /// Get the frontmatter of a note as a Python dict.
    ///
    /// Args:
    ///     path: Path to the note.
    ///
    /// Returns:
    ///     Dict of frontmatter key-value pairs, or None if no frontmatter.
    pub fn get_frontmatter_dict<'py>(&self, py: Python<'py>, path: String) -> PyResult<Option<Bound<'py, PyAny>>> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        match note.frontmatter() {
            Ok(Some(fm)) => {
                let json_str = serde_json::to_string(&fm)
                    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
                let json_module = py.import("json")?;
                let dict = json_module.call_method1("loads", (json_str,))?;
                Ok(Some(dict))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(PyRuntimeError::new_err(e.to_string())),
        }
    }

    /// Create a new note.
    ///
    /// Args:
    ///     path: Path for the new note.
    ///     content: Content of the note.
    ///
    /// Raises:
    ///     RuntimeError: If the note cannot be created.
    pub fn create_note(&self, path: String, content: String) -> PyResult<()> {
        let note_path = self.vault.normalize_note_path(&path);
        self.vault.create_note(&note_path, &content)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    }

    /// Delete a note.
    ///
    /// Args:
    ///     path: Path to the note to delete.
    ///
    /// Raises:
    ///     RuntimeError: If the note cannot be deleted.
    pub fn delete_note(&self, path: String) -> PyResult<()> {
        let note_path = self.vault.normalize_note_path(&path);
        self.vault.delete_note(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Rename a note (without link propagation).
    ///
    /// Args:
    ///     from_path: Current path of the note.
    ///     to_path: New path for the note.
    ///
    /// Raises:
    ///     RuntimeError: If the note cannot be renamed.
    pub fn rename_note(&self, from_path: String, to_path: String) -> PyResult<()> {
        let from = self.vault.normalize_note_path(&from_path);
        let to = self.vault.normalize_note_path(&to_path);
        self.vault.rename_note(&from, &to)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Resolve a note name or alias to a path.
    ///
    /// Args:
    ///     query: Note name, alias, or partial path.
    ///
    /// Returns:
    ///     The resolved path to the note.
    ///
    /// Raises:
    ///     RuntimeError: If the note cannot be resolved.
    pub fn resolve_note(&self, query: String) -> PyResult<String> {
        self.vault.resolve_note(&query)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    // ========================================================================
    // Parsing
    // ========================================================================

    /// Get all links from a note.
    ///
    /// Args:
    ///     path: Path to the note.
    ///
    /// Returns:
    ///     List of Link objects found in the note.
    pub fn get_links(&self, path: String) -> PyResult<Vec<Link>> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let links = parse_all_links(&note.content);
        Ok(links
            .into_iter()
            .map(|l| Link {
                target: l.target,
                alias: l.alias,
                heading: l.heading,
                block_id: l.block_id,
                embed: l.embed,
                line: l.line,
            })
            .collect())
    }

    /// Get all tags from a note.
    ///
    /// Args:
    ///     path: Path to the note.
    ///
    /// Returns:
    ///     List of Tag objects found in the note.
    pub fn get_tags(&self, path: String) -> PyResult<Vec<Tag>> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let tags = parse_tags(&note.content);
        Ok(tags
            .into_iter()
            .map(|t| Tag {
                name: t.name,
                line: t.line,
            })
            .collect())
    }

    /// Get all headings from a note.
    ///
    /// Args:
    ///     path: Path to the note.
    ///
    /// Returns:
    ///     List of Heading objects found in the note.
    pub fn get_headings(&self, path: String) -> PyResult<Vec<Heading>> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let headings = parse_headings(&note.content);
        Ok(headings
            .into_iter()
            .map(|h| Heading {
                text: h.text,
                level: h.level as usize,
                line: h.line,
                slug: h.slug,
            })
            .collect())
    }

    /// Get all block IDs from a note.
    ///
    /// Args:
    ///     path: Path to the note.
    ///
    /// Returns:
    ///     List of BlockId objects found in the note.
    pub fn get_block_ids(&self, path: String) -> PyResult<Vec<BlockId>> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let blocks = parse_block_ids(&note.content);
        Ok(blocks
            .into_iter()
            .map(|b| BlockId {
                id: b.id,
                line: b.line,
                block_type: format!("{:?}", b.block_type).to_lowercase(),
            })
            .collect())
    }

    /// Get all tasks from a note.
    ///
    /// Args:
    ///     path: Path to the note.
    ///
    /// Returns:
    ///     List of Task objects found in the note.
    pub fn get_tasks(&self, path: String) -> PyResult<Vec<Task>> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let task_config = ::vaultiel::config::TaskConfig::default();
        let tasks = parse_tasks(&note.content, &note_path, &task_config);

        Ok(tasks
            .into_iter()
            .map(|t| Task {
                file: t.location.file.to_string_lossy().to_string(),
                line: t.location.line,
                raw: t.raw,
                symbol: t.symbol,
                description: t.description,
                indent: t.indent,
                scheduled: t.scheduled,
                due: t.due,
                done: t.done,
                start: t.start,
                created: t.created,
                cancelled: t.cancelled,
                recurrence: t.recurrence,
                on_completion: t.on_completion,
                id: t.id,
                depends_on: t.depends_on,
                priority: t.priority.map(|p| format!("{:?}", p).to_lowercase()),
                tags: t.tags,
                block_id: t.block_id,
            })
            .collect())
    }

    // ========================================================================
    // Write Operations
    // ========================================================================

    /// Set the content of a note (replaces body, preserves frontmatter).
    pub fn set_content(&self, path: String, content: String) -> PyResult<()> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let updated = note.with_body(&content);
        updated.save(&self.vault.root)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Modify a frontmatter field.
    pub fn modify_frontmatter(&self, path: String, key: String, value: String) -> PyResult<()> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let mut fm = note.frontmatter()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
            .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&value)
            .unwrap_or(serde_yaml::Value::String(value));

        if let serde_yaml::Value::Mapping(ref mut map) = fm {
            map.insert(serde_yaml::Value::String(key), yaml_value);
        }

        let updated = note.with_frontmatter(&fm)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        updated.save(&self.vault.root)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Append content to a note.
    pub fn append_content(&self, path: String, content: String) -> PyResult<()> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let updated = note.append(&content);
        updated.save(&self.vault.root)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Replace first occurrence of pattern in note content.
    pub fn replace_content(&self, path: String, pattern: String, replacement: String) -> PyResult<()> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let new_content = note.content.replacen(&pattern, &replacement, 1);
        let updated = ::vaultiel::Note { path: note.path, content: new_content };
        updated.save(&self.vault.root)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Inspect a note — returns full JSON representation as a string.
    pub fn inspect(&self, path: String) -> PyResult<String> {
        let note_path = self.vault.normalize_note_path(&path);
        let note = self.vault.load_note(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let info = self.vault.note_info(&note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let frontmatter: Option<serde_json::Value> = match note.frontmatter() {
            Ok(Some(yaml)) => {
                let json_str = serde_json::to_string(&yaml).unwrap_or_default();
                serde_json::from_str(&json_str).ok()
            }
            _ => None,
        };

        let task_config = ::vaultiel::config::TaskConfig::default();
        let tasks = parse_tasks(&note.content, &note_path, &task_config);
        let links = ::vaultiel::parser::parse_all_links(&note.content);
        let tags = parse_tags(&note.content);
        let headings = parse_headings(&note.content);
        let block_ids = ::vaultiel::parser::parse_block_ids(&note.content);
        let inline_attrs = ::vaultiel::parser::parse_inline_attrs(&note.content);

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
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    // ========================================================================
    // Link Graph
    // ========================================================================

    /// Get incoming links to a note.
    ///
    /// Args:
    ///     path: Path to the note.
    ///
    /// Returns:
    ///     List of LinkRef objects representing links from other notes.
    pub fn get_incoming_links(&self, path: String) -> PyResult<Vec<LinkRef>> {
        let note_path = self.vault.normalize_note_path(&path);
        let graph = LinkGraph::build(&self.vault)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let incoming = graph.get_incoming(&note_path);
        Ok(incoming
            .iter()
            .map(|l| LinkRef {
                from_note: l.from.to_string_lossy().to_string(),
                line: l.link.line,
                context: l.context.as_string(),
                alias: l.link.alias.clone(),
                heading: l.link.heading.clone(),
                block_id: l.link.block_id.clone(),
                embed: l.link.embed,
            })
            .collect())
    }

    /// Get outgoing links from a note.
    ///
    /// Args:
    ///     path: Path to the note.
    ///
    /// Returns:
    ///     List of LinkRef objects representing links to other notes.
    pub fn get_outgoing_links(&self, path: String) -> PyResult<Vec<LinkRef>> {
        let note_path = self.vault.normalize_note_path(&path);
        let graph = LinkGraph::build(&self.vault)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        let outgoing = graph.get_outgoing(&note_path);
        Ok(outgoing
            .iter()
            .map(|l| LinkRef {
                from_note: note_path.to_string_lossy().to_string(),
                line: l.link.line,
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
    ///
    /// Adds a vaultiel field with a UUID and creation timestamp to the note's
    /// frontmatter if it doesn't already exist.
    ///
    /// Args:
    ///     path: Path to the note.
    ///     force: If True, overwrite existing metadata.
    ///
    /// Returns:
    ///     VaultielMetadata if metadata was added, None if already exists.
    pub fn init_metadata(&self, path: String, force: bool) -> PyResult<Option<VaultielMetadata>> {
        let note_path = self.vault.normalize_note_path(&path);
        let result = init_metadata(&self.vault, &note_path, force)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(result.map(|m| VaultielMetadata {
            id: m.id,
            created: m.created,
        }))
    }

    /// Get vaultiel metadata from a note.
    ///
    /// Args:
    ///     path: Path to the note.
    ///
    /// Returns:
    ///     VaultielMetadata if present, None otherwise.
    pub fn get_vaultiel_metadata(&self, path: String) -> PyResult<Option<VaultielMetadata>> {
        let note_path = self.vault.normalize_note_path(&path);
        let result = get_metadata(&self.vault, &note_path)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(result.map(|m| VaultielMetadata {
            id: m.id,
            created: m.created,
        }))
    }

    /// Find a note by its vaultiel ID.
    ///
    /// Args:
    ///     id: The UUID to search for.
    ///
    /// Returns:
    ///     Path to the note if found, None otherwise.
    pub fn find_by_id(&self, id: String) -> PyResult<Option<String>> {
        let result = find_by_id(&self.vault, &id)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

        Ok(result.map(|p| p.to_string_lossy().to_string()))
    }
}

// ============================================================================
// Standalone Functions
// ============================================================================

/// Parse links from markdown content.
///
/// Args:
///     content: Markdown content to parse.
///
/// Returns:
///     List of Link objects found in the content.
#[pyfunction]
pub fn parse_links(content: String) -> Vec<Link> {
    let links = parse_all_links(&content);
    links
        .into_iter()
        .map(|l| Link {
            target: l.target,
            alias: l.alias,
            heading: l.heading,
            block_id: l.block_id,
            embed: l.embed,
            line: l.line,
        })
        .collect()
}

/// Parse tags from markdown content.
///
/// Args:
///     content: Markdown content to parse.
///
/// Returns:
///     List of Tag objects found in the content.
#[pyfunction]
pub fn parse_content_tags(content: String) -> Vec<Tag> {
    let tags = parse_tags(&content);
    tags.into_iter()
        .map(|t| Tag {
            name: t.name,
            line: t.line,
        })
        .collect()
}

/// Parse headings from markdown content.
///
/// Args:
///     content: Markdown content to parse.
///
/// Returns:
///     List of Heading objects found in the content.
#[pyfunction]
pub fn parse_content_headings(content: String) -> Vec<Heading> {
    let headings = parse_headings(&content);
    headings
        .into_iter()
        .map(|h| Heading {
            text: h.text,
            level: h.level as usize,
            line: h.line,
            slug: h.slug,
        })
        .collect()
}

/// Parse block IDs from markdown content.
///
/// Args:
///     content: Markdown content to parse.
///
/// Returns:
///     List of BlockId objects found in the content.
#[pyfunction]
pub fn parse_content_block_ids(content: String) -> Vec<BlockId> {
    let blocks = parse_block_ids(&content);
    blocks
        .into_iter()
        .map(|b| BlockId {
            id: b.id,
            line: b.line,
            block_type: format!("{:?}", b.block_type).to_lowercase(),
        })
        .collect()
}

// ============================================================================
// Module Definition
// ============================================================================

/// Vaultiel Python module.
///
/// A library for programmatically interacting with Obsidian-style vaults.
#[pymodule]
fn vaultiel_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVault>()?;
    m.add_class::<Link>()?;
    m.add_class::<Tag>()?;
    m.add_class::<Heading>()?;
    m.add_class::<BlockId>()?;
    m.add_class::<Task>()?;
    m.add_class::<VaultielMetadata>()?;
    m.add_class::<LinkRef>()?;
    m.add_function(wrap_pyfunction!(parse_links, m)?)?;
    m.add_function(wrap_pyfunction!(parse_content_tags, m)?)?;
    m.add_function(wrap_pyfunction!(parse_content_headings, m)?)?;
    m.add_function(wrap_pyfunction!(parse_content_block_ids, m)?)?;
    Ok(())
}
