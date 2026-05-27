# Claude Instructions for Vaultiel

## Project Overview

Vaultiel is a Rust library for programmatically interacting with Obsidian-style vaults. It provides parsing, vault I/O, link graphs, task extraction, and metadata management. Consumed via napi-rs bindings (`vaultiel-node`) and a TypeScript Obsidian adapter (`vaultiel-obsidian`).

## Repository Structure

```
vaultiel/
├── CLAUDE.md             # This file
├── fixtures/             # Shared test vaults (used by all language bindings)
│   ├── minimal/          # Single basic note
│   ├── links/            # Notes with links, aliases, embeds, orphans
│   ├── unicode/          # Japanese and emoji filenames/content
│   └── frontmatter/      # Various frontmatter structures
├── vaultiel-rs/          # Rust core library (no CLI)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs        # Library exports
│   │   ├── parser/       # Parsing logic (frontmatter, wikilinks, tags, tasks, etc.)
│   │   ├── graph/        # Link graph construction and resolution
│   │   ├── vault.rs      # Vault operations
│   │   ├── note.rs       # Note struct and methods
│   │   ├── config.rs     # TaskConfig, EmojiFieldDef, EmojiValueType
│   │   ├── metadata.rs   # Vaultiel metadata (UUID-based note identification)
│   │   ├── error.rs      # Error types
│   │   └── types.rs      # Shared types (Link, Tag, Task, etc.)
│   └── tests/            # Rust fixture-based tests
├── vaultiel-node/        # @vaultiel/node (napi-rs bindings for Node.js)
└── vaultiel-obsidian/    # @vaultiel/obsidian (TypeScript, Obsidian APIs)
```

## Task System

Tasks use a **generic emoji metadata model**. All metadata fields are user-defined via `TaskConfig`:

```rust
// In config.rs
pub struct TaskConfig { pub fields: Vec<EmojiFieldDef> }
pub struct EmojiFieldDef { emoji, field_name, value_type, order }
pub enum EmojiValueType { Date, String, Text, Number, Flag{value}, Enum{value} }
```

- `TaskConfig::empty()` — no fields, no emoji parsing
- Task struct has `metadata: HashMap<String, String>` (no named fields)
- No default config — the consuming application provides all field definitions

## Testing

- **Unit tests**: Inline in source files with `#[cfg(test)]` modules
- **Fixture vaults**: Shared across all language bindings in `fixtures/` at repo root

Run tests:
```bash
cargo test -p vaultiel                   # All Rust tests
cargo test -p vaultiel parser            # Parser tests only
cargo check -p vaultiel-node             # Check node bindings compile
cd vaultiel-obsidian && npx vitest run   # TypeScript tests
```

## Implementation Guidance

- **Rust** is the core language. Use idiomatic Rust patterns.
- Prioritize correctness over performance.
- Match Obsidian's behavior where possible (link resolution, tag parsing, etc.).
