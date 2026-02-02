# Vaultiel - Project Plan

This document tracks implementation progress for Vaultiel. See [PROJECT_SPEC.md](./PROJECT_SPEC.md) for full feature specifications.

---

## Phase 1: Core CLI (MVP)

Foundation for all other features. Get basic note operations working.

### Project Setup
- [x] Initialize Rust project with Cargo
- [x] Set up clap for CLI argument parsing
- [x] Implement config loading (`~/.config/vaultiel.toml`)
- [x] Set up error handling and exit codes
- [x] Implement global flags (`--vault`, `--json`, `--yaml`, `--toml`, `--quiet`, `--verbose`)

### Parsing
- [x] Markdown file reading with frontmatter extraction
- [x] YAML frontmatter parsing
- [x] Wikilink parsing (`[[note]]`, `[[note|alias]]`, `[[note#heading]]`, `[[note#^block]]`)
- [x] Embed parsing (`![[file]]`, `![[note]]`, `![[image.png|400]]`)
- [x] Inline attribute parsing (`[key::value]`)
- [x] Tag parsing (`#tag`, `#tag/subtag`)
- [x] Block ID parsing (`^block-id`)
- [x] Heading parsing (levels, text, slug generation)
- [x] Code block detection (to skip parsing inside them)

### Commands: Note Operations
- [x] `list` - List notes with filtering (glob, tag, frontmatter, orphans, sort, limit)
- [x] `create` - Create new note with frontmatter/content
- [x] `delete` - Delete note (with `--no-propagate`, `--remove-links`, `--force`)
- [x] `search` - Subsequence matching (Obsidian-style)
- [x] `search` - Additional modes: fuzzy, exact, regex
- [x] `search` - Content search (`--content` flag)
- [x] `resolve` - Alias/name to path resolution

### Commands: Content Operations
- [x] `get-content` - Get note content (with frontmatter options)
- [x] `set-content` - Set note content (stdin, `--content`, `--file`)
- [x] `append-content` - Append to note
- [x] `prepend-content` - Prepend to note (after frontmatter)
- [x] `replace-content` - Replace by section heading
- [x] `replace-content` - Replace by regex pattern
- [x] `replace-content` - Replace by line range
- [x] `replace-content` - Replace by block ID

### Commands: Frontmatter Operations
- [x] `get-frontmatter` - Get frontmatter (json/yaml/toml output)
- [x] `get-frontmatter` - Include inline attributes
- [x] `get-frontmatter` - Get specific key
- [x] `modify-frontmatter` - Set value
- [x] `modify-frontmatter` - Add to list (`-v:add`)
- [x] `modify-frontmatter` - Remove from list (`-v:remove`)
- [x] `remove-frontmatter` - Remove a key

---

## Phase 2: Links, Tags & Blocks

Build the graph structure and traversal capabilities.

### Link Graph
- [x] Build in-memory link graph from vault
- [x] Track link context (body, frontmatter key, inline attribute, task)
- [x] Track link metadata (alias, heading, block_id, embed flag)
- [x] Incoming link resolution (which notes link to this note)

### Commands: Link Operations
- [x] `get-links` - Get incoming and outgoing links
- [x] `get-in-links` - Incoming links only
- [x] `get-out-links` - Outgoing links only
- [x] `get-links` - Context filtering (`--context`)
- [x] `get-links` - Embed filtering (`--embeds-only`, `--no-embeds`, `--media-only`)
- [x] `get-embeds` - Shorthand for embed listing

### Commands: Tag Operations
- [x] `get-tags` - Tags from specific note (with line/context)
- [x] `get-tags` - All tags in vault
- [x] `get-tags` - With counts (`--with-counts`)
- [x] `get-tags` - Nested hierarchy output (`--nested`)
- [ ] `search` - Tag filtering (`--tag`, `--tag-any`, `--no-tag`)

### Commands: Block Operations
- [x] `get-blocks` - List block IDs in a note
- [x] `get-block-refs` - Find references to a note's blocks

### Commands: Heading Operations
- [x] `get-headings` - List headings (flat)
- [x] `get-headings` - Nested hierarchy (`--nested`)
- [x] `get-headings` - Level filtering (`--min-level`, `--max-level`)
- [x] `get-section` - Extract section content

### Commands: Rename with Propagation
- [x] `rename` - Rename note file
- [x] `rename` - Update all incoming links throughout vault
- [x] `rename` - Handle heading/block references in links
- [x] `rename` - `--no-propagate` mode (just mv)
- [x] `rename` - `--dry-run` mode
- [ ] `rename-frontmatter` - Rename key across notes

---

## Phase 3: Tasks

Full Obsidian Tasks plugin compatibility.

### Task Parsing
- [x] Parse task markers (`- [ ]`, `- [x]`, `- [>]`, etc.)
- [x] Parse Obsidian Tasks metadata (due, scheduled, done dates)
- [x] Parse priority markers
- [x] Parse custom metadata fields (from config)
- [x] Extract links within tasks
- [x] Extract tags within tasks
- [x] Extract block IDs on tasks

### Task Hierarchy
- [x] Track indentation levels
- [x] Build parent/child relationships
- [x] Support nested output (tree structure)
- [x] Support flat output with parent references

### Commands
- [x] `get-tasks` - Basic task extraction
- [x] `get-tasks` - Filter by note (`--note`)
- [x] `get-tasks` - Filter by glob (`--glob`)
- [x] `get-tasks` - Filter by symbol (`--symbol`)
- [x] `get-tasks` - Filter by due date (`--due-before`, `--due-after`, `--due-on`)
- [x] `get-tasks` - Filter by scheduled date
- [x] `get-tasks` - Filter by done date
- [x] `get-tasks` - Filter by priority (`--priority`)
- [x] `get-tasks` - Filter by description text (`--contains`)
- [x] `get-tasks` - Filter by custom metadata (`--has`)
- [x] `get-tasks` - Filter by linked note (`--links-to`)
- [x] `get-tasks` - Filter by tag (`--tag`)
- [x] `get-tasks` - Filter by block ref (`--has-block-ref`, `--block-ref`)
- [x] `get-tasks` - Flat output (`--flat`)
- [x] `format-task` - Format task string for Obsidian
- [x] `format-task` - Relative date support (today, tomorrow, +3d)

---

## Phase 4: Vault Health & Info

Linting and diagnostics.

### Statistics
- [ ] `info` - Basic vault stats (note count, link count, etc.)
- [ ] `info` - Detailed stats (`--detailed`)
- [ ] `info` - Notes by folder breakdown
- [ ] `info` - Top tags, top linked notes

### Issue Detection
- [ ] Broken link detection
- [ ] Broken embed detection
- [ ] Broken heading link detection (`[[Note#Heading]]`)
- [ ] Broken block ref detection (`[[Note#^block]]`)
- [ ] Orphan note detection
- [ ] Duplicate alias detection
- [ ] Duplicate block ID detection (within a note)
- [ ] Empty note detection
- [ ] Missing frontmatter detection
- [ ] Invalid frontmatter detection (YAML parse errors)

### Commands
- [ ] `lint` - Run all checks
- [ ] `lint` - Filter checks (`--only`, `--ignore`)
- [ ] `lint` - Scope to notes (`--glob`)
- [ ] `lint` - Auto-fix (`--fix`)
- [ ] `lint` - CI mode (`--fail-on`)
- [ ] `lint` - GitHub Actions output (`--format github`)
- [ ] `find-orphans` - Shorthand with `--exclude` support
- [ ] `find-broken-links` - Shorthand with `--note` support

---

## Phase 5: Caching

Performance optimization for large vaults.

### Cache Infrastructure
- [ ] Design cache file format (JSON or binary)
- [ ] Implement cache location logic (global vs local)
- [ ] Atomic cache writes (temp file + rename)
- [ ] Lock file for concurrent access
- [ ] Cache versioning (invalidate on vaultiel upgrade)

### Indexing
- [ ] Index note paths and mtimes
- [ ] Index parsed frontmatter
- [ ] Index outgoing links with context
- [ ] Index tags
- [ ] Index block IDs
- [ ] Index tasks
- [ ] Index headings

### Incremental Updates
- [ ] Detect changed files via mtime
- [ ] Re-index only changed files
- [ ] Update incoming link references on change
- [ ] Handle file deletion
- [ ] Handle file creation

### Commands
- [ ] `cache status` - Show cache info
- [ ] `cache rebuild` - Force full rebuild
- [ ] `cache rebuild --verbose` - Progress output
- [ ] `cache clear` - Remove cache

### Integration
- [ ] Auto-enable caching for vaults > threshold
- [ ] `--trust-cache` flag for maximum speed
- [ ] Transparent cache usage in all read commands

---

## Phase 6: Metadata & IDs

Stable identifiers for external integrations.

### Vaultiel Metadata Field
- [ ] Define `vaultiel` frontmatter schema
- [ ] UUID generation
- [ ] Creation timestamp tracking

### Commands
- [ ] `init-metadata` - Add vaultiel field to a note
- [ ] `init-metadata --glob` - Batch initialization
- [ ] `get-by-id` - Find note by UUID

### Integration
- [ ] Preserve vaultiel field during operations
- [ ] `--include-vaultiel-field` flag on get-content
- [ ] ID stability across renames

---

## Phase 7: Bindings

Language bindings for integration.

### TypeScript Bindings
- [ ] Set up napi-rs or wasm-bindgen
- [ ] Expose core vault operations
- [ ] Expose link graph queries
- [ ] Expose task operations
- [ ] NPM package setup
- [ ] TypeScript type definitions
- [ ] Documentation and examples

### Python Bindings
- [ ] Set up PyO3
- [ ] Expose core vault operations
- [ ] Expose link graph queries
- [ ] Expose task operations
- [ ] PyPI package setup
- [ ] Type stubs (`.pyi` files)
- [ ] Documentation and examples

---

## Phase 8: Advanced Features

Future enhancements (lower priority).

### Graph Database Export
- [ ] `export-graph` - Neo4j Cypher format
- [ ] `export-graph` - JSON-LD format
- [ ] Incremental export (diff-based)

### Templating
- [ ] Template file format design
- [ ] Template discovery (`templates/` folder)
- [ ] Variable interpolation
- [ ] JavaScript execution (via TypeScript bindings)
- [ ] `create --template` implementation

### Sub-vault Support
- [ ] `mount` command design
- [ ] Namespace isolation
- [ ] Cross-vault link resolution
- [ ] Conflict handling

---

## Notes

### Priority Order
Phases 1-4 are the core product. Phase 5 (caching) is important for large vaults but can be deferred. Phases 6-8 are enhancements.

### Testing Strategy
- Unit tests for all parsers
- Integration tests for commands (use fixture vaults)
- Property-based tests for edge cases (unicode, special chars, deeply nested structures)

### Documentation
- README with quick start
- Man pages for CLI
- API docs for library crate
- Examples folder with common use cases
