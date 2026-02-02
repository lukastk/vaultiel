//! Caching system for large vault performance.
//!
//! The cache stores parsed vault data to avoid re-parsing on every command:
//! - Note index (paths, mtimes, frontmatter)
//! - Link graph (outgoing links with context)
//! - Tags
//! - Block IDs
//! - Tasks
//! - Headings

mod index;
mod types;

pub use index::VaultCache;
pub use types::*;

use crate::config::CacheConfig;
use crate::error::{Result, VaultError};
use std::fs;
use std::path::{Path, PathBuf};

/// Current cache format version. Increment when cache format changes.
pub const CACHE_VERSION: u32 = 1;

/// Get the cache directory for a vault.
pub fn get_cache_dir(vault_root: &Path, config: &CacheConfig) -> PathBuf {
    if config.location == "local" {
        vault_root.join(".vaultiel").join("cache")
    } else {
        // Global cache at ~/.cache/vaultiel/<vault-hash>/
        let vault_hash = compute_vault_hash(vault_root);
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("vaultiel")
            .join(vault_hash)
    }
}

/// Compute a hash of the vault path for the cache directory name.
fn compute_vault_hash(vault_root: &Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    vault_root.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Check if caching should be auto-enabled for a vault.
pub fn should_auto_enable_cache(note_count: usize, config: &CacheConfig) -> bool {
    config.enabled && note_count >= config.auto_threshold
}

/// Ensure the cache directory exists.
pub fn ensure_cache_dir(cache_dir: &Path) -> Result<()> {
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir).map_err(|e| {
            VaultError::CacheError(format!("Failed to create cache directory: {}", e))
        })?;
    }
    Ok(())
}

/// Remove the cache directory entirely.
pub fn clear_cache(cache_dir: &Path) -> Result<()> {
    if cache_dir.exists() {
        fs::remove_dir_all(cache_dir).map_err(|e| {
            VaultError::CacheError(format!("Failed to clear cache: {}", e))
        })?;
    }
    Ok(())
}

/// Lock file path for the cache.
pub fn lock_file_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("cache.lock")
}

/// Acquire a lock on the cache directory.
/// Returns a guard that releases the lock when dropped.
pub fn acquire_lock(cache_dir: &Path) -> Result<CacheLock> {
    ensure_cache_dir(cache_dir)?;
    let lock_path = lock_file_path(cache_dir);

    // Simple file-based locking
    // In production, you might want a more robust locking mechanism
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 50;
    const WAIT_MS: u64 = 100;

    loop {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(file) => {
                // Write our PID to the lock file
                use std::io::Write;
                let mut file = file;
                let _ = writeln!(file, "{}", std::process::id());
                return Ok(CacheLock { path: lock_path });
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                attempts += 1;
                if attempts >= MAX_ATTEMPTS {
                    return Err(VaultError::CacheError(
                        "Cache is locked by another process".to_string(),
                    ));
                }
                std::thread::sleep(std::time::Duration::from_millis(WAIT_MS));
            }
            Err(e) => {
                return Err(VaultError::CacheError(format!(
                    "Failed to acquire cache lock: {}",
                    e
                )));
            }
        }
    }
}

/// Guard that releases the cache lock when dropped.
pub struct CacheLock {
    path: PathBuf,
}

impl Drop for CacheLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Atomic write: write to temp file, then rename.
pub fn atomic_write(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        VaultError::CacheError("Invalid cache file path".to_string())
    })?;

    // Create temp file in same directory
    let temp_path = parent.join(format!(".tmp.{}", std::process::id()));

    fs::write(&temp_path, contents).map_err(|e| {
        VaultError::CacheError(format!("Failed to write temp file: {}", e))
    })?;

    fs::rename(&temp_path, path).map_err(|e| {
        // Clean up temp file on failure
        let _ = fs::remove_file(&temp_path);
        VaultError::CacheError(format!("Failed to rename temp file: {}", e))
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_compute_vault_hash() {
        let path1 = PathBuf::from("/path/to/vault");
        let path2 = PathBuf::from("/path/to/other");

        let hash1 = compute_vault_hash(&path1);
        let hash2 = compute_vault_hash(&path2);

        assert_ne!(hash1, hash2);
        assert_eq!(hash1.len(), 16);
    }

    #[test]
    fn test_get_cache_dir_global() {
        let config = CacheConfig::default();
        let vault_root = PathBuf::from("/test/vault");
        let cache_dir = get_cache_dir(&vault_root, &config);

        assert!(cache_dir.to_string_lossy().contains("vaultiel"));
    }

    #[test]
    fn test_get_cache_dir_local() {
        let config = CacheConfig {
            location: "local".to_string(),
            ..Default::default()
        };
        let vault_root = PathBuf::from("/test/vault");
        let cache_dir = get_cache_dir(&vault_root, &config);

        assert_eq!(cache_dir, PathBuf::from("/test/vault/.vaultiel/cache"));
    }

    #[test]
    fn test_atomic_write() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");

        atomic_write(&file_path, b"test content").unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_cache_lock() {
        let temp_dir = TempDir::new().unwrap();

        {
            let _lock = acquire_lock(temp_dir.path()).unwrap();
            assert!(lock_file_path(temp_dir.path()).exists());
        }

        // Lock should be released
        assert!(!lock_file_path(temp_dir.path()).exists());
    }
}
