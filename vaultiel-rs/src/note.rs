//! Note representation and operations.

use crate::error::Result;
use crate::parser::{
    self, parse_block_ids, parse_headings, parse_inline_properties, parse_all_links, parse_tags,
    split_frontmatter,
};
use crate::types::{BlockId, Heading, InlineProperty, Link, PropertyScope, Tag};
use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;
use std::path::{Path, PathBuf};

/// Represents a note in the vault.
#[derive(Debug, Clone)]
pub struct Note {
    /// Path relative to vault root (e.g., "proj/My Project.md").
    pub path: PathBuf,

    /// Raw content of the note.
    pub content: String,
}

impl Note {
    /// Create a new note from path and content.
    pub fn new(path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
        }
    }

    /// Load a note from disk.
    pub fn load(vault_root: &Path, relative_path: &Path) -> Result<Self> {
        let full_path = vault_root.join(relative_path);
        let content = std::fs::read_to_string(&full_path)?;
        Ok(Self {
            path: relative_path.to_path_buf(),
            content,
        })
    }

    /// Save the note to disk.
    pub fn save(&self, vault_root: &Path) -> Result<()> {
        let full_path = vault_root.join(&self.path);

        // Ensure parent directory exists
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&full_path, &self.content)?;
        Ok(())
    }

    /// Get the note name (filename without .md extension).
    pub fn name(&self) -> &str {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
    }

    /// Get the parent folder path.
    pub fn folder(&self) -> Option<&Path> {
        self.path.parent()
    }

    /// Check if the note has frontmatter.
    pub fn has_frontmatter(&self) -> bool {
        parser::extract_frontmatter(&self.content).is_some()
    }

    /// Get raw frontmatter YAML string.
    pub fn frontmatter_raw(&self) -> Option<&str> {
        parser::extract_frontmatter(&self.content)
    }

    /// Parse frontmatter as YAML value.
    pub fn frontmatter(&self) -> Result<Option<YamlValue>> {
        parser::parse_frontmatter_with_path(&self.content, &self.path)
    }

    /// Get content without frontmatter.
    pub fn body(&self) -> &str {
        split_frontmatter(&self.content).content
    }

    /// Get content including frontmatter.
    pub fn full_content(&self) -> &str {
        &self.content
    }

    /// Get the line number where the body starts.
    pub fn body_start_line(&self) -> usize {
        split_frontmatter(&self.content).content_start_line
    }

    /// Parse all links in the note.
    pub fn links(&self) -> Vec<Link> {
        parse_all_links(&self.content)
    }

    /// Parse all tags in the note.
    pub fn tags(&self) -> Vec<Tag> {
        parse_tags(&self.content)
    }

    /// Parse all block IDs in the note.
    pub fn block_ids(&self) -> Vec<BlockId> {
        parse_block_ids(&self.content)
    }

    /// Parse all headings in the note.
    pub fn headings(&self) -> Vec<Heading> {
        parse_headings(&self.content)
    }

    /// Parse all inline properties in the note.
    pub fn inline_properties(&self) -> Vec<InlineProperty> {
        parse_inline_properties(&self.content)
    }

    /// Update the note's frontmatter.
    pub fn with_frontmatter(&self, new_frontmatter: &YamlValue) -> Result<Self> {
        let new_content = parser::update_frontmatter(&self.content, new_frontmatter)?;
        Ok(Self {
            path: self.path.clone(),
            content: new_content,
        })
    }

    /// Update the note's body (content below frontmatter).
    pub fn with_body(&self, new_body: &str) -> Self {
        let split = split_frontmatter(&self.content);

        let new_content = if let Some(yaml) = split.yaml {
            format!("---\n{}\n---\n{}", yaml, new_body)
        } else {
            new_body.to_string()
        };

        Self {
            path: self.path.clone(),
            content: new_content,
        }
    }

    /// Append content to the note.
    pub fn append(&self, content: &str) -> Self {
        Self {
            path: self.path.clone(),
            content: format!("{}{}", self.content, content),
        }
    }

    /// Prepend content after frontmatter.
    pub fn prepend(&self, content: &str) -> Self {
        let split = split_frontmatter(&self.content);

        let new_content = if let Some(yaml) = split.yaml {
            format!("---\n{}\n---\n{}{}", yaml, content, split.content)
        } else {
            format!("{}{}", content, self.content)
        };

        Self {
            path: self.path.clone(),
            content: new_content,
        }
    }

    /// Replace content with the new full content.
    pub fn with_content(&self, new_content: impl Into<String>) -> Self {
        Self {
            path: self.path.clone(),
            content: new_content.into(),
        }
    }

    /// Remove a frontmatter key.
    ///
    /// Returns a new `Note` with the specified key removed from frontmatter.
    /// If the note has no frontmatter or the key doesn't exist, returns a clone.
    pub fn remove_frontmatter_key(&self, key: &str) -> Result<Self> {
        let fm = self.frontmatter()?;
        match fm {
            Some(YamlValue::Mapping(mut map)) => {
                map.remove(&YamlValue::String(key.to_string()));
                self.with_frontmatter(&YamlValue::Mapping(map))
            }
            Some(_) => Ok(self.clone()),
            None => Ok(self.clone()),
        }
    }

    /// Append a value to a frontmatter key's list.
    ///
    /// - If the key is absent, creates it as a single-element list.
    /// - If the key holds a scalar, converts to a list and appends.
    /// - If the key is already a list, appends the value.
    pub fn append_frontmatter_value(&self, key: &str, value: &YamlValue) -> Result<Self> {
        let fm = self.frontmatter()?;
        let mut map = match fm {
            Some(YamlValue::Mapping(map)) => map,
            Some(_) => serde_yaml::Mapping::new(),
            None => serde_yaml::Mapping::new(),
        };

        let yaml_key = YamlValue::String(key.to_string());
        let existing = map.remove(&yaml_key);

        let new_value = match existing {
            None => YamlValue::Sequence(vec![value.clone()]),
            Some(YamlValue::Sequence(mut seq)) => {
                seq.push(value.clone());
                YamlValue::Sequence(seq)
            }
            Some(scalar) => YamlValue::Sequence(vec![scalar, value.clone()]),
        };

        map.insert(yaml_key, new_value);
        self.with_frontmatter(&YamlValue::Mapping(map))
    }

    /// Set an inline property's value.
    ///
    /// Finds the inline property by key (or by index if multiple exist).
    /// If `index` is `None` and there are multiple properties with the same key, returns an error.
    /// Uses `start_col`/`end_col` for precise replacement on the target line.
    pub fn set_inline_property(&self, key: &str, new_value: &str, index: Option<usize>) -> Result<Self> {
        let props = self.inline_properties();
        let matching: Vec<_> = props.iter().enumerate()
            .filter(|(_, p)| p.key == key)
            .collect();

        let target = match index {
            Some(idx) => {
                props.get(idx).ok_or_else(|| crate::error::VaultError::Other(
                    format!("Inline property index {} out of range (note has {} inline properties)", idx, props.len())
                ))?
            }
            None => {
                if matching.is_empty() {
                    return Err(crate::error::VaultError::Other(
                        format!("No inline property found with key {:?}", key)
                    ));
                }
                if matching.len() > 1 {
                    return Err(crate::error::VaultError::Other(
                        format!("Multiple inline properties with key {:?} — specify an index", key)
                    ));
                }
                matching[0].1
            }
        };

        let formatted = crate::parser::inline_property::format_inline_property(key, new_value);

        let lines: Vec<&str> = self.content.lines().collect();
        let line_idx = target.line - 1;
        if line_idx >= lines.len() {
            return Err(crate::error::VaultError::Other(
                format!("Line {} is out of range", target.line)
            ));
        }

        let line = lines[line_idx];
        let new_line = format!("{}{}{}", &line[..target.start_col], formatted, &line[target.end_col..]);

        let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        new_lines[line_idx] = new_line;

        let mut new_content = new_lines.join("\n");
        if self.content.ends_with('\n') {
            new_content.push('\n');
        }

        Ok(Self {
            path: self.path.clone(),
            content: new_content,
        })
    }

    /// Remove an inline property.
    ///
    /// Finds by key (if provided) or by index. If finding by key and multiple
    /// properties share the same key, returns an error.
    pub fn remove_inline_property(&self, key: Option<&str>, index: Option<usize>) -> Result<Self> {
        let props = self.inline_properties();

        let target = match (key, index) {
            (_, Some(idx)) => {
                props.get(idx).ok_or_else(|| crate::error::VaultError::Other(
                    format!("Inline property index {} out of range (note has {} inline properties)", idx, props.len())
                ))?
            }
            (Some(k), None) => {
                let matching: Vec<_> = props.iter().filter(|p| p.key == k).collect();
                if matching.is_empty() {
                    return Err(crate::error::VaultError::Other(
                        format!("No inline property found with key {:?}", k)
                    ));
                }
                if matching.len() > 1 {
                    return Err(crate::error::VaultError::Other(
                        format!("Multiple inline properties with key {:?} — specify an index", k)
                    ));
                }
                matching[0]
            }
            (None, None) => {
                return Err(crate::error::VaultError::Other(
                    "Must specify either key or index to remove an inline property".to_string()
                ));
            }
        };

        let lines: Vec<&str> = self.content.lines().collect();
        let line_idx = target.line - 1;
        if line_idx >= lines.len() {
            return Err(crate::error::VaultError::Other(
                format!("Line {} is out of range", target.line)
            ));
        }

        let line = lines[line_idx];
        let new_line = format!("{}{}", &line[..target.start_col], &line[target.end_col..]);

        let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        new_lines[line_idx] = new_line;

        let mut new_content = new_lines.join("\n");
        if self.content.ends_with('\n') {
            new_content.push('\n');
        }

        Ok(Self {
            path: self.path.clone(),
            content: new_content,
        })
    }

    /// Rename all inline properties with `old_key` to `new_key`.
    pub fn rename_inline_property(&self, old_key: &str, new_key: &str) -> Result<Self> {
        let props = self.inline_properties();
        let matching: Vec<_> = props.iter().filter(|p| p.key == old_key).collect();

        if matching.is_empty() {
            return Ok(self.clone());
        }

        // Process in reverse order so column offsets remain valid
        let mut lines: Vec<String> = self.content.lines().map(|l| l.to_string()).collect();

        let mut sorted = matching.clone();
        sorted.sort_by(|a, b| b.line.cmp(&a.line).then(b.start_col.cmp(&a.start_col)));

        for prop in sorted {
            let line_idx = prop.line - 1;
            if line_idx >= lines.len() { continue; }

            let line = &lines[line_idx];
            let formatted = crate::parser::inline_property::format_inline_property(new_key, &prop.value);
            let new_line = format!("{}{}{}", &line[..prop.start_col], formatted, &line[prop.end_col..]);
            lines[line_idx] = new_line;
        }

        let mut new_content = lines.join("\n");
        if self.content.ends_with('\n') {
            new_content.push('\n');
        }

        Ok(Self {
            path: self.path.clone(),
            content: new_content,
        })
    }

    /// Rename a frontmatter key.
    ///
    /// Removes the old key and inserts the new key with the same value.
    /// If the old key doesn't exist, returns a clone.
    pub fn rename_frontmatter_key(&self, old_key: &str, new_key: &str) -> Result<Self> {
        let fm = self.frontmatter()?;
        match fm {
            Some(YamlValue::Mapping(mut map)) => {
                let old_yaml_key = YamlValue::String(old_key.to_string());
                if let Some(value) = map.remove(&old_yaml_key) {
                    map.insert(YamlValue::String(new_key.to_string()), value);
                    self.with_frontmatter(&YamlValue::Mapping(map))
                } else {
                    Ok(self.clone())
                }
            }
            _ => Ok(self.clone()),
        }
    }

    // ========================================================================
    // Property-agnostic operations
    // ========================================================================

    /// Get merged properties (frontmatter + inline).
    ///
    /// Frontmatter entries take precedence. For inline properties, keys already
    /// present in frontmatter are skipped; multiple inline values with the same
    /// key are collected into a `Sequence`.
    pub fn get_properties(&self) -> Result<std::collections::HashMap<String, YamlValue>> {
        let mut merged = std::collections::HashMap::new();

        // Start with frontmatter
        if let Some(YamlValue::Mapping(map)) = self.frontmatter()? {
            for (k, v) in map {
                if let YamlValue::String(key) = k {
                    merged.insert(key, v);
                }
            }
        }

        // Add inline properties (skip keys already in frontmatter)
        let inline = self.inline_properties();
        let mut inline_groups: std::collections::HashMap<String, Vec<YamlValue>> = std::collections::HashMap::new();
        for prop in &inline {
            inline_groups.entry(prop.key.clone()).or_default().push(
                YamlValue::String(prop.value.clone())
            );
        }

        for (key, values) in inline_groups {
            if !merged.contains_key(&key) {
                if values.len() == 1 {
                    merged.insert(key, values.into_iter().next().unwrap());
                } else {
                    merged.insert(key, YamlValue::Sequence(values));
                }
            }
        }

        Ok(merged)
    }

    /// Get a single property by key. Frontmatter checked first, then inline.
    pub fn get_property(&self, key: &str) -> Result<Option<YamlValue>> {
        // Check frontmatter first
        if let Some(YamlValue::Mapping(map)) = self.frontmatter()? {
            let yaml_key = YamlValue::String(key.to_string());
            if let Some(value) = map.get(&yaml_key) {
                return Ok(Some(value.clone()));
            }
        }

        // Check inline properties
        let inline = self.inline_properties();
        let matching: Vec<_> = inline.iter().filter(|p| p.key == key).collect();
        match matching.len() {
            0 => Ok(None),
            1 => Ok(Some(YamlValue::String(matching[0].value.clone()))),
            _ => {
                let values: Vec<YamlValue> = matching.iter()
                    .map(|p| YamlValue::String(p.value.clone()))
                    .collect();
                Ok(Some(YamlValue::Sequence(values)))
            }
        }
    }

    /// Set a property with scope control.
    ///
    /// - `Auto`: detect where key lives; error if ambiguous; default to frontmatter if new.
    /// - `Both`: error (ambiguous intent for set).
    /// - `Frontmatter` / `Inline`: direct delegation.
    pub fn set_property(&self, key: &str, value: &YamlValue, scope: &PropertyScope) -> Result<Self> {
        match scope {
            PropertyScope::Frontmatter => {
                let mut fm = self.frontmatter()?
                    .unwrap_or(YamlValue::Mapping(serde_yaml::Mapping::new()));
                if let YamlValue::Mapping(ref mut map) = fm {
                    map.insert(YamlValue::String(key.to_string()), value.clone());
                }
                self.with_frontmatter(&fm)
            }
            PropertyScope::Inline { index } => {
                let str_value = match value {
                    YamlValue::String(s) => s.clone(),
                    other => serde_yaml::to_string(other)
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                };
                self.set_inline_property(key, &str_value, *index)
            }
            PropertyScope::Auto => {
                let fm = self.frontmatter()?;
                let in_fm = match &fm {
                    Some(YamlValue::Mapping(map)) => map.contains_key(&YamlValue::String(key.to_string())),
                    _ => false,
                };
                let inline = self.inline_properties();
                let in_inline = inline.iter().any(|p| p.key == key);

                if in_fm && in_inline {
                    return Err(crate::error::VaultError::Other(
                        format!("Property {:?} exists in both frontmatter and inline — specify a scope", key)
                    ));
                }

                if in_fm {
                    self.set_property(key, value, &PropertyScope::Frontmatter)
                } else if in_inline {
                    self.set_property(key, value, &PropertyScope::Inline { index: None })
                } else {
                    // New key — default to frontmatter
                    self.set_property(key, value, &PropertyScope::Frontmatter)
                }
            }
            PropertyScope::Both => {
                Err(crate::error::VaultError::Other(
                    "Cannot use Both scope for set_property (ambiguous intent)".to_string()
                ))
            }
        }
    }

    /// Remove a property.
    ///
    /// - `Auto` / `Both`: remove from all locations.
    /// - `Frontmatter` / `Inline`: remove from specified scope only.
    pub fn remove_property(&self, key: &str, scope: &PropertyScope) -> Result<Self> {
        match scope {
            PropertyScope::Frontmatter => {
                self.remove_frontmatter_key(key)
            }
            PropertyScope::Inline { index } => {
                self.remove_inline_property(Some(key), *index)
            }
            PropertyScope::Auto | PropertyScope::Both => {
                // Remove from frontmatter
                let mut result = self.remove_frontmatter_key(key)?;

                // Remove all inline properties with this key (in reverse order)
                let props = result.inline_properties();
                let matching_indices: Vec<usize> = props.iter()
                    .enumerate()
                    .filter(|(_, p)| p.key == key)
                    .map(|(i, _)| i)
                    .collect();

                for idx in matching_indices.into_iter().rev() {
                    result = result.remove_inline_property(None, Some(idx))?;
                }

                Ok(result)
            }
        }
    }

    /// Rename a property key.
    ///
    /// - `Auto` / `Both`: rename in all locations.
    /// - `Frontmatter` / `Inline`: rename in specified scope only.
    pub fn rename_property(&self, old_key: &str, new_key: &str, scope: &PropertyScope) -> Result<Self> {
        match scope {
            PropertyScope::Frontmatter => {
                self.rename_frontmatter_key(old_key, new_key)
            }
            PropertyScope::Inline { .. } => {
                self.rename_inline_property(old_key, new_key)
            }
            PropertyScope::Auto | PropertyScope::Both => {
                let result = self.rename_frontmatter_key(old_key, new_key)?;
                result.rename_inline_property(old_key, new_key)
            }
        }
    }

    /// Change the task checkbox symbol on a specific line.
    ///
    /// `line` is 1-indexed (consistent with `Task.location.line`).
    /// The target line must match the pattern `- [.] ...` (with optional leading whitespace).
    /// Returns a new `Note` with the modified content.
    pub fn set_task_symbol(&self, line: usize, new_symbol: char) -> Result<Self> {
        if line == 0 {
            return Err(crate::error::VaultError::Other(
                "Line number must be 1-indexed (got 0)".to_string(),
            ));
        }

        let lines: Vec<&str> = self.content.lines().collect();

        if line > lines.len() {
            return Err(crate::error::VaultError::Other(format!(
                "Line {} is out of range (note has {} lines)",
                line,
                lines.len()
            )));
        }

        let target_line = lines[line - 1];

        // Match: optional whitespace, list marker, ` [`, any single char, `]`, rest
        let re = regex::Regex::new(r"^(\s*(?:[-*+]|\d+\.) \[).\](.*)$").unwrap();
        if !re.is_match(target_line) {
            return Err(crate::error::VaultError::Other(format!(
                "Line {} is not a task: {:?}",
                line, target_line
            )));
        }

        let new_line = re.replace(target_line, |caps: &regex::Captures| {
            format!("{}{}]{}", &caps[1], new_symbol, &caps[2])
        });

        let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        new_lines[line - 1] = new_line.to_string();

        // Preserve trailing newline if original content had one
        let mut new_content = new_lines.join("\n");
        if self.content.ends_with('\n') {
            new_content.push('\n');
        }

        Ok(Self {
            path: self.path.clone(),
            content: new_content,
        })
    }
}

/// Output representation of a note for CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteInfo {
    pub path: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

impl NoteInfo {
    pub fn from_path(vault_root: &Path, relative_path: &Path) -> Result<Self> {
        let full_path = vault_root.join(relative_path);
        let metadata = std::fs::metadata(&full_path)?;

        let modified = metadata
            .modified()
            .ok()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());

        let created = metadata
            .created()
            .ok()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());

        let name = relative_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        Ok(Self {
            path: relative_path.to_string_lossy().to_string(),
            name,
            modified,
            created,
            size_bytes: Some(metadata.len()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_name() {
        let note = Note::new("proj/My Project.md", "content");
        assert_eq!(note.name(), "My Project");
    }

    #[test]
    fn test_note_folder() {
        let note = Note::new("proj/sub/Note.md", "content");
        assert_eq!(note.folder(), Some(Path::new("proj/sub")));
    }

    #[test]
    fn test_note_body_no_frontmatter() {
        let note = Note::new("note.md", "Just content");
        assert_eq!(note.body(), "Just content");
        assert!(!note.has_frontmatter());
    }

    #[test]
    fn test_note_body_with_frontmatter() {
        let content = "---\ntitle: Test\n---\n\nBody content";
        let note = Note::new("note.md", content);
        assert!(note.has_frontmatter());
        assert_eq!(note.body(), "\nBody content");
    }

    #[test]
    fn test_note_frontmatter() {
        let content = "---\ntitle: Test\ntags:\n  - rust\n---\n\nBody";
        let note = Note::new("note.md", content);
        let fm = note.frontmatter().unwrap().unwrap();
        assert_eq!(fm["title"].as_str(), Some("Test"));
    }

    #[test]
    fn test_note_append() {
        let note = Note::new("note.md", "Hello");
        let updated = note.append(" World");
        assert_eq!(updated.content, "Hello World");
    }

    #[test]
    fn test_note_prepend_no_frontmatter() {
        let note = Note::new("note.md", "World");
        let updated = note.prepend("Hello ");
        assert_eq!(updated.content, "Hello World");
    }

    #[test]
    fn test_note_prepend_with_frontmatter() {
        let content = "---\ntitle: Test\n---\nWorld";
        let note = Note::new("note.md", content);
        let updated = note.prepend("Hello ");
        assert!(updated.content.contains("---\ntitle: Test\n---\nHello World"));
    }

    #[test]
    fn test_note_with_body() {
        let content = "---\ntitle: Test\n---\n\nOld body";
        let note = Note::new("note.md", content);
        let updated = note.with_body("New body");
        assert!(updated.content.contains("title: Test"));
        assert!(updated.content.contains("New body"));
        assert!(!updated.content.contains("Old body"));
    }

    #[test]
    fn test_note_links() {
        let note = Note::new("note.md", "See [[Other Note]] and [[Another|alias]].");
        let links = note.links();
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_note_tags() {
        let note = Note::new("note.md", "Tags: #rust #cli");
        let tags = note.tags();
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_set_task_symbol_check() {
        let content = "---\ntitle: Test\n---\n\n- [ ] My task\n- [ ] Another task\n";
        let note = Note::new("note.md", content);
        let updated = note.set_task_symbol(5, 'x').unwrap();
        assert!(updated.content.contains("- [x] My task"));
        assert!(updated.content.contains("- [ ] Another task"));
    }

    #[test]
    fn test_set_task_symbol_uncheck() {
        let content = "- [x] Done task\n- [ ] Open task\n";
        let note = Note::new("note.md", content);
        let updated = note.set_task_symbol(1, ' ').unwrap();
        assert!(updated.content.contains("- [ ] Done task"));
        assert!(updated.content.contains("- [ ] Open task"));
    }

    #[test]
    fn test_set_task_symbol_preserves_indentation() {
        let content = "- [ ] Top\n  - [ ] Indented task\n    - [x] Deeply indented\n";
        let note = Note::new("note.md", content);
        let updated = note.set_task_symbol(2, 'x').unwrap();
        assert!(updated.content.contains("  - [x] Indented task"));
        // Others unchanged
        assert!(updated.content.contains("- [ ] Top"));
        assert!(updated.content.contains("    - [x] Deeply indented"));
    }

    #[test]
    fn test_set_task_symbol_error_non_task_line() {
        let content = "# Heading\n- [ ] Task\n";
        let note = Note::new("note.md", content);
        let result = note.set_task_symbol(1, 'x');
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a task"));
    }

    #[test]
    fn test_set_task_symbol_error_out_of_range() {
        let content = "- [ ] Only task\n";
        let note = Note::new("note.md", content);
        let result = note.set_task_symbol(5, 'x');
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of range"));
    }

    #[test]
    fn test_set_task_symbol_error_zero_line() {
        let note = Note::new("note.md", "- [ ] Task\n");
        let result = note.set_task_symbol(0, 'x');
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("1-indexed"));
    }

    // ========================================================================
    // remove_frontmatter_key
    // ========================================================================

    #[test]
    fn test_remove_frontmatter_key() {
        let content = "---\ntitle: Test\ntags:\n  - rust\n---\n\nBody";
        let note = Note::new("note.md", content);
        let updated = note.remove_frontmatter_key("title").unwrap();
        let fm = updated.frontmatter().unwrap().unwrap();
        assert!(fm.get("title").is_none());
        assert!(fm.get("tags").is_some());
    }

    #[test]
    fn test_remove_frontmatter_key_nonexistent() {
        let content = "---\ntitle: Test\n---\n\nBody";
        let note = Note::new("note.md", content);
        let updated = note.remove_frontmatter_key("nonexistent").unwrap();
        let fm = updated.frontmatter().unwrap().unwrap();
        assert_eq!(fm["title"].as_str(), Some("Test"));
    }

    #[test]
    fn test_remove_frontmatter_key_no_frontmatter() {
        let note = Note::new("note.md", "Just content");
        let updated = note.remove_frontmatter_key("title").unwrap();
        assert_eq!(updated.content, "Just content");
    }

    // ========================================================================
    // append_frontmatter_value
    // ========================================================================

    #[test]
    fn test_append_frontmatter_value_new_key() {
        let content = "---\ntitle: Test\n---\n\nBody";
        let note = Note::new("note.md", content);
        let updated = note.append_frontmatter_value("tags", &YamlValue::String("rust".to_string())).unwrap();
        let fm = updated.frontmatter().unwrap().unwrap();
        let tags = fm["tags"].as_sequence().unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].as_str(), Some("rust"));
    }

    #[test]
    fn test_append_frontmatter_value_to_scalar() {
        let content = "---\ntag: existing\n---\n\nBody";
        let note = Note::new("note.md", content);
        let updated = note.append_frontmatter_value("tag", &YamlValue::String("new".to_string())).unwrap();
        let fm = updated.frontmatter().unwrap().unwrap();
        let tags = fm["tag"].as_sequence().unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].as_str(), Some("existing"));
        assert_eq!(tags[1].as_str(), Some("new"));
    }

    #[test]
    fn test_append_frontmatter_value_to_list() {
        let content = "---\ntags:\n  - a\n  - b\n---\n\nBody";
        let note = Note::new("note.md", content);
        let updated = note.append_frontmatter_value("tags", &YamlValue::String("c".to_string())).unwrap();
        let fm = updated.frontmatter().unwrap().unwrap();
        let tags = fm["tags"].as_sequence().unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[2].as_str(), Some("c"));
    }

    // ========================================================================
    // set_inline_property
    // ========================================================================

    #[test]
    fn test_set_inline_property() {
        let content = "Some text [status::active] here.";
        let note = Note::new("note.md", content);
        let updated = note.set_inline_property("status", "done", None).unwrap();
        assert!(updated.content.contains("[status::done]"));
        assert!(!updated.content.contains("[status::active]"));
    }

    #[test]
    fn test_set_inline_property_by_index() {
        let content = "[tag::a] [tag::b]";
        let note = Note::new("note.md", content);
        let updated = note.set_inline_property("tag", "c", Some(1)).unwrap();
        assert!(updated.content.contains("[tag::a]"));
        assert!(updated.content.contains("[tag::c]"));
        assert!(!updated.content.contains("[tag::b]"));
    }

    #[test]
    fn test_set_inline_property_ambiguous_error() {
        let content = "[tag::a] [tag::b]";
        let note = Note::new("note.md", content);
        let result = note.set_inline_property("tag", "c", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Multiple"));
    }

    #[test]
    fn test_set_inline_property_not_found() {
        let content = "No properties here";
        let note = Note::new("note.md", content);
        let result = note.set_inline_property("missing", "val", None);
        assert!(result.is_err());
    }

    // ========================================================================
    // remove_inline_property
    // ========================================================================

    #[test]
    fn test_remove_inline_property_by_key() {
        let content = "Text [status::active] more text";
        let note = Note::new("note.md", content);
        let updated = note.remove_inline_property(Some("status"), None).unwrap();
        assert!(!updated.content.contains("[status::active]"));
        assert!(updated.content.contains("Text  more text"));
    }

    #[test]
    fn test_remove_inline_property_by_index() {
        let content = "[tag::a] [tag::b]";
        let note = Note::new("note.md", content);
        let updated = note.remove_inline_property(None, Some(0)).unwrap();
        assert!(!updated.content.contains("[tag::a]"));
        assert!(updated.content.contains("[tag::b]"));
    }

    #[test]
    fn test_remove_inline_property_ambiguous_error() {
        let content = "[tag::a] [tag::b]";
        let note = Note::new("note.md", content);
        let result = note.remove_inline_property(Some("tag"), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Multiple"));
    }

    // ========================================================================
    // rename_inline_property
    // ========================================================================

    #[test]
    fn test_rename_inline_property() {
        let content = "[old-key::value1] some [old-key::value2]";
        let note = Note::new("note.md", content);
        let updated = note.rename_inline_property("old-key", "new-key").unwrap();
        assert!(updated.content.contains("[new-key::value1]"));
        assert!(updated.content.contains("[new-key::value2]"));
        assert!(!updated.content.contains("[old-key"));
    }

    #[test]
    fn test_rename_inline_property_nonexistent() {
        let content = "[key::value]";
        let note = Note::new("note.md", content);
        let updated = note.rename_inline_property("missing", "new").unwrap();
        assert_eq!(updated.content, content);
    }

    // ========================================================================
    // rename_frontmatter_key
    // ========================================================================

    #[test]
    fn test_rename_frontmatter_key() {
        let content = "---\nold-key: value\ntitle: Test\n---\n\nBody";
        let note = Note::new("note.md", content);
        let updated = note.rename_frontmatter_key("old-key", "new-key").unwrap();
        let fm = updated.frontmatter().unwrap().unwrap();
        assert!(fm.get("old-key").is_none());
        assert_eq!(fm["new-key"].as_str(), Some("value"));
        assert_eq!(fm["title"].as_str(), Some("Test"));
    }

    #[test]
    fn test_rename_frontmatter_key_nonexistent() {
        let content = "---\ntitle: Test\n---\n\nBody";
        let note = Note::new("note.md", content);
        let updated = note.rename_frontmatter_key("missing", "new").unwrap();
        let fm = updated.frontmatter().unwrap().unwrap();
        assert_eq!(fm["title"].as_str(), Some("Test"));
    }

    // ========================================================================
    // get_properties
    // ========================================================================

    #[test]
    fn test_get_properties_merge() {
        let content = "---\ntitle: Test\nstatus: active\n---\n\nSome text [priority::high] here.";
        let note = Note::new("note.md", content);
        let props = note.get_properties().unwrap();
        assert_eq!(props["title"].as_str(), Some("Test"));
        assert_eq!(props["status"].as_str(), Some("active"));
        assert_eq!(props["priority"].as_str(), Some("high"));
    }

    #[test]
    fn test_get_properties_frontmatter_precedence() {
        let content = "---\nstatus: active\n---\n\n[status::done]";
        let note = Note::new("note.md", content);
        let props = note.get_properties().unwrap();
        // Frontmatter takes precedence — inline is skipped
        assert_eq!(props["status"].as_str(), Some("active"));
    }

    #[test]
    fn test_get_properties_multiple_inline() {
        let content = "[tag::a] [tag::b]";
        let note = Note::new("note.md", content);
        let props = note.get_properties().unwrap();
        let tags = props["tag"].as_sequence().unwrap();
        assert_eq!(tags.len(), 2);
    }

    // ========================================================================
    // get_property
    // ========================================================================

    #[test]
    fn test_get_property_from_frontmatter() {
        let content = "---\ntitle: Test\n---\n\nBody";
        let note = Note::new("note.md", content);
        let val = note.get_property("title").unwrap();
        assert_eq!(val.unwrap().as_str(), Some("Test"));
    }

    #[test]
    fn test_get_property_from_inline() {
        let content = "Some text [status::active] here.";
        let note = Note::new("note.md", content);
        let val = note.get_property("status").unwrap();
        assert_eq!(val.unwrap().as_str(), Some("active"));
    }

    #[test]
    fn test_get_property_missing() {
        let content = "---\ntitle: Test\n---\n\nBody";
        let note = Note::new("note.md", content);
        let val = note.get_property("missing").unwrap();
        assert!(val.is_none());
    }

    #[test]
    fn test_get_property_frontmatter_takes_precedence() {
        let content = "---\nstatus: active\n---\n\n[status::done]";
        let note = Note::new("note.md", content);
        let val = note.get_property("status").unwrap();
        assert_eq!(val.unwrap().as_str(), Some("active"));
    }

    // ========================================================================
    // set_property
    // ========================================================================

    #[test]
    fn test_set_property_auto_fm_only() {
        let content = "---\ntitle: Old\n---\n\nBody";
        let note = Note::new("note.md", content);
        let updated = note.set_property("title", &YamlValue::String("New".to_string()), &PropertyScope::Auto).unwrap();
        let fm = updated.frontmatter().unwrap().unwrap();
        assert_eq!(fm["title"].as_str(), Some("New"));
    }

    #[test]
    fn test_set_property_auto_inline_only() {
        let content = "Some text [status::active] here.";
        let note = Note::new("note.md", content);
        let updated = note.set_property("status", &YamlValue::String("done".to_string()), &PropertyScope::Auto).unwrap();
        assert!(updated.content.contains("[status::done]"));
    }

    #[test]
    fn test_set_property_auto_both_error() {
        let content = "---\nstatus: active\n---\n\n[status::done]";
        let note = Note::new("note.md", content);
        let result = note.set_property("status", &YamlValue::String("new".to_string()), &PropertyScope::Auto);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("both"));
    }

    #[test]
    fn test_set_property_auto_new_defaults_fm() {
        let content = "---\ntitle: Test\n---\n\nBody";
        let note = Note::new("note.md", content);
        let updated = note.set_property("newkey", &YamlValue::String("val".to_string()), &PropertyScope::Auto).unwrap();
        let fm = updated.frontmatter().unwrap().unwrap();
        assert_eq!(fm["newkey"].as_str(), Some("val"));
    }

    #[test]
    fn test_set_property_both_scope_errors() {
        let content = "---\ntitle: Test\n---\n\nBody";
        let note = Note::new("note.md", content);
        let result = note.set_property("title", &YamlValue::String("val".to_string()), &PropertyScope::Both);
        assert!(result.is_err());
    }

    // ========================================================================
    // remove_property
    // ========================================================================

    #[test]
    fn test_remove_property_both_removes_all() {
        let content = "---\nstatus: active\n---\n\n[status::done]";
        let note = Note::new("note.md", content);
        let updated = note.remove_property("status", &PropertyScope::Both).unwrap();
        let fm = updated.frontmatter().unwrap().unwrap();
        assert!(fm.get("status").is_none());
        assert!(!updated.content.contains("[status::done]"));
    }

    #[test]
    fn test_remove_property_frontmatter_only() {
        let content = "---\nstatus: active\n---\n\n[status::done]";
        let note = Note::new("note.md", content);
        let updated = note.remove_property("status", &PropertyScope::Frontmatter).unwrap();
        let fm = updated.frontmatter().unwrap().unwrap();
        assert!(fm.get("status").is_none());
        // Inline should still be there
        assert!(updated.content.contains("[status::done]"));
    }

    // ========================================================================
    // rename_property
    // ========================================================================

    #[test]
    fn test_rename_property_both() {
        let content = "---\nold-key: value\n---\n\n[old-key::inline-val]";
        let note = Note::new("note.md", content);
        let updated = note.rename_property("old-key", "new-key", &PropertyScope::Both).unwrap();
        let fm = updated.frontmatter().unwrap().unwrap();
        assert!(fm.get("old-key").is_none());
        assert_eq!(fm["new-key"].as_str(), Some("value"));
        assert!(updated.content.contains("[new-key::inline-val]"));
        assert!(!updated.content.contains("[old-key"));
    }
}
