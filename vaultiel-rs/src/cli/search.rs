//! Search command implementation.

use crate::cli::args::{SearchArgs, SearchMode};
use crate::cli::list::matches_frontmatter_filter;
use crate::cli::output::Output;
use crate::error::Result;
use crate::vault::Vault;
use regex::Regex;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
    pub query: String,
    pub mode: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    pub score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_context: Option<String>,
}

pub fn run(vault: &Vault, args: &SearchArgs, output: &Output) -> Result<()> {
    let notes = vault.list_notes()?;
    let mut results: Vec<(PathBuf, f64, Option<String>)> = Vec::new();

    for path in notes {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let path_str = path.to_string_lossy().to_string();

        // Calculate score based on search mode
        let (score, context) = match args.mode {
            SearchMode::Subsequence => {
                let score = subsequence_score(&args.query, name);
                (score, None)
            }
            SearchMode::Fuzzy => {
                let score = fuzzy_score(&args.query, name);
                (score, None)
            }
            SearchMode::Exact => {
                let matches = name.to_lowercase().contains(&args.query.to_lowercase());
                (if matches { 1.0 } else { 0.0 }, None)
            }
            SearchMode::Regex => {
                match Regex::new(&args.query) {
                    Ok(re) => {
                        let matches = re.is_match(name) || re.is_match(&path_str);
                        (if matches { 1.0 } else { 0.0 }, None)
                    }
                    Err(_) => (0.0, None),
                }
            }
        };

        // If content search is enabled, also search content
        let (final_score, final_context) = if args.content && score == 0.0 {
            if let Ok(note) = vault.load_note(&path) {
                let content = note.body();
                let content_score = match args.mode {
                    SearchMode::Subsequence => subsequence_score(&args.query, content) * 0.5,
                    SearchMode::Fuzzy => fuzzy_score(&args.query, content) * 0.5,
                    SearchMode::Exact => {
                        if content.to_lowercase().contains(&args.query.to_lowercase()) {
                            0.5
                        } else {
                            0.0
                        }
                    }
                    SearchMode::Regex => {
                        match Regex::new(&args.query) {
                            Ok(re) => {
                                if re.is_match(content) {
                                    0.5
                                } else {
                                    0.0
                                }
                            }
                            Err(_) => 0.0,
                        }
                    }
                };

                if content_score > 0.0 {
                    // Extract context around match
                    let ctx = extract_match_context(content, &args.query);
                    (content_score, ctx)
                } else {
                    (score, context)
                }
            } else {
                (score, context)
            }
        } else {
            (score, context)
        };

        // Apply tag filters
        if !args.tag.is_empty() || !args.tag_any.is_empty() || !args.no_tag.is_empty() {
            if let Ok(note) = vault.load_note(&path) {
                let note_tags: Vec<String> = note.tags().iter().map(|t| t.name.clone()).collect();

                // --tag: must have ALL specified tags (AND logic)
                if !args.tag.is_empty() {
                    let has_all_tags = args.tag.iter().all(|required_tag| {
                        tag_matches(&note_tags, required_tag)
                    });
                    if !has_all_tags {
                        continue;
                    }
                }

                // --tag-any: must have AT LEAST ONE of the specified tags (OR logic)
                if !args.tag_any.is_empty() {
                    let has_any_tag = args.tag_any.iter().any(|required_tag| {
                        tag_matches(&note_tags, required_tag)
                    });
                    if !has_any_tag {
                        continue;
                    }
                }

                // --no-tag: must NOT have any of the specified tags
                if !args.no_tag.is_empty() {
                    let has_excluded_tag = args.no_tag.iter().any(|excluded_tag| {
                        tag_matches(&note_tags, excluded_tag)
                    });
                    if has_excluded_tag {
                        continue;
                    }
                }
            } else {
                continue;
            }
        }

        // Apply frontmatter filter (supports =, !=, ~= operators)
        if !args.frontmatter.is_empty() {
            if let Ok(note) = vault.load_note(&path) {
                if let Ok(Some(fm)) = note.frontmatter() {
                    let matches_all = args.frontmatter.iter().all(|filter| {
                        matches_frontmatter_filter(filter, &fm)
                    });
                    if !matches_all {
                        continue;
                    }
                } else {
                    continue;
                }
            } else {
                continue;
            }
        }

        if final_score > 0.0 {
            results.push((path, final_score, final_context));
        }
    }

    // Sort by score (descending)
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Apply limit
    let total = results.len();
    results.truncate(args.limit);

    let search_results: Vec<SearchResult> = results
        .into_iter()
        .map(|(path, score, context)| SearchResult {
            name: path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string(),
            path: path.to_string_lossy().to_string(),
            score,
            match_context: context,
        })
        .collect();

    let response = SearchResponse {
        results: search_results,
        total,
        query: args.query.clone(),
        mode: format!("{:?}", args.mode).to_lowercase(),
    };
    output.print(&response)?;

    Ok(())
}

/// Obsidian-style subsequence matching.
/// Returns a score between 0.0 and 1.0.
fn subsequence_score(query: &str, text: &str) -> f64 {
    let query_lower: Vec<char> = query.to_lowercase().chars().collect();
    let text_lower: Vec<char> = text.to_lowercase().chars().collect();

    if query_lower.is_empty() {
        return 1.0;
    }

    if text_lower.is_empty() {
        return 0.0;
    }

    let mut query_idx = 0;
    let mut match_positions: Vec<usize> = Vec::new();

    for (text_idx, text_char) in text_lower.iter().enumerate() {
        if query_idx < query_lower.len() && *text_char == query_lower[query_idx] {
            match_positions.push(text_idx);
            query_idx += 1;
        }
    }

    // All query characters must match
    if query_idx < query_lower.len() {
        return 0.0;
    }

    // Score based on:
    // - Consecutive matches are better
    // - Matches at word boundaries are better
    // - Shorter texts with same match are better

    let mut score = 1.0;

    // Bonus for consecutive matches
    let mut consecutive_bonus: f64 = 0.0;
    for i in 1..match_positions.len() {
        if match_positions[i] == match_positions[i - 1] + 1 {
            consecutive_bonus += 0.1;
        }
    }
    score += consecutive_bonus.min(0.5);

    // Bonus for starting at beginning
    if !match_positions.is_empty() && match_positions[0] == 0 {
        score += 0.2;
    }

    // Penalty for length difference
    let length_ratio = query_lower.len() as f64 / text_lower.len() as f64;
    score *= length_ratio.sqrt();

    score.min(1.0)
}

/// Simple fuzzy matching score.
fn fuzzy_score(query: &str, text: &str) -> f64 {
    // Use subsequence as base, but also consider edit distance for short queries
    let subseq = subsequence_score(query, text);

    if subseq > 0.0 {
        subseq
    } else {
        // For non-subsequence matches, check if query is close to text
        let query_lower = query.to_lowercase();
        let text_lower = text.to_lowercase();

        // Simple check: does text contain most of the query characters?
        let query_chars: std::collections::HashSet<char> = query_lower.chars().collect();
        let text_chars: std::collections::HashSet<char> = text_lower.chars().collect();

        let intersection = query_chars.intersection(&text_chars).count();
        let union = query_chars.union(&text_chars).count();

        if union > 0 {
            (intersection as f64 / union as f64) * 0.3 // Lower weight for fuzzy
        } else {
            0.0
        }
    }
}

/// Check if a note's tags match a required tag.
/// Supports nested tags: `project` matches `#project` and `#project/foo`.
fn tag_matches(note_tags: &[String], required_tag: &str) -> bool {
    let required = if required_tag.starts_with('#') {
        required_tag.to_lowercase()
    } else {
        format!("#{}", required_tag.to_lowercase())
    };

    note_tags.iter().any(|t| {
        let tag_lower = t.to_lowercase();
        tag_lower == required || tag_lower.starts_with(&format!("{}/", required))
    })
}

/// Extract context around a match in content.
fn extract_match_context(content: &str, query: &str) -> Option<String> {
    let lower_content = content.to_lowercase();
    let lower_query = query.to_lowercase();

    if let Some(pos) = lower_content.find(&lower_query) {
        let start = pos.saturating_sub(30);
        let end = (pos + query.len() + 30).min(content.len());

        // Find word boundaries
        let start = content[..start]
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(start);
        let end = content[end..]
            .find(|c: char| c.is_whitespace())
            .map(|i| end + i)
            .unwrap_or(end);

        let mut context = content[start..end].to_string();
        if start > 0 {
            context = format!("...{}", context);
        }
        if end < content.len() {
            context = format!("{}...", context);
        }

        Some(context.replace('\n', " ").trim().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subsequence_score_exact() {
        let score = subsequence_score("test", "test");
        assert!(score > 0.9);
    }

    #[test]
    fn test_subsequence_score_partial() {
        let score = subsequence_score("tst", "test");
        assert!(score > 0.0);
    }

    #[test]
    fn test_subsequence_score_no_match() {
        let score = subsequence_score("xyz", "test");
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_subsequence_score_case_insensitive() {
        let score = subsequence_score("TEST", "test");
        assert!(score > 0.9);
    }

    #[test]
    fn test_subsequence_obsidian_style() {
        // "vlt" should match "vaultiel"
        let score = subsequence_score("vlt", "vaultiel");
        assert!(score > 0.0);

        // "vi" should match "vaultiel" (v...i...e...l)
        let score = subsequence_score("vi", "vaultiel");
        assert!(score > 0.0);
    }
}
