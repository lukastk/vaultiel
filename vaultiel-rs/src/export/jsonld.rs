//! Export vault graph to JSON-LD format.

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::Result;
use crate::graph::LinkGraph;
use crate::parser::{parse_headings, parse_tags};
use crate::Vault;

/// Options for JSON-LD export.
#[derive(Debug, Clone, Default)]
pub struct JsonLdOptions {
    /// Include note tags.
    pub include_tags: bool,
    /// Include headings.
    pub include_headings: bool,
    /// Include frontmatter properties.
    pub include_frontmatter: bool,
    /// Base URI for the graph (e.g., "https://example.com/vault/").
    pub base_uri: Option<String>,
    /// Pretty-print the JSON output.
    pub pretty: bool,
}

/// A JSON-LD document representing the vault graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonLdGraph {
    #[serde(rename = "@context")]
    pub context: Value,
    #[serde(rename = "@graph")]
    pub graph: Vec<Value>,
}

/// Export the vault's link graph to JSON-LD format.
///
/// Generates a JSON-LD document with:
/// - A custom context defining the vocabulary
/// - Note nodes with properties and relationships
/// - Optional tag and heading information
pub fn export_jsonld<W: Write>(
    vault: &Vault,
    writer: &mut W,
    options: &JsonLdOptions,
) -> Result<ExportStats> {
    let link_graph = LinkGraph::build(vault)?;
    let notes = vault.list_notes()?;

    let base_uri = options.base_uri.clone().unwrap_or_else(|| {
        format!("file://{}/", vault.root.to_string_lossy())
    });

    let mut stats = ExportStats::default();
    let mut graph_nodes: Vec<Value> = Vec::new();

    // Collect tag and heading data if needed
    let mut note_tags: HashMap<PathBuf, Vec<String>> = HashMap::new();
    let mut note_headings: HashMap<PathBuf, Vec<HeadingData>> = HashMap::new();

    for path in &notes {
        if let Ok(note) = vault.load_note(path) {
            if options.include_tags {
                let tags: Vec<String> = parse_tags(&note.content)
                    .into_iter()
                    .map(|t| t.name)
                    .collect();
                note_tags.insert(path.clone(), tags);
            }

            if options.include_headings {
                let headings: Vec<HeadingData> = parse_headings(&note.content)
                    .into_iter()
                    .map(|h| HeadingData {
                        text: h.text,
                        level: h.level as u32,
                        slug: h.slug,
                    })
                    .collect();
                note_headings.insert(path.clone(), headings);
            }
        }
    }

    // Build note nodes
    for path in &notes {
        let path_str = path.to_string_lossy();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let note_uri = format!("{}note/{}", base_uri, urlencoding::encode(&path_str));

        let mut note_node = json!({
            "@id": note_uri,
            "@type": "Note",
            "path": path_str,
            "name": name,
        });

        // Add frontmatter properties
        if options.include_frontmatter {
            if let Ok(note) = vault.load_note(path) {
                if let Ok(Some(fm)) = note.frontmatter() {
                    if let serde_yaml::Value::Mapping(map) = fm {
                        for (key, value) in map {
                            if let serde_yaml::Value::String(k) = key {
                                // Skip the vaultiel field and complex nested values
                                if k == "vaultiel" {
                                    continue;
                                }
                                match value {
                                    serde_yaml::Value::String(s) => {
                                        note_node[&k] = json!(s);
                                    }
                                    serde_yaml::Value::Bool(b) => {
                                        note_node[&k] = json!(b);
                                    }
                                    serde_yaml::Value::Number(n) => {
                                        if let Some(i) = n.as_i64() {
                                            note_node[&k] = json!(i);
                                        } else if let Some(f) = n.as_f64() {
                                            note_node[&k] = json!(f);
                                        }
                                    }
                                    _ => {} // Skip sequences and nested mappings
                                }
                            }
                        }
                    }
                }
            }
        }

        // Add tags
        if options.include_tags {
            if let Some(tags) = note_tags.get(path) {
                if !tags.is_empty() {
                    note_node["tags"] = json!(tags);
                }
            }
        }

        // Add headings
        if options.include_headings {
            if let Some(headings) = note_headings.get(path) {
                if !headings.is_empty() {
                    let heading_values: Vec<Value> = headings
                        .iter()
                        .map(|h| {
                            json!({
                                "@type": "Heading",
                                "text": h.text,
                                "level": h.level,
                                "slug": h.slug,
                            })
                        })
                        .collect();
                    note_node["headings"] = json!(heading_values);
                }
            }
        }

        // Add outgoing links
        let outgoing = link_graph.get_outgoing(path);
        if !outgoing.is_empty() {
            let links: Vec<Value> = outgoing
                .iter()
                .filter_map(|link_info| {
                    // Resolve the target
                    vault.resolve_note(&link_info.link.target).ok().map(|target_path| {
                        let target_str = target_path.to_string_lossy();
                        let target_uri = format!("{}note/{}", base_uri, urlencoding::encode(&target_str));

                        let mut link_obj = json!({
                            "@type": "Link",
                            "target": {
                                "@id": target_uri
                            },
                            "context": link_info.context.as_string(),
                        });

                        if link_info.link.embed {
                            link_obj["embed"] = json!(true);
                        }

                        if let Some(alias) = &link_info.link.alias {
                            link_obj["alias"] = json!(alias);
                        }

                        if let Some(heading) = &link_info.link.heading {
                            link_obj["targetHeading"] = json!(heading);
                        }

                        if let Some(block_id) = &link_info.link.block_id {
                            link_obj["targetBlock"] = json!(block_id);
                        }

                        stats.links_created += 1;
                        link_obj
                    })
                })
                .collect();

            if !links.is_empty() {
                note_node["linksTo"] = json!(links);
            }
        }

        graph_nodes.push(note_node);
        stats.notes_created += 1;
    }

    // Build the JSON-LD document
    let context = json!({
        "@vocab": "https://vaultiel.dev/schema/",
        "Note": "https://vaultiel.dev/schema/Note",
        "Link": "https://vaultiel.dev/schema/Link",
        "Heading": "https://vaultiel.dev/schema/Heading",
        "path": {
            "@id": "https://vaultiel.dev/schema/path",
            "@type": "@id"
        },
        "name": "https://vaultiel.dev/schema/name",
        "tags": "https://vaultiel.dev/schema/tags",
        "headings": "https://vaultiel.dev/schema/headings",
        "linksTo": {
            "@id": "https://vaultiel.dev/schema/linksTo",
            "@type": "@id"
        },
        "target": {
            "@id": "https://vaultiel.dev/schema/target",
            "@type": "@id"
        },
        "context": "https://vaultiel.dev/schema/context",
        "alias": "https://vaultiel.dev/schema/alias",
        "embed": "https://vaultiel.dev/schema/embed",
        "targetHeading": "https://vaultiel.dev/schema/targetHeading",
        "targetBlock": "https://vaultiel.dev/schema/targetBlock",
        "text": "https://vaultiel.dev/schema/text",
        "level": "https://vaultiel.dev/schema/level",
        "slug": "https://vaultiel.dev/schema/slug"
    });

    let doc = JsonLdGraph {
        context,
        graph: graph_nodes,
    };

    // Write output
    if options.pretty {
        serde_json::to_writer_pretty(writer, &doc)?;
    } else {
        serde_json::to_writer(writer, &doc)?;
    }

    Ok(stats)
}

/// Statistics from the export operation.
#[derive(Debug, Clone, Default)]
pub struct ExportStats {
    pub notes_created: usize,
    pub links_created: usize,
}

#[derive(Debug, Clone)]
struct HeadingData {
    text: String,
    level: u32,
    slug: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = JsonLdOptions::default();
        assert!(!options.include_tags);
        assert!(!options.include_headings);
        assert!(!options.include_frontmatter);
        assert!(options.base_uri.is_none());
        assert!(!options.pretty);
    }
}
