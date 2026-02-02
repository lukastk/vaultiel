# Claude Instructions for Vaultiel

## Project Overview

Vaultiel is a CLI and library for programmatically interacting with Obsidian-style vaults. Think of it as `jq` for markdown notes with YAML frontmatter and wikilinks.

## Repository Structure

```
vaultiel/
├── PROJECT_SPEC.md       # Feature specification
├── PROJECT_PLAN.md       # Implementation roadmap
├── CLAUDE.md             # This file
├── phase_plans/          # Detailed plans for each phase
│   └── 01_core_cli.md
├── fixtures/             # Shared test vaults (used by all language bindings)
│   ├── minimal/          # Single basic note
│   ├── links/            # Notes with links, aliases, embeds, orphans
│   ├── unicode/          # Japanese and emoji filenames/content
│   └── frontmatter/      # Various frontmatter structures
├── vaultiel-rs/          # Rust core library + CLI
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs       # CLI entry point
│   │   ├── lib.rs        # Library exports
│   │   ├── cli/          # CLI command implementations
│   │   ├── parser/       # Parsing logic (frontmatter, wikilinks, tags, etc.)
│   │   ├── vault.rs      # Vault operations
│   │   ├── note.rs       # Note struct and methods
│   │   ├── config.rs     # Config file loading
│   │   ├── error.rs      # Error types and exit codes
│   │   └── types.rs      # Shared types (Link, Tag, etc.)
│   └── tests/            # Rust integration tests
│       └── integration_test.rs
├── vaultiel-ts/          # (Future) TypeScript bindings
└── vaultiel-py/          # (Future) Python bindings
```

## Key Documents

- **[PROJECT_SPEC.md](./PROJECT_SPEC.md)** - Full feature specification with command syntax, flags, output formats, and design decisions. This is the source of truth for how features should work.

- **[PROJECT_PLAN.md](./PROJECT_PLAN.md)** - Implementation roadmap with phased todo items. Use this to track progress and understand dependencies between features.

## Working on This Project

### Phase Plans

Before starting work on a phase, create a detailed phase plan in the `phase_plans/` directory. Naming format:

```
phase_plans/
├── 01_core_cli.md
├── 02_links_tags_blocks.md
├── 03_tasks.md
├── 04_vault_health.md
├── 05_caching.md
├── 06_metadata_ids.md
├── 07_bindings.md
└── 08_advanced_features.md
```

**Before beginning development on any phase, check the corresponding phase plan.** The phase plan should contain:
- Detailed breakdown of the work for that phase
- File/module structure decisions
- Any open questions resolved during planning
- Order of implementation within the phase

If a phase plan doesn't exist yet, create it before writing code.

### Ask Questions

If there is any ambiguity in instructions or specifications, **ask clarifying questions before proceeding**. It's better to confirm the intended behavior than to implement something incorrectly. Examples of things worth asking about:

- Edge cases not covered in the spec
- Unclear requirements or conflicting information
- Implementation approach when multiple valid options exist
- Whether a feature should match Obsidian's behavior exactly or diverge intentionally

### Implementation Guidance

- **Rust** is the core language. Use idiomatic Rust patterns.
- **clap** for CLI argument parsing.
- Prioritize correctness over performance initially, but keep caching (Phase 5) in mind for data structures.
- Match Obsidian's behavior where possible (link resolution, tag parsing, etc.) unless the spec explicitly says otherwise.
- All commands should support `--dry-run` for write operations.
- JSON is the default output format; support `--yaml` and `--toml` as alternatives.

### Testing

- **Unit tests**: Inline in source files with `#[cfg(test)]` modules
- **Integration tests**: In `vaultiel-rs/tests/`, using fixture vaults from `fixtures/`
- **Fixture vaults**: Shared across all language bindings in `fixtures/` at repo root
- Test edge cases: unicode, empty files, malformed frontmatter, deeply nested structures

Run tests:
```bash
cargo test --manifest-path vaultiel-rs/Cargo.toml           # All tests
cargo test --manifest-path vaultiel-rs/Cargo.toml parser    # Parser tests only
cargo test --manifest-path vaultiel-rs/Cargo.toml --test integration_test  # Integration tests
```
