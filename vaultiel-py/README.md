# Vaultiel Python Bindings

Python bindings for Vaultiel - a library for programmatically interacting with Obsidian-style vaults.

Built with [PyO3](https://pyo3.rs/) for high performance with full Obsidian compatibility.

## Installation

```bash
pip install vaultiel
```

## Quick Start

```python
from vaultiel import Vault

# Open a vault
vault = Vault("/path/to/your/vault")

# List all notes
notes = vault.list_notes()
print(f"Found {len(notes)} notes")

# Get note content
content = vault.get_content("my-note.md")
print(content)

# Get note body (without frontmatter)
body = vault.get_body("my-note.md")

# Get frontmatter as dict
frontmatter = vault.get_frontmatter_dict("my-note.md")
if frontmatter:
    print(f"Title: {frontmatter.get('title')}")
```

## Features

- **Fast**: Built on Rust for high performance
- **Full Obsidian compatibility**: Parses wikilinks, embeds, tags, block IDs, and tasks
- **Link graph**: Query incoming and outgoing links with context
- **Frontmatter**: Parse YAML frontmatter as Python dicts
- **Type hints**: Full type stub support for IDE completion

## API Reference

### Vault Class

The main entry point for vault operations.

```python
from vaultiel import Vault

vault = Vault("/path/to/vault")
```

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `root` | `str` | Vault root directory path |

#### Note Operations

```python
# List all notes
notes: list[str] = vault.list_notes()

# List notes matching glob pattern
proj_notes: list[str] = vault.list_notes_matching("proj/*.md")

# Check if note exists
exists: bool = vault.note_exists("my-note.md")

# Get full content (including frontmatter)
content: str = vault.get_content("my-note.md")

# Get body only (without frontmatter)
body: str = vault.get_body("my-note.md")

# Get frontmatter as JSON string
fm_json: str | None = vault.get_frontmatter("my-note.md")

# Get frontmatter as Python dict
fm_dict: dict | None = vault.get_frontmatter_dict("my-note.md")

# Create a note
vault.create_note("new-note.md", "---\ntitle: New Note\n---\n\nContent here.")

# Delete a note
vault.delete_note("old-note.md")

# Rename a note (no link propagation)
vault.rename_note("old-name.md", "new-name.md")

# Resolve name/alias to path
path: str = vault.resolve_note("My Note")  # or alias
```

#### Parsing

```python
from vaultiel import Link, Tag, Heading, BlockId, Task

# Get links from a note
links: list[Link] = vault.get_links("my-note.md")
for link in links:
    print(f"Link to {link.target} at line {link.line}")
    if link.alias:
        print(f"  Alias: {link.alias}")
    if link.heading:
        print(f"  Heading: #{link.heading}")
    if link.block_id:
        print(f"  Block: ^{link.block_id}")
    if link.embed:
        print("  (embedded)")

# Get tags from a note
tags: list[Tag] = vault.get_tags("my-note.md")
for tag in tags:
    print(f"#{tag.name} at line {tag.line}")

# Get headings from a note
headings: list[Heading] = vault.get_headings("my-note.md")
for h in headings:
    print(f"{'#' * h.level} {h.text} (slug: {h.slug})")

# Get block IDs from a note
blocks: list[BlockId] = vault.get_block_ids("my-note.md")
for block in blocks:
    print(f"^{block.id} ({block.block_type}) at line {block.line}")

# Get tasks from a note
tasks: list[Task] = vault.get_tasks("my-note.md")
for task in tasks:
    print(f"[{task.symbol}] {task.description}")
    if task.due:
        print(f"  Due: {task.due}")
    if task.scheduled:
        print(f"  Scheduled: {task.scheduled}")
    if task.priority:
        print(f"  Priority: {task.priority}")
    for tag in task.tags:
        print(f"  Tag: {tag}")
```

#### Link Graph

```python
from vaultiel import LinkRef

# Get incoming links (backlinks)
incoming: list[LinkRef] = vault.get_incoming_links("my-note.md")
for ref in incoming:
    print(f"Linked from {ref.from_note} at line {ref.line}")
    print(f"  Context: {ref.context}")  # body, frontmatter:key, task, etc.

# Get outgoing links
outgoing: list[LinkRef] = vault.get_outgoing_links("my-note.md")
```

#### Metadata

```python
from vaultiel import VaultielMetadata

# Initialize vaultiel metadata (UUID + timestamp)
metadata: VaultielMetadata | None = vault.init_metadata("my-note.md")
if metadata:
    print(f"ID: {metadata.id}")
    print(f"Created: {metadata.created}")

# Initialize with force (overwrite existing)
vault.init_metadata("my-note.md", force=True)

# Get existing metadata
metadata = vault.get_vaultiel_metadata("my-note.md")

# Find note by UUID
path: str | None = vault.find_by_id("550e8400-e29b-41d4-a716-446655440000")
```

### Standalone Parsing Functions

Parse content without a vault:

```python
from vaultiel import (
    parse_links,
    parse_content_tags,
    parse_content_headings,
    parse_content_block_ids,
)

content = """
# My Note

This links to [[Other Note]] and [[Another|with alias]].

Has #tags and #nested/tags too.

Important block ^my-block
"""

links = parse_links(content)
tags = parse_content_tags(content)
headings = parse_content_headings(content)
blocks = parse_content_block_ids(content)
```

### Data Classes

#### Link

```python
class Link:
    target: str              # Target path or name
    alias: str | None        # Display alias [[target|alias]]
    heading: str | None      # Heading reference [[note#heading]]
    block_id: str | None     # Block reference [[note#^block]]
    embed: bool              # True for ![[embeds]]
    line: int                # Line number (1-indexed)
```

#### Tag

```python
class Tag:
    name: str                # Tag name (without #)
    line: int                # Line number
```

#### Heading

```python
class Heading:
    text: str                # Heading text
    level: int               # 1-6
    line: int                # Line number
    slug: str                # URL slug (lowercase, hyphens)
```

#### BlockId

```python
class BlockId:
    id: str                  # Block ID (without ^)
    line: int                # Line number
    block_type: str          # paragraph, list-item, etc.
```

#### Task

```python
class Task:
    file: str                # Source file path
    line: int                # Line number
    raw: str                 # Raw task line
    symbol: str              # Task marker: [ ], [x], [>], etc.
    description: str         # Task text
    indent: int              # Indentation level
    scheduled: str | None    # Scheduled date (YYYY-MM-DD)
    due: str | None          # Due date
    done: str | None         # Completion date
    priority: str | None     # Priority level
    tags: list[str]          # Tags in task
    block_id: str | None     # Block ID on task
```

#### LinkRef

```python
class LinkRef:
    from_note: str           # Source note path
    line: int                # Line number
    context: str             # Where link appears (body, frontmatter:key, task)
    alias: str | None        # Link alias
    heading: str | None      # Heading reference
    block_id: str | None     # Block reference
    embed: bool              # True for embeds
```

#### VaultielMetadata

```python
class VaultielMetadata:
    id: str                  # UUID
    created: str             # ISO 8601 timestamp
```

## Examples

### Find Orphan Notes

```python
from vaultiel import Vault

vault = Vault("/path/to/vault")

notes = vault.list_notes()
orphans = []

for note in notes:
    incoming = vault.get_incoming_links(note)
    if not incoming:
        orphans.append(note)

print(f"Found {len(orphans)} orphan notes:")
for orphan in orphans:
    print(f"  - {orphan}")
```

### Extract All Tasks

```python
from vaultiel import Vault

vault = Vault("/path/to/vault")

all_tasks = []
for note in vault.list_notes():
    tasks = vault.get_tasks(note)
    all_tasks.extend(tasks)

# Filter incomplete tasks
incomplete = [t for t in all_tasks if t.symbol == "[ ]"]

# Filter by due date
import datetime
today = datetime.date.today().isoformat()
due_today = [t for t in incomplete if t.due == today]

print(f"Tasks due today: {len(due_today)}")
for task in due_today:
    print(f"  [{task.symbol}] {task.description}")
    print(f"    File: {task.file}:{task.line}")
```

### Build Link Graph

```python
from vaultiel import Vault
from collections import defaultdict

vault = Vault("/path/to/vault")

# Build adjacency list
graph = defaultdict(list)
for note in vault.list_notes():
    links = vault.get_links(note)
    for link in links:
        # Resolve link target to path
        try:
            target_path = vault.resolve_note(link.target)
            graph[note].append(target_path)
        except:
            pass  # Broken link

# Find most linked notes
incoming_counts = defaultdict(int)
for source, targets in graph.items():
    for target in targets:
        incoming_counts[target] += 1

top_10 = sorted(incoming_counts.items(), key=lambda x: -x[1])[:10]
print("Top 10 most linked notes:")
for note, count in top_10:
    print(f"  {count:3d} links: {note}")
```

### Process Frontmatter

```python
from vaultiel import Vault
import json

vault = Vault("/path/to/vault")

# Find all notes with a specific frontmatter key
project_notes = []
for note in vault.list_notes():
    fm = vault.get_frontmatter_dict(note)
    if fm and fm.get("type") == "project":
        project_notes.append({
            "path": note,
            "title": fm.get("title", note),
            "status": fm.get("status", "unknown"),
        })

print(json.dumps(project_notes, indent=2))
```

## Error Handling

Most methods raise `RuntimeError` on failure:

```python
from vaultiel import Vault

vault = Vault("/path/to/vault")

try:
    content = vault.get_content("nonexistent.md")
except RuntimeError as e:
    print(f"Error: {e}")

# Check before accessing
if vault.note_exists("maybe.md"):
    content = vault.get_content("maybe.md")
```

## Building from Source

Requires Rust (1.70+) and maturin:

```bash
# Install maturin
pip install maturin

# Build and install in development mode
cd vaultiel-py
maturin develop

# Build wheel
maturin build --release
```

## Comparison with CLI

The Python bindings provide a subset of CLI functionality focused on reading and querying:

| Feature | CLI | Python |
|---------|-----|--------|
| List notes | ✅ | ✅ |
| Read content | ✅ | ✅ |
| Parse links/tags/etc | ✅ | ✅ |
| Link graph | ✅ | ✅ |
| Create/delete/rename | ✅ | ✅ |
| Content modification | ✅ | - |
| Search | ✅ | - |
| Lint | ✅ | - |
| Cache | ✅ | - |
| Export | ✅ | - |

For advanced operations, use the CLI or contribute to the bindings!

## License

MIT
