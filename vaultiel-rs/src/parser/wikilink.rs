//! Wikilink and embed parsing.

use crate::parser::code_block::{find_code_block_ranges, is_in_code_block};
use crate::types::Link;
use regex::Regex;
use std::sync::LazyLock;

// Wikilink pattern: [[target]] or [[target|alias]] or [[target#heading]] or [[target#^block]]
// The target can contain most characters except | and ]
// After #, we have either a heading (no ^) or ^blockid
static WIKILINK: LazyLock<Regex> = LazyLock::new(|| {
    // Using non-greedy matching to capture the parts
    // (!)?                     - Optional ! for embeds (group 1)
    // \[\[                     - Opening [[
    // ([^\]\|#]+)              - Target path (group 2)
    // (?:#\^([a-zA-Z0-9_-]+))? - Block reference (group 3)
    // (?:#([^\]\|]+))?         - Heading reference (group 4)
    // (?:\|([^\]]+))?          - Alias (group 5)
    // \]\]                     - Closing ]]
    Regex::new(r"(!?)\[\[([^\]\|#]+)(?:#\^([a-zA-Z0-9_-]+))?(?:#([^\]\|]+))?(?:\|([^\]]+))?\]\]").unwrap()
});

// Pattern for embeds with size: ![[image.png|400]] or ![[image.png|400x300]]
#[allow(dead_code)]
static EMBED_WITH_SIZE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"!\[\[([^\]\|#]+)(?:#\^([a-zA-Z0-9_-]+))?(?:#([^\]\|]+))?(?:\|(\d+(?:x\d+)?|[^\]]+))?\]\]").unwrap()
});

/// Parse all wikilinks (not embeds) from content.
pub fn parse_links(content: &str) -> Vec<Link> {
    parse_all_links(content)
        .into_iter()
        .filter(|link| !link.embed)
        .collect()
}

/// Parse all embeds from content.
pub fn parse_embeds(content: &str) -> Vec<Link> {
    parse_all_links(content)
        .into_iter()
        .filter(|link| link.embed)
        .collect()
}

/// Parse all wikilinks and embeds from content.
pub fn parse_all_links(content: &str) -> Vec<Link> {
    let code_ranges = find_code_block_ranges(content);
    let mut links = Vec::new();

    for cap in WIKILINK.captures_iter(content) {
        let full_match = cap.get(0).unwrap();
        let start = full_match.start();
        let end = full_match.end();

        // Skip if inside code block
        if is_in_code_block(start, &code_ranges) {
            continue;
        }

        let is_embed = cap.get(1).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        let target = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");
        let block_id = cap.get(3).map(|m| m.as_str().to_string());
        let heading = cap.get(4).map(|m| m.as_str().to_string());
        let alias = cap.get(5).map(|m| m.as_str().to_string());

        // Calculate line number
        let line = content[..start].matches('\n').count() + 1;

        // Calculate column
        let line_start = content[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let start_col = start - line_start;
        let end_col = end - line_start;

        links.push(Link {
            target: target.to_string(),
            alias,
            heading,
            block_id,
            embed: is_embed,
            line,
            start_col,
            end_col,
        });
    }

    links
}

/// Check if a string looks like an image or media embed.
pub fn is_media_embed(target: &str) -> bool {
    let lower = target.to_lowercase();
    lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".gif")
        || lower.ends_with(".webp")
        || lower.ends_with(".svg")
        || lower.ends_with(".bmp")
        || lower.ends_with(".mp3")
        || lower.ends_with(".wav")
        || lower.ends_with(".ogg")
        || lower.ends_with(".mp4")
        || lower.ends_with(".webm")
        || lower.ends_with(".pdf")
}

/// Format a wikilink as a string.
pub fn format_wikilink(link: &Link) -> String {
    let mut result = String::new();

    if link.embed {
        result.push('!');
    }

    result.push_str("[[");
    result.push_str(&link.target);

    if let Some(ref heading) = link.heading {
        result.push('#');
        result.push_str(heading);
    }

    if let Some(ref block_id) = link.block_id {
        result.push_str("#^");
        result.push_str(block_id);
    }

    if let Some(ref alias) = link.alias {
        result.push('|');
        result.push_str(alias);
    }

    result.push_str("]]");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_link() {
        let content = "See [[My Note]] for details.";
        let links = parse_links(content);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "My Note");
        assert!(links[0].alias.is_none());
        assert!(!links[0].embed);
    }

    #[test]
    fn test_link_with_alias() {
        let content = "See [[My Note|the note]] for details.";
        let links = parse_links(content);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "My Note");
        assert_eq!(links[0].alias, Some("the note".to_string()));
    }

    #[test]
    fn test_link_with_heading() {
        let content = "See [[My Note#Section]] for details.";
        let links = parse_links(content);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "My Note");
        assert_eq!(links[0].heading, Some("Section".to_string()));
    }

    #[test]
    fn test_link_with_block_ref() {
        let content = "See [[My Note#^abc123]] for details.";
        let links = parse_links(content);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "My Note");
        assert_eq!(links[0].block_id, Some("abc123".to_string()));
    }

    #[test]
    fn test_link_with_heading_and_alias() {
        let content = "[[Note#Section|alias]]";
        let links = parse_links(content);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "Note");
        assert_eq!(links[0].heading, Some("Section".to_string()));
        assert_eq!(links[0].alias, Some("alias".to_string()));
    }

    #[test]
    fn test_embed() {
        let content = "![[image.png]]";
        let embeds = parse_embeds(content);
        assert_eq!(embeds.len(), 1);
        assert!(embeds[0].embed);
        assert_eq!(embeds[0].target, "image.png");
    }

    #[test]
    fn test_note_embed() {
        let content = "![[Other Note]]";
        let embeds = parse_embeds(content);
        assert_eq!(embeds.len(), 1);
        assert!(embeds[0].embed);
        assert_eq!(embeds[0].target, "Other Note");
    }

    #[test]
    fn test_embed_with_heading() {
        let content = "![[Note#Section]]";
        let embeds = parse_embeds(content);
        assert_eq!(embeds.len(), 1);
        assert_eq!(embeds[0].heading, Some("Section".to_string()));
    }

    #[test]
    fn test_multiple_links() {
        let content = "See [[Note A]] and [[Note B|B]] and ![[image.png]].";
        let all_links = parse_all_links(content);
        assert_eq!(all_links.len(), 3);

        let links = parse_links(content);
        assert_eq!(links.len(), 2);

        let embeds = parse_embeds(content);
        assert_eq!(embeds.len(), 1);
    }

    #[test]
    fn test_link_in_code_block_skipped() {
        let content = "See [[real link]]\n\n```\n[[fake link]]\n```\n\nMore text";
        let links = parse_links(content);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "real link");
    }

    #[test]
    fn test_link_in_inline_code_skipped() {
        let content = "See [[real link]] and `[[fake link]]` here.";
        let links = parse_links(content);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "real link");
    }

    #[test]
    fn test_link_line_numbers() {
        let content = "Line 1\n[[Link on line 2]]\nLine 3\n[[Link on line 4]]";
        let links = parse_links(content);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].line, 2);
        assert_eq!(links[1].line, 4);
    }

    #[test]
    fn test_format_wikilink() {
        let link = Link {
            target: "Note".to_string(),
            alias: Some("alias".to_string()),
            heading: Some("Section".to_string()),
            block_id: None,
            embed: false,
            line: 1,
            start_col: 0,
            end_col: 10,
        };
        assert_eq!(format_wikilink(&link), "[[Note#Section|alias]]");

        let embed = Link {
            target: "image.png".to_string(),
            alias: None,
            heading: None,
            block_id: None,
            embed: true,
            line: 1,
            start_col: 0,
            end_col: 10,
        };
        assert_eq!(format_wikilink(&embed), "![[image.png]]");
    }

    #[test]
    fn test_is_media_embed() {
        assert!(is_media_embed("image.png"));
        assert!(is_media_embed("photo.JPG"));
        assert!(is_media_embed("doc.pdf"));
        assert!(is_media_embed("audio.mp3"));
        assert!(is_media_embed("video.mp4"));
        assert!(!is_media_embed("Note"));
        assert!(!is_media_embed("Note.md"));
    }

    #[test]
    fn test_link_with_path() {
        let content = "[[folder/subfolder/note]]";
        let links = parse_links(content);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "folder/subfolder/note");
    }
}
