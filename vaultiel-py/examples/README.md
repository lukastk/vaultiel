# Vaultiel Python Examples

Example scripts demonstrating the Vaultiel Python bindings.

## Prerequisites

Install the vaultiel package:

```bash
pip install vaultiel

# Or from source:
cd vaultiel-py
pip install maturin
maturin develop
```

## Examples

### basic_operations.py

Demonstrates fundamental vault operations:
- Creating notes with frontmatter
- Listing and filtering notes
- Reading content and frontmatter
- Resolving note names and aliases
- Parsing links, tags, headings, and tasks

```bash
python basic_operations.py
```

### link_graph.py

Demonstrates link graph analysis:
- Building the complete link graph
- Finding incoming/outgoing links
- Calculating link statistics
- Finding orphan notes
- Analyzing link contexts

```bash
python link_graph.py
```

### task_analysis.py

Demonstrates task extraction and analysis:
- Extracting tasks from notes
- Filtering by status, date, priority
- Finding overdue tasks
- Analyzing task hierarchy
- Task statistics

```bash
python task_analysis.py
```

### standalone_parsing.py

Demonstrates parsing without a vault:
- Parsing links from arbitrary content
- Extracting tags from strings
- Building tables of contents from headings
- Processing content from other sources

```bash
python standalone_parsing.py
```

## Running All Examples

```bash
# Make sure you're in the examples directory
cd vaultiel-py/examples

# Run all examples
for script in *.py; do
    echo "=== Running $script ==="
    python "$script"
    echo
done
```

## Common Patterns

### Error Handling

```python
from vaultiel import Vault

vault = Vault("/path/to/vault")

# Check before accessing
if vault.note_exists("note.md"):
    content = vault.get_content("note.md")

# Or use try/except
try:
    content = vault.get_content("maybe.md")
except RuntimeError as e:
    print(f"Error: {e}")
```

### Working with Frontmatter

```python
# Get as dict (most convenient)
fm = vault.get_frontmatter_dict("note.md")
if fm:
    title = fm.get("title", "Untitled")
    tags = fm.get("tags", [])

# Get as JSON string
import json
fm_json = vault.get_frontmatter("note.md")
if fm_json:
    fm = json.loads(fm_json)
```

### Batch Processing

```python
from vaultiel import Vault

vault = Vault("/path/to/vault")

# Process all notes
for note in vault.list_notes():
    links = vault.get_links(note)
    # ... process links

# Process matching notes
for note in vault.list_notes_matching("projects/*.md"):
    tasks = vault.get_tasks(note)
    # ... process tasks
```

### Building Link Statistics

```python
from collections import defaultdict

vault = Vault("/path/to/vault")

incoming_counts = defaultdict(int)
for note in vault.list_notes():
    for ref in vault.get_outgoing_links(note):
        incoming_counts[ref.from_note] += 1

most_linked = sorted(incoming_counts.items(), key=lambda x: -x[1])[:10]
```

### Finding Orphans

```python
orphans = []
for note in vault.list_notes():
    incoming = vault.get_incoming_links(note)
    if not incoming:
        orphans.append(note)
```

### Task Filtering

```python
from datetime import date

today = str(date.today())

# Get all tasks
all_tasks = []
for note in vault.list_notes():
    all_tasks.extend(vault.get_tasks(note))

# Filter
incomplete = [t for t in all_tasks if t.symbol == "[ ]"]
due_today = [t for t in incomplete if t.due == today]
high_priority = [t for t in incomplete if t.priority == "high"]
overdue = [t for t in incomplete if t.due and t.due < today]
```

## Integration Ideas

### Export to Pandas DataFrame

```python
import pandas as pd
from vaultiel import Vault

vault = Vault("/path/to/vault")

# Create DataFrame of notes
notes_data = []
for note in vault.list_notes():
    fm = vault.get_frontmatter_dict(note)
    notes_data.append({
        "path": note,
        "title": fm.get("title") if fm else None,
        "type": fm.get("type") if fm else None,
        "tags": vault.get_tags(note),
        "link_count": len(vault.get_links(note)),
    })

df = pd.DataFrame(notes_data)
```

### Export to NetworkX Graph

```python
import networkx as nx
from vaultiel import Vault

vault = Vault("/path/to/vault")

G = nx.DiGraph()
for note in vault.list_notes():
    G.add_node(note)
    for link in vault.get_links(note):
        try:
            target = vault.resolve_note(link.target)
            G.add_edge(note, target)
        except:
            pass  # Broken link

# Analyze
print(f"Nodes: {G.number_of_nodes()}")
print(f"Edges: {G.number_of_edges()}")
print(f"Most connected: {max(G.degree(), key=lambda x: x[1])}")
```

### Webhook/API Integration

```python
from vaultiel import parse_links, parse_content_tags

def process_markdown(content: str) -> dict:
    """Process markdown content from an API."""
    links = parse_links(content)
    tags = parse_content_tags(content)

    return {
        "links": [l.target for l in links],
        "tags": [t.name for t in tags],
        "has_embeds": any(l.embed for l in links),
    }
```

## Cleanup

Each example creates a temporary vault. They are automatically cleaned up on restart, but you can manually remove them:

```bash
rm -rf /tmp/vaultiel-*-demo
```
