//! Task parsing for Obsidian Tasks plugin compatibility.

use crate::config::TaskConfig;
use crate::parser::code_block::find_code_block_ranges;
use crate::parser::wikilink::parse_links;
use crate::types::{Priority, Task, TaskLink, TaskLocation};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Regex for parsing task lines.
/// Matches: optional indent, "- [symbol] ", then the rest.
static TASK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\s*)- \[(.)\] (.*)$").unwrap()
});

/// Regex for extracting dates (ISO format: YYYY-MM-DD).
static DATE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap()
});

/// Regex for block ID at end of line.
static BLOCK_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\s+\^([a-zA-Z0-9_-]+)\s*$").unwrap()
});

/// Regex for tags in task description.
static TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"#[a-zA-Z_][a-zA-Z0-9_/-]*").unwrap()
});

/// Parse all tasks from content.
pub fn parse_tasks(content: &str, file_path: &PathBuf, config: &TaskConfig) -> Vec<Task> {
    let lines: Vec<&str> = content.lines().collect();
    let code_ranges = find_code_block_ranges(content);
    let mut tasks = Vec::new();
    let mut parent_stack: Vec<(usize, usize)> = Vec::new(); // (indent, line_number)

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1;

        // Skip lines inside code blocks
        if code_ranges.iter().any(|range| line_num >= range.start_line && line_num <= range.end_line) {
            continue;
        }

        if let Some(task) = parse_task_line(line, line_num, file_path, config) {
            // Determine parent based on indentation
            let indent = task.indent;

            // Pop items from stack that are at same or deeper indent
            while !parent_stack.is_empty() && parent_stack.last().unwrap().0 >= indent {
                parent_stack.pop();
            }

            let parent_line = parent_stack.last().map(|(_, line)| *line);

            let mut task = task;
            task.parent_line = parent_line;

            // Push this task onto stack as potential parent
            parent_stack.push((indent, line_num));

            tasks.push(task);
        } else {
            // Non-task line resets the parent stack at its indentation level
            let line_indent = count_indent(line);
            while !parent_stack.is_empty() && parent_stack.last().unwrap().0 >= line_indent {
                parent_stack.pop();
            }
        }
    }

    tasks
}

/// Parse a single task line.
fn parse_task_line(
    line: &str,
    line_num: usize,
    file_path: &PathBuf,
    config: &TaskConfig,
) -> Option<Task> {
    let caps = TASK_REGEX.captures(line)?;

    let indent_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
    let symbol = format!("[{}]", caps.get(2).map(|m| m.as_str()).unwrap_or(" "));
    let rest = caps.get(3).map(|m| m.as_str()).unwrap_or("");

    let indent = count_indent(indent_str);

    // Extract block ID first (always at end)
    let (rest, block_id) = extract_block_id(rest);

    // Extract metadata from the rest
    let (description, metadata) = extract_metadata(&rest, config);

    // Extract links from description
    let links = extract_task_links(&description);

    // Extract tags from description
    let tags = extract_task_tags(&description);

    Some(Task {
        location: TaskLocation {
            file: file_path.clone(),
            line: line_num,
        },
        raw: line.to_string(),
        symbol,
        description,
        indent,
        parent_line: None, // Set later by parse_tasks
        scheduled: metadata.scheduled,
        due: metadata.due,
        done: metadata.done,
        priority: metadata.priority,
        custom: metadata.custom,
        links,
        tags,
        block_id,
    })
}

/// Count indentation level (tabs or 4 spaces = 1 level).
fn count_indent(s: &str) -> usize {
    let mut spaces = 0;
    let mut tabs = 0;

    for c in s.chars() {
        match c {
            '\t' => tabs += 1,
            ' ' => spaces += 1,
            _ => break,
        }
    }

    tabs + (spaces / 4)
}

/// Extract block ID from the end of a line.
fn extract_block_id(text: &str) -> (String, Option<String>) {
    if let Some(caps) = BLOCK_ID_REGEX.captures(text) {
        let block_id = caps.get(1).map(|m| m.as_str().to_string());
        let without_block = BLOCK_ID_REGEX.replace(text, "").to_string();
        (without_block.trim_end().to_string(), block_id)
    } else {
        (text.to_string(), None)
    }
}

/// Extracted task metadata.
struct TaskMetadata {
    scheduled: Option<String>,
    due: Option<String>,
    done: Option<String>,
    priority: Option<Priority>,
    custom: HashMap<String, String>,
}

/// Extract metadata (dates, priority, custom) from task text.
fn extract_metadata(text: &str, config: &TaskConfig) -> (String, TaskMetadata) {
    let mut remaining = text.to_string();
    let mut metadata = TaskMetadata {
        scheduled: None,
        due: None,
        done: None,
        priority: None,
        custom: HashMap::new(),
    };

    // Extract custom metadata first (they appear before standard metadata)
    for (key, emoji) in &config.custom_metadata {
        if let Some(pos) = remaining.find(emoji) {
            // Find the value after the emoji (up to next emoji or end)
            let after_emoji = &remaining[pos + emoji.len()..];
            if let Some(date_match) = DATE_REGEX.find(after_emoji.trim_start()) {
                metadata.custom.insert(key.clone(), date_match.as_str().to_string());
                remaining = format!(
                    "{}{}",
                    &remaining[..pos],
                    &remaining[pos + emoji.len() + after_emoji.find(date_match.as_str()).unwrap() + date_match.len()..]
                );
            } else {
                // Try to extract non-date value (word or time like "2h", "30m")
                let value_end = after_emoji.trim_start()
                    .find(|c: char| c.is_whitespace() || is_emoji_start(c))
                    .unwrap_or(after_emoji.trim_start().len());
                if value_end > 0 {
                    let value = after_emoji.trim_start()[..value_end].to_string();
                    metadata.custom.insert(key.clone(), value.clone());
                    let trim_start = after_emoji.len() - after_emoji.trim_start().len();
                    remaining = format!(
                        "{}{}",
                        &remaining[..pos],
                        &remaining[pos + emoji.len() + trim_start + value_end..]
                    );
                }
            }
        }
    }

    // Extract scheduled date
    if let Some(pos) = remaining.find(&config.scheduled) {
        let after = &remaining[pos + config.scheduled.len()..];
        if let Some(date_match) = DATE_REGEX.find(after.trim_start()) {
            metadata.scheduled = Some(date_match.as_str().to_string());
            let trim_start = after.len() - after.trim_start().len();
            remaining = format!(
                "{}{}",
                &remaining[..pos],
                &remaining[pos + config.scheduled.len() + trim_start + date_match.end()..]
            );
        }
    }

    // Extract due date
    if let Some(pos) = remaining.find(&config.due) {
        let after = &remaining[pos + config.due.len()..];
        if let Some(date_match) = DATE_REGEX.find(after.trim_start()) {
            metadata.due = Some(date_match.as_str().to_string());
            let trim_start = after.len() - after.trim_start().len();
            remaining = format!(
                "{}{}",
                &remaining[..pos],
                &remaining[pos + config.due.len() + trim_start + date_match.end()..]
            );
        }
    }

    // Extract done date
    if let Some(pos) = remaining.find(&config.done) {
        let after = &remaining[pos + config.done.len()..];
        if let Some(date_match) = DATE_REGEX.find(after.trim_start()) {
            metadata.done = Some(date_match.as_str().to_string());
            let trim_start = after.len() - after.trim_start().len();
            remaining = format!(
                "{}{}",
                &remaining[..pos],
                &remaining[pos + config.done.len() + trim_start + date_match.end()..]
            );
        }
    }

    // Extract priority (check from highest to lowest)
    if remaining.contains(&config.priority_highest) {
        metadata.priority = Some(Priority::Highest);
        remaining = remaining.replace(&config.priority_highest, "");
    } else if remaining.contains(&config.priority_high) {
        metadata.priority = Some(Priority::High);
        remaining = remaining.replace(&config.priority_high, "");
    } else if remaining.contains(&config.priority_medium) {
        metadata.priority = Some(Priority::Medium);
        remaining = remaining.replace(&config.priority_medium, "");
    } else if remaining.contains(&config.priority_low) {
        metadata.priority = Some(Priority::Low);
        remaining = remaining.replace(&config.priority_low, "");
    } else if remaining.contains(&config.priority_lowest) {
        metadata.priority = Some(Priority::Lowest);
        remaining = remaining.replace(&config.priority_lowest, "");
    }

    // Clean up extra whitespace
    let description = remaining.split_whitespace().collect::<Vec<_>>().join(" ");

    (description, metadata)
}

/// Check if character is likely the start of an emoji.
fn is_emoji_start(c: char) -> bool {
    // Common emoji ranges
    matches!(c,
        '\u{1F300}'..='\u{1F9FF}' | // Misc Symbols, Emoticons, etc.
        '\u{2600}'..='\u{26FF}' |   // Misc Symbols
        '\u{2700}'..='\u{27BF}' |   // Dingbats
        '\u{231A}'..='\u{231B}' |   // Watch, Hourglass
        '\u{23E9}'..='\u{23F3}'     // Various symbols
    )
}

/// Extract links from task description.
fn extract_task_links(description: &str) -> Vec<TaskLink> {
    let links = parse_links(description);
    links
        .into_iter()
        .map(|link| TaskLink {
            to: link.target,
            alias: link.alias,
        })
        .collect()
}

/// Extract tags from task description.
fn extract_task_tags(description: &str) -> Vec<String> {
    TAG_REGEX
        .find_iter(description)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Build hierarchical task tree from flat task list.
pub fn build_task_hierarchy(tasks: Vec<Task>) -> Vec<crate::types::HierarchicalTask> {
    if tasks.is_empty() {
        return Vec::new();
    }

    // Group tasks by file
    let mut tasks_by_file: HashMap<PathBuf, Vec<Task>> = HashMap::new();
    for task in tasks {
        tasks_by_file
            .entry(task.location.file.clone())
            .or_default()
            .push(task);
    }

    let mut result = Vec::new();

    for (_file, file_tasks) in tasks_by_file {
        let mut hierarchical_tasks = build_file_hierarchy(file_tasks);
        result.append(&mut hierarchical_tasks);
    }

    result
}

/// Build hierarchy for tasks within a single file.
fn build_file_hierarchy(tasks: Vec<Task>) -> Vec<crate::types::HierarchicalTask> {
    use crate::types::HierarchicalTask;

    let mut result: Vec<HierarchicalTask> = Vec::new();
    let mut task_map: HashMap<usize, usize> = HashMap::new(); // line -> index in result

    for task in tasks {
        let line = task.location.line;
        let parent_line = task.parent_line;
        let h_task: HierarchicalTask = task.into();

        if let Some(parent_ln) = parent_line {
            // Find parent and add as child
            if let Some(&parent_idx) = task_map.get(&parent_ln) {
                add_child_to_tree(&mut result, parent_idx, h_task.clone());
            } else {
                // Parent not found, add as top-level
                task_map.insert(line, result.len());
                result.push(h_task);
            }
        } else {
            // Top-level task
            task_map.insert(line, result.len());
            result.push(h_task);
        }
    }

    result
}

/// Add a child task to the tree at the correct parent.
fn add_child_to_tree(
    result: &mut [crate::types::HierarchicalTask],
    parent_idx: usize,
    child: crate::types::HierarchicalTask,
) {
    // Find the actual parent (might be nested deeper)
    fn find_and_add(
        tasks: &mut [crate::types::HierarchicalTask],
        parent_line: usize,
        child: crate::types::HierarchicalTask,
    ) -> bool {
        for task in tasks.iter_mut() {
            if task.location.line == parent_line {
                task.children.push(child);
                return true;
            }
            if find_and_add(&mut task.children, parent_line, child.clone()) {
                return true;
            }
        }
        false
    }

    let parent_line = result[parent_idx].location.line;
    if !find_and_add(result, parent_line, child.clone()) {
        // Fallback: add to parent's children directly
        result[parent_idx].children.push(child);
    }
}

/// Format a task string for Obsidian.
pub fn format_task(
    description: &str,
    symbol: &str,
    scheduled: Option<&str>,
    due: Option<&str>,
    done: Option<&str>,
    priority: Option<Priority>,
    custom: &HashMap<String, String>,
    config: &TaskConfig,
) -> String {
    let mut parts = vec![format!("- {} {}", symbol, description)];

    // Add custom metadata first
    for (key, value) in custom {
        if let Some(emoji) = config.custom_metadata.get(key) {
            parts.push(format!("{} {}", emoji, value));
        }
    }

    // Add scheduled date
    if let Some(date) = scheduled {
        parts.push(format!("{} {}", config.scheduled, date));
    }

    // Add due date
    if let Some(date) = due {
        parts.push(format!("{} {}", config.due, date));
    }

    // Add done date
    if let Some(date) = done {
        parts.push(format!("{} {}", config.done, date));
    }

    // Add priority
    if let Some(p) = priority {
        let emoji = match p {
            Priority::Highest => &config.priority_highest,
            Priority::High => &config.priority_high,
            Priority::Medium => &config.priority_medium,
            Priority::Low => &config.priority_low,
            Priority::Lowest => &config.priority_lowest,
        };
        parts.push(emoji.to_string());
    }

    parts.join(" ")
}

/// Parse a relative date string into an ISO date.
pub fn parse_relative_date(date_str: &str, today: chrono::NaiveDate) -> Option<String> {
    let lower = date_str.to_lowercase();

    if lower == "today" {
        return Some(today.format("%Y-%m-%d").to_string());
    }

    if lower == "tomorrow" {
        return Some((today + chrono::Duration::days(1)).format("%Y-%m-%d").to_string());
    }

    if lower == "yesterday" {
        return Some((today - chrono::Duration::days(1)).format("%Y-%m-%d").to_string());
    }

    // Parse offset format: +3d, -1w, +2m
    if let Some(offset_str) = lower.strip_prefix('+').or_else(|| lower.strip_prefix('-')) {
        let is_negative = lower.starts_with('-');
        let len = offset_str.len();
        if len >= 2 {
            let unit = &offset_str[len - 1..];
            if let Ok(amount) = offset_str[..len - 1].parse::<i64>() {
                let amount = if is_negative { -amount } else { amount };
                let result_date = match unit {
                    "d" => today + chrono::Duration::days(amount),
                    "w" => today + chrono::Duration::weeks(amount),
                    "m" => {
                        // Approximate month as 30 days
                        today + chrono::Duration::days(amount * 30)
                    }
                    _ => return None,
                };
                return Some(result_date.format("%Y-%m-%d").to_string());
            }
        }
    }

    // Try to parse as ISO date directly
    if chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_ok() {
        return Some(date_str.to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> TaskConfig {
        TaskConfig::default()
    }

    #[test]
    fn test_parse_simple_task() {
        let content = "- [ ] A simple task";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &default_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].symbol, "[ ]");
        assert_eq!(tasks[0].description, "A simple task");
        assert_eq!(tasks[0].indent, 0);
    }

    #[test]
    fn test_parse_completed_task() {
        let content = "- [x] Completed task";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &default_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].symbol, "[x]");
    }

    #[test]
    fn test_parse_task_with_dates() {
        let content = "- [ ] Task with dates ⏳ 2026-02-05 📅 2026-02-10";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &default_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].scheduled, Some("2026-02-05".to_string()));
        assert_eq!(tasks[0].due, Some("2026-02-10".to_string()));
    }

    #[test]
    fn test_parse_task_with_priority() {
        let content = "- [ ] High priority task ⏫";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &default_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].priority, Some(Priority::High));
    }

    #[test]
    fn test_parse_task_with_links() {
        let content = "- [ ] Task linking to [[Note A]] and [[Note B|alias]]";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &default_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].links.len(), 2);
        assert_eq!(tasks[0].links[0].to, "Note A");
        assert_eq!(tasks[0].links[1].to, "Note B");
        assert_eq!(tasks[0].links[1].alias, Some("alias".to_string()));
    }

    #[test]
    fn test_parse_task_with_tags() {
        let content = "- [ ] Task with #tag1 and #project/subtag";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &default_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].tags, vec!["#tag1", "#project/subtag"]);
    }

    #[test]
    fn test_parse_task_with_block_id() {
        let content = "- [ ] Task with block ID ^abc123";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &default_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].block_id, Some("abc123".to_string()));
    }

    #[test]
    fn test_parse_nested_tasks() {
        let content = "- [ ] Parent task\n\t- [ ] Child task\n\t\t- [ ] Grandchild task";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &default_config());

        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].indent, 0);
        assert_eq!(tasks[0].parent_line, None);
        assert_eq!(tasks[1].indent, 1);
        assert_eq!(tasks[1].parent_line, Some(1));
        assert_eq!(tasks[2].indent, 2);
        assert_eq!(tasks[2].parent_line, Some(2));
    }

    #[test]
    fn test_count_indent() {
        assert_eq!(count_indent(""), 0);
        assert_eq!(count_indent("    "), 1);
        assert_eq!(count_indent("\t"), 1);
        assert_eq!(count_indent("\t\t"), 2);
        assert_eq!(count_indent("        "), 2);
        assert_eq!(count_indent("  "), 0); // 2 spaces is not enough for a level
    }

    #[test]
    fn test_format_task() {
        let config = default_config();
        let result = format_task(
            "My task",
            "[ ]",
            Some("2026-02-05"),
            Some("2026-02-10"),
            None,
            Some(Priority::High),
            &HashMap::new(),
            &config,
        );

        assert!(result.contains("- [ ] My task"));
        assert!(result.contains("⏳ 2026-02-05"));
        assert!(result.contains("📅 2026-02-10"));
        assert!(result.contains("⏫"));
    }

    #[test]
    fn test_parse_relative_date() {
        let today = chrono::NaiveDate::from_ymd_opt(2026, 2, 2).unwrap();

        assert_eq!(
            parse_relative_date("today", today),
            Some("2026-02-02".to_string())
        );
        assert_eq!(
            parse_relative_date("tomorrow", today),
            Some("2026-02-03".to_string())
        );
        assert_eq!(
            parse_relative_date("yesterday", today),
            Some("2026-02-01".to_string())
        );
        assert_eq!(
            parse_relative_date("+3d", today),
            Some("2026-02-05".to_string())
        );
        assert_eq!(
            parse_relative_date("-1w", today),
            Some("2026-01-26".to_string())
        );
    }

    #[test]
    fn test_build_task_hierarchy() {
        let tasks = vec![
            Task {
                location: TaskLocation {
                    file: PathBuf::from("test.md"),
                    line: 1,
                },
                raw: "- [ ] Parent".to_string(),
                symbol: "[ ]".to_string(),
                description: "Parent".to_string(),
                indent: 0,
                parent_line: None,
                scheduled: None,
                due: None,
                done: None,
                priority: None,
                custom: HashMap::new(),
                links: vec![],
                tags: vec![],
                block_id: None,
            },
            Task {
                location: TaskLocation {
                    file: PathBuf::from("test.md"),
                    line: 2,
                },
                raw: "- [ ] Child".to_string(),
                symbol: "[ ]".to_string(),
                description: "Child".to_string(),
                indent: 1,
                parent_line: Some(1),
                scheduled: None,
                due: None,
                done: None,
                priority: None,
                custom: HashMap::new(),
                links: vec![],
                tags: vec![],
                block_id: None,
            },
        ];

        let hierarchy = build_task_hierarchy(tasks);

        assert_eq!(hierarchy.len(), 1);
        assert_eq!(hierarchy[0].description, "Parent");
        assert_eq!(hierarchy[0].children.len(), 1);
        assert_eq!(hierarchy[0].children[0].description, "Child");
    }
}
