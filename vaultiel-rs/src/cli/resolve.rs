//! Resolve command implementation.

use crate::cli::args::ResolveArgs;
use crate::cli::output::Output;
use crate::error::{Result, VaultError};
use crate::vault::Vault;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct ResolveResponse {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<ResolveMatch>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolveMatch {
    pub path: String,
    pub match_type: String,
}

pub fn run(vault: &Vault, args: &ResolveArgs, output: &Output) -> Result<()> {
    let query = &args.query;

    if args.strict {
        // Strict mode: only exact path matches
        let path = vault.normalize_note_path(query);
        if vault.note_exists(&path) {
            let response = ResolveResponse {
                query: query.clone(),
                resolved: Some(path.to_string_lossy().to_string()),
                match_type: Some("path".to_string()),
                matches: None,
                error: None,
            };
            output.print(&response)?;
            return Ok(());
        } else {
            return Err(VaultError::NoteNotFound(path));
        }
    }

    // Normal resolution
    match vault.resolve_note(query) {
        Ok(path) => {
            let match_type = determine_match_type(vault, query, &path)?;
            let response = ResolveResponse {
                query: query.clone(),
                resolved: Some(path.to_string_lossy().to_string()),
                match_type: Some(match_type),
                matches: None,
                error: None,
            };
            output.print(&response)?;
            Ok(())
        }
        Err(VaultError::AmbiguousResolution {
            query: _,
            count: _,
            matches,
        }) => {
            if args.all {
                // Return all matches
                let match_list: Vec<ResolveMatch> = matches
                    .iter()
                    .map(|p| ResolveMatch {
                        path: p.to_string_lossy().to_string(),
                        match_type: determine_match_type(vault, query, p)
                            .unwrap_or_else(|_| "unknown".to_string()),
                    })
                    .collect();

                let response = ResolveResponse {
                    query: query.clone(),
                    resolved: None,
                    match_type: None,
                    matches: Some(match_list),
                    error: None,
                };
                output.print(&response)?;
                Ok(())
            } else {
                // Return error with matches
                let match_list: Vec<ResolveMatch> = matches
                    .iter()
                    .map(|p| ResolveMatch {
                        path: p.to_string_lossy().to_string(),
                        match_type: determine_match_type(vault, query, p)
                            .unwrap_or_else(|_| "unknown".to_string()),
                    })
                    .collect();

                let response = ResolveResponse {
                    query: query.clone(),
                    resolved: None,
                    match_type: None,
                    matches: Some(match_list.clone()),
                    error: Some(format!(
                        "ambiguous: {} notes match '{}'",
                        match_list.len(),
                        query
                    )),
                };
                output.print(&response)?;

                Err(VaultError::AmbiguousResolution {
                    query: query.clone(),
                    count: match_list.len(),
                    matches,
                })
            }
        }
        Err(e) => Err(e),
    }
}

fn determine_match_type(vault: &Vault, query: &str, path: &PathBuf) -> Result<String> {
    let query_lower = query.to_lowercase();

    // Check if it's an exact path match
    let normalized = vault.normalize_note_path(query);
    if normalized == *path {
        return Ok("path".to_string());
    }

    // Check if it's a name match
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if name.to_lowercase() == query_lower {
        return Ok("name".to_string());
    }

    // Must be an alias match
    Ok("alias".to_string())
}
