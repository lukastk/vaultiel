# Vaultiel Python Bindings

Python bindings for Vaultiel - a library for programmatically interacting with Obsidian-style vaults.

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

# Get links from a note
links = vault.get_links("my-note.md")
for link in links:
    print(f"Link to {link.target} at line {link.line}")

# Get tags
tags = vault.get_tags("my-note.md")
for tag in tags:
    print(f"Tag: #{tag.name}")

# Get tasks
tasks = vault.get_tasks("my-note.md")
for task in tasks:
    print(f"[{task.symbol}] {task.description}")
```

## Features

- **Fast**: Built on Rust for high performance
- **Full Obsidian compatibility**: Parses wikilinks, embeds, tags, block IDs, and tasks
- **Link graph**: Query incoming and outgoing links
- **Frontmatter**: Parse and manipulate YAML frontmatter
- **Type hints**: Full type stub support for IDE completion

## API Reference

### Vault Class

```python
class Vault:
    def __init__(self, path: str) -> None: ...

    # Properties
    @property
    def root(self) -> str: ...

    # Note operations
    def list_notes(self) -> list[str]: ...
    def list_notes_matching(self, pattern: str) -> list[str]: ...
    def note_exists(self, path: str) -> bool: ...
    def get_content(self, path: str) -> str: ...
    def get_body(self, path: str) -> str: ...
    def get_frontmatter(self, path: str) -> Optional[str]: ...
    def get_frontmatter_dict(self, path: str) -> Optional[dict]: ...
    def create_note(self, path: str, content: str) -> None: ...
    def delete_note(self, path: str) -> None: ...
    def rename_note(self, from_path: str, to_path: str) -> None: ...
    def resolve_note(self, query: str) -> str: ...

    # Parsing
    def get_links(self, path: str) -> list[Link]: ...
    def get_tags(self, path: str) -> list[Tag]: ...
    def get_headings(self, path: str) -> list[Heading]: ...
    def get_block_ids(self, path: str) -> list[BlockId]: ...
    def get_tasks(self, path: str) -> list[Task]: ...

    # Link graph
    def get_incoming_links(self, path: str) -> list[LinkRef]: ...
    def get_outgoing_links(self, path: str) -> list[LinkRef]: ...

    # Metadata
    def init_metadata(self, path: str, force: bool = False) -> Optional[VaultielMetadata]: ...
    def get_vaultiel_metadata(self, path: str) -> Optional[VaultielMetadata]: ...
    def find_by_id(self, id: str) -> Optional[str]: ...
```

### Standalone Functions

```python
# Parse content without a vault
def parse_links(content: str) -> list[Link]: ...
def parse_content_tags(content: str) -> list[Tag]: ...
def parse_content_headings(content: str) -> list[Heading]: ...
def parse_content_block_ids(content: str) -> list[BlockId]: ...
```

## Building from Source

Requires Rust and maturin:

```bash
pip install maturin
cd vaultiel-py
maturin develop
```

## License

MIT
