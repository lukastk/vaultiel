//! Link graph construction and querying.

use crate::error::Result;
use crate::parser::{parse_all_links, parse_frontmatter};
use crate::types::{Link, LinkContext};
use crate::vault::Vault;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::resolution::resolve_link_target;

/// Information about a link with its context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkInfo {
    /// The parsed link.
    #[serde(flatten)]
    pub link: Link,

    /// Where the link appears in the note.
    pub context: LinkContext,

    /// Resolved path to the target note (if it exists).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_path: Option<PathBuf>,
}

/// An incoming link from another note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingLink {
    /// The source note that contains the link.
    pub from: PathBuf,

    /// The parsed link.
    #[serde(flatten)]
    pub link: Link,

    /// Where the link appears in the source note.
    pub context: LinkContext,
}

/// A graph of links between notes in a vault.
#[derive(Debug, Default)]
pub struct LinkGraph {
    /// Map from note path to its outgoing links.
    outgoing: HashMap<PathBuf, Vec<LinkInfo>>,

    /// Map from target path to notes that link to it.
    /// The key is the normalized target (lowercase, no extension).
    incoming: HashMap<String, Vec<IncomingLink>>,

    /// Map from alias to note path.
    aliases: HashMap<String, PathBuf>,
}

impl LinkGraph {
    /// Build a link graph from a vault.
    pub fn build(vault: &Vault) -> Result<Self> {
        let mut graph = LinkGraph::default();

        // First pass: collect all aliases
        for path in vault.list_notes()? {
            if let Ok(note) = vault.load_note(&path) {
                if let Ok(Some(fm)) = parse_frontmatter(&note.content) {
                    if let Some(aliases) = fm.get("aliases") {
                        if let Some(arr) = aliases.as_sequence() {
                            for alias in arr {
                                if let Some(alias_str) = alias.as_str() {
                                    graph
                                        .aliases
                                        .insert(alias_str.to_lowercase(), path.clone());
                                }
                            }
                        } else if let Some(alias_str) = aliases.as_str() {
                            // Single alias as string
                            graph
                                .aliases
                                .insert(alias_str.to_lowercase(), path.clone());
                        }
                    }
                }
            }
        }

        // Second pass: build link graph
        for path in vault.list_notes()? {
            if let Ok(note) = vault.load_note(&path) {
                let links = Self::extract_links_with_context(&note.content, vault, &graph.aliases);

                // Build incoming index
                for link_info in &links {
                    let target_key = normalize_target(&link_info.link.target);

                    let incoming_link = IncomingLink {
                        from: path.clone(),
                        link: link_info.link.clone(),
                        context: link_info.context.clone(),
                    };

                    graph
                        .incoming
                        .entry(target_key)
                        .or_default()
                        .push(incoming_link);
                }

                graph.outgoing.insert(path, links);
            }
        }

        Ok(graph)
    }

    /// Extract all links from content with their context.
    fn extract_links_with_context(
        content: &str,
        vault: &Vault,
        aliases: &HashMap<String, PathBuf>,
    ) -> Vec<LinkInfo> {
        let mut links = Vec::new();

        // Parse links from body
        let body_links = parse_all_links(content);
        for link in body_links {
            let context = Self::determine_context(content, &link);
            let resolved_path = resolve_link_target(&link.target, vault, aliases);

            links.push(LinkInfo {
                link,
                context,
                resolved_path,
            });
        }

        // Parse links from frontmatter string values
        if let Ok(Some(fm)) = parse_frontmatter(content) {
            Self::extract_frontmatter_links(&fm, vault, aliases, &mut links, String::new());
        }

        links
    }

    /// Recursively extract links from frontmatter values.
    fn extract_frontmatter_links(
        value: &serde_yaml::Value,
        vault: &Vault,
        aliases: &HashMap<String, PathBuf>,
        links: &mut Vec<LinkInfo>,
        key_path: String,
    ) {
        match value {
            serde_yaml::Value::String(s) => {
                // Look for wikilinks in string values
                let fm_links = parse_all_links(s);
                for mut link in fm_links {
                    // Adjust line number - frontmatter links are at line 0 for now
                    // (accurate line tracking in frontmatter is complex)
                    link.line = 0;

                    let resolved_path = resolve_link_target(&link.target, vault, aliases);

                    let context = if key_path.is_empty() {
                        LinkContext::Body
                    } else {
                        LinkContext::FrontmatterScalar {
                            key: key_path.clone(),
                        }
                    };

                    links.push(LinkInfo {
                        link,
                        context,
                        resolved_path,
                    });
                }
            }
            serde_yaml::Value::Sequence(arr) => {
                for (i, item) in arr.iter().enumerate() {
                    if let serde_yaml::Value::String(s) = item {
                        let fm_links = parse_all_links(s);
                        for mut link in fm_links {
                            link.line = 0;
                            let resolved_path = resolve_link_target(&link.target, vault, aliases);

                            let context = LinkContext::FrontmatterList {
                                key: key_path.clone(),
                                index: i,
                            };

                            links.push(LinkInfo {
                                link,
                                context,
                                resolved_path,
                            });
                        }
                    } else {
                        Self::extract_frontmatter_links(
                            item,
                            vault,
                            aliases,
                            links,
                            format!("{}[{}]", key_path, i),
                        );
                    }
                }
            }
            serde_yaml::Value::Mapping(map) => {
                for (k, v) in map {
                    if let Some(key) = k.as_str() {
                        let new_path = if key_path.is_empty() {
                            key.to_string()
                        } else {
                            format!("{}.{}", key_path, key)
                        };
                        Self::extract_frontmatter_links(v, vault, aliases, links, new_path);
                    }
                }
            }
            _ => {}
        }
    }

    /// Determine the context of a link based on its position.
    fn determine_context(content: &str, link: &Link) -> LinkContext {
        // Get the line content
        let lines: Vec<&str> = content.lines().collect();
        if link.line == 0 || link.line > lines.len() {
            return LinkContext::Body;
        }

        let line_content = lines[link.line - 1];

        // Check if it's in a task
        let trimmed = line_content.trim_start();
        if trimmed.starts_with("- [ ]")
            || trimmed.starts_with("- [x]")
            || trimmed.starts_with("- [X]")
            || trimmed.starts_with("- [>]")
            || trimmed.starts_with("- [-]")
            || trimmed.starts_with("- [/]")
        {
            return LinkContext::Task;
        }

        // Check if it's in an inline attribute
        // Pattern: [key::...link...]
        if let Some(bracket_start) = line_content[..link.start_col].rfind('[') {
            let between = &line_content[bracket_start..link.start_col];
            if between.contains("::") && !between.contains(']') {
                // Extract the key
                if let Some(key_end) = between.find("::") {
                    let key = between[1..key_end].to_string();
                    return LinkContext::Inline { key };
                }
            }
        }

        LinkContext::Body
    }

    /// Get outgoing links for a note.
    pub fn get_outgoing(&self, path: &Path) -> Vec<&LinkInfo> {
        self.outgoing
            .get(path)
            .map(|links| links.iter().collect())
            .unwrap_or_default()
    }

    /// Get incoming links for a note.
    pub fn get_incoming(&self, path: &Path) -> Vec<&IncomingLink> {
        // Try multiple key formats
        let keys = [
            normalize_target(&path.to_string_lossy()),
            path.file_stem()
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default(),
        ];

        let mut result = Vec::new();
        for key in &keys {
            if let Some(links) = self.incoming.get(key) {
                result.extend(links.iter());
            }
        }

        // Deduplicate
        result.sort_by(|a, b| {
            (&a.from, a.link.line, a.link.start_col).cmp(&(&b.from, b.link.line, b.link.start_col))
        });
        result.dedup_by(|a, b| {
            a.from == b.from && a.link.line == b.link.line && a.link.start_col == b.link.start_col
        });

        result
    }

    /// Get all notes that have links.
    pub fn notes_with_links(&self) -> impl Iterator<Item = &PathBuf> {
        self.outgoing.keys()
    }

    /// Resolve an alias to a note path.
    pub fn resolve_alias(&self, alias: &str) -> Option<&PathBuf> {
        self.aliases.get(&alias.to_lowercase())
    }
}

/// Normalize a link target for use as a key.
fn normalize_target(target: &str) -> String {
    let target = target.to_lowercase();
    // Remove .md extension if present
    target
        .strip_suffix(".md")
        .unwrap_or(&target)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_vault() -> (TempDir, Vault) {
        let temp = TempDir::new().unwrap();
        let vault_path = temp.path();

        // Create test notes
        fs::write(
            vault_path.join("Note A.md"),
            "---\ntitle: Note A\naliases:\n  - alias-a\n---\n\n# Note A\n\nLinks to [[Note B]] and [[Note C#heading]].\n",
        )
        .unwrap();

        fs::write(
            vault_path.join("Note B.md"),
            "---\ntitle: Note B\nparent: \"[[Note A]]\"\n---\n\n# Note B\n\nLinks back to [[alias-a]].\n\n- [ ] Task with [[Note A]] link\n",
        )
        .unwrap();

        fs::write(
            vault_path.join("Note C.md"),
            "# Note C\n\nNo links here.\n\n## heading\n\nSome content.\n",
        )
        .unwrap();

        let vault = Vault::open(vault_path).unwrap();
        (temp, vault)
    }

    #[test]
    fn test_build_link_graph() {
        let (_temp, vault) = create_test_vault();
        let graph = LinkGraph::build(&vault).unwrap();

        // Check outgoing links from Note A
        let note_a_path = PathBuf::from("Note A.md");
        let outgoing = graph.get_outgoing(&note_a_path);
        assert_eq!(outgoing.len(), 2);
        assert!(outgoing.iter().any(|l| l.link.target == "Note B"));
        assert!(outgoing.iter().any(|l| l.link.target == "Note C"));
    }

    #[test]
    fn test_incoming_links() {
        let (_temp, vault) = create_test_vault();
        let graph = LinkGraph::build(&vault).unwrap();

        // Note A should have incoming links from Note B
        let note_a_path = PathBuf::from("Note A.md");
        let incoming = graph.get_incoming(&note_a_path);

        // Should have: alias-a reference and frontmatter parent reference
        assert!(incoming.len() >= 1);
        assert!(incoming.iter().any(|l| l.from == PathBuf::from("Note B.md")));
    }

    #[test]
    fn test_alias_resolution() {
        let (_temp, vault) = create_test_vault();
        let graph = LinkGraph::build(&vault).unwrap();

        let resolved = graph.resolve_alias("alias-a");
        assert_eq!(resolved, Some(&PathBuf::from("Note A.md")));
    }

    #[test]
    fn test_task_context() {
        let (_temp, vault) = create_test_vault();
        let graph = LinkGraph::build(&vault).unwrap();

        let note_b_path = PathBuf::from("Note B.md");
        let outgoing = graph.get_outgoing(&note_b_path);

        // One link should be in task context
        let task_links: Vec<_> = outgoing
            .iter()
            .filter(|l| matches!(l.context, LinkContext::Task))
            .collect();
        assert_eq!(task_links.len(), 1);
    }

    #[test]
    fn test_frontmatter_context() {
        let (_temp, vault) = create_test_vault();
        let graph = LinkGraph::build(&vault).unwrap();

        let note_b_path = PathBuf::from("Note B.md");
        let outgoing = graph.get_outgoing(&note_b_path);

        // One link should be in frontmatter context
        let fm_links: Vec<_> = outgoing
            .iter()
            .filter(|l| matches!(l.context, LinkContext::FrontmatterScalar { .. }))
            .collect();
        assert_eq!(fm_links.len(), 1);
    }
}
