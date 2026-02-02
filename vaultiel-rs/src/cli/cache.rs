//! Cache CLI commands.

use crate::cache::{clear_cache, get_cache_dir, VaultCache};
use crate::cli::output::Output;
use crate::config::TaskConfig;
use crate::error::{ExitCode, Result};
use crate::vault::Vault;
use serde::Serialize;

/// Show cache status.
pub fn status(vault: &Vault, output: &Output) -> Result<ExitCode> {
    let cache = VaultCache::load(&vault.root, &vault.config.cache)?;
    let status = cache.status(vault)?;
    output.print(&status)?;
    Ok(ExitCode::Success)
}

/// Rebuild the cache.
pub fn rebuild(vault: &Vault, verbose: bool, output: &Output) -> Result<ExitCode> {
    let mut cache = VaultCache::load(&vault.root, &vault.config.cache)?;
    let task_config = TaskConfig::from(&vault.config.tasks);

    let count = cache.rebuild(vault, &task_config, verbose)?;

    let result = RebuildResult {
        indexed_notes: count,
        cache_path: cache.cache_dir().to_path_buf(),
    };

    output.print(&result)?;
    Ok(ExitCode::Success)
}

/// Clear the cache.
pub fn clear(vault: &Vault, output: &Output) -> Result<ExitCode> {
    let cache_dir = get_cache_dir(&vault.root, &vault.config.cache);

    clear_cache(&cache_dir)?;

    let result = ClearResult {
        cleared: true,
        cache_path: cache_dir,
    };

    output.print(&result)?;
    Ok(ExitCode::Success)
}

#[derive(Debug, Serialize)]
struct RebuildResult {
    indexed_notes: usize,
    cache_path: std::path::PathBuf,
}

#[derive(Debug, Serialize)]
struct ClearResult {
    cleared: bool,
    cache_path: std::path::PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::output::Output;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_vault() -> (TempDir, Vault) {
        let temp_dir = TempDir::new().unwrap();

        let note_path = temp_dir.path().join("test.md");
        fs::write(
            &note_path,
            "---\ntitle: Test\n---\n\n# Test\n\nSome content.",
        )
        .unwrap();

        let config = crate::config::Config::default();
        let vault = Vault::new(temp_dir.path().to_path_buf(), config).unwrap();

        (temp_dir, vault)
    }

    #[test]
    fn test_cache_rebuild() {
        let (_temp_dir, vault) = create_test_vault();
        let output = Output::new(crate::cli::args::OutputFormat::Json, true);

        let result = rebuild(&vault, false, &output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cache_status() {
        let (_temp_dir, vault) = create_test_vault();
        let output = Output::new(crate::cli::args::OutputFormat::Json, true);

        // First rebuild to create cache
        let _ = rebuild(&vault, false, &output);

        let result = status(&vault, &output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cache_clear() {
        let (_temp_dir, vault) = create_test_vault();
        let output = Output::new(crate::cli::args::OutputFormat::Json, true);

        // First rebuild to create cache
        let _ = rebuild(&vault, false, &output);

        let result = clear(&vault, &output);
        assert!(result.is_ok());
    }
}
