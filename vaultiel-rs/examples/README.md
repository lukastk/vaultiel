# Vaultiel CLI Examples

Shell scripts demonstrating Vaultiel CLI usage patterns.

## Prerequisites

1. Install vaultiel:
   ```bash
   cargo install --path ..
   # or
   cargo install vaultiel
   ```

2. Ensure `jq` is installed for JSON processing:
   ```bash
   # macOS
   brew install jq

   # Ubuntu/Debian
   apt install jq
   ```

## Examples

### basic_operations.sh

Demonstrates fundamental note operations:
- Creating notes with frontmatter and content
- Listing and filtering notes
- Reading content and frontmatter
- Resolving note names and aliases
- Searching notes
- Modifying content

```bash
./basic_operations.sh
# or with custom vault:
VAULT=/path/to/vault ./basic_operations.sh
```

### link_graph.sh

Demonstrates link graph operations:
- Creating interconnected notes
- Getting outgoing links
- Getting incoming links (backlinks)
- Filtering links by context
- Finding orphan notes
- Checking for broken links
- Vault statistics

```bash
./link_graph.sh
```

### tasks.sh

Demonstrates task management:
- Creating notes with Obsidian Tasks format
- Extracting all tasks
- Filtering by status, date, priority
- Filtering by note or glob pattern
- Finding tasks that link to specific notes
- Hierarchical vs flat task views
- Formatting new tasks

```bash
./tasks.sh
```

### lint_and_health.sh

Demonstrates vault health checking:
- Running full lint checks
- Checking specific issue types
- Finding broken links
- Finding orphan notes
- Detecting duplicate block IDs
- Auto-fixing issues
- CI integration (GitHub Actions format)
- Exit codes for scripting

```bash
./lint_and_health.sh
```

### export_graph.sh

Demonstrates graph database export:
- Neo4j Cypher format (CREATE and MERGE)
- JSON-LD format (Linked Data)
- Including tags, headings, and frontmatter
- Custom base URIs for JSON-LD
- Piping output to other tools

```bash
./export_graph.sh
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `VAULT` | Path to vault directory | `/tmp/vaultiel-*-demo` |
| `OUTPUT_DIR` | Output directory for exports | `/tmp/vaultiel-exports` |

## Running All Examples

```bash
# Make scripts executable
chmod +x *.sh

# Run all examples
for script in *.sh; do
    echo "=== Running $script ==="
    ./$script
    echo
done
```

## Cleanup

Each script creates a temporary vault. To clean up:

```bash
rm -rf /tmp/vaultiel-*-demo /tmp/vaultiel-exports
```

## Common Patterns

### Pipe to jq for JSON processing

```bash
# Extract just note paths
vaultiel list | jq -r '.notes[].path'

# Count notes with a tag
vaultiel list --tag project | jq '.total'

# Get task descriptions
vaultiel get-tasks | jq '.[].description'
```

### Use in CI/CD

```bash
# Fail if broken links exist
vaultiel lint --fail-on broken-links

# Generate GitHub Actions annotations
vaultiel lint --format github
```

### Scripting with exit codes

```bash
# Check if note exists
if vaultiel resolve "My Note" > /dev/null 2>&1; then
    echo "Note exists"
else
    echo "Note not found"
fi
```

### Combine with other tools

```bash
# Export and load into Neo4j
vaultiel export-graph --format neo4j-cypher | cypher-shell

# Find notes modified today
vaultiel list --sort modified | jq -r '.notes[0:10][].path'

# Create daily note
vaultiel create "daily/$(date +%Y-%m-%d).md" --content "# $(date +%Y-%m-%d)"
```
