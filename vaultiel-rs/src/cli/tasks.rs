//! Task-related CLI commands.

use crate::cli::output::Output;
use crate::config::TaskConfig;
use crate::error::{ExitCode, Result};
use crate::parser::task::{build_task_hierarchy, format_task, FormatTaskParams, parse_relative_date, parse_tasks};
use crate::types::{HierarchicalTask, Priority, Task};
use crate::vault::Vault;
use chrono::Local;
use serde::Serialize;
use std::collections::HashMap;

/// Output for get-tasks command (flat).
#[derive(Debug, Serialize)]
pub struct TasksOutput {
    pub tasks: Vec<Task>,
}

/// Output for get-tasks command (hierarchical).
#[derive(Debug, Serialize)]
pub struct HierarchicalTasksOutput {
    pub tasks: Vec<HierarchicalTask>,
}

/// Output for format-task command.
#[derive(Debug, Serialize)]
pub struct FormatTaskOutput {
    pub formatted: String,
}

/// Filter options for task queries.
#[derive(Debug, Default)]
pub struct TaskFilter {
    pub symbols: Vec<String>,
    pub due_before: Option<String>,
    pub due_after: Option<String>,
    pub due_on: Option<String>,
    pub scheduled_before: Option<String>,
    pub scheduled_after: Option<String>,
    pub scheduled_on: Option<String>,
    pub done_before: Option<String>,
    pub done_after: Option<String>,
    pub done_on: Option<String>,
    pub start_before: Option<String>,
    pub start_after: Option<String>,
    pub start_on: Option<String>,
    pub created_before: Option<String>,
    pub created_after: Option<String>,
    pub created_on: Option<String>,
    pub has_recurrence: bool,
    pub id_filter: Option<String>,
    pub depends_on_filter: Option<String>,
    pub priority: Option<Priority>,
    pub contains: Option<String>,
    pub has_metadata: Vec<String>,
    pub links_to: Option<String>,
    pub tag: Option<String>,
    pub has_block_ref: bool,
    pub block_ref: Option<String>,
}

impl TaskFilter {
    pub fn matches(&self, task: &Task) -> bool {
        // Symbol filter
        if !self.symbols.is_empty() && !self.symbols.contains(&task.symbol) {
            return false;
        }

        // Due date filters
        if let Some(ref before) = self.due_before {
            if let Some(ref due) = task.due {
                if due >= before {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(ref after) = self.due_after {
            if let Some(ref due) = task.due {
                if due <= after {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(ref on) = self.due_on {
            if task.due.as_ref() != Some(on) {
                return false;
            }
        }

        // Scheduled date filters
        if let Some(ref before) = self.scheduled_before {
            if let Some(ref scheduled) = task.scheduled {
                if scheduled >= before {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(ref after) = self.scheduled_after {
            if let Some(ref scheduled) = task.scheduled {
                if scheduled <= after {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(ref on) = self.scheduled_on {
            if task.scheduled.as_ref() != Some(on) {
                return false;
            }
        }

        // Done date filters
        if let Some(ref before) = self.done_before {
            if let Some(ref done) = task.done {
                if done >= before {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(ref after) = self.done_after {
            if let Some(ref done) = task.done {
                if done <= after {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(ref on) = self.done_on {
            if task.done.as_ref() != Some(on) {
                return false;
            }
        }

        // Start date filters
        if let Some(ref before) = self.start_before {
            if let Some(ref start) = task.start {
                if start >= before {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(ref after) = self.start_after {
            if let Some(ref start) = task.start {
                if start <= after {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(ref on) = self.start_on {
            if task.start.as_ref() != Some(on) {
                return false;
            }
        }

        // Created date filters
        if let Some(ref before) = self.created_before {
            if let Some(ref created) = task.created {
                if created >= before {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(ref after) = self.created_after {
            if let Some(ref created) = task.created {
                if created <= after {
                    return false;
                }
            } else {
                return false;
            }
        }
        if let Some(ref on) = self.created_on {
            if task.created.as_ref() != Some(on) {
                return false;
            }
        }

        // Recurrence filter
        if self.has_recurrence && task.recurrence.is_none() {
            return false;
        }

        // ID filter
        if let Some(ref id) = self.id_filter {
            if task.id.as_ref() != Some(id) {
                return false;
            }
        }

        // Depends on filter
        if let Some(ref dep) = self.depends_on_filter {
            if !task.depends_on.contains(dep) {
                return false;
            }
        }

        // Priority filter
        if let Some(ref priority) = self.priority {
            if task.priority.as_ref() != Some(priority) {
                return false;
            }
        }

        // Contains filter (case-insensitive)
        if let Some(ref text) = self.contains {
            if !task.description.to_lowercase().contains(&text.to_lowercase()) {
                return false;
            }
        }

        // Has metadata filter
        for key in &self.has_metadata {
            if !task.custom.contains_key(key) {
                return false;
            }
        }

        // Links to filter
        if let Some(ref target) = self.links_to {
            let target_normalized = target.trim_end_matches(".md").to_lowercase();
            let has_link = task.links.iter().any(|link| {
                link.to.trim_end_matches(".md").to_lowercase() == target_normalized
            });
            if !has_link {
                return false;
            }
        }

        // Tag filter
        if let Some(ref tag) = self.tag {
            let tag_normalized = if tag.starts_with('#') {
                tag.clone()
            } else {
                format!("#{}", tag)
            };
            if !task.tags.iter().any(|t| t == &tag_normalized || t.starts_with(&format!("{}/", tag_normalized))) {
                return false;
            }
        }

        // Block ref filters
        if self.has_block_ref && task.block_id.is_none() {
            return false;
        }
        if let Some(ref block_id) = self.block_ref {
            if task.block_id.as_ref() != Some(block_id) {
                return false;
            }
        }

        true
    }
}

/// Get tasks from the vault.
pub fn get_tasks(
    vault: &Vault,
    note_path: Option<&str>,
    glob_pattern: Option<&str>,
    filter: TaskFilter,
    flat: bool,
    output: &Output,
) -> Result<ExitCode> {
    let task_config: TaskConfig = (&vault.config.tasks).into();
    let mut all_tasks = Vec::new();

    // Determine which notes to scan
    let notes = if let Some(path) = note_path {
        let note_path = vault.resolve_note(path)?;
        vec![note_path]
    } else if let Some(pattern) = glob_pattern {
        vault.list_notes_matching(pattern)?
    } else {
        vault.list_notes()?
    };

    // Parse tasks from each note
    for note_path in notes {
        if let Ok(note) = vault.load_note(&note_path) {
            let tasks = parse_tasks(&note.content, &note_path, &task_config);
            all_tasks.extend(tasks);
        }
    }

    // Apply filters
    let filtered_tasks: Vec<Task> = all_tasks
        .into_iter()
        .filter(|t| filter.matches(t))
        .collect();

    if flat {
        let result = TasksOutput {
            tasks: filtered_tasks,
        };
        output.print(&result)?;
    } else {
        let hierarchical = build_task_hierarchy(filtered_tasks);
        let result = HierarchicalTasksOutput {
            tasks: hierarchical,
        };
        output.print(&result)?;
    }

    Ok(ExitCode::Success)
}

/// Parameters for the format-task CLI command.
pub struct FormatTaskCommandParams<'a> {
    pub description: &'a str,
    pub symbol: &'a str,
    pub scheduled: Option<&'a str>,
    pub due: Option<&'a str>,
    pub done: Option<&'a str>,
    pub start: Option<&'a str>,
    pub created: Option<&'a str>,
    pub cancelled: Option<&'a str>,
    pub recurrence: Option<&'a str>,
    pub on_completion: Option<&'a str>,
    pub id: Option<&'a str>,
    pub depends_on: &'a [String],
    pub priority: Option<Priority>,
    pub custom: &'a HashMap<String, String>,
}

/// Format a task string for Obsidian.
pub fn format_task_command(
    params: &FormatTaskCommandParams,
    vault: &Vault,
    output: &Output,
) -> Result<ExitCode> {
    let task_config: TaskConfig = (&vault.config.tasks).into();
    let today = Local::now().date_naive();

    // Resolve relative dates
    let scheduled_resolved = params.scheduled.and_then(|s| parse_relative_date(s, today));
    let due_resolved = params.due.and_then(|s| parse_relative_date(s, today));
    let done_resolved = params.done.and_then(|s| parse_relative_date(s, today));
    let start_resolved = params.start.and_then(|s| parse_relative_date(s, today));
    let created_resolved = params.created.and_then(|s| parse_relative_date(s, today));
    let cancelled_resolved = params.cancelled.and_then(|s| parse_relative_date(s, today));

    let formatted = format_task(
        &FormatTaskParams {
            description: params.description,
            symbol: params.symbol,
            scheduled: scheduled_resolved.as_deref(),
            due: due_resolved.as_deref(),
            done: done_resolved.as_deref(),
            start: start_resolved.as_deref(),
            created: created_resolved.as_deref(),
            cancelled: cancelled_resolved.as_deref(),
            recurrence: params.recurrence,
            on_completion: params.on_completion,
            id: params.id,
            depends_on: params.depends_on,
            priority: params.priority,
            custom: params.custom,
        },
        &task_config,
    );

    let result = FormatTaskOutput { formatted };
    output.print(&result)?;

    Ok(ExitCode::Success)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TaskLocation;
    use std::path::PathBuf;

    fn make_task(description: &str) -> Task {
        Task {
            location: TaskLocation {
                file: PathBuf::from("test.md"),
                line: 1,
            },
            raw: format!("- [ ] {}", description),
            symbol: "[ ]".to_string(),
            description: description.to_string(),
            indent: 0,
            parent_line: None,
            scheduled: None,
            due: None,
            done: None,
            start: None,
            created: None,
            cancelled: None,
            recurrence: None,
            on_completion: None,
            id: None,
            depends_on: vec![],
            priority: None,
            custom: HashMap::new(),
            links: vec![],
            tags: vec![],
            block_id: None,
        }
    }

    #[test]
    fn test_filter_by_symbol() {
        let filter = TaskFilter {
            symbols: vec!["[x]".to_string()],
            ..Default::default()
        };

        let task1 = make_task("Open task");
        let mut task2 = make_task("Done task");
        task2.symbol = "[x]".to_string();

        assert!(!filter.matches(&task1));
        assert!(filter.matches(&task2));
    }

    #[test]
    fn test_filter_by_due_date() {
        let filter = TaskFilter {
            due_before: Some("2026-02-10".to_string()),
            ..Default::default()
        };

        let mut task1 = make_task("Task 1");
        task1.due = Some("2026-02-05".to_string());

        let mut task2 = make_task("Task 2");
        task2.due = Some("2026-02-15".to_string());

        let task3 = make_task("Task 3"); // No due date

        assert!(filter.matches(&task1));
        assert!(!filter.matches(&task2));
        assert!(!filter.matches(&task3));
    }

    #[test]
    fn test_filter_by_contains() {
        let filter = TaskFilter {
            contains: Some("important".to_string()),
            ..Default::default()
        };

        let task1 = make_task("This is important task");
        let task2 = make_task("Regular task");

        assert!(filter.matches(&task1));
        assert!(!filter.matches(&task2));
    }

    #[test]
    fn test_filter_by_priority() {
        let filter = TaskFilter {
            priority: Some(Priority::High),
            ..Default::default()
        };

        let mut task1 = make_task("High priority");
        task1.priority = Some(Priority::High);

        let mut task2 = make_task("Low priority");
        task2.priority = Some(Priority::Low);

        assert!(filter.matches(&task1));
        assert!(!filter.matches(&task2));
    }
}
