//! Vault health checks and issue detection.

use crate::graph::LinkGraph;
use crate::parser::{parse_block_ids, parse_headings};
use crate::vault::Vault;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Types of issues that can be detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IssueType {
    BrokenLinks,
    BrokenEmbeds,
    BrokenHeadingLinks,
    BrokenBlockRefs,
    Orphans,
    DuplicateAliases,
    DuplicateBlockIds,
    EmptyNotes,
    MissingFrontmatter,
    InvalidFrontmatter,
}

impl IssueType {
    /// All issue types.
    pub fn all() -> &'static [IssueType] {
        &[
            IssueType::BrokenLinks,
            IssueType::BrokenEmbeds,
            IssueType::BrokenHeadingLinks,
            IssueType::BrokenBlockRefs,
            IssueType::Orphans,
            IssueType::DuplicateAliases,
            IssueType::DuplicateBlockIds,
            IssueType::EmptyNotes,
            IssueType::MissingFrontmatter,
            IssueType::InvalidFrontmatter,
        ]
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Option<IssueType> {
        match s {
            "broken-links" => Some(IssueType::BrokenLinks),
            "broken-embeds" => Some(IssueType::BrokenEmbeds),
            "broken-heading-links" => Some(IssueType::BrokenHeadingLinks),
            "broken-block-refs" => Some(IssueType::BrokenBlockRefs),
            "orphans" => Some(IssueType::Orphans),
            "duplicate-aliases" => Some(IssueType::DuplicateAliases),
            "duplicate-block-ids" => Some(IssueType::DuplicateBlockIds),
            "empty-notes" => Some(IssueType::EmptyNotes),
            "missing-frontmatter" => Some(IssueType::MissingFrontmatter),
            "invalid-frontmatter" => Some(IssueType::InvalidFrontmatter),
            _ => None,
        }
    }

    /// Whether this issue type can be auto-fixed.
    pub fn is_fixable(&self) -> bool {
        matches!(
            self,
            IssueType::DuplicateBlockIds | IssueType::MissingFrontmatter
        )
    }
}

impl std::fmt::Display for IssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueType::BrokenLinks => write!(f, "broken-links"),
            IssueType::BrokenEmbeds => write!(f, "broken-embeds"),
            IssueType::BrokenHeadingLinks => write!(f, "broken-heading-links"),
            IssueType::BrokenBlockRefs => write!(f, "broken-block-refs"),
            IssueType::Orphans => write!(f, "orphans"),
            IssueType::DuplicateAliases => write!(f, "duplicate-aliases"),
            IssueType::DuplicateBlockIds => write!(f, "duplicate-block-ids"),
            IssueType::EmptyNotes => write!(f, "empty-notes"),
            IssueType::MissingFrontmatter => write!(f, "missing-frontmatter"),
            IssueType::InvalidFrontmatter => write!(f, "invalid-frontmatter"),
        }
    }
}

/// A detected issue in the vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    #[serde(rename = "type")]
    pub issue_type: IssueType,
    pub file: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub fixable: bool,
}

/// Summary of lint results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintSummary {
    pub total: usize,
    pub by_type: HashMap<String, usize>,
    pub fixable: usize,
}

/// Run health checks on the vault.
pub struct HealthChecker<'a> {
    vault: &'a Vault,
    include_types: Option<HashSet<IssueType>>,
    exclude_types: HashSet<IssueType>,
    glob_pattern: Option<String>,
}

impl<'a> HealthChecker<'a> {
    pub fn new(vault: &'a Vault) -> Self {
        Self {
            vault,
            include_types: None,
            exclude_types: HashSet::new(),
            glob_pattern: None,
        }
    }

    /// Only check specific issue types.
    pub fn only(mut self, types: Vec<IssueType>) -> Self {
        self.include_types = Some(types.into_iter().collect());
        self
    }

    /// Exclude specific issue types.
    pub fn ignore(mut self, types: Vec<IssueType>) -> Self {
        self.exclude_types = types.into_iter().collect();
        self
    }

    /// Only check notes matching glob pattern.
    pub fn glob(mut self, pattern: &str) -> Self {
        self.glob_pattern = Some(pattern.to_string());
        self
    }

    /// Check if an issue type should be checked.
    fn should_check(&self, issue_type: IssueType) -> bool {
        if self.exclude_types.contains(&issue_type) {
            return false;
        }
        if let Some(ref include) = self.include_types {
            return include.contains(&issue_type);
        }
        true
    }

    /// Run all enabled checks and return issues.
    pub fn run(&self) -> crate::error::Result<Vec<Issue>> {
        let mut issues = Vec::new();

        // Get notes to check
        let notes = if let Some(ref pattern) = self.glob_pattern {
            self.vault.list_notes_matching(pattern)?
        } else {
            self.vault.list_notes()?
        };

        // Build link graph for link-related checks
        let graph = LinkGraph::build(self.vault)?;

        // Collect all aliases for duplicate detection
        let mut alias_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

        // Check each note
        for note_path in &notes {
            // Load the note
            let note_result = self.vault.load_note(note_path);

            // Check for invalid frontmatter first
            if let Err(ref e) = note_result {
                if self.should_check(IssueType::InvalidFrontmatter) {
                    if e.to_string().contains("frontmatter") || e.to_string().contains("YAML") {
                        issues.push(Issue {
                            issue_type: IssueType::InvalidFrontmatter,
                            file: note_path.clone(),
                            line: None,
                            message: format!("Invalid frontmatter: {}", e),
                            target: None,
                            fixable: false,
                        });
                    }
                }
                continue;
            }

            let note = note_result.unwrap();

            // Check for missing frontmatter
            if self.should_check(IssueType::MissingFrontmatter) {
                if note.frontmatter().ok().flatten().is_none() {
                    issues.push(Issue {
                        issue_type: IssueType::MissingFrontmatter,
                        file: note_path.clone(),
                        line: None,
                        message: "Note has no frontmatter".to_string(),
                        target: None,
                        fixable: true,
                    });
                }
            }

            // Check for empty notes
            if self.should_check(IssueType::EmptyNotes) {
                let body = note.body();
                if body.trim().is_empty() {
                    issues.push(Issue {
                        issue_type: IssueType::EmptyNotes,
                        file: note_path.clone(),
                        line: None,
                        message: "Note has no content".to_string(),
                        target: None,
                        fixable: false,
                    });
                }
            }

            // Collect aliases
            if let Ok(Some(fm)) = note.frontmatter() {
                if let Some(aliases) = fm.get("aliases") {
                    if let Some(list) = aliases.as_sequence() {
                        for alias in list {
                            if let Some(s) = alias.as_str() {
                                alias_map
                                    .entry(s.to_lowercase())
                                    .or_default()
                                    .push(note_path.clone());
                            }
                        }
                    }
                }
            }

            // Check for duplicate block IDs
            if self.should_check(IssueType::DuplicateBlockIds) {
                let blocks = parse_block_ids(&note.content);
                let mut seen_ids: HashMap<String, usize> = HashMap::new();

                for block in blocks {
                    if let Some(&first_line) = seen_ids.get(&block.id) {
                        issues.push(Issue {
                            issue_type: IssueType::DuplicateBlockIds,
                            file: note_path.clone(),
                            line: Some(block.line),
                            message: format!(
                                "Block ID '^{}' already used on line {}",
                                block.id, first_line
                            ),
                            target: Some(block.id.clone()),
                            fixable: true,
                        });
                    } else {
                        seen_ids.insert(block.id.clone(), block.line);
                    }
                }
            }

            // Check outgoing links
            let outgoing = graph.get_outgoing(note_path);
            for link_info in outgoing {
                let link = &link_info.link;

                // Check for broken links/embeds
                if link_info.resolved_path.is_none() {
                    let issue_type = if link.embed {
                        IssueType::BrokenEmbeds
                    } else {
                        IssueType::BrokenLinks
                    };

                    if self.should_check(issue_type) {
                        let link_type = if link.embed { "embed" } else { "link" };
                        issues.push(Issue {
                            issue_type,
                            file: note_path.clone(),
                            line: Some(link.line),
                            message: format!(
                                "Broken {}: [[{}]] does not exist",
                                link_type, link.target
                            ),
                            target: Some(link.target.clone()),
                            fixable: false,
                        });
                    }
                } else if let Some(ref resolved) = link_info.resolved_path {
                    // Check heading links
                    if let Some(ref heading) = link.heading {
                        if self.should_check(IssueType::BrokenHeadingLinks) {
                            if let Ok(target_note) = self.vault.load_note(resolved) {
                                let headings = parse_headings(&target_note.content);
                                let heading_exists = headings.iter().any(|h| {
                                    h.text.eq_ignore_ascii_case(heading)
                                        || h.slug.eq_ignore_ascii_case(heading)
                                });
                                if !heading_exists {
                                    issues.push(Issue {
                                        issue_type: IssueType::BrokenHeadingLinks,
                                        file: note_path.clone(),
                                        line: Some(link.line),
                                        message: format!(
                                            "Heading '{}' not found in [[{}]]",
                                            heading, link.target
                                        ),
                                        target: Some(format!("{}#{}", link.target, heading)),
                                        fixable: false,
                                    });
                                }
                            }
                        }
                    }

                    // Check block refs
                    if let Some(ref block_id) = link.block_id {
                        if self.should_check(IssueType::BrokenBlockRefs) {
                            if let Ok(target_note) = self.vault.load_note(resolved) {
                                let blocks = parse_block_ids(&target_note.content);
                                let block_exists = blocks.iter().any(|b| b.id == *block_id);
                                if !block_exists {
                                    issues.push(Issue {
                                        issue_type: IssueType::BrokenBlockRefs,
                                        file: note_path.clone(),
                                        line: Some(link.line),
                                        message: format!(
                                            "Block ID '^{}' not found in [[{}]]",
                                            block_id, link.target
                                        ),
                                        target: Some(format!("{}#^{}", link.target, block_id)),
                                        fixable: false,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check for orphans (notes with no incoming links)
        if self.should_check(IssueType::Orphans) {
            for note_path in &notes {
                let incoming = graph.get_incoming(note_path);
                if incoming.is_empty() {
                    issues.push(Issue {
                        issue_type: IssueType::Orphans,
                        file: note_path.clone(),
                        line: None,
                        message: "Note has no incoming links".to_string(),
                        target: None,
                        fixable: false,
                    });
                }
            }
        }

        // Check for duplicate aliases
        if self.should_check(IssueType::DuplicateAliases) {
            for (alias, paths) in &alias_map {
                if paths.len() > 1 {
                    for path in paths {
                        issues.push(Issue {
                            issue_type: IssueType::DuplicateAliases,
                            file: path.clone(),
                            line: None,
                            message: format!(
                                "Alias '{}' also defined in: {}",
                                alias,
                                paths
                                    .iter()
                                    .filter(|p| *p != path)
                                    .map(|p| p.display().to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                            target: Some(alias.clone()),
                            fixable: false,
                        });
                    }
                }
            }
        }

        Ok(issues)
    }
}

/// Compute summary from issues.
pub fn compute_summary(issues: &[Issue]) -> LintSummary {
    let mut by_type: HashMap<String, usize> = HashMap::new();
    let mut fixable = 0;

    for issue in issues {
        *by_type.entry(issue.issue_type.to_string()).or_insert(0) += 1;
        if issue.fixable {
            fixable += 1;
        }
    }

    LintSummary {
        total: issues.len(),
        by_type,
        fixable,
    }
}

/// Format issues as GitHub Actions annotations.
pub fn format_github_actions(issues: &[Issue]) -> String {
    let mut output = String::new();

    for issue in issues {
        let level = match issue.issue_type {
            IssueType::Orphans | IssueType::EmptyNotes | IssueType::MissingFrontmatter => "warning",
            _ => "error",
        };

        let line_part = issue
            .line
            .map(|l| format!(",line={}", l))
            .unwrap_or_default();

        output.push_str(&format!(
            "::{} file={}{}::{}\n",
            level,
            issue.file.display(),
            line_part,
            issue.message
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_type_from_str() {
        assert_eq!(
            IssueType::from_str("broken-links"),
            Some(IssueType::BrokenLinks)
        );
        assert_eq!(IssueType::from_str("orphans"), Some(IssueType::Orphans));
        assert_eq!(IssueType::from_str("invalid"), None);
    }

    #[test]
    fn test_issue_type_display() {
        assert_eq!(IssueType::BrokenLinks.to_string(), "broken-links");
        assert_eq!(IssueType::DuplicateBlockIds.to_string(), "duplicate-block-ids");
    }

    #[test]
    fn test_issue_type_fixable() {
        assert!(IssueType::DuplicateBlockIds.is_fixable());
        assert!(IssueType::MissingFrontmatter.is_fixable());
        assert!(!IssueType::BrokenLinks.is_fixable());
        assert!(!IssueType::Orphans.is_fixable());
    }

    #[test]
    fn test_compute_summary() {
        let issues = vec![
            Issue {
                issue_type: IssueType::BrokenLinks,
                file: PathBuf::from("a.md"),
                line: Some(1),
                message: "test".to_string(),
                target: None,
                fixable: false,
            },
            Issue {
                issue_type: IssueType::BrokenLinks,
                file: PathBuf::from("b.md"),
                line: Some(2),
                message: "test".to_string(),
                target: None,
                fixable: false,
            },
            Issue {
                issue_type: IssueType::MissingFrontmatter,
                file: PathBuf::from("c.md"),
                line: None,
                message: "test".to_string(),
                target: None,
                fixable: true,
            },
        ];

        let summary = compute_summary(&issues);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.by_type.get("broken-links"), Some(&2));
        assert_eq!(summary.by_type.get("missing-frontmatter"), Some(&1));
        assert_eq!(summary.fixable, 1);
    }

    #[test]
    fn test_format_github_actions() {
        let issues = vec![
            Issue {
                issue_type: IssueType::BrokenLinks,
                file: PathBuf::from("test.md"),
                line: Some(10),
                message: "Broken link".to_string(),
                target: None,
                fixable: false,
            },
            Issue {
                issue_type: IssueType::Orphans,
                file: PathBuf::from("orphan.md"),
                line: None,
                message: "No incoming links".to_string(),
                target: None,
                fixable: false,
            },
        ];

        let output = format_github_actions(&issues);
        assert!(output.contains("::error file=test.md,line=10::Broken link"));
        assert!(output.contains("::warning file=orphan.md::No incoming links"));
    }
}
