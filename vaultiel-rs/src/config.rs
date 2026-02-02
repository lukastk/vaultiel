//! Configuration loading for Vaultiel.

use crate::error::{Result, VaultError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub vault: VaultConfig,

    #[serde(default)]
    pub tasks: TasksConfig,

    #[serde(default)]
    pub cache: CacheConfig,
}

/// Vault-related configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VaultConfig {
    /// Default vault path.
    pub default: Option<PathBuf>,
}

/// Task symbol configuration (Obsidian Tasks plugin compatibility).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksConfig {
    #[serde(default = "default_due_symbol")]
    pub due: String,

    #[serde(default = "default_scheduled_symbol")]
    pub scheduled: String,

    #[serde(default = "default_done_symbol")]
    pub done: String,

    #[serde(default = "default_priority_highest_symbol")]
    pub priority_highest: String,

    #[serde(default = "default_priority_high_symbol")]
    pub priority_high: String,

    #[serde(default = "default_priority_medium_symbol")]
    pub priority_medium: String,

    #[serde(default = "default_priority_low_symbol")]
    pub priority_low: String,

    #[serde(default = "default_priority_lowest_symbol")]
    pub priority_lowest: String,

    /// Custom metadata fields with their symbols.
    #[serde(default)]
    pub custom_metadata: HashMap<String, String>,
}

impl Default for TasksConfig {
    fn default() -> Self {
        Self {
            due: default_due_symbol(),
            scheduled: default_scheduled_symbol(),
            done: default_done_symbol(),
            priority_highest: default_priority_highest_symbol(),
            priority_high: default_priority_high_symbol(),
            priority_medium: default_priority_medium_symbol(),
            priority_low: default_priority_low_symbol(),
            priority_lowest: default_priority_lowest_symbol(),
            custom_metadata: HashMap::new(),
        }
    }
}

fn default_due_symbol() -> String {
    "📅".to_string()
}
fn default_scheduled_symbol() -> String {
    "⏳".to_string()
}
fn default_done_symbol() -> String {
    "✅".to_string()
}
fn default_priority_highest_symbol() -> String {
    "🔺".to_string()
}
fn default_priority_high_symbol() -> String {
    "⏫".to_string()
}
fn default_priority_medium_symbol() -> String {
    "🔼".to_string()
}
fn default_priority_low_symbol() -> String {
    "🔽".to_string()
}
fn default_priority_lowest_symbol() -> String {
    "⏬".to_string()
}

/// Cache configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,

    #[serde(default = "default_cache_location")]
    pub location: String,

    #[serde(default = "default_cache_auto_threshold")]
    pub auto_threshold: usize,

    #[serde(default)]
    pub trust_mode: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            location: default_cache_location(),
            auto_threshold: default_cache_auto_threshold(),
            trust_mode: false,
        }
    }
}

fn default_cache_enabled() -> bool {
    true
}
fn default_cache_location() -> String {
    "global".to_string()
}
fn default_cache_auto_threshold() -> usize {
    500
}

impl Config {
    /// Load configuration from the default location (~/.config/vaultiel.toml).
    pub fn load() -> Result<Self> {
        let config_path = Self::default_config_path();

        if config_path.exists() {
            Self::load_from(&config_path)
        } else {
            Ok(Self::default())
        }
    }

    /// Load configuration from a specific path.
    pub fn load_from(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(|e| {
            VaultError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        toml::from_str(&content).map_err(|e| {
            VaultError::ConfigError(format!("Failed to parse config file: {}", e))
        })
    }

    /// Returns the default config file path (~/.config/vaultiel.toml).
    pub fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("vaultiel.toml")
    }

    /// Resolve the vault path from CLI argument, config, or current directory.
    pub fn resolve_vault_path(&self, cli_vault: Option<&Path>) -> Result<PathBuf> {
        // Priority: CLI flag > config > current directory
        if let Some(path) = cli_vault {
            let path = path.to_path_buf();
            if path.is_dir() {
                return Ok(path);
            } else {
                return Err(VaultError::InvalidVaultPath(path));
            }
        }

        if let Some(ref default) = self.vault.default {
            if default.is_dir() {
                return Ok(default.clone());
            } else {
                return Err(VaultError::VaultNotFound(default.clone()));
            }
        }

        // Fall back to current directory
        let cwd = std::env::current_dir()?;
        Ok(cwd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.vault.default.is_none());
        assert_eq!(config.tasks.due, "📅");
        assert!(config.cache.enabled);
    }

    #[test]
    fn test_parse_config() {
        let toml = r#"
[vault]
default = "/path/to/vault"

[tasks]
due = "📅"
scheduled = "⏳"

[tasks.custom_metadata]
time_estimate = "⏲️"

[cache]
enabled = true
location = "global"
"#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(
            config.vault.default,
            Some(PathBuf::from("/path/to/vault"))
        );
        assert_eq!(
            config.tasks.custom_metadata.get("time_estimate"),
            Some(&"⏲️".to_string())
        );
    }
}
