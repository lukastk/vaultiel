//! Export vault graph to Neo4j Cypher format.

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;

use crate::error::Result;
use crate::graph::LinkGraph;
use crate::parser::{parse_headings, parse_tags};
use crate::Vault;

/// Options for Cypher export.
#[derive(Debug, Clone, Default)]
pub struct CypherOptions {
    /// Include note tags as Tag nodes with TAGGED relationships.
    pub include_tags: bool,
    /// Include headings as Heading nodes with HAS_HEADING relationships.
    pub include_headings: bool,
    /// Include frontmatter properties on Note nodes.
    pub include_frontmatter: bool,
    /// Use MERGE instead of CREATE for idempotent imports.
    pub use_merge: bool,
}

/// Export the vault's link graph to Neo4j Cypher format.
///
/// Generates Cypher statements that create:
/// - Note nodes with path, name, and optional frontmatter properties
/// - LINKS_TO relationships between notes
/// - Optionally: Tag nodes and TAGGED relationships
/// - Optionally: Heading nodes and HAS_HEADING relationships
pub fn export_cypher<W: Write>(
    vault: &Vault,
    writer: &mut W,
    options: &CypherOptions,
) -> Result<ExportStats> {
    let graph = LinkGraph::build(vault)?;
    let notes = vault.list_notes()?;

    let mut stats = ExportStats::default();
    let keyword = if options.use_merge { "MERGE" } else { "CREATE" };

    // Collect all tags if needed
    let mut all_tags: HashSet<String> = HashSet::new();
    let mut note_tags: HashMap<PathBuf, Vec<String>> = HashMap::new();
    let mut note_headings: HashMap<PathBuf, Vec<HeadingInfo>> = HashMap::new();

    // First pass: collect metadata
    for path in &notes {
        if let Ok(note) = vault.load_note(path) {
            if options.include_tags {
                let tags: Vec<String> = parse_tags(&note.content)
                    .into_iter()
                    .map(|t| t.name)
                    .collect();
                for tag in &tags {
                    all_tags.insert(tag.clone());
                }
                note_tags.insert(path.clone(), tags);
            }

            if options.include_headings {
                let headings: Vec<HeadingInfo> = parse_headings(&note.content)
                    .into_iter()
                    .map(|h| HeadingInfo {
                        text: h.text,
                        level: h.level,
                        slug: h.slug,
                    })
                    .collect();
                note_headings.insert(path.clone(), headings);
            }
        }
    }

    // Write header comment
    writeln!(writer, "// Vaultiel Graph Export - Neo4j Cypher")?;
    writeln!(writer, "// Vault: {}", vault.root.display())?;
    writeln!(writer, "// Notes: {}", notes.len())?;
    writeln!(writer)?;

    // Create constraints for idempotency
    if options.use_merge {
        writeln!(writer, "// Constraints (run once)")?;
        writeln!(writer, "CREATE CONSTRAINT note_path IF NOT EXISTS FOR (n:Note) REQUIRE n.path IS UNIQUE;")?;
        if options.include_tags {
            writeln!(writer, "CREATE CONSTRAINT tag_name IF NOT EXISTS FOR (t:Tag) REQUIRE t.name IS UNIQUE;")?;
        }
        writeln!(writer)?;
    }

    // Create Tag nodes
    if options.include_tags && !all_tags.is_empty() {
        writeln!(writer, "// Tag nodes")?;
        for tag in &all_tags {
            writeln!(writer, "{} (:Tag {{name: {}}});", keyword, escape_string(tag))?;
            stats.tags_created += 1;
        }
        writeln!(writer)?;
    }

    // Create Note nodes
    writeln!(writer, "// Note nodes")?;
    for path in &notes {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let path_str = path.to_string_lossy();

        let mut props = vec![
            format!("path: {}", escape_string(&path_str)),
            format!("name: {}", escape_string(name)),
        ];

        // Add frontmatter properties
        if options.include_frontmatter {
            if let Ok(note) = vault.load_note(path) {
                if let Ok(Some(fm)) = note.frontmatter() {
                    if let serde_yaml::Value::Mapping(map) = fm {
                        for (key, value) in map {
                            if let serde_yaml::Value::String(k) = key {
                                // Skip complex nested values and vaultiel metadata
                                if k == "vaultiel" {
                                    continue;
                                }
                                match value {
                                    serde_yaml::Value::String(s) => {
                                        props.push(format!("{}: {}", sanitize_property_name(&k), escape_string(&s)));
                                    }
                                    serde_yaml::Value::Bool(b) => {
                                        props.push(format!("{}: {}", sanitize_property_name(&k), b));
                                    }
                                    serde_yaml::Value::Number(n) => {
                                        if let Some(i) = n.as_i64() {
                                            props.push(format!("{}: {}", sanitize_property_name(&k), i));
                                        } else if let Some(f) = n.as_f64() {
                                            props.push(format!("{}: {}", sanitize_property_name(&k), f));
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

        writeln!(writer, "{} (:Note {{{}}});", keyword, props.join(", "))?;
        stats.notes_created += 1;
    }
    writeln!(writer)?;

    // Create Heading nodes and relationships
    if options.include_headings {
        writeln!(writer, "// Heading nodes and relationships")?;
        for (path, headings) in &note_headings {
            let path_str = path.to_string_lossy();
            for heading in headings {
                let heading_id = format!("{}#{}", path_str, heading.slug);
                writeln!(
                    writer,
                    "{} (:Heading {{id: {}, text: {}, level: {}, slug: {}}});",
                    keyword,
                    escape_string(&heading_id),
                    escape_string(&heading.text),
                    heading.level,
                    escape_string(&heading.slug)
                )?;
                writeln!(
                    writer,
                    "MATCH (n:Note {{path: {}}}), (h:Heading {{id: {}}}) {} (n)-[:HAS_HEADING]->(h);",
                    escape_string(&path_str),
                    escape_string(&heading_id),
                    keyword
                )?;
                stats.headings_created += 1;
            }
        }
        writeln!(writer)?;
    }

    // Create TAGGED relationships
    if options.include_tags {
        writeln!(writer, "// Tag relationships")?;
        for (path, tags) in &note_tags {
            let path_str = path.to_string_lossy();
            for tag in tags {
                writeln!(
                    writer,
                    "MATCH (n:Note {{path: {}}}), (t:Tag {{name: {}}}) {} (n)-[:TAGGED]->(t);",
                    escape_string(&path_str),
                    escape_string(tag),
                    keyword
                )?;
                stats.tag_relationships += 1;
            }
        }
        writeln!(writer)?;
    }

    // Create LINKS_TO relationships
    writeln!(writer, "// Link relationships")?;
    for source_path in &notes {
        let outgoing = graph.get_outgoing(source_path);
        let source_str = source_path.to_string_lossy();

        for link_info in outgoing {
            // Resolve the target path
            if let Ok(target_path) = vault.resolve_note(&link_info.link.target) {
                let target_str = target_path.to_string_lossy();

                let mut rel_props = vec![
                    format!("context: {}", escape_string(&link_info.context.as_string())),
                ];

                if link_info.link.embed {
                    rel_props.push("embed: true".to_string());
                }

                if let Some(alias) = &link_info.link.alias {
                    rel_props.push(format!("alias: {}", escape_string(alias)));
                }

                if let Some(heading) = &link_info.link.heading {
                    rel_props.push(format!("heading: {}", escape_string(heading)));
                }

                if let Some(block_id) = &link_info.link.block_id {
                    rel_props.push(format!("blockId: {}", escape_string(block_id)));
                }

                writeln!(
                    writer,
                    "MATCH (a:Note {{path: {}}}), (b:Note {{path: {}}}) {} (a)-[:LINKS_TO {{{}}}]->(b);",
                    escape_string(&source_str),
                    escape_string(&target_str),
                    keyword,
                    rel_props.join(", ")
                )?;
                stats.links_created += 1;
            }
        }
    }

    writeln!(writer)?;
    writeln!(writer, "// Export complete")?;
    writeln!(writer, "// Notes: {}, Links: {}, Tags: {}, Headings: {}",
        stats.notes_created, stats.links_created, stats.tags_created, stats.headings_created)?;

    Ok(stats)
}

/// Statistics from the export operation.
#[derive(Debug, Clone, Default)]
pub struct ExportStats {
    pub notes_created: usize,
    pub links_created: usize,
    pub tags_created: usize,
    pub tag_relationships: usize,
    pub headings_created: usize,
}

#[derive(Debug, Clone)]
struct HeadingInfo {
    text: String,
    level: u8,
    slug: String,
}

/// Escape a string for Cypher.
fn escape_string(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("'{}'", escaped)
}

/// Sanitize a property name for Neo4j.
fn sanitize_property_name(name: &str) -> String {
    // Replace invalid characters with underscores
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Ensure it doesn't start with a digit
    if sanitized.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        format!("_{}", sanitized)
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("hello"), "'hello'");
        assert_eq!(escape_string("it's"), "'it\\'s'");
        assert_eq!(escape_string("line\nbreak"), "'line\\nbreak'");
    }

    #[test]
    fn test_sanitize_property_name() {
        assert_eq!(sanitize_property_name("title"), "title");
        assert_eq!(sanitize_property_name("my-key"), "my_key");
        assert_eq!(sanitize_property_name("123abc"), "_123abc");
    }
}
