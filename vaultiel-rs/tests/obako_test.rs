//! Obako compatibility integration tests.
//!
//! Tests vaultiel against the `fixtures/obako/` vault which contains
//! representative notes for all 13 Obako note types, all checkbox symbols,
//! all Obsidian Tasks metadata markers, inline attributes, and more.

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

/// Run vaultiel CLI command and return (stdout, stderr, exit_code).
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

/// Parse stdout as JSON value.
fn parse_json(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout).expect("Failed to parse JSON output")
}

// ============================================================================
// Frontmatter per note type
// ============================================================================

mod frontmatter_per_type {
    use super::*;

    #[test]
    fn proj_stream() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "proj/Stream Project.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["notetype"], "proj");
        assert_eq!(json["proj-status"], "stream");
        assert_eq!(json["parent"], "[[Personal projects]]");
    }

    #[test]
    fn proj_active() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "proj/Active Project.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["notetype"], "proj");
        assert_eq!(json["proj-status"], "active");
        assert_eq!(json["is-passive"], false);
        assert_eq!(json["proj-start-date"], "2026-01-15");
        assert_eq!(json["proj-end-date"], "2026-03-15");
        assert_eq!(json["parent"], "[[proj/Stream Project]]");
    }

    #[test]
    fn proj_done() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "proj/Done Project.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["notetype"], "proj");
        assert_eq!(json["proj-status"], "done");
        assert_eq!(json["proj-completion-date"], "2025-12-14");
    }

    #[test]
    fn mod_active() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "mod/Active Module.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["notetype"], "mod");
        assert_eq!(json["mod-status"], "active");
        assert_eq!(json["mod-start-date"], "2026-02-20");
        assert_eq!(json["parent"], "[[proj/Active Project]]");
    }

    #[test]
    fn plan_daily() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "plan/2026-02-27.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["notetype"], "plan");
        assert_eq!(json["cons"], false);
        assert_eq!(json["planner-active"], true);
    }

    #[test]
    fn plan_weekly() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "plan/2026 w9.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["notetype"], "plan");
        assert_eq!(json["cons"], true);
        assert_eq!(json["planner-active"], false);
        assert_eq!(json["week"], 9);
    }

    #[test]
    fn log_daily() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "log/2026-02-27 Daily log.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["notetype"], "log");
        assert_eq!(json["cons"], false);
        assert_eq!(json["is-hp-cons"], false);
        assert_eq!(json["is-link-cons"], false);
        assert_eq!(json["parent"], "[[proj/Active Project]]");
    }

    #[test]
    fn cap_capture() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "cap/2026-02-27_14:30:22 Quick thought.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["notetype"], "cap");
        assert_eq!(json["cons"], false);
        assert_eq!(json["is-hp-cons"], false);
        assert_eq!(json["is-link-cons"], true);
        assert_eq!(json["parent"], "[[proj/Active Project]]");
    }

    #[test]
    fn pad_scratch() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "pad/My scratch pad.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["notetype"], "pad");
        assert_eq!(json["pad-in-writing"], true);
        assert_eq!(json["cons"], false);
        assert_eq!(json["is-hp-cons"], true);
        assert_eq!(json["is-link-cons"], false);
    }

    #[test]
    fn memo_note() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "memo/Design decision.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["notetype"], "memo");
        assert_eq!(json["memo-complete"], false);
    }

    #[test]
    fn doc_note() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "doc/My document.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["notetype"], "doc");
        assert_eq!(json["doc-status"], "writing");
    }

    #[test]
    fn ref_paper() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "ref/A Paper.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["notetype"], "ref");
        assert_eq!(json["ref-type"], "paper");
        assert_eq!(json["reading-status"], "reading");
        assert_eq!(json["reading-priority"], 3);
        assert_eq!(json["reading-category"], "nonfiction");
        assert_eq!(json["on-reading-list"], true);
        assert!(json["read-date"].is_null());
        assert_eq!(json["is-reproduction"], false);
        // authors should be an array
        let authors = json["authors"].as_array().expect("authors should be array");
        assert_eq!(authors.len(), 2);
        assert_eq!(authors[0], "Smith");
        assert_eq!(authors[1], "Jones");
    }

    #[test]
    fn ent_entity() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "ent/Some Person.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["notetype"], "ent");
        assert!(json["url"].is_null());
    }

    #[test]
    fn con_concept() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "con/Some Concept.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["notetype"], "con");
    }

    #[test]
    fn box_note() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "box/20260213_mafds6__boxyard.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["notetype"], "box");
        assert_eq!(json["box-name"], "boxyard");
        assert_eq!(json["box-missing"], false);
        assert_eq!(json["box-storage-location"], "hetzner-box");
        // box-groups should be an array of wikilinks
        let groups = json["box-groups"].as_array().expect("box-groups should be array");
        assert_eq!(groups.len(), 2);
        let first = groups[0].as_str().unwrap();
        assert!(first.contains("proj"), "first group should contain 'proj': {}", first);
    }

    #[test]
    fn archived_note() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "Archived Note.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["archived"], true);
        assert_eq!(json["cons"], true);
        assert_eq!(json["is-hp-cons"], true);
        assert_eq!(json["is-link-cons"], true);
    }
}

// ============================================================================
// Wikilinks in frontmatter
// ============================================================================

mod frontmatter_wikilinks {
    use super::*;

    #[test]
    fn parent_is_wikilink() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "proj/Active Project.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let parent = json["parent"].as_str().unwrap();
        assert!(parent.starts_with("[[") && parent.ends_with("]]"),
            "parent should be a wikilink: {}", parent);
    }

    #[test]
    fn box_groups_wikilink_array() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "box/20260213_mafds6__boxyard.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let groups = json["box-groups"].as_array().expect("box-groups should be array");
        for group in groups {
            let s = group.as_str().unwrap();
            assert!(s.starts_with("[[") && s.ends_with("]]"),
                "each box-group should be a wikilink: {}", s);
        }
    }

    #[test]
    fn links_array_wikilinks() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "proj/Active Project.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let links = json["links"].as_array().expect("links should be array");
        assert_eq!(links.len(), 1);
        let link = links[0].as_str().unwrap();
        assert!(link.starts_with("[[") && link.ends_with("]]"),
            "links entry should be a wikilink: {}", link);
    }
}

// ============================================================================
// Inline attributes
// ============================================================================

mod inline_attributes {
    use super::*;

    #[test]
    fn parse_inline_attrs() {
        let (stdout, _, code) = run_vaultiel("obako", &["inspect", "pad/My scratch pad.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let attrs = json["inline_attrs"].as_array().expect("inline_attrs should be array");
        assert!(attrs.len() >= 2, "expected at least 2 inline attrs, got {}", attrs.len());

        // Check that resources and related keys exist
        let keys: Vec<&str> = attrs.iter()
            .filter_map(|a| a["key"].as_str())
            .collect();
        assert!(keys.contains(&"resources"), "should have 'resources' attr: {:?}", keys);
        assert!(keys.contains(&"related"), "should have 'related' attr: {:?}", keys);
    }
}

// ============================================================================
// Task checkbox symbols
// ============================================================================

mod task_symbols {
    use super::*;

    #[test]
    fn all_nine_symbols() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "All Checkbox Symbols.md", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");

        let symbols: Vec<&str> = tasks.iter()
            .filter_map(|t| t["symbol"].as_str())
            .collect();

        assert!(symbols.contains(&"[ ]"), "missing [ ]: {:?}", symbols);
        assert!(symbols.contains(&"[x]"), "missing [x]: {:?}", symbols);
        assert!(symbols.contains(&"[>]"), "missing [>]: {:?}", symbols);
        assert!(symbols.contains(&"[+]"), "missing [+]: {:?}", symbols);
        assert!(symbols.contains(&"[d]"), "missing [d]: {:?}", symbols);
        assert!(symbols.contains(&"[A]"), "missing [A]: {:?}", symbols);
        assert!(symbols.contains(&"[-]"), "missing [-]: {:?}", symbols);
        assert!(symbols.contains(&"[f]"), "missing [f]: {:?}", symbols);
        assert!(symbols.contains(&"[N]"), "missing [N]: {:?}", symbols);
    }

    #[test]
    fn filter_by_symbol() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "All Checkbox Symbols.md", "--symbol", "[f]", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0]["symbol"], "[f]");
    }
}

// ============================================================================
// Task metadata markers
// ============================================================================

mod task_metadata {
    use super::*;

    #[test]
    fn due_date() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "All Task Metadata.md", "--due-on", "2026-03-15", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert!(!tasks.is_empty(), "should find task with due 2026-03-15");
        assert_eq!(tasks[0]["due"], "2026-03-15");
    }

    #[test]
    fn scheduled_date() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "All Task Metadata.md", "--scheduled-on", "2026-03-01", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert!(!tasks.is_empty(), "should find task with scheduled 2026-03-01");
        assert_eq!(tasks[0]["scheduled"], "2026-03-01");
    }

    #[test]
    fn start_date() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "All Task Metadata.md", "--start-on", "2026-02-20", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert!(!tasks.is_empty(), "should find task with start 2026-02-20");
        assert_eq!(tasks[0]["start"], "2026-02-20");
    }

    #[test]
    fn created_date() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "All Task Metadata.md", "--created-on", "2026-02-15", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert!(!tasks.is_empty(), "should find task with created 2026-02-15");
        assert_eq!(tasks[0]["created"], "2026-02-15");
    }

    #[test]
    fn done_date() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "All Task Metadata.md", "--done-on", "2026-02-25", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert!(!tasks.is_empty(), "should find task with done 2026-02-25");
        assert_eq!(tasks[0]["done"], "2026-02-25");
    }

    #[test]
    fn cancelled_date() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "All Task Metadata.md", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        let cancelled_tasks: Vec<&serde_json::Value> = tasks.iter()
            .filter(|t| t["cancelled"].is_string())
            .collect();
        assert!(!cancelled_tasks.is_empty(), "should find task with cancelled date");
        assert_eq!(cancelled_tasks[0]["cancelled"], "2026-02-26");
    }

    #[test]
    fn priorities() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "All Task Metadata.md", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");

        let priorities: Vec<&str> = tasks.iter()
            .filter_map(|t| t["priority"].as_str())
            .collect();

        assert!(priorities.contains(&"highest"), "missing highest priority: {:?}", priorities);
        assert!(priorities.contains(&"high"), "missing high priority: {:?}", priorities);
        assert!(priorities.contains(&"medium"), "missing medium priority: {:?}", priorities);
        assert!(priorities.contains(&"low"), "missing low priority: {:?}", priorities);
        assert!(priorities.contains(&"lowest"), "missing lowest priority: {:?}", priorities);
    }

    #[test]
    fn recurrence() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "All Task Metadata.md", "--has-recurrence", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert!(!tasks.is_empty(), "should find tasks with recurrence");
        let rec = tasks[0]["recurrence"].as_str().unwrap();
        assert!(rec.contains("every"), "recurrence should contain 'every': {}", rec);
    }

    #[test]
    fn id_and_depends() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "All Task Metadata.md", "--id", "task-001", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert!(!tasks.is_empty(), "should find task with id task-001");
        assert_eq!(tasks[0]["id"], "task-001");
    }

    #[test]
    fn on_completion() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "All Task Metadata.md", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        let oc_tasks: Vec<&serde_json::Value> = tasks.iter()
            .filter(|t| t["on_completion"].is_string())
            .collect();
        assert!(!oc_tasks.is_empty(), "should find task with on_completion");
        assert_eq!(oc_tasks[0]["on_completion"], "delete");
    }

    #[test]
    fn custom_metadata_time_estimate() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "All Task Metadata.md", "--has", "time_estimate", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert!(tasks.len() >= 2, "should find at least 2 tasks with time_estimate, got {}", tasks.len());
    }

    #[test]
    fn all_fields_combined() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "All Task Metadata.md", "--id", "full-task", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert_eq!(tasks.len(), 1, "should find exactly 1 task with id full-task");
        let task = &tasks[0];
        assert_eq!(task["id"], "full-task");
        assert_eq!(task["priority"], "high");
        assert_eq!(task["due"], "2026-03-15");
        assert_eq!(task["scheduled"], "2026-03-01");
        assert_eq!(task["start"], "2026-02-20");
        assert_eq!(task["created"], "2026-02-15");
        assert!(task["recurrence"].as_str().unwrap().contains("every week"));
        assert_eq!(task["on_completion"], "delete");
    }
}

// ============================================================================
// Task hierarchy
// ============================================================================

mod task_hierarchy {
    use super::*;

    #[test]
    fn nested_subtasks() {
        // Hierarchical (default) output should show parent-child
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "Mixed Task Note.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");

        // Find the completed review task that has children
        let parent_task = tasks.iter().find(|t| {
            t["description"].as_str().map_or(false, |d| d.contains("Completed review task"))
        });
        assert!(parent_task.is_some(), "should find parent task 'Completed review task'");
        let parent = parent_task.unwrap();
        let children = parent["children"].as_array();
        assert!(children.is_some(), "parent task should have children array");
        assert_eq!(children.unwrap().len(), 2, "should have 2 child tasks");
    }

    #[test]
    fn flat_mode_no_nesting() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "Mixed Task Note.md", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        // All tasks should be at top level in flat mode
        for task in tasks {
            assert!(task.get("children").is_none() || task["children"].is_null(),
                "flat mode should not have children: {}", task);
        }
    }
}

// ============================================================================
// Tags in tasks
// ============================================================================

mod task_tags {
    use super::*;

    #[test]
    fn tags_extracted() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "Mixed Task Note.md", "--tag", "#tray", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert_eq!(tasks.len(), 1, "should find exactly 1 task with #tray tag");
        assert!(tasks[0]["description"].as_str().unwrap().contains("Fix the build pipeline"));
    }

    #[test]
    fn hp_cons_tag() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "Mixed Task Note.md", "--tag", "#hp-cons", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert_eq!(tasks.len(), 1);
    }
}

// ============================================================================
// Block refs in tasks
// ============================================================================

mod task_block_refs {
    use super::*;

    #[test]
    fn block_ref_detected() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "Mixed Task Note.md", "--has-block-ref", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert!(tasks.len() >= 2, "should find at least 2 tasks with block refs, got {}", tasks.len());
    }

    #[test]
    fn specific_block_ref() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "Mixed Task Note.md", "--block-ref", "abc123", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert_eq!(tasks.len(), 1, "should find exactly 1 task with ^abc123");
    }
}

// ============================================================================
// Format task round-trip and ordering
// ============================================================================

mod format_task {
    use super::*;

    #[test]
    fn canonical_order() {
        let (stdout, _, code) = run_vaultiel("obako", &[
            "format-task",
            "--desc", "Test task",
            "--id", "test-id",
            "--depends-on", "dep-1",
            "--priority", "high",
            "--recurrence", "every week",
            "--on-completion", "delete",
            "--created", "2026-02-15",
            "--start", "2026-02-20",
            "--scheduled", "2026-03-01",
            "--due", "2026-03-15",
            "--done", "2026-03-10",
        ]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let formatted = json["formatted"].as_str().expect("should have task field");

        // Verify ordering: id before priority before recurrence before dates
        let id_pos = formatted.find("🆔").expect("should have id");
        let dep_pos = formatted.find("⛔").expect("should have depends_on");
        let pri_pos = formatted.find("⏫").expect("should have priority");
        let rec_pos = formatted.find("🔁").expect("should have recurrence");
        let oc_pos = formatted.find("🏁").expect("should have on_completion");
        let created_pos = formatted.find("➕").expect("should have created");
        let start_pos = formatted.find("🛫").expect("should have start");
        let sched_pos = formatted.find("⏳").expect("should have scheduled");
        let due_pos = formatted.find("📅").expect("should have due");
        let done_pos = formatted.find("✅").expect("should have done");

        assert!(id_pos < dep_pos, "id should come before depends_on");
        assert!(dep_pos < pri_pos, "depends_on should come before priority");
        assert!(pri_pos < rec_pos, "priority should come before recurrence");
        assert!(rec_pos < oc_pos, "recurrence should come before on_completion");
        assert!(oc_pos < created_pos, "on_completion should come before created");
        assert!(created_pos < start_pos, "created should come before start");
        assert!(start_pos < sched_pos, "start should come before scheduled");
        assert!(sched_pos < due_pos, "scheduled should come before due");
        assert!(due_pos < done_pos, "due should come before done");
    }

    #[test]
    fn custom_metadata_before_recognized_fields() {
        let (stdout, _, code) = run_vaultiel("obako", &[
            "format-task",
            "--desc", "Test task",
            "--custom", "time_estimate=2h",
            "--priority", "high",
            "--due", "2026-03-15",
        ]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let formatted = json["formatted"].as_str().expect("should have task field");

        let custom_pos = formatted.find("⏲️").expect("should have custom metadata");
        let pri_pos = formatted.find("⏫").expect("should have priority");
        let due_pos = formatted.find("📅").expect("should have due");

        assert!(custom_pos < pri_pos, "custom metadata should come before priority");
        assert!(pri_pos < due_pos, "priority should come before due");
    }

    #[test]
    fn round_trip() {
        // Format a task, then parse it back by creating a temp note
        let (stdout, _, code) = run_vaultiel("obako", &[
            "format-task",
            "--desc", "Round trip test",
            "--scheduled", "2026-03-01",
            "--due", "2026-03-15",
            "--priority", "medium",
            "--custom", "time_estimate=1h",
        ]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let formatted = json["formatted"].as_str().expect("should have task field");

        // The formatted task should contain all our fields
        assert!(formatted.contains("Round trip test"));
        assert!(formatted.contains("⏳ 2026-03-01"));
        assert!(formatted.contains("📅 2026-03-15"));
        assert!(formatted.contains("🔼")); // medium priority
        assert!(formatted.contains("⏲️ 1h"));
    }
}

// ============================================================================
// Frontmatter queries
// ============================================================================

mod frontmatter_queries {
    use super::*;

    #[test]
    fn filter_by_notetype() {
        let (stdout, _, code) = run_vaultiel("obako", &["list", "--frontmatter", "notetype=proj"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let notes = json["notes"].as_array().expect("notes should be array");
        assert_eq!(notes.len(), 4, "should find 4 project notes, got {}", notes.len());
    }

    #[test]
    fn compound_filter() {
        let (stdout, _, code) = run_vaultiel("obako", &[
            "list",
            "--frontmatter", "notetype=proj",
            "--frontmatter", "proj-status=active",
        ]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let notes = json["notes"].as_array().expect("notes should be array");
        assert_eq!(notes.len(), 1, "should find 1 active project");
        assert!(notes[0]["path"].as_str().unwrap().contains("Active Project"));
    }

    #[test]
    fn negation_filter() {
        let (stdout, _, code) = run_vaultiel("obako", &[
            "list",
            "--frontmatter", "notetype=proj",
            "--frontmatter", "proj-status!=done",
        ]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let notes = json["notes"].as_array().expect("notes should be array");
        // Should exclude Done Project (proj-status=done)
        assert_eq!(notes.len(), 3, "should find 3 non-done projects, got {}", notes.len());
        let paths: Vec<&str> = notes.iter()
            .filter_map(|n| n["path"].as_str())
            .collect();
        assert!(!paths.iter().any(|p| p.contains("Done Project")),
            "should not include Done Project: {:?}", paths);
    }

    #[test]
    fn list_contains_filter() {
        // box-groups contains wikilink with "proj"
        let (stdout, _, code) = run_vaultiel("obako", &[
            "list",
            "--frontmatter", "box-groups~=proj",
        ]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let notes = json["notes"].as_array().expect("notes should be array");
        assert!(notes.len() >= 1, "should find box notes with proj group");
    }
}

// ============================================================================
// Inspect command
// ============================================================================

mod inspect_command {
    use super::*;

    #[test]
    fn full_json_output() {
        let (stdout, _, code) = run_vaultiel("obako", &["inspect", "proj/Active Project.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);

        // All top-level fields should be present
        assert!(json["path"].is_string(), "should have path");
        assert!(json["name"].is_string(), "should have name");
        assert!(json["frontmatter"].is_object(), "should have frontmatter");
        assert!(json["inline_attrs"].is_array(), "should have inline_attrs");
        assert!(json["headings"].is_array(), "should have headings");
        assert!(json["tasks"].is_array(), "should have tasks");
        assert!(json["links"].is_object(), "should have links");
        assert!(json["tags"].is_array(), "should have tags");
        assert!(json["block_ids"].is_array(), "should have block_ids");
        assert!(json["stats"].is_object(), "should have stats");

        // Stats fields
        assert!(json["stats"]["lines"].is_number());
        assert!(json["stats"]["words"].is_number());
        assert!(json["stats"]["size_bytes"].is_number());

        // Links structure
        assert!(json["links"]["outgoing"].is_array());
        assert!(json["links"]["incoming"].is_array());

        // Frontmatter should match
        assert_eq!(json["frontmatter"]["notetype"], "proj");
        assert_eq!(json["frontmatter"]["proj-status"], "active");
    }

    #[test]
    fn inspect_has_tasks() {
        let (stdout, _, code) = run_vaultiel("obako", &["inspect", "proj/Active Project.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert!(tasks.len() >= 3, "Active Project should have at least 3 tasks");
    }

    #[test]
    fn inspect_has_headings() {
        let (stdout, _, code) = run_vaultiel("obako", &["inspect", "proj/Active Project.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let headings = json["headings"].as_array().expect("headings should be array");
        // Should have "Active Project" (h1) and "Tasks" (h2)
        assert!(headings.len() >= 2, "should have at least 2 headings, got {}", headings.len());
    }

    #[test]
    fn inspect_has_outgoing_links() {
        let (stdout, _, code) = run_vaultiel("obako", &["inspect", "pad/My scratch pad.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let outgoing = json["links"]["outgoing"].as_array().expect("outgoing should be array");
        assert!(outgoing.len() >= 2, "scratch pad should have outgoing links, got {}", outgoing.len());
    }
}

// ============================================================================
// Task links
// ============================================================================

mod task_links {
    use super::*;

    #[test]
    fn links_populated_in_tasks() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-tasks", "--note", "Mixed Task Note.md", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");

        // Find the task that links to proj/Active Project
        let linking_task = tasks.iter().find(|t| {
            t["links"].as_array().map_or(false, |links| {
                links.iter().any(|l| l["to"].as_str().unwrap() == "proj/Active Project")
            })
        });
        assert!(linking_task.is_some(), "should find a task linking to proj/Active Project");
    }

    #[test]
    fn links_to_filter_obako() {
        let (stdout, _, code) = run_vaultiel("obako", &[
            "get-tasks", "--links-to", "proj/Active Project", "--flat"
        ]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        // Mixed Task Note has 2 tasks linking to proj/Active Project
        assert!(tasks.len() >= 2, "should find at least 2 tasks linking to proj/Active Project, got {}", tasks.len());
        for task in tasks {
            let links = task["links"].as_array().unwrap();
            let has_target = links.iter().any(|l| {
                l["to"].as_str().unwrap().to_lowercase().contains("active project")
            });
            assert!(has_target, "task should link to Active Project: {}", task["description"]);
        }
    }

    #[test]
    fn links_to_filter_memo() {
        let (stdout, _, code) = run_vaultiel("obako", &[
            "get-tasks", "--links-to", "memo/Design decision", "--flat"
        ]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert_eq!(tasks.len(), 1, "should find 1 task linking to memo/Design decision");
        assert!(tasks[0]["description"].as_str().unwrap().contains("Design decision"));
    }

    #[test]
    fn task_link_alias_preserved() {
        // In Project Tasks fixture, "Review [[Code Review|PR #123]]" has an alias
        let (stdout, _, code) = run_vaultiel("tasks", &["get-tasks", "--links-to", "Code Review", "--flat"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        let tasks = json["tasks"].as_array().expect("tasks should be array");
        assert_eq!(tasks.len(), 1);
        let link = tasks[0]["links"].as_array().unwrap().iter()
            .find(|l| l["to"].as_str().unwrap() == "Code Review").unwrap();
        assert_eq!(link["alias"].as_str().unwrap(), "PR #123");
    }
}

// ============================================================================
// Consolidation fields
// ============================================================================

mod consolidation_fields {
    use super::*;

    #[test]
    fn cons_fields_readable() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "log/2026-02-27 Daily log.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["cons"], false);
        assert_eq!(json["is-hp-cons"], false);
        assert_eq!(json["is-link-cons"], false);
    }

    #[test]
    fn cons_true_values() {
        let (stdout, _, code) = run_vaultiel("obako", &["get-frontmatter", "Archived Note.md"]);
        assert_eq!(code, 0);
        let json = parse_json(&stdout);
        assert_eq!(json["cons"], true);
        assert_eq!(json["is-hp-cons"], true);
        assert_eq!(json["is-link-cons"], true);
    }
}
