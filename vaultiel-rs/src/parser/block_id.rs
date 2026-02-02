//! Block ID parsing (^block-id).

use crate::parser::code_block::{find_code_block_ranges, is_line_in_fenced_code_block};
use crate::types::{BlockId, BlockType};
use regex::Regex;
use std::sync::LazyLock;

// Block ID pattern: ^id at the end of a line
// ID can contain letters, numbers, underscores, and hyphens
static BLOCK_ID: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\s+\^([a-zA-Z0-9_-]+)\s*$").unwrap()
});

/// Parse all block IDs from content.
pub fn parse_block_ids(content: &str) -> Vec<BlockId> {
    let code_ranges = find_code_block_ranges(content);
    let mut block_ids = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1; // 1-indexed

        // Skip lines inside fenced code blocks
        if is_line_in_fenced_code_block(line_num, &code_ranges) {
            continue;
        }

        if let Some(cap) = BLOCK_ID.captures(line) {
            let id = cap.get(1).unwrap().as_str().to_string();
            let block_type = determine_block_type(line, content, line_idx);

            block_ids.push(BlockId {
                id,
                line: line_num,
                block_type,
            });
        }
    }

    block_ids
}

/// Determine the type of block based on the line content and context.
fn determine_block_type(line: &str, _content: &str, _line_idx: usize) -> BlockType {
    let trimmed = line.trim_start();

    if trimmed.starts_with('#') {
        return BlockType::Heading;
    }

    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        return BlockType::ListItem;
    }

    // Check for numbered list
    if trimmed
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .count()
        > 0
    {
        let rest = trimmed.trim_start_matches(|c: char| c.is_ascii_digit());
        if rest.starts_with(". ") {
            return BlockType::ListItem;
        }
    }

    if trimmed.starts_with('>') {
        return BlockType::Blockquote;
    }

    if trimmed.starts_with('|') && trimmed.ends_with('|') {
        return BlockType::Table;
    }

    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        return BlockType::CodeBlock;
    }

    BlockType::Paragraph
}

/// Find a block by its ID.
pub fn find_block_by_id<'a>(content: &'a str, block_id: &str) -> Option<(usize, &'a str)> {
    let block_ids = parse_block_ids(content);

    for block in block_ids {
        if block.id == block_id {
            // Return the line content
            if let Some(line) = content.lines().nth(block.line - 1) {
                return Some((block.line, line));
            }
        }
    }

    None
}

/// Get the range of lines for a block (handles multi-line blocks like lists).
pub fn get_block_range(content: &str, block_id: &str) -> Option<(usize, usize)> {
    // For now, just return the single line
    // TODO: Handle multi-line blocks (lists, blockquotes, etc.)
    find_block_by_id(content, block_id).map(|(line, _)| (line, line))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_block_id() {
        let content = "Some paragraph text ^abc123";
        let blocks = parse_block_ids(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].id, "abc123");
        assert_eq!(blocks[0].line, 1);
        assert_eq!(blocks[0].block_type, BlockType::Paragraph);
    }

    #[test]
    fn test_block_id_on_list_item() {
        let content = "- List item ^list-id";
        let blocks = parse_block_ids(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].id, "list-id");
        assert_eq!(blocks[0].block_type, BlockType::ListItem);
    }

    #[test]
    fn test_block_id_on_heading() {
        let content = "# Heading ^head-id";
        let blocks = parse_block_ids(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].id, "head-id");
        assert_eq!(blocks[0].block_type, BlockType::Heading);
    }

    #[test]
    fn test_block_id_on_blockquote() {
        let content = "> Quote text ^quote-id";
        let blocks = parse_block_ids(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, BlockType::Blockquote);
    }

    #[test]
    fn test_multiple_block_ids() {
        let content = "Para 1 ^id1\n\nPara 2 ^id2\n\n- Item ^id3";
        let blocks = parse_block_ids(content);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].line, 1);
        assert_eq!(blocks[1].line, 3);
        assert_eq!(blocks[2].line, 5);
    }

    #[test]
    fn test_block_id_in_code_block_skipped() {
        let content = "Real paragraph ^real-id\n\n```\nCode ^fake-id\n```";
        let blocks = parse_block_ids(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].id, "real-id");
    }

    #[test]
    fn test_block_id_must_be_at_end() {
        let content = "Some ^id text continues";
        let blocks = parse_block_ids(content);
        // ^id is not at the end of the line, so it shouldn't match
        assert_eq!(blocks.len(), 0);
    }

    #[test]
    fn test_block_id_with_whitespace_after() {
        let content = "Some text ^id   ";
        let blocks = parse_block_ids(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].id, "id");
    }

    #[test]
    fn test_find_block_by_id() {
        let content = "Line 1\nLine 2 with ^target\nLine 3";
        let result = find_block_by_id(content, "target");
        assert!(result.is_some());
        let (line, text) = result.unwrap();
        assert_eq!(line, 2);
        assert!(text.contains("Line 2"));
    }

    #[test]
    fn test_numbered_list_block_type() {
        let content = "1. First item ^item1";
        let blocks = parse_block_ids(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, BlockType::ListItem);
    }
}
