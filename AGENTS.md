## Project Overview

Vaultiel is a Rust toolkit for programmatically interacting with Obsidian-style vaults. The core crate (`vaultiel-rs`) provides parsing, vault I/O, link graphs, task extraction, search, and metadata management. It is consumed via napi-rs bindings (`vaultiel-node`), a TypeScript Obsidian adapter (`vaultiel-obsidian`), and a standalone CLI crate (`vaultiel-cli`) that builds the `vaultiel` binary (~40 subcommands for read/parse/graph/write/metadata operations).

## Repository Structure

```
vaultiel/
├── AGENTS.md             # This file
├── fixtures/             # Shared test vaults (used by all language bindings)
│   ├── minimal/          # Single basic note
│   ├── links/            # Notes with links, aliases, embeds, orphans
│   ├── unicode/          # Japanese and emoji filenames/content
│   ├── frontmatter/      # Various frontmatter structures
│   ├── tasks/            # Notes with tasks and task hierarchies
│   └── obako/            # Obako-flavored vault fixture
├── vaultiel-rs/          # Rust core library
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs        # Library exports
│   │   ├── parser/       # Parsing logic (frontmatter, wikilinks, tags, tasks, etc.)
│   │   ├── graph/        # Link graph construction and resolution
│   │   ├── search/       # Query parser + matcher (subsequence/fuzzy/exact/regex)
│   │   ├── vault.rs      # Vault operations
│   │   ├── note.rs       # Note struct and methods
│   │   ├── config.rs     # TaskConfig, EmojiFieldDef, EmojiValueType
│   │   ├── metadata.rs   # Vaultiel metadata (UUID-based note identification)
│   │   ├── error.rs      # Error types
│   │   └── types.rs      # Shared types (Link, Tag, Task, etc.)
│   │                     # (tests are inline #[cfg(test)] modules, no tests/ dir)
├── vaultiel-cli/         # Rust CLI crate — builds the `vaultiel` binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs       # Clap CLI definition + dispatch
│       └── commands/     # read.rs, parse.rs, graph.rs, write.rs, meta.rs
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
