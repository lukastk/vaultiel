# Vaultiel

> A CLI and library for programmatically interacting with Obsidian-style vaults.
> Think of it as `jq` for markdown notes with YAML frontmatter and wikilinks.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

**Vaultiel** provides a programmatic interface to Obsidian vaults, enabling:

- **Note operations**: List, create, delete, rename (with automatic link propagation)
- **Content manipulation**: Get, set, append, prepend, replace by section/pattern/block
- **Frontmatter**: Parse and modify YAML frontmatter and inline attributes
- **Link graph**: Traverse incoming/outgoing links with rich context metadata
- **Embeds**: Track `![[embed]]` syntax for notes, images, PDFs, and media
- **Tags**: Extract and filter by hierarchical tags (`#tag/subtag`)
- **Block references**: Full support for `^block-id` syntax
- **Headings**: Extract headings and section content
- **Tasks**: Parse Obsidian Tasks plugin format with hierarchy support
- **Search**: Subsequence matching (Obsidian-style), fuzzy, exact, and regex modes
- **Vault health**: Lint for broken links, orphans, duplicate IDs, and more
- **Caching**: Optional indexing for fast operations on large vaults
- **Graph export**: Export to Neo4j Cypher or JSON-LD format

## Installation

### CLI (Rust)

```bash
# From crates.io (coming soon)
cargo install vaultiel

# From source
git clone https://github.com/lukas/vaultiel
cd vaultiel/vaultiel-rs
cargo install --path .
```

### Python

```bash
pip install vaultiel
```

### Node.js / TypeScript

```bash
npm install @vaultiel/node
```

## Quick Start

### CLI

```bash
# Set your default vault
echo '[vault]
default = "/path/to/your/vault"' > ~/.config/vaultiel.toml

# List all notes
vaultiel list

# Search for notes
vaultiel search "project"

# Get note content
vaultiel get-content "My Note.md"

# Get links from a note
vaultiel get-links "My Note.md"

# Find broken links
vaultiel lint --only broken-links

# Export vault graph
vaultiel export-graph --format neo4j-cypher > vault.cypher
```

### Python

```python
from vaultiel import Vault

vault = Vault("/path/to/vault")

# List notes
notes = vault.list_notes()

# Get content and links
content = vault.get_content("My Note.md")
links = vault.get_links("My Note.md")

# Get incoming links (backlinks)
backlinks = vault.get_incoming_links("My Note.md")

# Parse tasks
tasks = vault.get_tasks("My Note.md")
for task in tasks:
    print(f"[{task.symbol}] {task.description}")
```

### TypeScript / Node.js

```typescript
import { Vault } from '@vaultiel/node';

const vault = new Vault('/path/to/vault');

// List notes
const notes = vault.listNotes();

// Get content and links
const content = vault.getContent('My Note.md');
const links = vault.getLinks('My Note.md');

// Get incoming links (backlinks)
const backlinks = vault.getIncomingLinks('My Note.md');

// Parse tasks
const tasks = vault.getTasks('My Note.md');
tasks.forEach(task => {
    console.log(`[${task.symbol}] ${task.description}`);
});
```

## Configuration

Create `~/.config/vaultiel.toml`:

```toml
[vault]
default = "/path/to/your/vault"

[tasks]
# Obsidian Tasks plugin symbols (these are the defaults)
due = "📅"
scheduled = "⏳"
done = "✅"
priority_highest = "🔺"
priority_high = "⏫"
priority_medium = "🔼"
priority_low = "🔽"
priority_lowest = "⏬"

# Custom metadata fields
[tasks.custom_metadata]
time_estimate = "⏲️"

[cache]
enabled = true
location = "global"  # or "local" for vault-specific cache
```

## CLI Commands

### Note Operations

| Command | Description |
|---------|-------------|
| `list` | List notes with filtering (glob, tag, frontmatter, orphans) |
| `create` | Create a new note |
| `delete` | Delete a note (with optional link cleanup) |
| `rename` | Rename and propagate link changes |
| `search` | Search notes (subsequence, fuzzy, exact, regex) |
| `resolve` | Resolve note name/alias to path |

### Content Operations

| Command | Description |
|---------|-------------|
| `get-content` | Get note content |
| `set-content` | Set note content |
| `append-content` | Append to note |
| `prepend-content` | Prepend to note (after frontmatter) |
| `replace-content` | Replace by section, pattern, lines, or block |

### Frontmatter Operations

| Command | Description |
|---------|-------------|
| `get-frontmatter` | Get frontmatter (JSON/YAML/TOML) |
| `modify-frontmatter` | Set or modify frontmatter fields |
| `remove-frontmatter` | Remove a frontmatter field |
| `rename-frontmatter` | Rename a key across notes |

### Link & Graph Operations

| Command | Description |
|---------|-------------|
| `get-links` | Get incoming and outgoing links |
| `get-in-links` | Get incoming links (backlinks) |
| `get-out-links` | Get outgoing links |
| `get-embeds` | Get embedded content |
| `get-tags` | Get tags from note or vault |
| `get-blocks` | Get block IDs |
| `get-block-refs` | Get references to blocks |
| `get-headings` | Get headings |
| `get-section` | Extract section content |

### Task Operations

| Command | Description |
|---------|-------------|
| `get-tasks` | Extract tasks with filtering |
| `format-task` | Format a task string |

### Vault Health

| Command | Description |
|---------|-------------|
| `info` | Vault statistics |
| `lint` | Check for issues (broken links, orphans, etc.) |
| `find-orphans` | Find notes with no incoming links |
| `find-broken-links` | Find broken links |

### Cache & Export

| Command | Description |
|---------|-------------|
| `cache status` | Show cache information |
| `cache rebuild` | Rebuild the cache |
| `cache clear` | Clear the cache |
| `export-graph` | Export to Neo4j Cypher or JSON-LD |

### Metadata

| Command | Description |
|---------|-------------|
| `init-metadata` | Initialize vaultiel metadata (UUID) |
| `get-metadata` | Get vaultiel metadata |
| `get-by-id` | Find note by UUID |

## Project Structure

```
vaultiel/
├── vaultiel-rs/     # Rust core library + CLI
├── vaultiel-py/     # Python bindings (PyO3)
├── vaultiel-node/   # Node.js bindings (napi-rs)
├── fixtures/        # Test vaults for all bindings
├── phase_plans/     # Implementation plans
├── PROJECT_SPEC.md  # Full specification
└── PROJECT_PLAN.md  # Implementation roadmap
```

## Features by Package

| Feature | CLI | Python | Node.js |
|---------|-----|--------|---------|
| Note CRUD | ✅ | ✅ | ✅ |
| Content operations | ✅ | ✅ | ✅ |
| Frontmatter | ✅ | ✅ | ✅ |
| Link graph | ✅ | ✅ | ✅ |
| Tags | ✅ | ✅ | ✅ |
| Block IDs | ✅ | ✅ | ✅ |
| Headings | ✅ | ✅ | ✅ |
| Tasks | ✅ | ✅ | ✅ |
| Search | ✅ | - | - |
| Lint | ✅ | - | - |
| Cache | ✅ | - | - |
| Graph export | ✅ | - | - |
| Metadata (UUID) | ✅ | ✅ | ✅ |

## Exit Codes (CLI)

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Note not found |
| 3 | Note already exists |
| 4 | Ambiguous resolution |
| 5 | Invalid frontmatter |
| 10 | Lint issues found |

## Examples

### Finding Broken Links

```bash
# Find all broken links
vaultiel lint --only broken-links

# Find broken links in specific files
vaultiel lint --only broken-links --glob "projects/*.md"

# CI integration
vaultiel lint --fail-on broken-links --format github
```

### Task Extraction

```bash
# All incomplete tasks
vaultiel get-tasks --symbol "[ ]"

# Tasks due this week
vaultiel get-tasks --due-before $(date -v+7d +%Y-%m-%d)

# Tasks in project files with high priority
vaultiel get-tasks --glob "projects/*.md" --priority high
```

### Link Graph Analysis

```bash
# Get backlinks to a note
vaultiel get-in-links "Projects/My Project.md"

# Find orphan notes (no incoming links)
vaultiel find-orphans --exclude "templates/*"

# Export for graph visualization
vaultiel export-graph --format json-ld --pretty > vault.jsonld
```

### Content Manipulation

```bash
# Replace a section
vaultiel replace-content "My Note.md" \
  --section "## Status" \
  --content "## Status\n\nCompleted!"

# Append to a daily note
echo "- Meeting notes..." | vaultiel append-content "Daily/2026-02-03.md"

# Modify frontmatter
vaultiel modify-frontmatter "My Note.md" -k status --value complete
```

## Documentation

- [CLI Reference](./vaultiel-rs/README.md)
- [Python API](./vaultiel-py/README.md)
- [Node.js API](./vaultiel-node/README.md)
- [Full Specification](./PROJECT_SPEC.md)

## Contributing

Contributions are welcome! Please see the [PROJECT_PLAN.md](./PROJECT_PLAN.md) for current implementation status and planned features.

## License

MIT
