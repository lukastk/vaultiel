//! Link target resolution logic.
//!
//! Obsidian resolves links in this order:
//! 1. Exact path match (if target contains `/`)
//! 2. Exact filename match (case-insensitive)
//! 3. Alias match from frontmatter

use crate::vault::Vault;
use std::collections::HashMap;
use std::path::PathBuf;

/// Resolve a link target to a note path.
///
/// Returns `Some(path)` if the target resolves to an existing note,
/// or `None` if it's a broken link.
pub fn resolve_link_target(
    target: &str,
    vault: &Vault,
    aliases: &HashMap<String, PathBuf>,
) -> Option<PathBuf> {
    // Remove any heading or block reference for resolution
    let target = target.split('#').next().unwrap_or(target);

    // Normalize the target
    let target_lower = target.to_lowercase();
    let target_with_ext = if target_lower.ends_with(".md") {
        target.to_string()
    } else {
        format!("{}.md", target)
    };

    // 1. If target contains `/`, try exact path match
    if target.contains('/') {
        let path = PathBuf::from(&target_with_ext);
        if vault.note_exists(&path) {
            return Some(path);
        }
        // Also try without normalizing case
        let path = PathBuf::from(target_with_ext);
        if vault.note_exists(&path) {
            return Some(path);
        }
    }

    // 2. Try filename match (case-insensitive)
    if let Ok(notes) = vault.list_notes() {
        // First try exact filename match
        let target_stem = target_lower
            .strip_suffix(".md")
            .unwrap_or(&target_lower);

        for note_path in &notes {
            if let Some(stem) = note_path.file_stem() {
                if stem.to_string_lossy().to_lowercase() == target_stem {
                    return Some(note_path.clone());
                }
            }
        }
    }

    // 3. Try alias match
    if let Some(path) = aliases.get(&target_lower) {
        return Some(path.clone());
    }

    None
}

/// Check if a target looks like a media file (image, audio, video, PDF).
pub fn is_media_target(target: &str) -> bool {
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
        || lower.ends_with(".m4a")
        || lower.ends_with(".flac")
        || lower.ends_with(".mp4")
        || lower.ends_with(".webm")
        || lower.ends_with(".mov")
        || lower.ends_with(".avi")
        || lower.ends_with(".pdf")
}

/// Get the media type for a target.
pub fn get_media_type(target: &str) -> Option<&'static str> {
    let lower = target.to_lowercase();

    if lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".gif")
        || lower.ends_with(".webp")
        || lower.ends_with(".svg")
        || lower.ends_with(".bmp")
    {
        Some("image")
    } else if lower.ends_with(".mp3")
        || lower.ends_with(".wav")
        || lower.ends_with(".ogg")
        || lower.ends_with(".m4a")
        || lower.ends_with(".flac")
    {
        Some("audio")
    } else if lower.ends_with(".mp4")
        || lower.ends_with(".webm")
        || lower.ends_with(".mov")
        || lower.ends_with(".avi")
    {
        Some("video")
    } else if lower.ends_with(".pdf") {
        Some("pdf")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_media_target() {
        assert!(is_media_target("image.png"));
        assert!(is_media_target("photo.JPG"));
        assert!(is_media_target("audio.mp3"));
        assert!(is_media_target("video.mp4"));
        assert!(is_media_target("document.pdf"));
        assert!(!is_media_target("Note"));
        assert!(!is_media_target("Note.md"));
    }

    #[test]
    fn test_get_media_type() {
        assert_eq!(get_media_type("image.png"), Some("image"));
        assert_eq!(get_media_type("audio.mp3"), Some("audio"));
        assert_eq!(get_media_type("video.mp4"), Some("video"));
        assert_eq!(get_media_type("doc.pdf"), Some("pdf"));
        assert_eq!(get_media_type("Note.md"), None);
    }
}
