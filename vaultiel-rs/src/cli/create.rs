//! Create command implementation.

use crate::cli::args::CreateArgs;
use crate::cli::output::{DryRunResponse, Output};
use crate::error::{Result, VaultError};
use crate::parser::serialize_frontmatter;
use crate::vault::Vault;
use serde::Serialize;
use serde_yaml::Value as YamlValue;

#[derive(Debug, Serialize)]
pub struct CreateResponse {
    pub path: String,
    pub message: String,
}

pub fn run(vault: &Vault, args: &CreateArgs, output: &Output) -> Result<()> {
    let path = vault.normalize_note_path(&args.path);

    // Check if already exists
    if vault.note_exists(&path) {
        return Err(VaultError::NoteAlreadyExists(path));
    }

    // Build content
    let mut content = String::new();

    // Add frontmatter if provided
    if let Some(ref fm_json) = args.frontmatter {
        let fm_value: YamlValue = serde_json::from_str(fm_json).map_err(|e| {
            VaultError::Other(format!("Invalid frontmatter JSON: {}", e))
        })?;
        content.push_str(&serialize_frontmatter(&fm_value)?);
    }

    // Add content if provided
    if let Some(ref body) = args.content {
        // Unescape newlines
        let body = body.replace("\\n", "\n");
        content.push_str(&body);
    }

    // If no content at all, create minimal note
    if content.is_empty() {
        content = String::new();
    }

    if args.dry_run {
        let response = DryRunResponse {
            action: "create".to_string(),
            path: path.to_string_lossy().to_string(),
            content: Some(content),
            changes: None,
        };
        output.print(&response)?;
        return Ok(());
    }

    // Create the note
    vault.create_note(&path, &content)?;

    // Open in Obsidian if requested
    if args.open {
        open_in_obsidian(vault, &path)?;
    }

    let response = CreateResponse {
        path: path.to_string_lossy().to_string(),
        message: "Note created successfully".to_string(),
    };
    output.print(&response)?;

    Ok(())
}

fn open_in_obsidian(vault: &Vault, path: &std::path::Path) -> Result<()> {
    // Get vault name from path
    let vault_name = vault
        .root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("vault");

    let file_path = path.to_string_lossy();

    // URL encode the parameters
    let vault_encoded = urlencoding_encode(vault_name);
    let file_encoded = urlencoding_encode(&file_path);

    let url = format!(
        "obsidian://open?vault={}&file={}",
        vault_encoded, file_encoded
    );

    // Try to open the URL
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&url).spawn().ok();
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .ok();
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .ok();
    }

    Ok(())
}

// Simple URL encoding for vault/file names
fn urlencoding_encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            ' ' => result.push_str("%20"),
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    result
}
