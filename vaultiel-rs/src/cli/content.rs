//! Content commands implementation.

use crate::cli::args::{
    AppendContentArgs, GetContentArgs, PrependContentArgs, ReplaceContentArgs, SetContentArgs,
};
use crate::cli::output::{DryRunResponse, Output};
use crate::error::{Result, VaultError};
use crate::parser::{find_code_block_ranges, parse_headings, split_frontmatter};
use crate::vault::Vault;
use regex::Regex;
use serde::Serialize;
use std::io::{self, Read};

#[derive(Debug, Serialize)]
pub struct ContentResponse {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ModifyResponse {
    pub path: String,
    pub message: String,
}

/// Read content from args (--content, --file, or stdin).
fn read_input_content(content_arg: &Option<String>, file_arg: &Option<std::path::PathBuf>) -> Result<String> {
    if let Some(content) = content_arg {
        // Unescape newlines
        Ok(content.replace("\\n", "\n"))
    } else if let Some(path) = file_arg {
        Ok(std::fs::read_to_string(path)?)
    } else {
        // Try to read from stdin if it's not a terminal
        if atty::isnt(atty::Stream::Stdin) {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        } else {
            Err(VaultError::NoContentProvided)
        }
    }
}

// === get-content ===

pub fn get_content(vault: &Vault, args: &GetContentArgs, output: &Output) -> Result<()> {
    let path = vault.normalize_note_path(&args.path);
    let note = vault.load_note(&path)?;

    let content = if args.include_frontmatter {
        if args.include_vaultiel_field {
            // Include full content with vaultiel field
            note.full_content().to_string()
        } else {
            // Exclude vaultiel field from frontmatter
            filter_vaultiel_from_content(&note.content)
        }
    } else {
        note.body().to_string()
    };

    // For get-content, output raw text, not JSON
    output.print_raw(&content);
    Ok(())
}

/// Filter out the vaultiel field from frontmatter.
fn filter_vaultiel_from_content(content: &str) -> String {
    let split = split_frontmatter(content);

    match split.yaml {
        Some(fm_str) => {
            // Parse, remove vaultiel, and re-serialize
            if let Ok(mut fm) = serde_yaml::from_str::<serde_yaml::Value>(fm_str) {
                if let serde_yaml::Value::Mapping(ref mut map) = fm {
                    map.remove(&serde_yaml::Value::String("vaultiel".to_string()));
                }

                // Re-serialize
                if let Ok(new_fm) = serde_yaml::to_string(&fm) {
                    let new_fm = new_fm.trim();
                    if new_fm == "{}" || new_fm.is_empty() {
                        // Empty frontmatter after removing vaultiel
                        format!("---\n---\n{}", split.content)
                    } else {
                        format!("---\n{}\n---\n{}", new_fm, split.content)
                    }
                } else {
                    content.to_string()
                }
            } else {
                content.to_string()
            }
        }
        None => content.to_string(),
    }
}

// === set-content ===

pub fn set_content(vault: &Vault, args: &SetContentArgs, output: &Output) -> Result<()> {
    let path = vault.normalize_note_path(&args.path);
    let note = vault.load_note(&path)?;

    let new_content = read_input_content(&args.content, &args.file)?;

    let final_content = if args.below_frontmatter {
        // Preserve frontmatter, replace body
        note.with_body(&new_content).content
    } else if args.frontmatter_only {
        // Replace frontmatter, preserve body
        let split = split_frontmatter(&note.content);
        if new_content.starts_with("---") {
            format!("{}{}", new_content.trim_end(), split.content)
        } else {
            format!("---\n{}\n---\n{}", new_content.trim(), split.content)
        }
    } else {
        // Replace entire content
        new_content
    };

    if args.dry_run {
        let response = DryRunResponse {
            action: "set-content".to_string(),
            path: path.to_string_lossy().to_string(),
            content: Some(final_content),
            changes: None,
        };
        output.print(&response)?;
        return Ok(());
    }

    let updated_note = note.with_content(final_content);
    vault.save_note(&updated_note)?;

    let response = ModifyResponse {
        path: path.to_string_lossy().to_string(),
        message: "Content updated successfully".to_string(),
    };
    output.print(&response)?;

    Ok(())
}

// === append-content ===

pub fn append_content(vault: &Vault, args: &AppendContentArgs, output: &Output) -> Result<()> {
    let path = vault.normalize_note_path(&args.path);
    let note = vault.load_note(&path)?;

    let content_to_append = read_input_content(&args.content, &args.file)?;
    let updated_note = note.append(&content_to_append);

    if args.dry_run {
        let response = DryRunResponse {
            action: "append-content".to_string(),
            path: path.to_string_lossy().to_string(),
            content: Some(updated_note.content),
            changes: None,
        };
        output.print(&response)?;
        return Ok(());
    }

    vault.save_note(&updated_note)?;

    let response = ModifyResponse {
        path: path.to_string_lossy().to_string(),
        message: "Content appended successfully".to_string(),
    };
    output.print(&response)?;

    Ok(())
}

// === prepend-content ===

pub fn prepend_content(vault: &Vault, args: &PrependContentArgs, output: &Output) -> Result<()> {
    let path = vault.normalize_note_path(&args.path);
    let note = vault.load_note(&path)?;

    let content_to_prepend = read_input_content(&args.content, &args.file)?;
    let updated_note = note.prepend(&content_to_prepend);

    if args.dry_run {
        let response = DryRunResponse {
            action: "prepend-content".to_string(),
            path: path.to_string_lossy().to_string(),
            content: Some(updated_note.content),
            changes: None,
        };
        output.print(&response)?;
        return Ok(());
    }

    vault.save_note(&updated_note)?;

    let response = ModifyResponse {
        path: path.to_string_lossy().to_string(),
        message: "Content prepended successfully".to_string(),
    };
    output.print(&response)?;

    Ok(())
}

// === replace-content ===

pub fn replace_content(vault: &Vault, args: &ReplaceContentArgs, output: &Output) -> Result<()> {
    let path = vault.normalize_note_path(&args.path);
    let note = vault.load_note(&path)?;

    let replacement = read_input_content(&args.content, &args.file)?;

    let new_content = if let Some(ref section) = args.section {
        replace_section(&note.content, section, &replacement)?
    } else if let Some(ref pattern) = args.pattern {
        replace_pattern(&note.content, pattern, &replacement, false)?
    } else if let Some(ref pattern_all) = args.pattern_all {
        replace_pattern(&note.content, pattern_all, &replacement, true)?
    } else if let Some(ref lines_str) = args.lines {
        replace_lines(&note.content, lines_str, &replacement)?
    } else if let Some(ref block_id) = args.block {
        replace_block(&note.content, block_id, &replacement)?
    } else {
        return Err(VaultError::Other(
            "Must specify --section, --pattern, --pattern-all, --lines, or --block".to_string(),
        ));
    };

    if args.dry_run {
        let response = DryRunResponse {
            action: "replace-content".to_string(),
            path: path.to_string_lossy().to_string(),
            content: Some(new_content),
            changes: None,
        };
        output.print(&response)?;
        return Ok(());
    }

    let updated_note = note.with_content(new_content);
    vault.save_note(&updated_note)?;

    let response = ModifyResponse {
        path: path.to_string_lossy().to_string(),
        message: "Content replaced successfully".to_string(),
    };
    output.print(&response)?;

    Ok(())
}

/// Replace a section under a heading.
fn replace_section(content: &str, heading: &str, replacement: &str) -> Result<String> {
    let headings = parse_headings(content);

    // Find the target heading
    let heading_text = heading.trim_start_matches('#').trim();
    let target = headings
        .iter()
        .find(|h| h.text.to_lowercase() == heading_text.to_lowercase())
        .ok_or_else(|| VaultError::SectionNotFound(heading.to_string()))?;

    let lines: Vec<&str> = content.lines().collect();
    let start_line = target.line - 1; // 0-indexed

    // Find the end of the section (next heading at same or higher level)
    let end_line = headings
        .iter()
        .filter(|h| h.line > target.line && h.level <= target.level)
        .map(|h| h.line - 1)
        .next()
        .unwrap_or(lines.len());

    // Build new content
    let mut result = String::new();

    // Add lines before section
    for line in &lines[..start_line] {
        result.push_str(line);
        result.push('\n');
    }

    // Add replacement (including heading if not in replacement)
    let replacement = replacement.trim();
    if !replacement.starts_with('#') {
        // Preserve the original heading
        result.push_str(lines[start_line]);
        result.push('\n');
    }
    result.push_str(replacement);
    if !replacement.ends_with('\n') {
        result.push('\n');
    }

    // Add lines after section
    for line in &lines[end_line..] {
        result.push_str(line);
        result.push('\n');
    }

    // Remove trailing newline if original didn't have one
    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    Ok(result)
}

/// Replace pattern matches.
fn replace_pattern(content: &str, pattern: &str, replacement: &str, all: bool) -> Result<String> {
    let re = Regex::new(pattern)?;
    let code_ranges = find_code_block_ranges(content);

    if all {
        // Replace all matches not in code blocks
        let mut result = content.to_string();
        let mut offset: i64 = 0;

        for m in re.find_iter(content) {
            let start = (m.start() as i64 + offset) as usize;
            let end = (m.end() as i64 + offset) as usize;

            // Check if in code block
            if code_ranges.iter().any(|r| m.start() >= r.start && m.end() <= r.end) {
                continue;
            }

            result = format!("{}{}{}", &result[..start], replacement, &result[end..]);
            offset += replacement.len() as i64 - (m.end() - m.start()) as i64;
        }

        Ok(result)
    } else {
        // Replace first match not in code block
        for m in re.find_iter(content) {
            if code_ranges.iter().any(|r| m.start() >= r.start && m.end() <= r.end) {
                continue;
            }

            return Ok(format!(
                "{}{}{}",
                &content[..m.start()],
                replacement,
                &content[m.end()..]
            ));
        }

        // No match found outside code blocks
        Err(VaultError::Other(format!(
            "Pattern '{}' not found outside code blocks",
            pattern
        )))
    }
}

/// Replace a line range.
fn replace_lines(content: &str, range_str: &str, replacement: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    // Parse range: "10-15", "10-", "-15", "10"
    let (start, end) = parse_line_range(range_str, total_lines)?;

    let mut result = String::new();

    // Add lines before range
    for line in &lines[..start] {
        result.push_str(line);
        result.push('\n');
    }

    // Add replacement
    result.push_str(replacement.trim_end());
    result.push('\n');

    // Add lines after range
    for line in &lines[end..] {
        result.push_str(line);
        result.push('\n');
    }

    // Remove trailing newline if original didn't have one
    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    Ok(result)
}

fn parse_line_range(range_str: &str, total_lines: usize) -> Result<(usize, usize)> {
    let range_str = range_str.trim();

    if let Some((start_str, end_str)) = range_str.split_once('-') {
        let start = if start_str.is_empty() {
            0
        } else {
            start_str
                .parse::<usize>()
                .map_err(|_| VaultError::InvalidLineRange(range_str.to_string()))?
                .saturating_sub(1) // Convert to 0-indexed
        };

        let end = if end_str.is_empty() {
            total_lines
        } else {
            end_str
                .parse::<usize>()
                .map_err(|_| VaultError::InvalidLineRange(range_str.to_string()))?
                .min(total_lines)
        };

        if start > end {
            return Err(VaultError::InvalidLineRange(range_str.to_string()));
        }

        Ok((start, end))
    } else {
        // Single line
        let line = range_str
            .parse::<usize>()
            .map_err(|_| VaultError::InvalidLineRange(range_str.to_string()))?
            .saturating_sub(1);

        if line >= total_lines {
            return Err(VaultError::InvalidLineRange(range_str.to_string()));
        }

        Ok((line, line + 1))
    }
}

/// Replace a block by its ID.
fn replace_block(content: &str, block_id: &str, replacement: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();

    // Find the line with the block ID
    let block_pattern = format!(r"\^{}\s*$", regex::escape(block_id));
    let re = Regex::new(&block_pattern)?;

    for (idx, line) in lines.iter().enumerate() {
        if re.is_match(line) {
            let mut result = String::new();

            // Add lines before block
            for l in &lines[..idx] {
                result.push_str(l);
                result.push('\n');
            }

            // Add replacement
            result.push_str(replacement.trim_end());
            result.push('\n');

            // Add lines after block
            for l in &lines[idx + 1..] {
                result.push_str(l);
                result.push('\n');
            }

            // Remove trailing newline if original didn't have one
            if !content.ends_with('\n') && result.ends_with('\n') {
                result.pop();
            }

            return Ok(result);
        }
    }

    Err(VaultError::BlockNotFound(block_id.to_string()))
}

// Check if stdin is a terminal (for reading from pipe)
mod atty {
    pub enum Stream {
        Stdin,
    }

    pub fn isnt(stream: Stream) -> bool {
        match stream {
            Stream::Stdin => {
                #[cfg(unix)]
                {
                    use std::os::unix::io::AsRawFd;
                    unsafe { libc::isatty(std::io::stdin().as_raw_fd()) == 0 }
                }
                #[cfg(windows)]
                {
                    // Simplified check for Windows
                    false
                }
                #[cfg(not(any(unix, windows)))]
                {
                    false
                }
            }
        }
    }
}
