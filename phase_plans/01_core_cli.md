# Phase 1: Core CLI (MVP)

## Overview

This phase establishes the foundation for Vaultiel: project structure, configuration, parsing infrastructure, and basic note/content/frontmatter operations. By the end of this phase, users can list notes, create/delete notes, search, and manipulate content and frontmatter.

## Architecture Decisions

### Crate Structure

Single crate with clear separation between CLI and library:

```
vaultiel-rs/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports (for future bindings)
│   ├── cli/                 # CLI command implementations
│   │   ├── mod.rs
│   │   ├── args.rs          # Clap argument definitions
│   │   ├── output.rs        # JSON/YAML/TOML output formatting
│   │   ├── list.rs
│   │   ├── create.rs
│   │   ├── delete.rs
│   │   ├── search.rs
│   │   ├── resolve.rs
│   │   ├── content.rs       # get/set/append/prepend/replace-content
│   │   └── frontmatter.rs   # get/modify/remove-frontmatter
│   ├── config.rs            # Config file loading
│   ├── error.rs             # Error types and exit codes
│   ├── vault.rs             # Vault struct (directory + config)
│   ├── note.rs              # Note struct (path + parsed content)
│   ├── parser/              # All parsing logic
│   │   ├── mod.rs
│   │   ├── frontmatter.rs   # YAML frontmatter
│   │   ├── wikilink.rs      # [[link]] and [[link|alias]]
│   │   ├── embed.rs         # ![[embed]]
│   │   ├── tag.rs           # #tag and #tag/subtag
│   │   ├── block_id.rs      # ^block-id
│   │   ├── heading.rs       # # Heading parsing
│   │   ├── inline_attr.rs   # [key::value]
│   │   └── code_block.rs    # Detect code blocks (to skip parsing inside)
│   ├── search/              # Search algorithms
│   │   ├── mod.rs
│   │   ├── subsequence.rs   # Obsidian-style subsequence matching
│   │   ├── fuzzy.rs         # Fuzzy matching
│   │   ├── exact.rs         # Exact matching
│   │   └── regex.rs         # Regex matching
│   └── types.rs             # Shared types (Link, Tag, BlockId, etc.)
└── tests/
    ├── fixtures/            # Test vault fixtures
    └── integration/         # Integration tests
```

### Key Design Decisions

1. **Library-first**: Core logic lives in `lib.rs` and modules. CLI is a thin wrapper. This prepares for Phase 7 bindings.

2. **Lazy parsing**: Notes are parsed on-demand, not all at once. A `Note` struct holds raw content; parsing methods extract specific elements.

3. **Immutable operations**: Parse operations return new data; modification operations return new content strings. The caller decides whether to write to disk.

4. **Vault as context**: The `Vault` struct holds the root path and config. Commands receive a `&Vault` reference.

## Dependencies

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
toml = "0.8"
glob = "0.3"
regex = "1"
thiserror = "1"
dirs = "5"                    # For ~/.config path
chrono = { version = "0.4", features = ["serde"] }
unicode-normalization = "0.1" # For slug generation

[dev-dependencies]
tempfile = "3"
pretty_assertions = "1"
```

## Implementation Order

### Step 1: Project Setup
1. `cargo init`
2. Add dependencies to `Cargo.toml`
3. Create module structure (empty files)
4. Implement `error.rs` with error types and exit codes

### Step 2: Configuration
1. Define `Config` struct in `config.rs`
2. Implement config file loading from `~/.config/vaultiel.toml`
3. Handle missing config (use defaults)
4. Implement vault path resolution (CLI flag > config > current dir)

### Step 3: Core Types
1. Define `Vault` struct (root path + config)
2. Define `Note` struct (relative path + content)
3. Define shared types in `types.rs`:
   - `Link` (target, alias, heading, block_id, embed flag, line number)
   - `Tag` (name, line number)
   - `BlockId` (id, line number)
   - `Heading` (text, level, line number, slug)
   - `InlineAttr` (key, value, line number)

### Step 4: Parsers (in order of dependency)
1. **Code block detection** - needed to skip parsing inside code blocks
2. **Frontmatter parser** - extracts YAML between `---` delimiters
3. **Heading parser** - finds `#` headings, generates slugs
4. **Wikilink parser** - `[[target]]`, `[[target|alias]]`, `[[target#heading]]`, `[[target#^block]]`
5. **Embed parser** - `![[target]]` (reuses wikilink parser with embed flag)
6. **Tag parser** - `#tag`, `#tag/subtag` (but not inside code or links)
7. **Block ID parser** - `^block-id` at end of lines
8. **Inline attribute parser** - `[key::value]`

### Step 5: CLI Framework
1. Set up clap with derive macros in `cli/args.rs`
2. Define global flags (`--vault`, `--json`, `--yaml`, `--toml`, `-q`, `-v`)
3. Implement output formatting in `cli/output.rs`
4. Wire up main.rs to dispatch commands

### Step 6: Commands (in order)
1. **`list`** - Simplest command; validates the full pipeline works
2. **`get-content`** - Read a note's content
3. **`get-frontmatter`** - Parse and return frontmatter
4. **`create`** - Create a new note
5. **`delete`** - Delete a note (no propagation yet - that's Phase 2)
6. **`set-content`** - Overwrite note content
7. **`append-content`** - Append to note
8. **`prepend-content`** - Prepend after frontmatter
9. **`replace-content`** - Section/pattern/lines/block replacement
10. **`modify-frontmatter`** - Modify frontmatter fields
11. **`remove-frontmatter`** - Remove frontmatter fields
12. **`search`** - Implement search algorithms
13. **`resolve`** - Alias/name resolution

## Open Questions (Resolved)

1. **How to handle note paths?**
   - Decision: Paths are always relative to vault root, without `.md` extension in user-facing commands. Internally, we normalize to include `.md`.

2. **Should `list` iterate the filesystem or build an index?**
   - Decision: Iterate filesystem for now. Caching comes in Phase 5.

3. **How to handle concurrent writes?**
   - Decision: No locking in Phase 1. Single-user CLI assumption. Caching phase will add lock files.

4. **Frontmatter with wikilinks?**
   - Decision: Wikilinks in frontmatter values are stored as strings. They're parsed when needed (e.g., for link graph in Phase 2).

## Testing Strategy

### Unit Tests
- Each parser gets comprehensive unit tests
- Test edge cases: empty input, malformed input, unicode, nested structures
- Test code block skipping (links inside code blocks should not be parsed)

### Integration Tests
Create fixture vaults in `tests/fixtures/`:
- `minimal/` - Single note for basic tests
- `links/` - Notes with various link types
- `frontmatter/` - Notes with various frontmatter structures
- `unicode/` - Notes with unicode in names and content

### Test Commands
```bash
cargo test                    # Run all tests
cargo test parser             # Run parser tests only
cargo test --test integration # Run integration tests
```

## Success Criteria

Phase 1 is complete when:
- [x] All commands from Phase 1 of PROJECT_PLAN.md are implemented
- [x] All parsers handle edge cases correctly
- [x] Unit test coverage for all parsers (110 tests passing)
- [ ] Integration tests for all commands (TODO)
- [x] `--dry-run` works for all write operations
- [x] Output formats work (`--json`, `--yaml`, `--toml`)
- [x] Exit codes are correct per spec
- [x] Can be installed via `cargo install --path .`

## Implementation Progress

### Completed
- Project structure and module organization
- Error types with exit codes (error.rs)
- Configuration loading (config.rs)
- Core types: Link, Tag, BlockId, Heading, InlineAttr (types.rs)
- Vault and Note structs (vault.rs, note.rs)
- All parsers with unit tests:
  - Code block detection (handles fenced and inline code)
  - Frontmatter parsing (YAML between `---` delimiters)
  - Wikilink and embed parsing
  - Tag parsing (`#tag`, `#tag/subtag`)
  - Block ID parsing (`^block-id`)
  - Heading parsing with slug generation
  - Inline attribute parsing (`[key::value]`)
- CLI with all commands:
  - list, create, delete
  - search, resolve
  - get-content, set-content, append-content, prepend-content, replace-content
  - get-frontmatter, modify-frontmatter, remove-frontmatter
- Output formatting (JSON, YAML, TOML)
- Dry-run support for write operations
- Proper exit codes

### Remaining
- Integration tests with fixture vaults
- Edge case testing for CLI commands
