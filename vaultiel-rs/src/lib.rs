//! Vaultiel - A library for programmatically interacting with Obsidian-style vaults.
//!
//! # Overview
//!
//! Vaultiel provides a programmatic interface to Obsidian vaults, enabling:
//! - Note creation, modification, and renaming (with automatic link propagation)
//! - Frontmatter manipulation (YAML + inline attributes)
//! - Link graph traversal (with rich context metadata)
//! - Tag extraction and filtering
//! - Block reference support
//! - Task extraction and formatting
//! - Search (subsequence matching like Obsidian, plus other algorithms)
//!
//! # Example
//!
//! ```no_run
//! use std::path::PathBuf;
//! use vaultiel::{Vault, Note};
//!
//! // Open a vault
//! let vault = Vault::open("/path/to/vault").unwrap();
//!
//! // List all notes
//! for path in vault.list_notes().unwrap() {
//!     println!("{}", path.display());
//! }
//!
//! // Load and parse a note
//! let note = vault.load_note(&PathBuf::from("my-note.md")).unwrap();
//! println!("Links: {:?}", note.links());
//! println!("Tags: {:?}", note.tags());
//! ```

pub mod cli;
pub mod config;
pub mod error;
pub mod graph;
pub mod note;
pub mod parser;
pub mod types;
pub mod vault;

// Re-export main types at crate root
pub use config::Config;
pub use error::{Result, VaultError};
pub use note::Note;
pub use types::*;
pub use vault::Vault;
