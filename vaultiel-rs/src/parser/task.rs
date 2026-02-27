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
        start: metadata.start,
        created: metadata.created,
        cancelled: metadata.cancelled,
        recurrence: metadata.recurrence,
        on_completion: metadata.on_completion,
        id: metadata.id,
        depends_on: metadata.depends_on,
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
    start: Option<String>,
    created: Option<String>,
    cancelled: Option<String>,
    recurrence: Option<String>,
    on_completion: Option<String>,
    id: Option<String>,
    depends_on: Vec<String>,
    priority: Option<Priority>,
    custom: HashMap<String, String>,
}

/// Extract a date field: find emoji, extract ISO date after it, remove from string.
fn extract_date_field(remaining: &mut String, emoji: &str) -> Option<String> {
    if let Some(pos) = remaining.find(emoji) {
        let after = &remaining[pos + emoji.len()..];
        if let Some(date_match) = DATE_REGEX.find(after.trim_start()) {
            let value = date_match.as_str().to_string();
            let trim_start = after.len() - after.trim_start().len();
            *remaining = format!(
                "{}{}",
                &remaining[..pos],
                &remaining[pos + emoji.len() + trim_start + date_match.end()..]
            );
            return Some(value);
        }
    }
    None
}

/// Extract a text field: find emoji, extract text until next emoji, remove from string.
fn extract_text_field(remaining: &mut String, emoji: &str) -> Option<String> {
    if let Some(pos) = remaining.find(emoji) {
        let after = &remaining[pos + emoji.len()..];
        let after_trimmed = after.trim_start();
        // Find the next emoji (start of another field)
        let value_end = after_trimmed
            .find(|c: char| is_emoji_start(c))
            .unwrap_or(after_trimmed.len());
        let value = after_trimmed[..value_end].trim().to_string();
        if !value.is_empty() {
            let trim_start = after.len() - after_trimmed.len();
            *remaining = format!(
                "{}{}",
                &remaining[..pos],
                &remaining[pos + emoji.len() + trim_start + value_end..]
            );
            return Some(value);
        }
    }
    None
}

/// Extract metadata (dates, priority, custom) from task text.
fn extract_metadata(text: &str, config: &TaskConfig) -> (String, TaskMetadata) {
    let mut remaining = text.to_string();
    let mut metadata = TaskMetadata {
        scheduled: None,
        due: None,
        done: None,
        start: None,
        created: None,
        cancelled: None,
        recurrence: None,
        on_completion: None,
        id: None,
        depends_on: Vec::new(),
        priority: None,
        custom: HashMap::new(),
    };

    // Extract custom metadata first (they appear before standard metadata)
    for (key, emoji) in &config.custom_metadata {
        if let Some(pos) = remaining.find(emoji) {
            // Find the value after the emoji — scope to text before next emoji
            let after_emoji = &remaining[pos + emoji.len()..];
            let trimmed = after_emoji.trim_start();
            let trim_start = after_emoji.len() - trimmed.len();

            // Find end of immediate value (up to next emoji or whitespace-then-emoji)
            let value_end = trimmed
                .find(|c: char| is_emoji_start(c))
                .unwrap_or(trimmed.len());
            let immediate_value = trimmed[..value_end].trim();

            if !immediate_value.is_empty() {
                // Check if it's a date within the scoped value
                let value = if let Some(date_match) = DATE_REGEX.find(immediate_value) {
                    date_match.as_str().to_string()
                } else {
                    // Extract first word/token (e.g., "2h", "30m", "1")
                    let word_end = immediate_value
                        .find(|c: char| c.is_whitespace())
                        .unwrap_or(immediate_value.len());
                    immediate_value[..word_end].to_string()
                };
                metadata.custom.insert(key.clone(), value.clone());
                // Remove the emoji and its value from remaining
                let consume_len = trim_start + value_end;
                remaining = format!(
                    "{}{}",
                    &remaining[..pos],
                    &remaining[pos + emoji.len() + consume_len..]
                );
            }
        }
    }

    // Extract text-based fields (before date fields, since they consume until next emoji)
    // ID: single word/value
    metadata.id = extract_text_field(&mut remaining, &config.id);

    // Depends on: comma-separated IDs
    if let Some(raw) = extract_text_field(&mut remaining, &config.depends_on) {
        metadata.depends_on = raw.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    }

    // Recurrence: multi-word text (e.g., "every 2 weeks")
    metadata.recurrence = extract_text_field(&mut remaining, &config.recurrence);

    // On completion: single word (e.g., "delete")
    metadata.on_completion = extract_text_field(&mut remaining, &config.on_completion);

    // Extract date fields
    metadata.start = extract_date_field(&mut remaining, &config.start);
    metadata.created = extract_date_field(&mut remaining, &config.created);
    metadata.scheduled = extract_date_field(&mut remaining, &config.scheduled);
    metadata.due = extract_date_field(&mut remaining, &config.due);
    metadata.cancelled = extract_date_field(&mut remaining, &config.cancelled);
    metadata.done = extract_date_field(&mut remaining, &config.done);

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
    // Common emoji ranges used in Obsidian Tasks
    matches!(c,
        '\u{1F100}'..='\u{1F1FF}' | // Enclosed Alphanumeric Supplement (🆔 U+1F194)
        '\u{1F300}'..='\u{1F9FF}' | // Misc Symbols, Emoticons, etc.
        '\u{2600}'..='\u{26FF}' |   // Misc Symbols (⛔ U+26D4)
        '\u{2700}'..='\u{27BF}' |   // Dingbats (➕ U+2795, ❌ U+274C, ✅ U+2705)
        '\u{231A}'..='\u{231B}' |   // Watch, Hourglass
        '\u{23E9}'..='\u{23F3}'     // Various symbols (⏳ U+23F3, ⏫ U+23EB, ⏬ U+23EC, ⏲ U+23F2)
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

/// Parameters for formatting a task.
#[derive(Debug)]
pub struct FormatTaskParams<'a> {
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

impl<'a> Default for FormatTaskParams<'a> {
    fn default() -> Self {
        // Use a leaked empty HashMap for the default — this is a static empty reference
        static EMPTY_MAP: std::sync::LazyLock<HashMap<String, String>> =
            std::sync::LazyLock::new(HashMap::new);
        Self {
            description: "",
            symbol: "[ ]",
            scheduled: None,
            due: None,
            done: None,
            start: None,
            created: None,
            cancelled: None,
            recurrence: None,
            on_completion: None,
            id: None,
            depends_on: &[],
            priority: None,
            custom: &EMPTY_MAP,
        }
    }
}

/// Format a task string for Obsidian.
///
/// Uses Obsidian Tasks canonical order:
/// description → custom metadata → id → depends_on → priority → recurrence →
/// on_completion → created → start → scheduled → due → cancelled → done
pub fn format_task(params: &FormatTaskParams, config: &TaskConfig) -> String {
    let mut parts = vec![format!("- {} {}", params.symbol, params.description)];

    // Custom metadata (before recognized fields so Obsidian Tasks can parse from end)
    for (key, value) in params.custom {
        if let Some(emoji) = config.custom_metadata.get(key) {
            parts.push(format!("{} {}", emoji, value));
        }
    }

    // Obsidian Tasks canonical field order
    if let Some(id) = params.id {
        parts.push(format!("{} {}", config.id, id));
    }

    if !params.depends_on.is_empty() {
        parts.push(format!("{} {}", config.depends_on, params.depends_on.join(",")));
    }

    if let Some(p) = params.priority {
        let emoji = match p {
            Priority::Highest => &config.priority_highest,
            Priority::High => &config.priority_high,
            Priority::Medium => &config.priority_medium,
            Priority::Low => &config.priority_low,
            Priority::Lowest => &config.priority_lowest,
        };
        parts.push(emoji.to_string());
    }

    if let Some(recurrence) = params.recurrence {
        parts.push(format!("{} {}", config.recurrence, recurrence));
    }

    if let Some(on_completion) = params.on_completion {
        parts.push(format!("{} {}", config.on_completion, on_completion));
    }

    if let Some(date) = params.created {
        parts.push(format!("{} {}", config.created, date));
    }

    if let Some(date) = params.start {
        parts.push(format!("{} {}", config.start, date));
    }

    if let Some(date) = params.scheduled {
        parts.push(format!("{} {}", config.scheduled, date));
    }

    if let Some(date) = params.due {
        parts.push(format!("{} {}", config.due, date));
    }

    if let Some(date) = params.cancelled {
        parts.push(format!("{} {}", config.cancelled, date));
    }

    if let Some(date) = params.done {
        parts.push(format!("{} {}", config.done, date));
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
        let empty_custom = HashMap::new();
        let empty_deps: Vec<String> = vec![];
        let result = format_task(
            &FormatTaskParams {
                description: "My task",
                symbol: "[ ]",
                scheduled: Some("2026-02-05"),
                due: Some("2026-02-10"),
                priority: Some(Priority::High),
                custom: &empty_custom,
                depends_on: &empty_deps,
                ..Default::default()
            },
            &config,
        );

        assert!(result.contains("- [ ] My task"));
        assert!(result.contains("⏳ 2026-02-05"));
        assert!(result.contains("📅 2026-02-10"));
        assert!(result.contains("⏫"));
    }

    #[test]
    fn test_parse_task_with_start_date() {
        let content = "- [ ] Task with start 🛫 2026-03-01";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &default_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].start, Some("2026-03-01".to_string()));
    }

    #[test]
    fn test_parse_task_with_created_date() {
        let content = "- [ ] Task ➕ 2026-02-20";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &default_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].created, Some("2026-02-20".to_string()));
    }

    #[test]
    fn test_parse_task_with_cancelled_date() {
        let content = "- [-] Cancelled task ❌ 2026-02-25";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &default_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].cancelled, Some("2026-02-25".to_string()));
    }

    #[test]
    fn test_parse_task_with_recurrence() {
        let content = "- [ ] Recurring task 🔁 every week 📅 2026-03-01";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &default_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].recurrence, Some("every week".to_string()));
        assert_eq!(tasks[0].due, Some("2026-03-01".to_string()));
    }

    #[test]
    fn test_parse_task_with_id_and_depends() {
        let content = "- [ ] Task 🆔 abc123 ⛔ def456,ghi789 📅 2026-03-01";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &default_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, Some("abc123".to_string()));
        assert_eq!(tasks[0].depends_on, vec!["def456", "ghi789"]);
        assert_eq!(tasks[0].due, Some("2026-03-01".to_string()));
    }

    #[test]
    fn test_parse_task_with_on_completion() {
        let content = "- [ ] Task 🏁 delete 📅 2026-03-01";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &default_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].on_completion, Some("delete".to_string()));
        assert_eq!(tasks[0].due, Some("2026-03-01".to_string()));
    }

    #[test]
    fn test_parse_task_all_fields() {
        let content = "- [ ] Full task 🆔 myid ⛔ dep1 ⏫ 🔁 every day 🏁 delete ➕ 2026-01-01 🛫 2026-02-01 ⏳ 2026-02-15 📅 2026-03-01 ❌ 2026-02-20 ✅ 2026-02-25";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &default_config());

        assert_eq!(tasks.len(), 1);
        let task = &tasks[0];
        assert_eq!(task.description, "Full task");
        assert_eq!(task.id, Some("myid".to_string()));
        assert_eq!(task.depends_on, vec!["dep1"]);
        assert_eq!(task.priority, Some(Priority::High));
        assert_eq!(task.recurrence, Some("every day".to_string()));
        assert_eq!(task.on_completion, Some("delete".to_string()));
        assert_eq!(task.created, Some("2026-01-01".to_string()));
        assert_eq!(task.start, Some("2026-02-01".to_string()));
        assert_eq!(task.scheduled, Some("2026-02-15".to_string()));
        assert_eq!(task.due, Some("2026-03-01".to_string()));
        assert_eq!(task.cancelled, Some("2026-02-20".to_string()));
        assert_eq!(task.done, Some("2026-02-25".to_string()));
    }

    #[test]
    fn test_format_task_canonical_order() {
        let config = default_config();
        let empty_custom = HashMap::new();
        let deps = vec!["dep1".to_string()];
        let result = format_task(
            &FormatTaskParams {
                description: "Task",
                symbol: "[ ]",
                id: Some("myid"),
                depends_on: &deps,
                priority: Some(Priority::High),
                recurrence: Some("every week"),
                on_completion: Some("delete"),
                created: Some("2026-01-01"),
                start: Some("2026-02-01"),
                scheduled: Some("2026-02-15"),
                due: Some("2026-03-01"),
                cancelled: Some("2026-02-20"),
                done: Some("2026-02-25"),
                custom: &empty_custom,
            },
            &config,
        );

        // Verify canonical order: id → depends → priority → recurrence → on_completion → created → start → scheduled → due → cancelled → done
        let id_pos = result.find("🆔").unwrap();
        let dep_pos = result.find("⛔").unwrap();
        let pri_pos = result.find("⏫").unwrap();
        let rec_pos = result.find("🔁").unwrap();
        let oc_pos = result.find("🏁").unwrap();
        let cre_pos = result.find("➕").unwrap();
        let start_pos = result.find("🛫").unwrap();
        let sch_pos = result.find("⏳").unwrap();
        let due_pos = result.find("📅").unwrap();
        let can_pos = result.find("❌").unwrap();
        let done_pos = result.find("✅").unwrap();

        assert!(id_pos < dep_pos);
        assert!(dep_pos < pri_pos);
        assert!(pri_pos < rec_pos);
        assert!(rec_pos < oc_pos);
        assert!(oc_pos < cre_pos);
        assert!(cre_pos < start_pos);
        assert!(start_pos < sch_pos);
        assert!(sch_pos < due_pos);
        assert!(due_pos < can_pos);
        assert!(can_pos < done_pos);
    }

    #[test]
    fn test_format_task_round_trip() {
        let config = default_config();
        let empty_custom = HashMap::new();
        let deps = vec!["dep1".to_string()];
        let formatted = format_task(
            &FormatTaskParams {
                description: "Round trip task",
                symbol: "[ ]",
                id: Some("myid"),
                depends_on: &deps,
                priority: Some(Priority::High),
                recurrence: Some("every week"),
                created: Some("2026-01-01"),
                start: Some("2026-02-01"),
                scheduled: Some("2026-02-15"),
                due: Some("2026-03-01"),
                custom: &empty_custom,
                ..Default::default()
            },
            &config,
        );

        // Parse it back
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(&formatted, &path, &config);

        assert_eq!(tasks.len(), 1);
        let task = &tasks[0];
        assert_eq!(task.description, "Round trip task");
        assert_eq!(task.id, Some("myid".to_string()));
        assert_eq!(task.depends_on, vec!["dep1"]);
        assert_eq!(task.priority, Some(Priority::High));
        assert_eq!(task.recurrence, Some("every week".to_string()));
        assert_eq!(task.created, Some("2026-01-01".to_string()));
        assert_eq!(task.start, Some("2026-02-01".to_string()));
        assert_eq!(task.scheduled, Some("2026-02-15".to_string()));
        assert_eq!(task.due, Some("2026-03-01".to_string()));
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
            },
        ];

        let hierarchy = build_task_hierarchy(tasks);

        assert_eq!(hierarchy.len(), 1);
        assert_eq!(hierarchy[0].description, "Parent");
        assert_eq!(hierarchy[0].children.len(), 1);
        assert_eq!(hierarchy[0].children[0].description, "Child");
    }
}
