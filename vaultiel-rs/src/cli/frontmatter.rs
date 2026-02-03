//! Frontmatter commands implementation.

use crate::cli::args::{GetFrontmatterArgs, ModifyFrontmatterArgs, OutputFormat, RemoveFrontmatterArgs, RenameFrontmatterArgs};
use crate::cli::output::{DryRunResponse, Output};
use crate::error::{Result, VaultError};
use crate::vault::Vault;
use serde::Serialize;
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct FrontmatterResponse {
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ModifyResponse {
    pub path: String,
    pub message: String,
}

// === get-frontmatter ===

pub fn get_frontmatter(vault: &Vault, args: &GetFrontmatterArgs, output: &Output) -> Result<()> {
    let path = vault.normalize_note_path(&args.path);
    let note = vault.load_note(&path)?;

    let fm = note.frontmatter()?.ok_or_else(|| {
        VaultError::InvalidFrontmatter {
            path: path.clone(),
            message: "Note has no frontmatter".to_string(),
        }
    })?;

    // Merge inline attributes if not excluded
    let mut data: serde_json::Value = serde_json::to_value(&fm)?;

    if !args.no_inline {
        let inline_attrs = note.inline_attrs();
        if let serde_json::Value::Object(ref mut map) = data {
            for attr in inline_attrs {
                // Only add if not already in frontmatter
                if !map.contains_key(&attr.key) {
                    map.insert(
                        attr.key.clone(),
                        serde_json::Value::String(attr.value.clone()),
                    );
                }
            }
        }
    }

    // If specific key requested, extract just that
    if let Some(ref key) = args.key {
        if let serde_json::Value::Object(map) = &data {
            if let Some(value) = map.get(key) {
                output_value(value, args.format.unwrap_or(OutputFormat::Json), output)?;
            } else {
                return Err(VaultError::Other(format!("Key '{}' not found in frontmatter", key)));
            }
        }
        return Ok(());
    }

    // Output based on format
    let format = args.format.unwrap_or(OutputFormat::Json);
    output_value(&data, format, output)?;

    Ok(())
}

fn output_value(value: &serde_json::Value, format: OutputFormat, output: &Output) -> Result<()> {
    match format {
        OutputFormat::Json => {
            output.print(value)?;
        }
        OutputFormat::Yaml => {
            let yaml_value: YamlValue = serde_json::from_value(value.clone())?;
            let yaml_str = serde_yaml::to_string(&yaml_value)?;
            output.print_raw(yaml_str.trim());
        }
        OutputFormat::Toml => {
            // TOML requires a table at the root
            let toml_str = if value.is_object() {
                toml::to_string_pretty(&value)?
            } else {
                format!("value = {}", serde_json::to_string(value)?)
            };
            output.print_raw(toml_str.trim());
        }
    }
    Ok(())
}

// === modify-frontmatter ===

pub fn modify_frontmatter(vault: &Vault, args: &ModifyFrontmatterArgs, output: &Output) -> Result<()> {
    let path = vault.normalize_note_path(&args.path);
    let note = vault.load_note(&path)?;

    let mut fm = note.frontmatter()?.unwrap_or(YamlValue::Mapping(Default::default()));

    // Ensure it's a mapping
    let mapping = fm.as_mapping_mut().ok_or_else(|| {
        VaultError::InvalidFrontmatter {
            path: path.clone(),
            message: "Frontmatter is not a mapping".to_string(),
        }
    })?;

    let key = YamlValue::String(args.key.clone());

    if let Some(ref value) = args.value {
        // Set value
        let yaml_value = parse_yaml_value(value);
        mapping.insert(key, yaml_value);
    } else if let Some(ref add_value) = args.add_value {
        // Add to list
        let yaml_add = parse_yaml_value(add_value);

        let entry = mapping.entry(key.clone()).or_insert_with(|| {
            YamlValue::Sequence(Vec::new())
        });

        if let YamlValue::Sequence(seq) = entry {
            // Only add if not already present
            if !seq.contains(&yaml_add) {
                seq.push(yaml_add);
            }
        } else {
            // Convert scalar to list
            let old_value = entry.clone();
            *entry = YamlValue::Sequence(vec![old_value, yaml_add]);
        }
    } else if let Some(ref remove_value) = args.remove_value {
        // Remove from list
        let yaml_remove = parse_yaml_value(remove_value);

        if let Some(YamlValue::Sequence(seq)) = mapping.get_mut(&key) {
            seq.retain(|v| v != &yaml_remove);
        }
    } else {
        return Err(VaultError::Other(
            "Must specify --value, --add, or --remove".to_string(),
        ));
    }

    let updated_note = note.with_frontmatter(&fm)?;

    if args.dry_run {
        let response = DryRunResponse {
            action: "modify-frontmatter".to_string(),
            path: path.to_string_lossy().to_string(),
            content: Some(updated_note.content),
            changes: None,
        };
        output.print(&response)?;
        return Ok(());
    }

    vault.save_note(&updated_note)?;

    let response = ModifyResponse {
        path: path.to_string_lossy().to_string(),
        message: "Frontmatter modified successfully".to_string(),
    };
    output.print(&response)?;

    Ok(())
}

// === remove-frontmatter ===

pub fn remove_frontmatter(vault: &Vault, args: &RemoveFrontmatterArgs, output: &Output) -> Result<()> {
    let path = vault.normalize_note_path(&args.path);
    let note = vault.load_note(&path)?;

    let mut fm = note.frontmatter()?.ok_or_else(|| {
        VaultError::InvalidFrontmatter {
            path: path.clone(),
            message: "Note has no frontmatter".to_string(),
        }
    })?;

    let mapping = fm.as_mapping_mut().ok_or_else(|| {
        VaultError::InvalidFrontmatter {
            path: path.clone(),
            message: "Frontmatter is not a mapping".to_string(),
        }
    })?;

    let key = YamlValue::String(args.key.clone());
    if mapping.remove(&key).is_none() {
        return Err(VaultError::Other(format!(
            "Key '{}' not found in frontmatter",
            args.key
        )));
    }

    let updated_note = note.with_frontmatter(&fm)?;

    if args.dry_run {
        let response = DryRunResponse {
            action: "remove-frontmatter".to_string(),
            path: path.to_string_lossy().to_string(),
            content: Some(updated_note.content),
            changes: None,
        };
        output.print(&response)?;
        return Ok(());
    }

    vault.save_note(&updated_note)?;

    let response = ModifyResponse {
        path: path.to_string_lossy().to_string(),
        message: "Frontmatter field removed successfully".to_string(),
    };
    output.print(&response)?;

    Ok(())
}

// === rename-frontmatter ===

#[derive(Debug, Serialize)]
pub struct RenameFrontmatterResponse {
    pub renamed: Vec<String>,
    pub skipped: Vec<String>,
    pub total_renamed: usize,
}

pub fn rename_frontmatter(vault: &Vault, args: &RenameFrontmatterArgs, output: &Output) -> Result<()> {
    // Get notes to process
    let notes = if let Some(ref pattern) = args.glob {
        vault.list_notes_matching(pattern)?
    } else {
        vault.list_notes()?
    };

    let mut renamed: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    for note_path in notes {
        let note = match vault.load_note(&note_path) {
            Ok(n) => n,
            Err(_) => {
                skipped.push(note_path.to_string_lossy().to_string());
                continue;
            }
        };

        // Get frontmatter
        let fm = match note.frontmatter() {
            Ok(Some(f)) => f,
            _ => {
                // No frontmatter, skip
                continue;
            }
        };

        // Check if the old key exists
        let has_old_key = if let YamlValue::Mapping(ref map) = fm {
            map.get(YamlValue::String(args.from.clone())).is_some()
        } else {
            false
        };

        if !has_old_key {
            continue;
        }

        // Check if new key already exists
        let has_new_key = if let YamlValue::Mapping(ref map) = fm {
            map.get(YamlValue::String(args.to.clone())).is_some()
        } else {
            false
        };

        if has_new_key {
            skipped.push(format!(
                "{} (key '{}' already exists)",
                note_path.display(),
                args.to
            ));
            continue;
        }

        // Perform the rename
        let mut new_fm = fm.clone();
        if let YamlValue::Mapping(ref mut map) = new_fm {
            if let Some(value) = map.remove(&YamlValue::String(args.from.clone())) {
                map.insert(YamlValue::String(args.to.clone()), value);
            }
        }

        if args.dry_run {
            renamed.push(format!("{} (dry-run)", note_path.display()));
        } else {
            let updated_note = note.with_frontmatter(&new_fm)?;
            vault.save_note(&updated_note)?;
            renamed.push(note_path.to_string_lossy().to_string());
        }
    }

    let response = RenameFrontmatterResponse {
        total_renamed: renamed.len(),
        renamed,
        skipped,
    };

    output.print(&response)?;

    Ok(())
}

/// Parse a string value into a YAML value, trying to preserve types.
fn parse_yaml_value(s: &str) -> YamlValue {
    // Try to parse as JSON first (handles booleans, numbers, arrays, objects)
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(s) {
        if let Ok(yaml) = serde_json::from_value(json) {
            return yaml;
        }
    }

    // Try common patterns
    let trimmed = s.trim();

    // Boolean
    if trimmed.eq_ignore_ascii_case("true") {
        return YamlValue::Bool(true);
    }
    if trimmed.eq_ignore_ascii_case("false") {
        return YamlValue::Bool(false);
    }

    // Null
    if trimmed.eq_ignore_ascii_case("null") || trimmed == "~" {
        return YamlValue::Null;
    }

    // Integer
    if let Ok(i) = trimmed.parse::<i64>() {
        return YamlValue::Number(i.into());
    }

    // Float
    if let Ok(f) = trimmed.parse::<f64>() {
        // serde_yaml uses serde_json::Number internally
        if let Some(n) = serde_json::Number::from_f64(f) {
            // Convert through JSON
            if let Ok(yaml_num) = serde_yaml::from_str::<YamlValue>(&n.to_string()) {
                return yaml_num;
            }
        }
    }

    // Default to string
    YamlValue::String(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml_value_string() {
        match parse_yaml_value("hello") {
            YamlValue::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_parse_yaml_value_bool() {
        match parse_yaml_value("true") {
            YamlValue::Bool(b) => assert!(b),
            _ => panic!("Expected bool"),
        }
    }

    #[test]
    fn test_parse_yaml_value_number() {
        match parse_yaml_value("42") {
            YamlValue::Number(n) => assert_eq!(n.as_i64(), Some(42)),
            _ => panic!("Expected number"),
        }
    }

    #[test]
    fn test_parse_yaml_value_array() {
        match parse_yaml_value("[1, 2, 3]") {
            YamlValue::Sequence(seq) => assert_eq!(seq.len(), 3),
            _ => panic!("Expected sequence"),
        }
    }
}
