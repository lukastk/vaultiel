//! Integration tests for Vaultiel CLI using fixture vaults.

use std::path::PathBuf;
use std::process::Command;

/// Get the path to a fixture vault.
fn fixture_path(name: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .join("fixtures")
        .join(name)
}

/// Run vaultiel CLI command and return stdout.
fn run_vaultiel(vault: &str, args: &[&str]) -> (String, String, i32) {
    let vault_path = fixture_path(vault);
    let binary = env!("CARGO_BIN_EXE_vaultiel");

    let output = Command::new(binary)
        .arg("--vault")
        .arg(&vault_path)
        .args(args)
        .output()
        .expect("Failed to execute vaultiel");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);

    (stdout, stderr, code)
}

mod list_command {
    use super::*;

    #[test]
    fn list_minimal_vault() {
        let (stdout, _, code) = run_vaultiel("minimal", &["list"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"total\": 1"));
        assert!(stdout.contains("Note.md"));
    }

    #[test]
    fn list_links_vault() {
        let (stdout, _, code) = run_vaultiel("links", &["list"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("Hub.md"));
        assert!(stdout.contains("Page A.md"));
        assert!(stdout.contains("Page B.md"));
        assert!(stdout.contains("Orphan.md"));
    }

    #[test]
    fn list_unicode_vault() {
        let (stdout, _, code) = run_vaultiel("unicode", &["list"]);
        assert_eq!(code, 0);
        // Should handle unicode filenames
        assert!(stdout.contains(".md"));
    }
}

mod get_content_command {
    use super::*;

    #[test]
    fn get_content_basic() {
        let (stdout, _, code) = run_vaultiel("minimal", &["get-content", "Note"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("simple note"));
        assert!(stdout.contains("#tag"));
    }

    #[test]
    fn get_content_not_found() {
        let (_, stderr, code) = run_vaultiel("minimal", &["get-content", "NonExistent"]);
        assert_eq!(code, 2); // NOTE_NOT_FOUND exit code
        assert!(stderr.contains("not found"));
    }
}

mod get_frontmatter_command {
    use super::*;

    #[test]
    fn get_frontmatter_complex() {
        let (stdout, _, code) = run_vaultiel("frontmatter", &["get-frontmatter", "Complex"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"title\""));
        assert!(stdout.contains("Complex Frontmatter"));
        assert!(stdout.contains("\"rating\""));
    }

    #[test]
    fn get_frontmatter_no_frontmatter() {
        let (_, stderr, code) = run_vaultiel("frontmatter", &["get-frontmatter", "No Frontmatter"]);
        // Notes without frontmatter return an error
        assert_eq!(code, 5); // INVALID_FRONTMATTER exit code
        assert!(stderr.contains("no frontmatter"));
    }

    #[test]
    fn get_frontmatter_minimal() {
        let (stdout, _, code) = run_vaultiel("frontmatter", &["get-frontmatter", "Minimal"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"title\""));
        assert!(stdout.contains("Minimal"));
    }
}

mod search_command {
    use super::*;

    #[test]
    fn search_finds_note() {
        let (stdout, _, code) = run_vaultiel("links", &["search", "hub"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("Hub.md"));
    }

    #[test]
    fn search_no_results() {
        let (stdout, _, code) = run_vaultiel("links", &["search", "zzzznotfound"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"total\": 0"));
    }

    #[test]
    fn search_subsequence() {
        let (stdout, _, code) = run_vaultiel("links", &["search", "pga"]); // matches "Page A"
        assert_eq!(code, 0);
        assert!(stdout.contains("Page A.md"));
    }
}

mod resolve_command {
    use super::*;

    #[test]
    fn resolve_by_name() {
        let (stdout, _, code) = run_vaultiel("links", &["resolve", "Hub"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("Hub.md"));
    }

    #[test]
    fn resolve_by_alias() {
        let (stdout, _, code) = run_vaultiel("links", &["resolve", "central"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("Hub.md"));
    }
}

// Phase 2: Links, Tags & Blocks

mod get_links_command {
    use super::*;

    #[test]
    fn get_links_hub() {
        let (stdout, _, code) = run_vaultiel("links", &["get-links", "Hub"]);
        assert_eq!(code, 0);
        // Hub has incoming link from Page A and Page B
        assert!(stdout.contains("\"incoming\""));
        assert!(stdout.contains("\"outgoing\""));
        // Check outgoing links exist
        assert!(stdout.contains("Page A"));
        assert!(stdout.contains("Page B"));
    }

    #[test]
    fn get_in_links() {
        let (stdout, _, code) = run_vaultiel("links", &["get-in-links", "Hub"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"incoming\""));
        // Page A and Page B both link to Hub
        assert!(stdout.contains("Page A.md"));
    }

    #[test]
    fn get_out_links() {
        let (stdout, _, code) = run_vaultiel("links", &["get-out-links", "Hub"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"outgoing\""));
        // Hub links to Page A, Page B, and embeds
        assert!(stdout.contains("Page A"));
        assert!(stdout.contains("Page B"));
    }

    #[test]
    fn get_out_links_embeds_only() {
        let (stdout, _, code) = run_vaultiel("links", &["get-out-links", "Hub", "--embeds-only"]);
        assert_eq!(code, 0);
        // Should only show embeds (Embedded Note and image.png)
        assert!(stdout.contains("Embedded Note"));
        // Should not contain non-embed links
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let outgoing = json["outgoing"].as_array().unwrap();
        for link in outgoing {
            assert!(link["embed"].as_bool().unwrap());
        }
    }
}

mod get_embeds_command {
    use super::*;

    #[test]
    fn get_embeds() {
        let (stdout, _, code) = run_vaultiel("links", &["get-embeds", "Hub"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"embeds\""));
        assert!(stdout.contains("Embedded Note"));
        assert!(stdout.contains("image.png"));
    }

    #[test]
    fn get_embeds_media_only() {
        let (stdout, _, code) = run_vaultiel("links", &["get-embeds", "Hub", "--media-only"]);
        assert_eq!(code, 0);
        // Should only contain image embed
        assert!(stdout.contains("image.png"));
        assert!(!stdout.contains("Embedded Note"));
    }

    #[test]
    fn get_embeds_notes_only() {
        let (stdout, _, code) = run_vaultiel("links", &["get-embeds", "Hub", "--notes-only"]);
        assert_eq!(code, 0);
        // Should only contain note embed
        assert!(stdout.contains("Embedded Note"));
        assert!(!stdout.contains("image.png"));
    }
}

mod get_tags_command {
    use super::*;

    #[test]
    fn get_tags_from_note() {
        let (stdout, _, code) = run_vaultiel("minimal", &["get-tags", "Note"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"tags\""));
        assert!(stdout.contains("#tag"));
    }

    #[test]
    fn get_tags_vault_wide() {
        let (stdout, _, code) = run_vaultiel("minimal", &["get-tags"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"tags\""));
    }

    #[test]
    fn get_tags_with_counts() {
        let (stdout, _, code) = run_vaultiel("minimal", &["get-tags", "--with-counts"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"count\""));
    }
}

mod get_blocks_command {
    use super::*;

    #[test]
    fn get_blocks_from_note() {
        let (stdout, _, code) = run_vaultiel("links", &["get-blocks", "Page A"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"blocks\""));
        assert!(stdout.contains("block-a1"));
    }

    #[test]
    fn get_blocks_no_blocks() {
        let (stdout, _, code) = run_vaultiel("minimal", &["get-blocks", "Note"]);
        assert_eq!(code, 0);
        // Should return empty blocks array
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert!(json["blocks"].as_array().unwrap().is_empty());
    }
}

mod get_headings_command {
    use super::*;

    #[test]
    fn get_headings_flat() {
        let (stdout, _, code) = run_vaultiel("links", &["get-headings", "Hub"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"headings\""));
        assert!(stdout.contains("Hub Note"));
        assert!(stdout.contains("Section One"));
        assert!(stdout.contains("Section Two"));
    }

    #[test]
    fn get_headings_nested() {
        let (stdout, _, code) = run_vaultiel("links", &["get-headings", "Hub", "--nested"]);
        assert_eq!(code, 0);
        // With nested output, Section One and Two should be under Hub Note
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let top_level = json["headings"].as_array().unwrap();
        assert_eq!(top_level.len(), 1); // Hub Note at top level
        assert_eq!(top_level[0]["text"], "Hub Note");
        assert!(top_level[0]["children"].as_array().unwrap().len() >= 2);
    }

    #[test]
    fn get_headings_level_filter() {
        let (stdout, _, code) = run_vaultiel("links", &["get-headings", "Hub", "--min-level", "2"]);
        assert_eq!(code, 0);
        // Should not contain H1 (Hub Note)
        assert!(!stdout.contains("Hub Note"));
        // Should contain H2s
        assert!(stdout.contains("Section One"));
    }
}

mod get_section_command {
    use super::*;

    #[test]
    fn get_section_by_text() {
        let (stdout, _, code) = run_vaultiel("links", &["get-section", "Hub", "Section One"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("Section One"));
        assert!(stdout.contains("Content in section one"));
    }

    #[test]
    fn get_section_content_only() {
        let (stdout, _, code) = run_vaultiel("links", &["get-section", "Hub", "Section One", "--content-only"]);
        assert_eq!(code, 0);
        // Content should not start with the heading
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let content = json["content"].as_str().unwrap();
        assert!(!content.starts_with("##"));
        assert!(content.contains("Content in section one"));
    }

    #[test]
    fn get_section_not_found() {
        let (_, stderr, code) = run_vaultiel("links", &["get-section", "Hub", "NonExistent"]);
        assert_eq!(code, 1); // GENERAL_ERROR exit code (HeadingNotFound)
        assert!(stderr.contains("not found"));
    }
}

mod rename_command {
    use std::fs;
    use tempfile::TempDir;

    fn setup_temp_vault() -> TempDir {
        let temp = TempDir::new().unwrap();
        let vault_path = temp.path();

        // Create a simple vault structure
        fs::write(vault_path.join("Source.md"), "# Source\n\nThis is the source note.").unwrap();
        fs::write(vault_path.join("Linker.md"), "# Linker\n\nThis links to [[Source]].").unwrap();

        temp
    }

    #[test]
    fn rename_dry_run() {
        let temp = setup_temp_vault();
        let binary = env!("CARGO_BIN_EXE_vaultiel");

        let output = std::process::Command::new(binary)
            .arg("--vault")
            .arg(temp.path())
            .args(["rename", "Source", "Target", "--dry-run"])
            .output()
            .expect("Failed to execute vaultiel");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let code = output.status.code().unwrap_or(-1);

        assert_eq!(code, 0);
        assert!(stdout.contains("\"action\": \"rename\""));
        assert!(stdout.contains("Source.md"));
        assert!(stdout.contains("Target.md"));
        assert!(stdout.contains("would_update"));

        // Verify file wasn't actually renamed
        assert!(temp.path().join("Source.md").exists());
        assert!(!temp.path().join("Target.md").exists());
    }

    #[test]
    fn rename_no_propagate() {
        let temp = setup_temp_vault();
        let binary = env!("CARGO_BIN_EXE_vaultiel");

        let output = std::process::Command::new(binary)
            .arg("--vault")
            .arg(temp.path())
            .args(["rename", "Source", "Target", "--no-propagate"])
            .output()
            .expect("Failed to execute vaultiel");

        let code = output.status.code().unwrap_or(-1);
        assert_eq!(code, 0);

        // Verify file was renamed
        assert!(!temp.path().join("Source.md").exists());
        assert!(temp.path().join("Target.md").exists());

        // Verify Linker still has old link (no propagation)
        let linker_content = fs::read_to_string(temp.path().join("Linker.md")).unwrap();
        assert!(linker_content.contains("[[Source]]"));
    }
}

// Phase 3: Tasks

mod get_tasks_command {
    use super::*;

    #[test]
    fn get_tasks_all() {
        let (stdout, _, code) = run_vaultiel("tasks", &["get-tasks"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"tasks\""));
        // Should contain hierarchical output by default
        assert!(stdout.contains("\"children\""));
    }

    #[test]
    fn get_tasks_flat() {
        let (stdout, _, code) = run_vaultiel("tasks", &["get-tasks", "--flat"]);
        assert_eq!(code, 0);
        // Flat output should not have children arrays
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let tasks = json["tasks"].as_array().unwrap();
        assert!(!tasks.is_empty());
        // Check first task doesn't have children key (or it's missing)
        for task in tasks {
            assert!(task.get("children").is_none());
        }
    }

    #[test]
    fn get_tasks_filter_by_symbol() {
        let (stdout, _, code) = run_vaultiel("tasks", &["get-tasks", "--symbol", "[x]", "--flat"]);
        assert_eq!(code, 0);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let tasks = json["tasks"].as_array().unwrap();
        // All tasks should have [x] symbol
        for task in tasks {
            assert_eq!(task["symbol"], "[x]");
        }
    }

    #[test]
    fn get_tasks_filter_by_priority() {
        let (stdout, _, code) = run_vaultiel("tasks", &["get-tasks", "--priority", "high", "--flat"]);
        assert_eq!(code, 0);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let tasks = json["tasks"].as_array().unwrap();
        assert!(!tasks.is_empty());
        for task in tasks {
            assert_eq!(task["priority"], "high");
        }
    }

    #[test]
    fn get_tasks_with_links() {
        let (stdout, _, code) = run_vaultiel("tasks", &["get-tasks", "--flat"]);
        assert_eq!(code, 0);
        // Should have tasks with links
        assert!(stdout.contains("\"links\""));
        assert!(stdout.contains("\"to\""));
    }

    #[test]
    fn get_tasks_with_tags() {
        let (stdout, _, code) = run_vaultiel("tasks", &["get-tasks", "--flat"]);
        assert_eq!(code, 0);
        // Should have tasks with tags
        assert!(stdout.contains("\"tags\""));
        assert!(stdout.contains("#high-priority"));
    }

    #[test]
    fn get_tasks_with_block_ids() {
        let (stdout, _, code) = run_vaultiel("tasks", &["get-tasks", "--has-block-ref", "--flat"]);
        assert_eq!(code, 0);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let tasks = json["tasks"].as_array().unwrap();
        // All returned tasks should have block_id
        for task in tasks {
            assert!(task["block_id"].is_string());
        }
    }
}

mod format_task_command {
    use super::*;

    #[test]
    fn format_simple_task() {
        let (stdout, _, code) = run_vaultiel("tasks", &["format-task", "--desc", "Test task"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"formatted\""));
        assert!(stdout.contains("- [ ] Test task"));
    }

    #[test]
    fn format_task_with_due() {
        let (stdout, _, code) = run_vaultiel("tasks", &["format-task", "--desc", "Task with due", "--due", "2026-02-15"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("2026-02-15"));
        assert!(stdout.contains("📅"));
    }

    #[test]
    fn format_task_with_priority() {
        let (stdout, _, code) = run_vaultiel("tasks", &["format-task", "--desc", "High priority", "--priority", "high"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("⏫")); // High priority emoji
    }

    #[test]
    fn format_task_relative_date() {
        let (stdout, _, code) = run_vaultiel("tasks", &["format-task", "--desc", "Tomorrow task", "--due", "tomorrow"]);
        assert_eq!(code, 0);
        // Should contain a date (tomorrow from today)
        assert!(stdout.contains("📅"));
        assert!(stdout.contains("2026-02")); // Should be in February 2026
    }
}

// Phase 4: Vault Health & Info

mod info_command {
    use super::*;

    #[test]
    fn info_basic() {
        let (stdout, _, code) = run_vaultiel("links", &["info"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"note_count\""));
        assert!(stdout.contains("\"link_count\""));
        assert!(stdout.contains("\"tag_count\""));
        assert!(stdout.contains("\"orphan_count\""));
    }

    #[test]
    fn info_detailed() {
        let (stdout, _, code) = run_vaultiel("links", &["info", "--detailed"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"notes_by_folder\""));
        assert!(stdout.contains("\"top_tags\""));
        assert!(stdout.contains("\"top_linked\""));
        assert!(stdout.contains("\"recently_modified\""));
    }
}

mod lint_command {
    use super::*;

    #[test]
    fn lint_all_checks() {
        let (stdout, _, code) = run_vaultiel("links", &["lint"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"issues\""));
        assert!(stdout.contains("\"summary\""));
    }

    #[test]
    fn lint_only_orphans() {
        let (stdout, _, code) = run_vaultiel("links", &["lint", "--only", "orphans"]);
        assert_eq!(code, 0);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let issues = json["issues"].as_array().unwrap();
        for issue in issues {
            assert_eq!(issue["type"], "orphans");
        }
    }

    #[test]
    fn lint_ignore_orphans() {
        let (stdout, _, code) = run_vaultiel("links", &["lint", "--ignore", "orphans"]);
        assert_eq!(code, 0);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let issues = json["issues"].as_array().unwrap();
        for issue in issues {
            assert_ne!(issue["type"], "orphans");
        }
    }

    #[test]
    fn lint_fail_on() {
        let (_, _, code) = run_vaultiel("links", &["lint", "--fail-on", "broken-links"]);
        assert_eq!(code, 10); // LINT_ISSUES_FOUND exit code
    }

    #[test]
    fn lint_text_format() {
        let (stdout, _, code) = run_vaultiel("links", &["lint", "--format", "text"]);
        assert_eq!(code, 0);
        // Text format includes issue type in brackets
        assert!(stdout.contains("["));
    }

    #[test]
    fn lint_github_format() {
        let (stdout, _, code) = run_vaultiel("links", &["lint", "--format", "github"]);
        assert_eq!(code, 0);
        // GitHub format uses ::error or ::warning
        assert!(stdout.contains("::"));
    }
}

mod find_orphans_command {
    use super::*;

    #[test]
    fn find_orphans_basic() {
        let (stdout, _, code) = run_vaultiel("links", &["find-orphans"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"orphans\""));
        assert!(stdout.contains("\"count\""));
        assert!(stdout.contains("Orphan.md"));
    }

    #[test]
    fn find_orphans_exclude() {
        let (stdout, _, code) = run_vaultiel("links", &["find-orphans", "--exclude", "Orphan*"]);
        assert_eq!(code, 0);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["count"], 0);
    }
}

mod find_broken_links_command {
    use super::*;

    #[test]
    fn find_broken_links_basic() {
        let (stdout, _, code) = run_vaultiel("links", &["find-broken-links"]);
        assert_eq!(code, 0);
        assert!(stdout.contains("\"broken_links\""));
        assert!(stdout.contains("\"count\""));
    }

    #[test]
    fn find_broken_links_in_note() {
        let (stdout, _, code) = run_vaultiel("links", &["find-broken-links", "--note", "Hub"]);
        assert_eq!(code, 0);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let links = json["broken_links"].as_array().unwrap();
        // Hub.md has one broken embed (image.png)
        for link in links {
            assert!(link["file"].as_str().unwrap().contains("Hub"));
        }
    }
}

mod cache_command {
    use std::fs;
    use tempfile::TempDir;

    /// Create a temp vault for cache tests to avoid modifying fixture vaults.
    fn create_temp_vault() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Create a test note
        let note_content = r#"---
title: Test Note
tags:
  - rust
  - cli
aliases:
  - my-test
---

# Test Heading

This is a [[link]] to another note.

- [ ] A task 📅 2026-02-10

Some content with a #tag.

A paragraph with ^block-id
"#;
        fs::write(temp_dir.path().join("test.md"), note_content).unwrap();

        // Create another note
        let note2_content = r#"---
title: Second Note
---

# Second

Content with [[test]] link back.
"#;
        fs::write(temp_dir.path().join("second.md"), note2_content).unwrap();

        temp_dir
    }

    fn run_cache_cmd(vault_path: &std::path::Path, args: &[&str]) -> (String, String, i32) {
        let binary = env!("CARGO_BIN_EXE_vaultiel");

        let output = std::process::Command::new(binary)
            .arg("--vault")
            .arg(vault_path)
            .args(args)
            .output()
            .expect("Failed to execute vaultiel");

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let code = output.status.code().unwrap_or(-1);

        (stdout, stderr, code)
    }

    #[test]
    fn cache_rebuild() {
        let temp_vault = create_temp_vault();

        let (stdout, _, code) = run_cache_cmd(temp_vault.path(), &["cache", "rebuild"]);
        assert_eq!(code, 0);

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["indexed_notes"], 2);
    }

    #[test]
    fn cache_status() {
        let temp_vault = create_temp_vault();

        // First rebuild
        let (_, _, code) = run_cache_cmd(temp_vault.path(), &["cache", "rebuild"]);
        assert_eq!(code, 0);

        // Then check status
        let (stdout, _, code) = run_cache_cmd(temp_vault.path(), &["cache", "status"]);
        assert_eq!(code, 0);

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["indexed_notes"], 2);
        assert!(json["cache_path"].as_str().unwrap().contains("vaultiel"));
    }

    #[test]
    fn cache_clear() {
        let temp_vault = create_temp_vault();

        // First rebuild
        let (_, _, code) = run_cache_cmd(temp_vault.path(), &["cache", "rebuild"]);
        assert_eq!(code, 0);

        // Then clear
        let (stdout, _, code) = run_cache_cmd(temp_vault.path(), &["cache", "clear"]);
        assert_eq!(code, 0);

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert!(json["cleared"].as_bool().unwrap());
    }

    #[test]
    fn cache_rebuild_progress() {
        let temp_vault = create_temp_vault();

        let (stdout, stderr, code) = run_cache_cmd(
            temp_vault.path(),
            &["cache", "rebuild", "--progress"],
        );
        assert_eq!(code, 0);

        // Progress output goes to stderr
        assert!(stderr.contains("Indexing"));

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["indexed_notes"], 2);
    }
}

mod metadata_command {
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_vault() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Note without metadata
        let note_content = r#"---
title: Test Note
tags:
  - rust
---

# Test

Content.
"#;
        fs::write(temp_dir.path().join("test.md"), note_content).unwrap();

        // Note with existing metadata
        let note_with_meta = r#"---
title: With Meta
vaultiel:
  id: "existing-uuid-12345"
  created: "2026-01-01T00:00:00Z"
---

# With Meta

Content.
"#;
        fs::write(temp_dir.path().join("with-meta.md"), note_with_meta).unwrap();

        // Another note
        let note2 = r#"---
title: Another
---

# Another

Content.
"#;
        fs::write(temp_dir.path().join("another.md"), note2).unwrap();

        temp_dir
    }

    fn run_cmd(vault_path: &std::path::Path, args: &[&str]) -> (String, String, i32) {
        let binary = env!("CARGO_BIN_EXE_vaultiel");

        let output = std::process::Command::new(binary)
            .arg("--vault")
            .arg(vault_path)
            .args(args)
            .output()
            .expect("Failed to execute vaultiel");

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let code = output.status.code().unwrap_or(-1);

        (stdout, stderr, code)
    }

    #[test]
    fn init_metadata_single() {
        let temp_vault = create_temp_vault();

        let (stdout, _, code) = run_cmd(temp_vault.path(), &["init-metadata", "test"]);
        assert_eq!(code, 0);

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["action"], "initialized");
        assert!(json["metadata"]["id"].as_str().is_some());
    }

    #[test]
    fn init_metadata_skips_existing() {
        let temp_vault = create_temp_vault();

        let (stdout, _, code) = run_cmd(temp_vault.path(), &["init-metadata", "with-meta"]);
        assert_eq!(code, 0);

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["action"], "skipped");
        assert_eq!(json["metadata"]["id"], "existing-uuid-12345");
    }

    #[test]
    fn init_metadata_force() {
        let temp_vault = create_temp_vault();

        let (stdout, _, code) = run_cmd(
            temp_vault.path(),
            &["init-metadata", "with-meta", "--force"],
        );
        assert_eq!(code, 0);

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["action"], "initialized");
        // New UUID should be different
        assert_ne!(json["metadata"]["id"], "existing-uuid-12345");
    }

    #[test]
    fn init_metadata_glob() {
        let temp_vault = create_temp_vault();

        let (stdout, _, code) = run_cmd(temp_vault.path(), &["init-metadata", "--glob", "*.md"]);
        assert_eq!(code, 0);

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["total"], 3);
        // with-meta should be skipped, others initialized
        assert_eq!(json["initialized"], 2);
        assert_eq!(json["skipped"], 1);
    }

    #[test]
    fn init_metadata_dry_run() {
        let temp_vault = create_temp_vault();

        let (stdout, _, code) = run_cmd(
            temp_vault.path(),
            &["init-metadata", "test", "--dry-run"],
        );
        assert_eq!(code, 0);

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(json["action"], "would_initialize");

        // Verify no actual change was made
        let (stdout2, _, _) = run_cmd(temp_vault.path(), &["get-metadata", "test"]);
        let json2: serde_json::Value = serde_json::from_str(&stdout2).unwrap();
        assert!(!json2["has_metadata"].as_bool().unwrap());
    }

    #[test]
    fn get_by_id_found() {
        let temp_vault = create_temp_vault();

        let (stdout, _, code) = run_cmd(temp_vault.path(), &["get-by-id", "existing-uuid-12345"]);
        assert_eq!(code, 0);

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert!(json["found"].as_bool().unwrap());
        assert!(json["path"].as_str().unwrap().contains("with-meta"));
    }

    #[test]
    fn get_by_id_not_found() {
        let temp_vault = create_temp_vault();

        let (stdout, _, code) = run_cmd(temp_vault.path(), &["get-by-id", "nonexistent-uuid"]);
        assert_eq!(code, 0);

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert!(!json["found"].as_bool().unwrap());
        assert!(json["path"].is_null());
    }

    #[test]
    fn get_metadata() {
        let temp_vault = create_temp_vault();

        let (stdout, _, code) = run_cmd(temp_vault.path(), &["get-metadata", "with-meta"]);
        assert_eq!(code, 0);

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert!(json["has_metadata"].as_bool().unwrap());
        assert_eq!(json["metadata"]["id"], "existing-uuid-12345");
    }

    #[test]
    fn get_metadata_none() {
        let temp_vault = create_temp_vault();

        let (stdout, _, code) = run_cmd(temp_vault.path(), &["get-metadata", "test"]);
        assert_eq!(code, 0);

        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert!(!json["has_metadata"].as_bool().unwrap());
    }
}
