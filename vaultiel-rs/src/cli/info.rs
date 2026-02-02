//! Vault info command.

use crate::cli::output::Output;
use crate::config::TaskConfig;
use crate::error::{ExitCode, Result};
use crate::graph::LinkGraph;
use crate::health::{HealthChecker, IssueType};
use crate::parser::{parse_tags, parse_tasks};
use crate::vault::Vault;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Basic vault info output.
#[derive(Debug, Serialize)]
pub struct InfoOutput {
    pub vault_path: PathBuf,
    pub note_count: usize,
    pub total_size_bytes: u64,
    pub link_count: usize,
    pub tag_count: usize,
    pub task_count: usize,
    pub orphan_count: usize,
    pub broken_link_count: usize,
}

/// Detailed vault info output.
#[derive(Debug, Serialize)]
pub struct DetailedInfoOutput {
    pub vault_path: PathBuf,
    pub note_count: usize,
    pub total_size_bytes: u64,
    pub link_count: usize,
    pub tag_count: usize,
    pub task_count: usize,
    pub orphan_count: usize,
    pub broken_link_count: usize,
    pub notes_by_folder: HashMap<String, usize>,
    pub top_tags: Vec<TagCount>,
    pub top_linked: Vec<LinkedNote>,
    pub recently_modified: Vec<RecentNote>,
}

#[derive(Debug, Serialize)]
pub struct TagCount {
    pub tag: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct LinkedNote {
    pub note: PathBuf,
    pub incoming: usize,
}

#[derive(Debug, Serialize)]
pub struct RecentNote {
    pub note: PathBuf,
    pub modified: String,
}

/// Get vault info.
pub fn info(vault: &Vault, detailed: bool, output: &Output) -> Result<ExitCode> {
    let notes = vault.list_notes()?;
    let note_count = notes.len();

    // Calculate total size
    let mut total_size_bytes: u64 = 0;
    for note_path in &notes {
        let full_path = vault.root.join(note_path);
        if let Ok(metadata) = std::fs::metadata(&full_path) {
            total_size_bytes += metadata.len();
        }
    }

    // Build link graph
    let graph = LinkGraph::build(vault)?;

    // Count links
    let mut link_count = 0;
    for note_path in &notes {
        link_count += graph.get_outgoing(note_path).len();
    }

    // Count unique tags
    let mut all_tags: std::collections::HashSet<String> = std::collections::HashSet::new();
    let task_config = TaskConfig::default();

    for note_path in &notes {
        if let Ok(note) = vault.load_note(note_path) {
            let tags = parse_tags(&note.content);
            for tag in tags {
                all_tags.insert(tag.name);
            }
        }
    }
    let tag_count = all_tags.len();

    // Count tasks
    let mut task_count = 0;
    for note_path in &notes {
        if let Ok(note) = vault.load_note(note_path) {
            let tasks = parse_tasks(&note.content, note_path, &task_config);
            task_count += tasks.len();
        }
    }

    // Count orphans
    let mut orphan_count = 0;
    for note_path in &notes {
        if graph.get_incoming(note_path).is_empty() {
            orphan_count += 1;
        }
    }

    // Count broken links
    let checker = HealthChecker::new(vault).only(vec![IssueType::BrokenLinks]);
    let issues = checker.run()?;
    let broken_link_count = issues.len();

    if detailed {
        // Notes by folder
        let mut notes_by_folder: HashMap<String, usize> = HashMap::new();
        for note_path in &notes {
            let folder = note_path
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or(".")
                .to_string();
            let folder = if folder.is_empty() { ".".to_string() } else { folder };
            *notes_by_folder.entry(folder).or_insert(0) += 1;
        }

        // Top tags
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        for note_path in &notes {
            if let Ok(note) = vault.load_note(note_path) {
                let tags = parse_tags(&note.content);
                for tag in tags {
                    *tag_counts.entry(tag.name).or_insert(0) += 1;
                }
            }
        }
        let mut top_tags: Vec<TagCount> = tag_counts
            .into_iter()
            .map(|(tag, count)| TagCount { tag, count })
            .collect();
        top_tags.sort_by(|a, b| b.count.cmp(&a.count));
        top_tags.truncate(10);

        // Top linked notes
        let mut incoming_counts: HashMap<PathBuf, usize> = HashMap::new();
        for note_path in &notes {
            let incoming = graph.get_incoming(note_path);
            incoming_counts.insert(note_path.clone(), incoming.len());
        }
        let mut top_linked: Vec<LinkedNote> = incoming_counts
            .into_iter()
            .filter(|(_, count)| *count > 0)
            .map(|(note, incoming)| LinkedNote { note, incoming })
            .collect();
        top_linked.sort_by(|a, b| b.incoming.cmp(&a.incoming));
        top_linked.truncate(10);

        // Recently modified
        let mut recent_notes: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();
        for note_path in &notes {
            let full_path = vault.root.join(note_path);
            if let Ok(metadata) = std::fs::metadata(&full_path) {
                if let Ok(modified) = metadata.modified() {
                    recent_notes.push((note_path.clone(), modified));
                }
            }
        }
        recent_notes.sort_by(|a, b| b.1.cmp(&a.1));
        recent_notes.truncate(10);

        let recently_modified: Vec<RecentNote> = recent_notes
            .into_iter()
            .map(|(note, modified)| {
                let datetime: chrono::DateTime<chrono::Utc> = modified.into();
                RecentNote {
                    note,
                    modified: datetime.to_rfc3339(),
                }
            })
            .collect();

        let result = DetailedInfoOutput {
            vault_path: vault.root.to_path_buf(),
            note_count,
            total_size_bytes,
            link_count,
            tag_count,
            task_count,
            orphan_count,
            broken_link_count,
            notes_by_folder,
            top_tags,
            top_linked,
            recently_modified,
        };
        output.print(&result)?;
    } else {
        let result = InfoOutput {
            vault_path: vault.root.to_path_buf(),
            note_count,
            total_size_bytes,
            link_count,
            tag_count,
            task_count,
            orphan_count,
            broken_link_count,
        };
        output.print(&result)?;
    }

    Ok(ExitCode::Success)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_count_serialization() {
        let tc = TagCount {
            tag: "#rust".to_string(),
            count: 5,
        };
        let json = serde_json::to_string(&tc).unwrap();
        assert!(json.contains("\"tag\":\"#rust\""));
        assert!(json.contains("\"count\":5"));
    }
}
