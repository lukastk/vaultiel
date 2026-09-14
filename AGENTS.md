## Project Overview

Vaultiel is a Rust toolkit for programmatically interacting with Obsidian-style vaults. The core crate (`vaultiel`, in `vaultiel-rs/`) provides parsing, vault I/O, link graphs, task extraction, search, and metadata management. It is consumed via napi-rs bindings (`vaultiel-node`), a TypeScript Obsidian adapter (`vaultiel-obsidian`), and a standalone CLI crate (`vaultiel-cli`) that builds the `vaultiel` binary (32 subcommands for read/parse/graph/write/metadata operations).

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
├── vaultiel-node/            # @vaultiel/node (napi-rs bindings for Node.js)
├── vaultiel-node-commands/   # TypeScript command layer over the node bindings (was vaultiel-cli)
├── vaultiel-obsidian/        # @vaultiel/obsidian (TypeScript, Obsidian APIs)
└── PROJECT_PLAN.md, PROJECT_SPEC.md, phase_plans/  # Planning / spec docs
```

## `vaultiel-obsidian`: go through Obsidian's own APIs, not the raw vault

Inside Obsidian, the adapter must produce **exactly what a UI action would**, because
callers (mysystem, obako) assume a write is indistinguishable from the user doing it by
hand. Two rules that came out of real corruption/regression bugs:

- **`renameNote` uses `app.fileManager.renameFile`, never `app.vault.rename`.** The
  latter moves the file but leaves every inbound `[[wikilink]]` dangling; `fileManager`
  is Obsidian's own rename path and rewrites every link (plain and aliased) pointing at
  the note, subject to the user's "Automatically update internal links" setting. This
  surfaced as a mysystem bug: dragging a date-range planner in NoteTimeline renames its
  note (the range is in the filename), and links to it silently broke.
- **Frontmatter writes are atomic and position-independent** (`Vault.modifyFrontmatterWith`)
  for *all* callers — the non-atomic path was a whole class of corruption.
  `Vault.getFrontmatterError` detects malformed frontmatter, and note writes carry a loud
  illegal-filename guard.

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
cargo build -p vaultiel-cli              # Build the CLI (the crate has no tests)
cd vaultiel-obsidian && npx vitest run   # TypeScript tests
```

**The `vaultiel` on PATH is an installed copy, not the workspace build.** It lives at
`~/.local/bin/vaultiel`, installed by `mysystem/dev_scripts/initialise.sh` with
`cargo install --path vaultiel-cli --root ~/.local --force`. After changing the CLI (or the
core crate it links), reinstall with that same command, or `vaultiel` keeps running the old
binary. `initialise.sh` skips the install when `~/.local/.vaultiel-commit` already matches
this repo's HEAD, so uncommitted changes are never picked up by re-running it. Every
subcommand requires `--vault <path>`.

## Implementation Guidance

- **Rust** is the core language. Use idiomatic Rust patterns.
- Prioritize correctness over performance.
- Match Obsidian's behavior where possible (link resolution, tag parsing, etc.).
