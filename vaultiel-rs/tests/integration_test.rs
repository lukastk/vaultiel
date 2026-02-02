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
