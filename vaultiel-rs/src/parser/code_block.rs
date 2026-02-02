//! Code block detection for skipping parsing inside code.

use regex::Regex;
use std::sync::LazyLock;

/// A range of characters that are inside a code block or inline code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBlockRange {
    /// Start byte offset (inclusive).
    pub start: usize,
    /// End byte offset (exclusive).
    pub end: usize,
    /// Line number where the code block starts (1-indexed).
    pub start_line: usize,
    /// Line number where the code block ends (1-indexed).
    pub end_line: usize,
    /// Whether this is a fenced code block (vs inline code).
    pub is_fenced: bool,
}

// Matches the opening of a fenced code block: ``` or ~~~ at start of line
static FENCE_OPEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^(`{3,}|~{3,})").unwrap()
});

// Matches inline code - simple pattern for single backticks
static INLINE_CODE_SINGLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"`[^`\n]+`").unwrap()
});

// Matches inline code with double backticks (can contain single backticks)
static INLINE_CODE_DOUBLE: LazyLock<Regex> = LazyLock::new(|| {
    // Matches `` followed by (non-backtick OR single-backtick-followed-by-non-backtick)* followed by ``
    Regex::new(r"``(?:[^`]|`[^`])*``").unwrap()
});

/// Find all code block and inline code ranges in content.
///
/// These ranges should be excluded when parsing for links, tags, etc.
pub fn find_code_block_ranges(content: &str) -> Vec<CodeBlockRange> {
    let mut ranges = Vec::new();

    // Find fenced code blocks by manually tracking opening and closing fences
    let mut pos = 0;
    while pos < content.len() {
        if let Some(open_match) = FENCE_OPEN.find(&content[pos..]) {
            let fence_char = content[pos + open_match.start()..].chars().next().unwrap();
            let fence_len = open_match.len();
            let abs_start = pos + open_match.start();

            // Find the end of the opening line
            let line_end = content[abs_start..]
                .find('\n')
                .map(|i| abs_start + i + 1)
                .unwrap_or(content.len());

            // Look for matching closing fence
            let mut search_pos = line_end;
            let mut found_close = false;

            while search_pos < content.len() {
                // Look for a line starting with the same fence
                if let Some(newline_pos) = content[search_pos..].find('\n') {
                    let next_line_start = search_pos + newline_pos + 1;
                    if next_line_start < content.len() {
                        let rest = &content[next_line_start..];
                        // Check if line starts with matching fence
                        if rest.starts_with(&fence_char.to_string().repeat(fence_len)) {
                            // Verify it's a proper closing fence (only fence chars and whitespace)
                            let close_line_end = rest.find('\n').unwrap_or(rest.len());
                            let close_line = &rest[..close_line_end];
                            let trimmed = close_line.trim();
                            if trimmed.chars().all(|c| c == fence_char) && trimmed.len() >= fence_len {
                                let abs_end = next_line_start + close_line_end;

                                let start_line = content[..abs_start].matches('\n').count() + 1;
                                let end_line = content[..abs_end].matches('\n').count() + 1;

                                ranges.push(CodeBlockRange {
                                    start: abs_start,
                                    end: abs_end,
                                    start_line,
                                    end_line,
                                    is_fenced: true,
                                });

                                pos = abs_end;
                                found_close = true;
                                break;
                            }
                        }
                    }
                    search_pos = search_pos + newline_pos + 1;
                } else {
                    break;
                }
            }

            if !found_close {
                // No closing fence found, move past this potential opener
                pos = line_end;
            }
        } else {
            break;
        }
    }

    // Find inline code (but not inside fenced blocks)
    for m in INLINE_CODE_DOUBLE.find_iter(content) {
        let start = m.start();
        let end = m.end();

        // Skip if inside a fenced code block
        if ranges.iter().any(|r| r.is_fenced && start >= r.start && end <= r.end) {
            continue;
        }

        let start_line = content[..start].matches('\n').count() + 1;
        let end_line = content[..end].matches('\n').count() + 1;

        ranges.push(CodeBlockRange {
            start,
            end,
            start_line,
            end_line,
            is_fenced: false,
        });
    }

    for m in INLINE_CODE_SINGLE.find_iter(content) {
        let start = m.start();
        let end = m.end();

        // Skip if overlapping with any existing range (fenced block or double backtick)
        if ranges.iter().any(|r| {
            // Check if this match overlaps with existing range
            (start >= r.start && start < r.end) || (end > r.start && end <= r.end)
        }) {
            continue;
        }

        let start_line = content[..start].matches('\n').count() + 1;
        let end_line = content[..end].matches('\n').count() + 1;

        ranges.push(CodeBlockRange {
            start,
            end,
            start_line,
            end_line,
            is_fenced: false,
        });
    }

    // Sort by start position
    ranges.sort_by_key(|r| r.start);

    ranges
}

/// Check if a byte offset is inside any code block.
pub fn is_in_code_block(offset: usize, ranges: &[CodeBlockRange]) -> bool {
    ranges.iter().any(|r| offset >= r.start && offset < r.end)
}

/// Check if a line number is inside any fenced code block.
pub fn is_line_in_fenced_code_block(line: usize, ranges: &[CodeBlockRange]) -> bool {
    ranges
        .iter()
        .any(|r| r.is_fenced && line >= r.start_line && line <= r.end_line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fenced_code_block() {
        let content = r#"Some text

```rust
let x = [[not a link]];
```

More text"#;

        let ranges = find_code_block_ranges(content);
        assert_eq!(ranges.len(), 1);
        assert!(ranges[0].is_fenced);
        assert_eq!(ranges[0].start_line, 3);
    }

    #[test]
    fn test_inline_code() {
        let content = "Some `inline [[code]]` here";
        let ranges = find_code_block_ranges(content);
        assert_eq!(ranges.len(), 1);
        assert!(!ranges[0].is_fenced);
    }

    #[test]
    fn test_double_backtick_inline() {
        let content = "Some ``inline `code` with backticks`` here";
        let ranges = find_code_block_ranges(content);
        assert_eq!(ranges.len(), 1);
    }

    #[test]
    fn test_nested_fenced_blocks() {
        let content = r#"```
outer
```

text

~~~
inner
~~~"#;

        let ranges = find_code_block_ranges(content);
        assert_eq!(ranges.len(), 2);
    }

    #[test]
    fn test_is_in_code_block() {
        let content = "before `code` after";
        let ranges = find_code_block_ranges(content);

        assert!(!is_in_code_block(0, &ranges)); // 'b' in 'before'
        assert!(is_in_code_block(8, &ranges)); // 'c' in 'code'
        assert!(!is_in_code_block(14, &ranges)); // 'a' in 'after'
    }
}
