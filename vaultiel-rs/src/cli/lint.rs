//! Lint command for vault health checks.

use crate::cli::output::Output;
use crate::error::{ExitCode, Result};
use crate::health::{compute_summary, fix_issue, format_github_actions, FixResult, HealthChecker, Issue, IssueType, LintSummary};
use crate::vault::Vault;
use serde::Serialize;

/// Output for lint command.
#[derive(Debug, Serialize)]
pub struct LintOutput {
    pub issues: Vec<Issue>,
    pub summary: LintSummary,
}

/// Output format for lint command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintFormat {
    Json,
    Text,
    Github,
}

impl LintFormat {
    pub fn from_str(s: &str) -> Option<LintFormat> {
        match s.to_lowercase().as_str() {
            "json" => Some(LintFormat::Json),
            "text" => Some(LintFormat::Text),
            "github" => Some(LintFormat::Github),
            _ => None,
        }
    }
}

/// Output for lint fix results.
#[derive(Debug, Serialize)]
pub struct LintFixOutput {
    pub issues: Vec<Issue>,
    pub fixed: Vec<FixResult>,
    pub summary: LintSummary,
}

/// Run lint checks on the vault.
pub fn lint(
    vault: &Vault,
    only: &[String],
    ignore: &[String],
    glob_pattern: Option<&str>,
    fail_on: &[String],
    format: LintFormat,
    fix: bool,
    output: &Output,
) -> Result<ExitCode> {
    // Parse issue types
    let only_types: Vec<IssueType> = only
        .iter()
        .filter_map(|s| IssueType::from_str(s))
        .collect();

    let ignore_types: Vec<IssueType> = ignore
        .iter()
        .filter_map(|s| IssueType::from_str(s))
        .collect();

    let fail_on_types: Vec<IssueType> = fail_on
        .iter()
        .filter_map(|s| IssueType::from_str(s))
        .collect();

    // Build health checker
    let mut checker = HealthChecker::new(vault);

    if !only_types.is_empty() {
        checker = checker.only(only_types);
    }

    if !ignore_types.is_empty() {
        checker = checker.ignore(ignore_types);
    }

    if let Some(pattern) = glob_pattern {
        checker = checker.glob(pattern);
    }

    // Run checks
    let issues = checker.run()?;

    // Apply fixes if requested
    let mut fix_results: Vec<FixResult> = Vec::new();
    let mut remaining_issues = issues.clone();

    if fix {
        for issue in &issues {
            if issue.fixable {
                match fix_issue(vault, issue) {
                    Ok(result) => {
                        if result.success {
                            // Remove fixed issue from remaining
                            remaining_issues.retain(|i| {
                                !(i.file == issue.file
                                    && i.issue_type == issue.issue_type
                                    && i.line == issue.line)
                            });
                        }
                        fix_results.push(result);
                    }
                    Err(e) => {
                        fix_results.push(FixResult {
                            file: issue.file.clone(),
                            issue_type: issue.issue_type,
                            success: false,
                            message: format!("Fix failed: {}", e),
                        });
                    }
                }
            }
        }
    }

    let summary = compute_summary(&remaining_issues);

    // Output based on format
    match format {
        LintFormat::Json => {
            if fix && !fix_results.is_empty() {
                let result = LintFixOutput {
                    issues: remaining_issues.clone(),
                    fixed: fix_results.clone(),
                    summary: summary.clone(),
                };
                output.print(&result)?;
            } else {
                let result = LintOutput { issues: remaining_issues.clone(), summary: summary.clone() };
                output.print(&result)?;
            }
        }
        LintFormat::Text => {
            // Show fix results first
            if !fix_results.is_empty() {
                println!("Fixed issues:");
                for result in &fix_results {
                    let status = if result.success { "✓" } else { "✗" };
                    println!(
                        "  {} [{}] {}: {}",
                        status,
                        result.issue_type,
                        result.file.display(),
                        result.message
                    );
                }
                println!();
            }

            if remaining_issues.is_empty() {
                println!("No issues found.");
            } else {
                for issue in &remaining_issues {
                    let line_info = issue
                        .line
                        .map(|l| format!(":{}", l))
                        .unwrap_or_default();
                    println!(
                        "[{}] {}{}",
                        issue.issue_type,
                        issue.file.display(),
                        line_info
                    );
                    println!("  {}", issue.message);
                    if issue.fixable && !fix {
                        println!("  (auto-fixable with --fix)");
                    }
                    println!();
                }
                println!("---");
                println!("Total: {} issues ({} fixable)", summary.total, summary.fixable);
                for (issue_type, count) in &summary.by_type {
                    println!("  {}: {}", issue_type, count);
                }
            }
        }
        LintFormat::Github => {
            print!("{}", format_github_actions(&remaining_issues));
        }
    }

    // Check if we should fail (only for remaining unfixed issues)
    if !fail_on_types.is_empty() {
        for issue in &remaining_issues {
            if fail_on_types.contains(&issue.issue_type) {
                return Ok(ExitCode::LintIssuesFound);
            }
        }
    }

    Ok(ExitCode::Success)
}

/// Find orphan notes (shorthand for lint --only orphans).
pub fn find_orphans(
    vault: &Vault,
    exclude_patterns: &[String],
    output: &Output,
) -> Result<ExitCode> {
    let checker = HealthChecker::new(vault).only(vec![IssueType::Orphans]);
    let mut issues = checker.run()?;

    // Filter by exclude patterns
    if !exclude_patterns.is_empty() {
        issues.retain(|issue| {
            let path_str = issue.file.to_string_lossy();
            !exclude_patterns.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|p| p.matches(&path_str))
                    .unwrap_or(false)
            })
        });
    }

    #[derive(Serialize)]
    struct OrphansOutput {
        orphans: Vec<std::path::PathBuf>,
        count: usize,
    }

    let orphans: Vec<std::path::PathBuf> = issues.iter().map(|i| i.file.clone()).collect();
    let count = orphans.len();

    let result = OrphansOutput { orphans, count };
    output.print(&result)?;

    Ok(ExitCode::Success)
}

/// Find broken links (shorthand for lint --only broken-links).
pub fn find_broken_links(
    vault: &Vault,
    note_path: Option<&str>,
    output: &Output,
) -> Result<ExitCode> {
    let mut checker = HealthChecker::new(vault).only(vec![IssueType::BrokenLinks, IssueType::BrokenEmbeds]);

    if let Some(path) = note_path {
        // For a specific note, we need to resolve it and then filter
        let resolved = vault.resolve_note(path)?;
        // Use a glob that matches only this file
        let pattern = resolved.to_string_lossy().to_string();
        checker = checker.glob(&pattern);
    }

    let issues = checker.run()?;

    #[derive(Serialize)]
    struct BrokenLinksOutput {
        broken_links: Vec<BrokenLink>,
        count: usize,
    }

    #[derive(Serialize)]
    struct BrokenLink {
        file: std::path::PathBuf,
        line: Option<usize>,
        target: String,
        is_embed: bool,
    }

    let broken_links: Vec<BrokenLink> = issues
        .iter()
        .map(|i| BrokenLink {
            file: i.file.clone(),
            line: i.line,
            target: i.target.clone().unwrap_or_default(),
            is_embed: i.issue_type == IssueType::BrokenEmbeds,
        })
        .collect();

    let count = broken_links.len();

    let result = BrokenLinksOutput {
        broken_links,
        count,
    };
    output.print(&result)?;

    Ok(ExitCode::Success)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_format_from_str() {
        assert_eq!(LintFormat::from_str("json"), Some(LintFormat::Json));
        assert_eq!(LintFormat::from_str("text"), Some(LintFormat::Text));
        assert_eq!(LintFormat::from_str("github"), Some(LintFormat::Github));
        assert_eq!(LintFormat::from_str("invalid"), None);
    }
}
