//! Task parsing with generic emoji metadata fields.

use crate::config::{EmojiValueType, TaskConfig};
use crate::parser::code_block::find_code_block_ranges;
use crate::parser::wikilink::parse_links;
use crate::types::{Task, TaskLink, TaskLocation};
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

/// Regex for numbers (integer or decimal).
static NUMBER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-?\d+(?:\.\d+)?").unwrap()
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
        metadata,
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

/// Remove an emoji and its associated value text from the remaining string.
fn remove_range(remaining: &mut String, start: usize, end: usize) {
    *remaining = format!("{}{}", &remaining[..start], &remaining[end..]);
}

/// Extract a date value after an emoji position.
fn extract_date_after(remaining: &mut String, emoji: &str) -> Option<String> {
    if let Some(pos) = remaining.find(emoji) {
        let after = &remaining[pos + emoji.len()..];
        if let Some(date_match) = DATE_REGEX.find(after.trim_start()) {
            let value = date_match.as_str().to_string();
            let trim_start = after.len() - after.trim_start().len();
            let end = pos + emoji.len() + trim_start + date_match.end();
            remove_range(remaining, pos, end);
            return Some(value);
        }
    }
    None
}

/// Extract a single word/token after an emoji.
fn extract_word_after(remaining: &mut String, emoji: &str) -> Option<String> {
    if let Some(pos) = remaining.find(emoji) {
        let after = &remaining[pos + emoji.len()..];
        let trimmed = after.trim_start();
        let trim_start = after.len() - trimmed.len();
        // Find end of word (whitespace or emoji)
        let word_end = trimmed
            .find(|c: char| c.is_whitespace())
            .unwrap_or(trimmed.len());
        let value = trimmed[..word_end].trim().to_string();
        if !value.is_empty() {
            let end = pos + emoji.len() + trim_start + word_end;
            remove_range(remaining, pos, end);
            return Some(value);
        }
    }
    None
}

/// Extract multi-word text until next registered emoji or end of string.
fn extract_text_until_next_emoji(remaining: &mut String, emoji: &str, all_emojis: &[&str]) -> Option<String> {
    if let Some(pos) = remaining.find(emoji) {
        let after_emoji_start = pos + emoji.len();
        let after = &remaining[after_emoji_start..];
        let trimmed = after.trim_start();
        let trim_start = after.len() - trimmed.len();

        // Find the next registered emoji
        let value_end = find_next_emoji_pos_in_slice(trimmed, all_emojis).unwrap_or(trimmed.len());
        let value = trimmed[..value_end].trim().to_string();
        if !value.is_empty() {
            let end = after_emoji_start + trim_start + value_end;
            remove_range(remaining, pos, end);
            return Some(value);
        }
    }
    None
}

/// Find the position of the next registered emoji in a slice.
fn find_next_emoji_pos_in_slice(text: &str, emojis: &[&str]) -> Option<usize> {
    let mut earliest: Option<usize> = None;
    for emoji in emojis {
        if let Some(pos) = text.find(emoji) {
            if earliest.is_none() || pos < earliest.unwrap() {
                earliest = Some(pos);
            }
        }
    }
    earliest
}

/// Extract a number after an emoji.
fn extract_number_after(remaining: &mut String, emoji: &str) -> Option<String> {
    if let Some(pos) = remaining.find(emoji) {
        let after = &remaining[pos + emoji.len()..];
        if let Some(num_match) = NUMBER_REGEX.find(after.trim_start()) {
            let value = num_match.as_str().to_string();
            let trim_start = after.len() - after.trim_start().len();
            let end = pos + emoji.len() + trim_start + num_match.end();
            remove_range(remaining, pos, end);
            return Some(value);
        }
    }
    None
}

/// Remove just the emoji (for flag/enum types with no inline value).
fn remove_emoji(remaining: &mut String, emoji: &str) -> bool {
    if let Some(pos) = remaining.find(emoji) {
        let end = pos + emoji.len();
        remove_range(remaining, pos, end);
        true
    } else {
        false
    }
}

/// Extract metadata generically from task text using config field definitions.
fn extract_metadata(text: &str, config: &TaskConfig) -> (String, HashMap<String, String>) {
    let mut remaining = text.to_string();
    let mut metadata = HashMap::new();
    let all_emojis = config.all_emojis();

    // Process fields in order
    for field in config.sorted_fields() {
        match &field.value_type {
            EmojiValueType::Date => {
                if let Some(value) = extract_date_after(&mut remaining, &field.emoji) {
                    metadata.insert(field.field_name.clone(), value);
                }
            }
            EmojiValueType::String => {
                if let Some(value) = extract_word_after(&mut remaining, &field.emoji) {
                    metadata.insert(field.field_name.clone(), value);
                }
            }
            EmojiValueType::Text => {
                if let Some(value) = extract_text_until_next_emoji(&mut remaining, &field.emoji, &all_emojis) {
                    metadata.insert(field.field_name.clone(), value);
                }
            }
            EmojiValueType::Number => {
                if let Some(value) = extract_number_after(&mut remaining, &field.emoji) {
                    metadata.insert(field.field_name.clone(), value);
                }
            }
            EmojiValueType::Flag { value } => {
                if remove_emoji(&mut remaining, &field.emoji) {
                    metadata.insert(field.field_name.clone(), value.clone());
                }
            }
            EmojiValueType::Enum { value } => {
                if remove_emoji(&mut remaining, &field.emoji) {
                    metadata.insert(field.field_name.clone(), value.clone());
                }
            }
        }
    }

    // Clean up extra whitespace
    let description = remaining.split_whitespace().collect::<Vec<_>>().join(" ");

    (description, metadata)
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
    pub metadata: &'a HashMap<String, String>,
}

impl<'a> Default for FormatTaskParams<'a> {
    fn default() -> Self {
        static EMPTY_MAP: std::sync::LazyLock<HashMap<String, String>> =
            std::sync::LazyLock::new(HashMap::new);
        Self {
            description: "",
            symbol: "[ ]",
            metadata: &EMPTY_MAP,
        }
    }
}

/// Format a task string for Obsidian.
///
/// Iterates sorted fields and emits present metadata in order.
pub fn format_task(params: &FormatTaskParams, config: &TaskConfig) -> String {
    let mut parts = vec![format!("- {} {}", params.symbol, params.description)];

    for field in config.sorted_fields() {
        if let Some(value) = params.metadata.get(&field.field_name) {
            match &field.value_type {
                EmojiValueType::Flag { .. } | EmojiValueType::Enum { .. } => {
                    // Flag/enum: just emit the emoji, no value after it
                    parts.push(field.emoji.clone());
                }
                _ => {
                    // Date/String/Text/Number: emit emoji + value
                    parts.push(format!("{} {}", field.emoji, value));
                }
            }
        }
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
    use crate::config::EmojiFieldDef;

    /// Build a config matching the old Obsidian Tasks defaults for testing.
    fn obsidian_tasks_config() -> TaskConfig {
        TaskConfig {
            fields: vec![
                EmojiFieldDef { emoji: "🆔".to_string(), field_name: "id".to_string(), value_type: EmojiValueType::String, order: 10 },
                EmojiFieldDef { emoji: "⛔".to_string(), field_name: "depends_on".to_string(), value_type: EmojiValueType::Text, order: 20 },
                EmojiFieldDef { emoji: "🔺".to_string(), field_name: "priority".to_string(), value_type: EmojiValueType::Flag { value: "highest".to_string() }, order: 30 },
                EmojiFieldDef { emoji: "⏫".to_string(), field_name: "priority".to_string(), value_type: EmojiValueType::Enum { value: "high".to_string() }, order: 31 },
                EmojiFieldDef { emoji: "🔼".to_string(), field_name: "priority".to_string(), value_type: EmojiValueType::Enum { value: "medium".to_string() }, order: 32 },
                EmojiFieldDef { emoji: "🔽".to_string(), field_name: "priority".to_string(), value_type: EmojiValueType::Enum { value: "low".to_string() }, order: 33 },
                EmojiFieldDef { emoji: "⏬".to_string(), field_name: "priority".to_string(), value_type: EmojiValueType::Enum { value: "lowest".to_string() }, order: 34 },
                EmojiFieldDef { emoji: "🔁".to_string(), field_name: "recurrence".to_string(), value_type: EmojiValueType::Text, order: 40 },
                EmojiFieldDef { emoji: "🏁".to_string(), field_name: "on_completion".to_string(), value_type: EmojiValueType::Text, order: 50 },
                EmojiFieldDef { emoji: "➕".to_string(), field_name: "created".to_string(), value_type: EmojiValueType::Date, order: 60 },
                EmojiFieldDef { emoji: "🛫".to_string(), field_name: "start".to_string(), value_type: EmojiValueType::Date, order: 70 },
                EmojiFieldDef { emoji: "⏳".to_string(), field_name: "scheduled".to_string(), value_type: EmojiValueType::Date, order: 80 },
                EmojiFieldDef { emoji: "📅".to_string(), field_name: "due".to_string(), value_type: EmojiValueType::Date, order: 90 },
                EmojiFieldDef { emoji: "❌".to_string(), field_name: "cancelled".to_string(), value_type: EmojiValueType::Date, order: 100 },
                EmojiFieldDef { emoji: "✅".to_string(), field_name: "done".to_string(), value_type: EmojiValueType::Date, order: 110 },
            ],
        }
    }

    #[test]
    fn test_parse_simple_task() {
        let content = "- [ ] A simple task";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &obsidian_tasks_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].symbol, "[ ]");
        assert_eq!(tasks[0].description, "A simple task");
        assert_eq!(tasks[0].indent, 0);
        assert!(tasks[0].metadata.is_empty());
    }

    #[test]
    fn test_parse_completed_task() {
        let content = "- [x] Completed task";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &obsidian_tasks_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].symbol, "[x]");
    }

    #[test]
    fn test_parse_task_with_dates() {
        let content = "- [ ] Task with dates ⏳ 2026-02-05 📅 2026-02-10";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &obsidian_tasks_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].metadata.get("scheduled"), Some(&"2026-02-05".to_string()));
        assert_eq!(tasks[0].metadata.get("due"), Some(&"2026-02-10".to_string()));
    }

    #[test]
    fn test_parse_task_with_priority() {
        let content = "- [ ] High priority task ⏫";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &obsidian_tasks_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].metadata.get("priority"), Some(&"high".to_string()));
    }

    #[test]
    fn test_parse_task_with_links() {
        let content = "- [ ] Task linking to [[Note A]] and [[Note B|alias]]";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &obsidian_tasks_config());

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
        let tasks = parse_tasks(content, &path, &obsidian_tasks_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].tags, vec!["#tag1", "#project/subtag"]);
    }

    #[test]
    fn test_parse_task_with_block_id() {
        let content = "- [ ] Task with block ID ^abc123";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &obsidian_tasks_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].block_id, Some("abc123".to_string()));
    }

    #[test]
    fn test_parse_nested_tasks() {
        let content = "- [ ] Parent task\n\t- [ ] Child task\n\t\t- [ ] Grandchild task";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &obsidian_tasks_config());

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
        let config = obsidian_tasks_config();
        let mut metadata = HashMap::new();
        metadata.insert("scheduled".to_string(), "2026-02-05".to_string());
        metadata.insert("due".to_string(), "2026-02-10".to_string());
        metadata.insert("priority".to_string(), "high".to_string());

        let result = format_task(
            &FormatTaskParams {
                description: "My task",
                symbol: "[ ]",
                metadata: &metadata,
            },
            &config,
        );

        assert!(result.contains("- [ ] My task"));
        assert!(result.contains("⏳ 2026-02-05"));
        assert!(result.contains("📅 2026-02-10"));
        // Priority "high" matches ⏫ enum field
        assert!(result.contains("⏫"));
    }

    #[test]
    fn test_parse_task_with_recurrence() {
        let content = "- [ ] Recurring task 🔁 every week 📅 2026-03-01";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &obsidian_tasks_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].metadata.get("recurrence"), Some(&"every week".to_string()));
        assert_eq!(tasks[0].metadata.get("due"), Some(&"2026-03-01".to_string()));
    }

    #[test]
    fn test_parse_task_with_id_and_depends() {
        let content = "- [ ] Task 🆔 abc123 ⛔ def456,ghi789 📅 2026-03-01";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &obsidian_tasks_config());

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].metadata.get("id"), Some(&"abc123".to_string()));
        assert_eq!(tasks[0].metadata.get("depends_on"), Some(&"def456,ghi789".to_string()));
        assert_eq!(tasks[0].metadata.get("due"), Some(&"2026-03-01".to_string()));
    }

    #[test]
    fn test_parse_task_all_fields() {
        let content = "- [ ] Full task 🆔 myid ⛔ dep1 ⏫ 🔁 every day 🏁 delete ➕ 2026-01-01 🛫 2026-02-01 ⏳ 2026-02-15 📅 2026-03-01 ❌ 2026-02-20 ✅ 2026-02-25";
        let path = PathBuf::from("test.md");
        let tasks = parse_tasks(content, &path, &obsidian_tasks_config());

        assert_eq!(tasks.len(), 1);
        let task = &tasks[0];
        assert_eq!(task.description, "Full task");
        assert_eq!(task.metadata.get("id"), Some(&"myid".to_string()));
        assert_eq!(task.metadata.get("depends_on"), Some(&"dep1".to_string()));
        assert_eq!(task.metadata.get("priority"), Some(&"high".to_string()));
        assert_eq!(task.metadata.get("recurrence"), Some(&"every day".to_string()));
        assert_eq!(task.metadata.get("on_completion"), Some(&"delete".to_string()));
        assert_eq!(task.metadata.get("created"), Some(&"2026-01-01".to_string()));
        assert_eq!(task.metadata.get("start"), Some(&"2026-02-01".to_string()));
        assert_eq!(task.metadata.get("scheduled"), Some(&"2026-02-15".to_string()));
        assert_eq!(task.metadata.get("due"), Some(&"2026-03-01".to_string()));
        assert_eq!(task.metadata.get("cancelled"), Some(&"2026-02-20".to_string()));
        assert_eq!(task.metadata.get("done"), Some(&"2026-02-25".to_string()));
    }

    #[test]
    fn test_format_task_canonical_order() {
        let config = obsidian_tasks_config();
        let mut metadata = HashMap::new();
        metadata.insert("id".to_string(), "myid".to_string());
        metadata.insert("depends_on".to_string(), "dep1".to_string());
        metadata.insert("priority".to_string(), "high".to_string());
        metadata.insert("recurrence".to_string(), "every week".to_string());
        metadata.insert("on_completion".to_string(), "delete".to_string());
        metadata.insert("created".to_string(), "2026-01-01".to_string());
        metadata.insert("start".to_string(), "2026-02-01".to_string());
        metadata.insert("scheduled".to_string(), "2026-02-15".to_string());
        metadata.insert("due".to_string(), "2026-03-01".to_string());
        metadata.insert("cancelled".to_string(), "2026-02-20".to_string());
        metadata.insert("done".to_string(), "2026-02-25".to_string());

        let result = format_task(
            &FormatTaskParams {
                description: "Task",
                symbol: "[ ]",
                metadata: &metadata,
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
    fn test_empty_config_parses_simple_task() {
        let content = "- [ ] Simple task with 📅 2026-01-01";
        let path = PathBuf::from("test.md");
        let config = TaskConfig::empty();
        let tasks = parse_tasks(content, &path, &config);

        assert_eq!(tasks.len(), 1);
        // With no fields registered, emoji stays in description
        assert!(tasks[0].description.contains("📅"));
        assert!(tasks[0].metadata.is_empty());
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
                metadata: HashMap::new(),
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
                metadata: HashMap::new(),
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
