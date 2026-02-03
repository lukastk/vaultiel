# Vaultiel CLI

The Rust core library and command-line interface for Vaultiel.

## Installation

### From Source

```bash
git clone https://github.com/lukas/vaultiel
cd vaultiel/vaultiel-rs
cargo install --path .
```

### From Crates.io

```bash
cargo install vaultiel
```

## Configuration

Create `~/.config/vaultiel.toml`:

```toml
[vault]
# Default vault path (supports ~ expansion)
default = "~/Documents/Obsidian/MyVault"

[tasks]
# Obsidian Tasks plugin symbols (defaults shown)
due = "📅"
scheduled = "⏳"
done = "✅"
priority_highest = "🔺"
priority_high = "⏫"
priority_medium = "🔼"
priority_low = "🔽"
priority_lowest = "⏬"

# Custom metadata fields that appear before standard task metadata
[tasks.custom_metadata]
time_estimate = "⏲️"
# Add more as: key = "emoji"

[cache]
enabled = true                # Enable caching (default: true for vaults >500 notes)
location = "global"           # "global" (~/.cache) or "local" (.vaultiel/)
```

## Global Flags

All commands support these global flags:

| Flag | Description |
|------|-------------|
| `--vault PATH` | Override default vault for this command |
| `--json` | Output as JSON (default) |
| `--yaml` | Output as YAML |
| `--toml` | Output as TOML |
| `-q, --quiet` | Suppress non-essential output |
| `--no-color` | Disable colored output |

## Commands

### Note Operations

#### `vaultiel list`

List notes in the vault with filtering.

```bash
vaultiel list                                # All notes
vaultiel list --glob "proj/*.md"             # Notes matching glob
vaultiel list --tag "#rust"                  # Notes with tag
vaultiel list --frontmatter notetype=proj    # Notes with frontmatter value
vaultiel list --has-links                    # Notes with outgoing links
vaultiel list --orphans                      # Notes with no incoming links
vaultiel list --sort modified --reverse      # Sort by modified, newest first
vaultiel list --limit 20                     # Limit results
```

**Flags:**
- `--glob PATTERN` — Filter by glob pattern
- `--tag TAG` — Filter by tag (repeatable, AND logic)
- `--frontmatter KEY=VALUE` — Filter by frontmatter field
- `--has-links` — Only notes with outgoing links
- `--has-backlinks` — Only notes with incoming links
- `--orphans` — Only notes with no incoming links
- `--sort FIELD` — Sort by: `path`, `modified`, `created`, `name`
- `--reverse` — Reverse sort order
- `--limit N` — Limit results

---

#### `vaultiel create NOTE_PATH`

Create a new note.

```bash
vaultiel create "proj/My Project.md" --frontmatter '{"status": "active"}'
vaultiel create "daily/2026-02-03.md" --content "# Today\n\nNotes here."
vaultiel create "note.md" --dry-run          # Preview without creating
```

**Flags:**
- `--frontmatter JSON` — Initial frontmatter
- `--content TEXT` — Initial content
- `--open` — Open in Obsidian after creation
- `--dry-run` — Preview what would be created

---

#### `vaultiel delete NOTE_PATH`

Delete a note with optional link cleanup.

```bash
vaultiel delete "proj/Old.md"
vaultiel delete "proj/Old.md" --remove-links  # Remove links from other notes
vaultiel delete "proj/Old.md" --no-propagate  # Just delete, don't check links
vaultiel delete "proj/Old.md" --force         # Skip confirmation
vaultiel delete "proj/Old.md" --dry-run       # Preview what would happen
```

**Flags:**
- `--remove-links` — Remove all links to this note from other files
- `--no-propagate` — Just delete the file, don't check for broken links
- `--force` — Skip confirmation
- `--dry-run` — Preview changes

---

#### `vaultiel rename FROM TO`

Rename a note and update all links.

```bash
vaultiel rename "proj/Old.md" "proj/New.md"
vaultiel rename "proj/Old.md" "proj/New.md" --no-propagate  # Just mv
vaultiel rename "proj/Old.md" "proj/New.md" --dry-run       # Preview
```

**Flags:**
- `--no-propagate` — Don't update links in other notes
- `--dry-run` — Preview changes

---

#### `vaultiel search QUERY`

Search notes using various algorithms.

```bash
vaultiel search "vaultiel"                   # Subsequence match (default)
vaultiel search "vaultiel" --mode fuzzy      # Fuzzy matching
vaultiel search "vaultiel" --mode exact      # Exact match
vaultiel search "error.*handling" --mode regex
vaultiel search "vaultiel" --content         # Search content, not just names
vaultiel search "" --tag "#rust"             # Find notes by tag
vaultiel search "" --tag "#rust" --tag-any "#python"  # OR logic
vaultiel search "" --no-tag "#archive"       # Exclude tags
```

**Flags:**
- `--limit N` — Number of results (default: 1)
- `--mode MODE` — `subsequence` (default), `fuzzy`, `exact`, `regex`
- `--content` — Search note content
- `--tag TAG` — Filter by tag (AND logic)
- `--tag-any TAG` — Filter by tag (OR logic)
- `--no-tag TAG` — Exclude notes with tag
- `--frontmatter KEY=VALUE` — Filter by frontmatter

---

#### `vaultiel resolve NOTE_NAME`

Resolve a note name or alias to its file path.

```bash
vaultiel resolve "Vaultiel"            # Find by note name
vaultiel resolve "vault-cli"           # Find by alias
vaultiel resolve "proj/Vaultiel"       # With folder prefix
vaultiel resolve "Daily" --all         # Return all matches
```

**Flags:**
- `--all` — Return all matches (for ambiguous queries)
- `--strict` — Only match exact paths

---

### Content Operations

#### `vaultiel get-content NOTE_PATH`

Get note content.

```bash
vaultiel get-content "note.md"
vaultiel get-content "note.md" --include-frontmatter
vaultiel get-content "note.md" --include-frontmatter --include-vaultiel-field
```

**Flags:**
- `--include-frontmatter` — Include YAML frontmatter
- `--include-vaultiel-field` — Include vaultiel metadata field (excluded by default)

---

#### `vaultiel set-content NOTE_PATH`

Set note content.

```bash
echo "New content" | vaultiel set-content "note.md"
vaultiel set-content "note.md" --content "New content"
vaultiel set-content "note.md" --file content.md
vaultiel set-content "note.md" --content "Body only" --below-frontmatter
vaultiel set-content "note.md" --frontmatter-only --content "---\ntitle: New\n---"
```

**Flags:**
- `--content TEXT` — Content to set
- `--file PATH` — Read content from file
- `--below-frontmatter` — Only replace body, preserve frontmatter
- `--frontmatter-only` — Only replace frontmatter
- `--dry-run` — Preview changes

---

#### `vaultiel append-content NOTE_PATH`

Append content to a note.

```bash
echo "New paragraph" | vaultiel append-content "note.md"
vaultiel append-content "note.md" --content "\n## New Section"
vaultiel append-content "note.md" --file additions.md
```

**Flags:**
- `--content TEXT` — Content to append
- `--file PATH` — Read content from file
- `--dry-run` — Preview changes

---

#### `vaultiel prepend-content NOTE_PATH`

Prepend content (after frontmatter).

```bash
vaultiel prepend-content "note.md" --content "## Notice\n\nImportant!\n\n"
```

**Flags:**
- `--content TEXT` — Content to prepend
- `--file PATH` — Read content from file
- `--dry-run` — Preview changes

---

#### `vaultiel replace-content NOTE_PATH`

Replace content by section, pattern, lines, or block.

```bash
# Replace a section (heading to next same-level heading)
vaultiel replace-content "note.md" --section "## Status" --content "## Status\n\nDone!"

# Replace regex pattern (first match)
vaultiel replace-content "note.md" --pattern "TODO:.*" --content "DONE"

# Replace all matches
vaultiel replace-content "note.md" --pattern-all "old-word" --content "new-word"

# Replace line range
vaultiel replace-content "note.md" --lines 10-15 --content "Replaced lines"

# Replace by block ID
vaultiel replace-content "note.md" --block "my-block" --content "New block ^my-block"
```

**Flags:**
- `--section HEADING` — Replace section under heading
- `--pattern REGEX` — Replace first regex match
- `--pattern-all REGEX` — Replace all regex matches
- `--lines RANGE` — Replace line range (e.g., `10-15`, `10-`, `-15`)
- `--block ID` — Replace block by ID
- `--content TEXT` — Replacement content
- `--file PATH` — Read replacement from file
- `--dry-run` — Preview changes

---

### Frontmatter Operations

#### `vaultiel get-frontmatter NOTE_PATH`

Get note frontmatter.

```bash
vaultiel get-frontmatter "note.md"
vaultiel get-frontmatter "note.md" --format yaml
vaultiel get-frontmatter "note.md" --key title
vaultiel get-frontmatter "note.md" --no-inline  # Exclude inline attributes
```

**Flags:**
- `--format FORMAT` — Output: `json`, `yaml`, `toml`
- `--key KEY` — Get specific key only
- `--no-inline` — Exclude inline attributes `[key::value]`

---

#### `vaultiel modify-frontmatter NOTE_PATH`

Modify frontmatter fields.

```bash
vaultiel modify-frontmatter "note.md" -k status --value active
vaultiel modify-frontmatter "note.md" -k tags --add rust
vaultiel modify-frontmatter "note.md" -k tags --remove old-tag
```

**Flags:**
- `-k KEY` — Key to modify
- `--value VALUE` — Value to set
- `--add VALUE` — Add to list
- `--remove VALUE` — Remove from list
- `--dry-run` — Preview changes

---

#### `vaultiel remove-frontmatter NOTE_PATH`

Remove a frontmatter field.

```bash
vaultiel remove-frontmatter "note.md" -k obsolete-field
```

**Flags:**
- `-k KEY` — Key to remove
- `--dry-run` — Preview changes

---

#### `vaultiel rename-frontmatter`

Rename a frontmatter key across notes.

```bash
vaultiel rename-frontmatter --from old-key --to new-key
vaultiel rename-frontmatter --from old-key --to new-key --glob "proj/*.md"
vaultiel rename-frontmatter --from old-key --to new-key --dry-run
```

**Flags:**
- `--from KEY` — Original key name
- `--to KEY` — New key name
- `--glob PATTERN` — Apply to matching notes
- `--dry-run` — Preview changes

---

### Link Operations

#### `vaultiel get-links NOTE_PATH`

Get all links with rich context metadata.

```bash
vaultiel get-links "note.md"
vaultiel get-links "note.md" --context body           # Body links only
vaultiel get-links "note.md" --context "frontmatter:*"
vaultiel get-links "note.md" --embeds-only            # Only embeds
vaultiel get-links "note.md" --no-embeds              # Exclude embeds
vaultiel get-links "note.md" --media-only             # Images/audio/video
```

**Link Context Types:**

| Context | Description |
|---------|-------------|
| `body` | In note body |
| `frontmatter:<key>` | In frontmatter field |
| `frontmatter:<key>[<n>]` | In frontmatter list |
| `inline:<key>` | In inline attribute |
| `task` | Inside a task item |

**Flags:**
- `--context PATTERN` — Filter by context (supports wildcards)
- `--embeds-only` — Only embeds (`![[...]]`)
- `--no-embeds` — Exclude embeds
- `--media-only` — Only image/audio/video/PDF embeds

---

#### `vaultiel get-in-links NOTE_PATH`

Get incoming links (backlinks).

```bash
vaultiel get-in-links "note.md"
```

---

#### `vaultiel get-out-links NOTE_PATH`

Get outgoing links.

```bash
vaultiel get-out-links "note.md"
```

---

#### `vaultiel get-embeds NOTE_PATH`

Get embeds (shorthand for `get-out-links --embeds-only`).

```bash
vaultiel get-embeds "note.md"
vaultiel get-embeds "note.md" --media-only   # Images, PDFs, etc.
vaultiel get-embeds "note.md" --notes-only   # Note embeds only
```

---

### Tag Operations

#### `vaultiel get-tags [NOTE_PATH]`

Get tags from a note or the entire vault.

```bash
vaultiel get-tags "note.md"               # Tags in specific note
vaultiel get-tags                         # All tags in vault
vaultiel get-tags --with-counts           # Include usage counts
vaultiel get-tags --nested                # Hierarchical output
vaultiel get-tags --glob "proj/*.md"      # Tags in matching notes
```

**Flags:**
- `--with-counts` — Include usage counts
- `--nested` — Hierarchical output
- `--glob PATTERN` — Filter notes

---

### Block Operations

#### `vaultiel get-blocks NOTE_PATH`

Get all block IDs in a note.

```bash
vaultiel get-blocks "note.md"
```

#### `vaultiel get-block-refs NOTE_PATH`

Get references to a note's blocks.

```bash
vaultiel get-block-refs "note.md"
```

---

### Heading Operations

#### `vaultiel get-headings NOTE_PATH`

Get all headings in a note.

```bash
vaultiel get-headings "note.md"
vaultiel get-headings "note.md" --nested
vaultiel get-headings "note.md" --min-level 2 --max-level 3
```

**Flags:**
- `--nested` — Hierarchical output
- `--min-level N` — Minimum heading level (1-6)
- `--max-level N` — Maximum heading level (1-6)

---

#### `vaultiel get-section NOTE_PATH HEADING`

Extract section content.

```bash
vaultiel get-section "note.md" "## Configuration"
vaultiel get-section "note.md" "## Configuration" --content-only
vaultiel get-section "note.md" "configuration" --by-slug
vaultiel get-section "note.md" "## Config" --exclude-subheadings
```

**Flags:**
- `--content-only` — Exclude the heading line
- `--by-slug` — Match by heading slug
- `--include-subheadings` — Include nested headings (default)
- `--exclude-subheadings` — Stop at first subheading

---

### Task Operations

#### `vaultiel get-tasks`

Extract tasks with filtering.

```bash
vaultiel get-tasks
vaultiel get-tasks --note "note.md"
vaultiel get-tasks --glob "proj/*.md"
vaultiel get-tasks --symbol "[ ]"              # Incomplete only
vaultiel get-tasks --symbol "[x]" --symbol "[A]"
vaultiel get-tasks --due-before 2026-02-10
vaultiel get-tasks --due-on today
vaultiel get-tasks --scheduled-on tomorrow
vaultiel get-tasks --priority high
vaultiel get-tasks --contains "inbox"
vaultiel get-tasks --links-to "proj/Vaultiel.md"
vaultiel get-tasks --tag "#urgent"
vaultiel get-tasks --has-block-ref
vaultiel get-tasks --flat                       # Non-hierarchical output
```

**Flags:**
- `--note PATH` — Tasks in specific note
- `--glob PATTERN` — Tasks in matching notes
- `--symbol SYMBOL` — Filter by marker (repeatable)
- `--due-before DATE`, `--due-after DATE`, `--due-on DATE`
- `--scheduled-before DATE`, `--scheduled-after DATE`, `--scheduled-on DATE`
- `--done-before DATE`, `--done-after DATE`, `--done-on DATE`
- `--priority LEVEL` — `highest`, `high`, `medium`, `low`, `lowest`
- `--contains TEXT` — Filter by description
- `--has METADATA` — Filter by custom metadata
- `--links-to PATH` — Tasks linking to note
- `--tag TAG` — Tasks with tag
- `--has-block-ref` — Tasks with block reference
- `--block-ref ID` — Tasks with specific block ref
- `--flat` — Flat list instead of hierarchy

**Date formats:** `YYYY-MM-DD`, `today`, `tomorrow`, `yesterday`, `+3d`, `-1w`

---

#### `vaultiel format-task`

Format a task string for Obsidian.

```bash
vaultiel format-task --desc "Clear inbox" --scheduled tomorrow --priority high
# Output: - [ ] Clear inbox ⏳ 2026-02-04 ⏫

vaultiel format-task --desc "Done" --symbol "[x]" --done today
# Output: - [x] Done ✅ 2026-02-03
```

**Flags:**
- `--desc TEXT` — Task description (required)
- `--symbol SYMBOL` — Task marker (default: `[ ]`)
- `--scheduled DATE` — Scheduled date
- `--due DATE` — Due date
- `--done DATE` — Done date
- `--priority LEVEL` — Priority

---

### Vault Health

#### `vaultiel info`

Display vault statistics.

```bash
vaultiel info
vaultiel info --detailed
```

**Flags:**
- `--detailed` — Extended statistics

---

#### `vaultiel lint`

Check vault health.

```bash
vaultiel lint
vaultiel lint --fix                            # Auto-fix where possible
vaultiel lint --only broken-links
vaultiel lint --ignore orphans
vaultiel lint --glob "proj/*.md"
vaultiel lint --fail-on broken-links           # CI mode
vaultiel lint --format github                  # GitHub Actions annotations
```

**Issue Types:**

| Issue | Description | Auto-fix |
|-------|-------------|----------|
| `broken-links` | Links to non-existent notes | No |
| `broken-embeds` | Embeds of non-existent files | No |
| `broken-heading-links` | Invalid `[[Note#Heading]]` | No |
| `broken-block-refs` | Invalid `[[Note#^block]]` | No |
| `orphans` | Notes with no backlinks | No |
| `duplicate-aliases` | Same alias in multiple notes | No |
| `duplicate-block-ids` | Same `^id` used twice in note | Yes |
| `empty-notes` | Notes with no content | No |
| `missing-frontmatter` | Notes without frontmatter | Yes |
| `invalid-frontmatter` | Malformed YAML | No |

**Flags:**
- `--fix` — Auto-fix fixable issues
- `--only TYPE` — Check specific type (repeatable)
- `--ignore TYPE` — Skip type (repeatable)
- `--glob PATTERN` — Check matching notes
- `--fail-on TYPE` — Exit non-zero if found
- `--format FORMAT` — `json`, `text`, `github`

---

#### `vaultiel find-orphans`

Find notes with no incoming links.

```bash
vaultiel find-orphans
vaultiel find-orphans --exclude "templates/*"
```

---

#### `vaultiel find-broken-links`

Find broken links.

```bash
vaultiel find-broken-links
vaultiel find-broken-links --note "proj/Vaultiel.md"
```

---

### Cache Operations

#### `vaultiel cache status`

Show cache information.

```bash
vaultiel cache status
```

#### `vaultiel cache rebuild`

Rebuild the cache.

```bash
vaultiel cache rebuild
vaultiel cache rebuild --progress
```

#### `vaultiel cache clear`

Clear the cache.

```bash
vaultiel cache clear
```

---

### Export

#### `vaultiel export-graph`

Export vault to graph database format.

```bash
vaultiel export-graph --format neo4j-cypher > vault.cypher
vaultiel export-graph --format json-ld --pretty > vault.jsonld
vaultiel export-graph --format neo4j-cypher --use-merge  # MERGE instead of CREATE
vaultiel export-graph --format json-ld --base-uri "https://example.com/vault/"
vaultiel export-graph --include-tags --include-headings --include-frontmatter
```

**Flags:**
- `--format FORMAT` — `neo4j-cypher` or `json-ld`
- `--output FILE` — Write to file instead of stdout
- `--pretty` — Pretty-print JSON-LD
- `--use-merge` — Use MERGE instead of CREATE (Neo4j)
- `--base-uri URI` — Base URI for JSON-LD
- `--include-tags` — Include tag nodes/relationships
- `--include-headings` — Include heading data
- `--include-frontmatter` — Include frontmatter properties

---

### Metadata

#### `vaultiel init-metadata NOTE_PATH`

Initialize vaultiel metadata (UUID + timestamp).

```bash
vaultiel init-metadata "note.md"
vaultiel init-metadata --glob "**/*.md"
vaultiel init-metadata "note.md" --force     # Overwrite existing
vaultiel init-metadata "note.md" --dry-run
```

**Flags:**
- `--glob PATTERN` — Initialize matching notes
- `--force` — Overwrite existing metadata
- `--dry-run` — Preview changes

---

#### `vaultiel get-metadata NOTE_PATH`

Get vaultiel metadata from a note.

```bash
vaultiel get-metadata "note.md"
```

---

#### `vaultiel get-by-id UUID`

Find a note by its vaultiel ID.

```bash
vaultiel get-by-id "550e8400-e29b-41d4-a716-446655440000"
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Note not found |
| 3 | Note already exists |
| 4 | Ambiguous resolution |
| 5 | Invalid frontmatter |
| 10 | Lint issues found |

## Library Usage

Vaultiel can also be used as a Rust library:

```rust
use vaultiel::{Vault, Config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let vault = Vault::new("/path/to/vault", config)?;

    // List notes
    let notes = vault.list_notes()?;

    // Load a note
    let note = vault.load_note("my-note.md")?;
    println!("Content: {}", note.body());

    // Parse links
    let links = note.links();
    for link in links {
        println!("Link to: {}", link.target);
    }

    Ok(())
}
```

## Building

```bash
# Build
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

## License

MIT
