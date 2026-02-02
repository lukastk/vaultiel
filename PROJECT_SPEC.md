# Vaultiel - Project Specification

> A CLI and library for programmatically interacting with Obsidian-style vaults.
> Think of it as `jq` for markdown notes with YAML frontmatter and wikilinks.

---

## Overview

**Vaultiel** provides a programmatic interface to Obsidian vaults, enabling:
- Note listing, creation, deletion, and renaming (with automatic link propagation)
- Content operations (get, set, append, prepend, replace by section/pattern/block)
- Frontmatter manipulation (YAML + inline attributes)
- Link graph traversal (with rich context metadata for embeds, headings, blocks)
- Embed tracking (`![[embed]]` syntax for notes, images, PDFs, media)
- Tag extraction and filtering
- Block reference support (`^block-id`)
- Heading extraction and section-based operations
- Task extraction and formatting (with hierarchy, link, and tag awareness)
- Search (subsequence matching like Obsidian, plus other algorithms)
- Alias resolution (note name/alias to file path)
- Vault health checks (broken links, orphans, duplicate IDs, linting)
- Optional caching for large vault performance

### Target Bindings
1. **CLI** (primary)
2. **Rust library** (core)
3. **TypeScript bindings** (for Obsidian plugin integration)
4. **Python bindings** (for scripting and automation)

---

## Configuration

### Global Config
Location: `~/.config/vaultiel.toml`

```toml
[vault]
default = "/path/to/default/vault"

[tasks]
# Obsidian Tasks plugin symbols (defaults)
due = "📅"
scheduled = "⏳"
done = "✅"
priority_highest = "🔺"
priority_high = "⏫"
priority_medium = "🔼"
priority_low = "🔽"
priority_lowest = "⏬"

# Custom metadata fields (appear BEFORE standard task metadata)
[tasks.custom_metadata]
time_estimate = "⏲️"
# Add more as needed
```

### Global Flags
```bash
--vault PATH    # Override default vault for this command
--json          # Output as JSON (default for most commands)
--yaml          # Output as YAML
--toml          # Output as TOML
--quiet, -q     # Suppress non-essential output
--verbose, -v   # Increase output verbosity (repeatable: -vv, -vvv)
--no-color      # Disable colored output
```

### Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error (invalid arguments, file not found, etc.) |
| 2 | Note not found |
| 3 | Note already exists (for `create`) |
| 4 | Ambiguous resolution (multiple notes match) |
| 5 | Invalid frontmatter (parse error) |
| 10 | Lint issues found (with `--fail-on`) |

Exit codes enable scripting:
```bash
vaultiel resolve "Daily" 2>/dev/null || echo "Note not found or ambiguous"
vaultiel lint --fail-on broken-links && echo "No broken links!"
```

---

## Core Commands

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
vaultiel list --sort modified                # Sort by modification time
vaultiel list --limit 20                     # Limit results
```

Flags:
- `--glob PATTERN` — Filter to notes matching glob pattern
- `--tag TAG` — Filter by tag (repeatable for AND logic)
- `--frontmatter KEY=VALUE` — Filter by frontmatter field
- `--has-links` — Only notes with outgoing links
- `--has-backlinks` — Only notes with incoming links
- `--orphans` — Only notes with no incoming links
- `--sort FIELD` — Sort by: `path` (default), `modified`, `created`, `name`
- `--reverse` — Reverse sort order
- `--limit N` — Limit number of results

Output:
```json
{
  "notes": [
    {
      "path": "proj/Vaultiel.md",
      "name": "Vaultiel",
      "modified": "2026-02-02T18:30:00Z",
      "created": "2026-01-15T10:00:00Z",
      "size_bytes": 12345
    },
    {
      "path": "log/2026-02-02 Work.md",
      "name": "2026-02-02 Work",
      "modified": "2026-02-02T17:45:00Z",
      "created": "2026-02-02T09:00:00Z",
      "size_bytes": 3456
    }
  ],
  "total": 2
}
```

---

#### `vaultiel create NOTE_PATH`
Create a new note. Errors if the note already exists.

```bash
vaultiel create "proj/My Project.md" --frontmatter '{"notetype": "proj"}'
vaultiel create "log/2026-02-02 Daily.md" --content "# Today\n\nNotes here."
vaultiel create "cap/quick thought.md" --template "capture"  # future: templating
```

Flags:
- `--frontmatter JSON` — Initial frontmatter
- `--content TEXT` — Initial content (below frontmatter)
- `--template NAME` — Use a template (future feature)
- `--open` — Open in Obsidian after creation (`obsidian://open?vault=...&file=...`)
- `--dry-run` — Return what would be created without writing

---

#### `vaultiel delete NOTE_PATH`
Delete a note with optional link cleanup.

```bash
vaultiel delete "proj/Old Project.md"
vaultiel delete "proj/Old Project.md" --no-propagate    # Just rm, don't touch other files
vaultiel delete "proj/Old Project.md" --remove-links    # Remove links from other notes
vaultiel delete "proj/Old Project.md" --dry-run         # Show what would happen
```

Flags:
- `--no-propagate` — Just delete the file, don't modify other notes (effectively `rm`)
- `--remove-links` — Remove all links to this note from other files (default: warn only)
- `--force` — Skip confirmation prompt
- `--dry-run` — Show what would be deleted/modified without making changes

**Default behavior:** Deletes the note and warns about any incoming links that will become broken. Use `--remove-links` to automatically clean them up, or `--no-propagate` to skip the check entirely.

---

#### `vaultiel rename NOTE_PATH NEW_PATH`
Rename a note and propagate link changes throughout the vault.

```bash
vaultiel rename "proj/Old Name.md" "proj/New Name.md"
vaultiel rename "proj/Old Name.md" "proj/New Name.md" --no-propagate  # just mv
```

Flags:
- `--no-propagate` — Don't update links in other notes (effectively `mv`)
- `--dry-run` — Show what would change without modifying files.

---

#### `vaultiel search QUERY`
Search notes using subsequence matching (Obsidian-style) by default.

```bash
vaultiel search "vaultiel"
vaultiel search "vaultiel" --limit 10
vaultiel search "vaultiel" --mode fuzzy
vaultiel search "vaultiel" --mode exact
```

Flags:
- `--limit N` — Number of results (default: 1)
- `--mode MODE` — Search algorithm: `subsequence` (default), `fuzzy`, `exact`, `regex`
- `--content` — Search note content (not just titles/paths)
- `--frontmatter KEY=VALUE` — Filter by frontmatter field

---

#### `vaultiel resolve NOTE_NAME`
Resolve a note name or alias to its file path.

```bash
vaultiel resolve "Vaultiel"                    # Find by note name
vaultiel resolve "vault-cli"                   # Find by alias
vaultiel resolve "proj/Vaultiel"               # With folder prefix
vaultiel resolve "Vaultiel" --all              # Return all matches if ambiguous
```

Obsidian notes can have aliases defined in frontmatter:
```yaml
aliases:
  - vault-cli
  - vaultiel-tool
```

The `resolve` command finds the actual file path for a note name or alias, handling:
- Exact path matches
- Note name matches (without `.md` extension)
- Alias matches
- Folder disambiguation

Output:
```json
{
  "query": "vault-cli",
  "resolved": "proj/Vaultiel.md",
  "match_type": "alias"
}
```

When multiple notes match (ambiguous):
```json
{
  "query": "Daily",
  "resolved": null,
  "matches": [
    {"path": "log/2026-02-01 Daily.md", "match_type": "name"},
    {"path": "log/2026-02-02 Daily.md", "match_type": "name"},
    {"path": "templates/Daily.md", "match_type": "name"}
  ],
  "error": "ambiguous: 3 notes match 'Daily'"
}
```

Flags:
- `--all` — Return all matches instead of erroring on ambiguity
- `--strict` — Only match exact paths, not aliases or partial names

---

### Content Operations

#### `vaultiel get-content NOTE_PATH`
Get note content.

```bash
vaultiel get-content "proj/Vaultiel.md"
vaultiel get-content "proj/Vaultiel.md" --include-frontmatter
vaultiel get-content "proj/Vaultiel.md" --include-vaultiel-field
```

Flags:
- `--include-frontmatter` — Include YAML frontmatter block
- `--include-vaultiel-field` — Include the `vaultiel` metadata field (excluded by default)

---

#### `vaultiel set-content NOTE_PATH`
Set note content.

```bash
echo "New content" | vaultiel set-content "proj/Vaultiel.md"
vaultiel set-content "proj/Vaultiel.md" --content "New content"
vaultiel set-content "proj/Vaultiel.md" --content "New content" --below-frontmatter
vaultiel set-content "proj/Vaultiel.md" --file content.md
```

Flags:
- `--content TEXT` — Content to set (or pipe via stdin)
- `--file PATH` — Read content from file
- `--below-frontmatter` — Only replace content below frontmatter (preserve frontmatter)
- `--frontmatter-only` — Only replace frontmatter (preserve content)
- `--dry-run` — Return modified content without writing

---

#### `vaultiel append-content NOTE_PATH`
Append content to a note.

```bash
vaultiel append-content "log/2026-02-02.md" --content "\n## Evening\n\nMore notes."
echo "Appended text" | vaultiel append-content "log/2026-02-02.md"
vaultiel append-content "log/2026-02-02.md" --file notes.md
```

Flags:
- `--content TEXT` — Content to append (or pipe via stdin)
- `--file PATH` — Read content from file
- `--dry-run` — Return modified content without writing

---

#### `vaultiel prepend-content NOTE_PATH`
Prepend content to a note (after frontmatter).

```bash
vaultiel prepend-content "proj/Vaultiel.md" --content "## Notice\n\nThis is important.\n\n"
echo "Prepended text" | vaultiel prepend-content "proj/Vaultiel.md"
vaultiel prepend-content "proj/Vaultiel.md" --file header.md
```

Flags:
- `--content TEXT` — Content to prepend (or pipe via stdin)
- `--file PATH` — Read content from file
- `--dry-run` — Return modified content without writing

Note: Content is inserted immediately after the frontmatter block (if present) or at the beginning of the file.

---

#### `vaultiel replace-content NOTE_PATH`
Replace content matching a pattern or heading section.

```bash
# Replace by heading section
vaultiel replace-content "proj/Vaultiel.md" --section "## Status" --content "## Status\n\nActive development."

# Replace by pattern (first match)
vaultiel replace-content "proj/Vaultiel.md" --pattern "TODO:.*" --content "DONE: Completed"

# Replace by line range
vaultiel replace-content "proj/Vaultiel.md" --lines 10-15 --content "New content for these lines"

# Replace a specific block (by block ID)
vaultiel replace-content "proj/Vaultiel.md" --block "api-design" --content "Updated API design notes ^api-design"
```

Flags:
- `--section HEADING` — Replace entire section under heading (until next same-level heading)
- `--pattern REGEX` — Replace first match of regex pattern
- `--pattern-all REGEX` — Replace all matches of regex pattern
- `--lines RANGE` — Replace line range (e.g., `10-15`, `10-`, `-15`)
- `--block BLOCK_ID` — Replace the block with the given `^block-id`
- `--content TEXT` — Replacement content (or pipe via stdin)
- `--file PATH` — Read replacement content from file
- `--dry-run` — Return modified content without writing

---

### Frontmatter Operations

#### `vaultiel get-frontmatter NOTE_PATH`
Get note frontmatter.

```bash
vaultiel get-frontmatter "proj/Vaultiel.md"
vaultiel get-frontmatter "proj/Vaultiel.md" --format yaml
vaultiel get-frontmatter "proj/Vaultiel.md" --no-inline
```

Flags:
- `--format FORMAT` — Output format: `json` (default), `yaml`, `toml`
- `--no-inline` — Exclude inline attributes (`[key::value]` syntax)
- `--key KEY` — Get specific key only

---

#### `vaultiel modify-frontmatter NOTE_PATH`
Modify frontmatter fields.

```bash
vaultiel modify-frontmatter "proj/Vaultiel.md" -k proj-status -v active
vaultiel modify-frontmatter "proj/Vaultiel.md" -k tags -v:add rust
vaultiel modify-frontmatter "proj/Vaultiel.md" -k tags -v:remove old-tag
```

Flags:
- `-k KEY` — Key to modify
- `-v VALUE` — Value to set
- `-v:add VALUE` — Add to list
- `-v:remove VALUE` — Remove from list
- `--dry-run` — Return modified content without writing

---

#### `vaultiel remove-frontmatter NOTE_PATH`
Remove a frontmatter field.

```bash
vaultiel remove-frontmatter "proj/Vaultiel.md" -k obsolete-field
```

Flags:
- `-k KEY` — Key to remove
- `--dry-run` — Return modified content without writing

---

#### `vaultiel rename-frontmatter`
Rename a frontmatter key across notes.

```bash
vaultiel rename-frontmatter --from old-key --to new-key
vaultiel rename-frontmatter --from old-key --to new-key --note "proj/Vaultiel.md"
vaultiel rename-frontmatter --from old-key --to new-key --glob "proj/*.md"
```

Flags:
- `--from KEY` — Original key name
- `--to KEY` — New key name
- `--note PATH` — Apply to specific note only
- `--glob PATTERN` — Apply to notes matching glob
- `--dry-run` — Show what would change

---

### Link Operations

#### `vaultiel get-links NOTE_PATH`
Get all links (incoming and outgoing) with rich context metadata.

```bash
vaultiel get-links "proj/Vaultiel.md"
```

**Link Context Types:**

Links can appear in different contexts, and vaultiel tracks where each link is located:

| Context | Description | Example |
|---------|-------------|---------|
| `body` | In the note body (markdown content) | `See [[Other Note]]` |
| `frontmatter:<key>` | In a frontmatter field (scalar) | `parent: "[[Vaultiel]]"` |
| `frontmatter:<key>[<index>]` | In a frontmatter list field | `links: ["[[Note1]]", "[[Note2]]"]` |
| `inline:<key>` | In an inline attribute | `[related::[[Note]]]` |
| `task` | Inside a task item | `- [ ] Review [[PR Link]]` |

**Output Example:**
```json
{
  "incoming": [
    {
      "from": "pad/Vaultiel spec.md",
      "line": 5,
      "context": "frontmatter:parent",
      "alias": null,
      "block_id": null,
      "heading": null,
      "embed": false
    },
    {
      "from": "log/2026-01-28 Work.md",
      "line": 8,
      "context": "frontmatter:links[0]",
      "alias": null,
      "block_id": null,
      "heading": null,
      "embed": false
    },
    {
      "from": "proj/Obako.md",
      "line": 45,
      "context": "body",
      "alias": "vaultiel project",
      "block_id": null,
      "heading": null,
      "embed": false
    },
    {
      "from": "mod/Build tooling.md",
      "line": 23,
      "context": "body",
      "alias": null,
      "block_id": "api-design",
      "heading": null,
      "embed": false
    },
    {
      "from": "pad/Planning.md",
      "line": 15,
      "context": "body",
      "alias": null,
      "block_id": null,
      "heading": "Configuration",
      "embed": false
    },
    {
      "from": "proj/Overview.md",
      "line": 8,
      "context": "body",
      "alias": null,
      "block_id": null,
      "heading": null,
      "embed": true
    }
  ],
  "outgoing": [
    {
      "to": "pad/Vaultiel spec.md",
      "line": 12,
      "context": "body",
      "alias": null,
      "block_id": null,
      "heading": null,
      "embed": false
    },
    {
      "to": "proj/Obako.md",
      "line": 3,
      "context": "frontmatter:parent",
      "alias": null,
      "block_id": null,
      "heading": null,
      "embed": false
    },
    {
      "to": "ent/Rust.md",
      "line": 34,
      "context": "inline:related",
      "alias": null,
      "block_id": null,
      "heading": null,
      "embed": false
    },
    {
      "to": "assets/diagram.png",
      "line": 56,
      "context": "body",
      "alias": null,
      "block_id": null,
      "heading": null,
      "embed": true
    }
  ]
}
```

**Filtering by context:**
```bash
vaultiel get-links "proj/Vaultiel.md" --context body           # Body links only
vaultiel get-links "proj/Vaultiel.md" --context "frontmatter:*" # All frontmatter links
vaultiel get-links "proj/Vaultiel.md" --context "frontmatter:parent"  # Just parent field
```

Flags:
- `--context PATTERN` — Filter by context (supports wildcards)
- `--include-aliases` — Include link aliases in output (default: true)
- `--include-blocks` — Include block references (default: true)

---

#### `vaultiel get-in-links NOTE_PATH`
Get incoming links only. Same output format as `get-links`.

---

#### `vaultiel get-out-links NOTE_PATH`
Get outgoing links only. Same output format as `get-links`.

---

#### Embeds

Obsidian distinguishes between links (`[[note]]`) and embeds (`![[note]]` or `![[image.png]]`). Embeds inline the referenced content.

**Embed types:**
- Note embeds: `![[note]]` or `![[note#heading]]` or `![[note#^block]]`
- Image embeds: `![[image.png]]`, `![[photo.jpg|400]]` (with size)
- PDF embeds: `![[document.pdf]]` or `![[document.pdf#page=5]]`
- Audio/video embeds: `![[audio.mp3]]`, `![[video.mp4]]`

**Embed tracking in links:**

The `get-links` output includes an `embed` field:

```json
{
  "to": "proj/Vaultiel.md",
  "line": 23,
  "context": "body",
  "alias": null,
  "block_id": null,
  "heading": null,
  "embed": true
}
```

**Filtering embeds:**
```bash
vaultiel get-links "proj/Vaultiel.md" --embeds-only      # Only embeds
vaultiel get-links "proj/Vaultiel.md" --no-embeds        # Exclude embeds
vaultiel get-links "proj/Vaultiel.md" --media-only       # Only image/audio/video/pdf embeds
```

---

#### `vaultiel get-embeds NOTE_PATH`
Get all embeds in a note (shorthand for `get-out-links --embeds-only`).

```bash
vaultiel get-embeds "proj/Vaultiel.md"
vaultiel get-embeds "proj/Vaultiel.md" --media-only      # Images, audio, video, PDFs
vaultiel get-embeds "proj/Vaultiel.md" --notes-only      # Note embeds only
```

Output:
```json
{
  "embeds": [
    {
      "target": "assets/diagram.png",
      "line": 15,
      "type": "image",
      "size": "400",
      "context": "body"
    },
    {
      "target": "proj/API Design.md",
      "line": 28,
      "type": "note",
      "heading": "Overview",
      "context": "body"
    }
  ]
}
```

---

### Tag Operations

#### `vaultiel get-tags [NOTE_PATH]`
Get tags from a note or the entire vault.

```bash
vaultiel get-tags "proj/Vaultiel.md"          # Tags in specific note
vaultiel get-tags                              # All tags in vault
vaultiel get-tags --with-counts                # Include usage counts
vaultiel get-tags --glob "proj/*.md"           # Tags in matching notes
```

Output:
```json
{
  "tags": [
    {"tag": "#rust", "count": 15},
    {"tag": "#tray", "count": 42},
    {"tag": "#tray/autonomy", "count": 12}
  ]
}
```

For a specific note, includes location context:
```json
{
  "tags": [
    {"tag": "#hp-cons", "line": 15, "context": "body"},
    {"tag": "#tray/personal", "line": 23, "context": "body"}
  ]
}
```

Flags:
- `--with-counts` — Include usage counts (vault-wide)
- `--glob PATTERN` — Filter to notes matching glob
- `--nested` — Return as nested hierarchy (e.g., `tray` → `tray/autonomy`)

---

#### `vaultiel search` (tag filtering)
Search now supports tag filtering:

```bash
vaultiel search "" --tag "#rust"                    # Notes with #rust tag
vaultiel search "" --tag "#tray" --tag "#hp-cons"   # Notes with both tags
vaultiel search "" --tag "#tray/*"                  # Notes with any #tray subtag
```

Additional flags:
- `--tag TAG` — Filter by tag (repeatable for AND logic)
- `--tag-any TAG` — Filter by tag (repeatable for OR logic)
- `--no-tag TAG` — Exclude notes with tag

---

### Block Reference Operations

Vaultiel supports Obsidian's block reference syntax (`^block-id`).

#### Block IDs in Links
Links can reference specific blocks:

```markdown
See [[proj/Vaultiel#^implementation-notes]]
```

#### `vaultiel get-blocks NOTE_PATH`
Get all block IDs defined in a note.

```bash
vaultiel get-blocks "proj/Vaultiel.md"
```

Output:
```json
{
  "blocks": [
    {"id": "implementation-notes", "line": 45, "type": "paragraph"},
    {"id": "api-design", "line": 78, "type": "list-item"},
    {"id": "15gv8g", "line": 102, "type": "paragraph"}
  ]
}
```

#### `vaultiel get-block-refs NOTE_PATH`
Get all block references pointing to a note's blocks.

```bash
vaultiel get-block-refs "proj/Vaultiel.md"
```

Output:
```json
{
  "refs": [
    {
      "block_id": "implementation-notes",
      "from": "pad/Planning notes.md",
      "line": 23,
      "context": "body"
    }
  ]
}
```

Block references are also included in `get-links` output with `block_id` field.

---

### Heading Operations

Obsidian supports linking to headings via `[[Note#Heading]]`. Vaultiel provides commands to work with headings.

#### `vaultiel get-headings NOTE_PATH`
Get all headings in a note.

```bash
vaultiel get-headings "proj/Vaultiel.md"
vaultiel get-headings "proj/Vaultiel.md" --min-level 2 --max-level 3
```

Output:
```json
{
  "headings": [
    {"text": "Overview", "level": 2, "line": 8, "slug": "overview"},
    {"text": "Configuration", "level": 2, "line": 29, "slug": "configuration"},
    {"text": "Global Config", "level": 3, "line": 31, "slug": "global-config"},
    {"text": "Core Commands", "level": 2, "line": 64, "slug": "core-commands"}
  ]
}
```

Flags:
- `--min-level N` — Minimum heading level (1-6)
- `--max-level N` — Maximum heading level (1-6)
- `--nested` — Return as nested hierarchy instead of flat list

**Nested output:**
```json
{
  "headings": [
    {
      "text": "Overview",
      "level": 2,
      "line": 8,
      "slug": "overview",
      "children": []
    },
    {
      "text": "Configuration",
      "level": 2,
      "line": 29,
      "slug": "configuration",
      "children": [
        {"text": "Global Config", "level": 3, "line": 31, "slug": "global-config", "children": []}
      ]
    }
  ]
}
```

---

#### `vaultiel get-section NOTE_PATH HEADING`
Get the content of a specific section (heading and all content until the next same-level or higher heading).

```bash
vaultiel get-section "proj/Vaultiel.md" "## Configuration"
vaultiel get-section "proj/Vaultiel.md" "## Configuration" --include-subheadings
vaultiel get-section "proj/Vaultiel.md" "## Configuration" --content-only
```

Flags:
- `--include-subheadings` — Include nested subheadings (default: true)
- `--exclude-subheadings` — Stop at first subheading
- `--content-only` — Exclude the heading line itself
- `--by-slug SLUG` — Find heading by slug instead of exact text match

---

#### Heading Links

Heading links (`[[Note#Heading]]`) are tracked in link operations:

```json
{
  "from": "pad/Planning.md",
  "line": 15,
  "context": "body",
  "alias": null,
  "block_id": null,
  "heading": "Configuration"
}
```

The `heading` field is populated when the link targets a specific heading.

---

### Task Operations

#### `vaultiel get-tasks`
Extract tasks from the vault with filtering. Tasks are link-aware and can contain references to notes.

```bash
vaultiel get-tasks
vaultiel get-tasks --note "proj/Vaultiel.md"
vaultiel get-tasks --glob "mod/*.md"
vaultiel get-tasks --symbol "[ ]"              # Incomplete only
vaultiel get-tasks --symbol "[x]" --symbol "[A]"  # Multiple symbols
vaultiel get-tasks --due-before 2026-02-10
vaultiel get-tasks --scheduled-on 2026-02-02
vaultiel get-tasks --priority high
vaultiel get-tasks --contains "inbox"
vaultiel get-tasks --links-to "proj/Vaultiel.md"  # Tasks that link to a note
vaultiel get-tasks --tag "#hp-cons"               # Tasks with specific tag
```

Flags:
- `--note PATH` — Filter to tasks in specific note
- `--glob PATTERN` — Filter to tasks in notes matching glob
- `--symbol SYMBOL` — Filter by task marker (`[ ]`, `[x]`, `[d]`, `[>]`, etc.)
- `--due-before DATE` — Due date before (exclusive)
- `--due-after DATE` — Due date after (exclusive)
- `--due-on DATE` — Due on specific date
- `--due-before-inclusive DATE` — Due date before (inclusive)
- `--due-after-inclusive DATE` — Due date after (inclusive)
- `--scheduled-before DATE`, `--scheduled-after DATE`, `--scheduled-on DATE` — Scheduled date filters
- `--done-before DATE`, `--done-after DATE`, `--done-on DATE` — Completion date filters
- `--priority LEVEL` — Filter by priority (highest, high, medium, low, lowest)
- `--contains TEXT` — Filter by task description text
- `--has METADATA_KEY` — Filter by presence of custom metadata (e.g., `--has time_estimate`)
- `--links-to NOTE_PATH` — Filter to tasks containing a link to the specified note
- `--tag TAG` — Filter to tasks containing the specified tag
- `--has-block-ref` — Filter to tasks with block references (`^block-id`)
- `--block-ref ID` — Filter to tasks with a specific block reference
- `--flat` — Return flat list instead of hierarchical (each task has `indent`/`parent_task` but no `children`)

**Output:**

Tasks include full location metadata and any links/tags/block-refs they contain:

```json
[
  {
    "location": {
      "file": "mod/Build CLI.md",
      "line": 15
    },
    "raw": "- [ ] Implement rename command for [[proj/Vaultiel]] ⏲️ 2h ⏳ 2026-02-05 📅 2026-02-10 ⏫ #hp-cons ^task-123",
    "symbol": "[ ]",
    "description": "Implement rename command for [[proj/Vaultiel]]",
    "scheduled": "2026-02-05",
    "due": "2026-02-10",
    "priority": "high",
    "custom": {
      "time_estimate": "2h"
    },
    "links": [
      {"to": "proj/Vaultiel.md", "alias": null}
    ],
    "tags": ["#hp-cons"],
    "block_id": "task-123"
  },
  {
    "location": {
      "file": "proj/Obako.md",
      "line": 42
    },
    "raw": "- [+] Review [[pad/Vaultiel spec|spec]] changes ^762mze",
    "symbol": "[+]",
    "description": "Review [[pad/Vaultiel spec|spec]] changes",
    "scheduled": null,
    "due": null,
    "priority": null,
    "custom": {},
    "links": [
      {"to": "pad/Vaultiel spec.md", "alias": "spec"}
    ],
    "tags": [],
    "block_id": "762mze"
  }
]
```

**Task Location Context:**

The `location.file` tells you which note the task lives in. This is essential for:
- Understanding task provenance
- Navigating to the task in Obsidian
- Filtering tasks by their parent module/project

---

#### Task Hierarchy

Tasks can be nested via indentation. Vaultiel tracks the full hierarchy:

```markdown
- [ ] Parent task
	- [ ] Child task 1
	- [ ] Child task 2
		- [ ] Grandchild task
	- [x] Child task 3 (completed)
```

**Output with hierarchy:**

```json
{
  "location": {
    "file": "mod/Build CLI.md",
    "line": 10
  },
  "raw": "- [ ] Parent task",
  "symbol": "[ ]",
  "description": "Parent task",
  "indent": 0,
  "parent_task": null,
  "children": [
    {
      "location": {
        "file": "mod/Build CLI.md",
        "line": 11
      },
      "raw": "- [ ] Child task 1",
      "symbol": "[ ]",
      "description": "Child task 1",
      "indent": 1,
      "parent_task": {"line": 10},
      "children": [],
      "scheduled": null,
      "due": null,
      "priority": null,
      "custom": {},
      "links": [],
      "tags": [],
      "block_id": null
    },
    {
      "location": {
        "file": "mod/Build CLI.md",
        "line": 12
      },
      "raw": "- [ ] Child task 2",
      "symbol": "[ ]",
      "description": "Child task 2",
      "indent": 1,
      "parent_task": {"line": 10},
      "children": [
        {
          "location": {"file": "mod/Build CLI.md", "line": 13},
          "raw": "- [ ] Grandchild task",
          "symbol": "[ ]",
          "description": "Grandchild task",
          "indent": 2,
          "parent_task": {"line": 12},
          "children": [],
          "...": "..."
        }
      ],
      "...": "..."
    }
  ],
  "scheduled": null,
  "due": null,
  "priority": null,
  "custom": {},
  "links": [],
  "tags": [],
  "block_id": null
}
```

**Hierarchy fields:**
- `indent` — Indentation level (0 = top-level, 1 = one indent, etc.)
- `parent_task` — Reference to parent task (by line number in same file), or `null` if top-level
- `children` — Array of child tasks (recursive structure)

**Flattening:**
```bash
vaultiel get-tasks --flat  # Returns flat list, each task has indent/parent_task but no children array
```

By default, `get-tasks` returns a hierarchical structure (top-level tasks with nested children). Use `--flat` for a flat list where each task stands alone but retains its `indent` and `parent_task` metadata.

---

#### `vaultiel format-task`
Format a task string for pasting into Obsidian.

```bash
vaultiel format-task --desc "Clear your inbox" --scheduled tomorrow --priority high
# Output: - [ ] Clear your inbox ⏳ 2026-02-03 ⏫

vaultiel format-task --desc "Review PR" --due 2026-02-10 --time-estimate 30m
# Output: - [ ] Review PR ⏲️ 30m 📅 2026-02-10

vaultiel format-task --desc "Done task" --symbol "[x]" --done today
# Output: - [x] Done task ✅ 2026-02-02
```

Flags:
- `--desc TEXT` — Task description (required)
- `--symbol SYMBOL` — Task marker (default: `[ ]`)
- `--scheduled DATE` — Scheduled date (supports `today`, `tomorrow`, `YYYY-MM-DD`)
- `--due DATE` — Due date
- `--done DATE` — Completion date
- `--priority LEVEL` — Priority level
- `--time-estimate DURATION` — Time estimate (custom metadata)
- `--CUSTOM_KEY VALUE` — Any custom metadata defined in config

---

### Vault Info & Health

#### `vaultiel info`
Display vault information and statistics.

```bash
vaultiel info
vaultiel info --detailed
```

Output:
```json
{
  "vault_path": "/Users/lukas/store/obsidian/myvault",
  "note_count": 1543,
  "total_size_bytes": 15234567,
  "link_count": 8921,
  "tag_count": 234,
  "task_count": 456,
  "orphan_count": 23,
  "broken_link_count": 5
}
```

Detailed output (`--detailed`) adds:
```json
{
  "...": "...",
  "notes_by_folder": {
    "proj": 45,
    "log": 365,
    "cap": 234,
    "mod": 89
  },
  "top_tags": [
    {"tag": "#tray", "count": 142},
    {"tag": "#rust", "count": 89}
  ],
  "top_linked": [
    {"note": "proj/Vaultiel.md", "incoming": 45},
    {"note": "ent/Rust.md", "incoming": 38}
  ],
  "recently_modified": [
    {"note": "log/2026-02-02 Work.md", "modified": "2026-02-02T18:30:00Z"}
  ]
}
```

Flags:
- `--detailed` — Include extended statistics
- `--format FORMAT` — Output format: `json` (default), `yaml`, `text`

---

#### `vaultiel lint`
Check vault health and report issues.

```bash
vaultiel lint
vaultiel lint --fix                           # Auto-fix where possible
vaultiel lint --only broken-links             # Check specific issue type
vaultiel lint --ignore orphans                # Skip specific checks
vaultiel lint --glob "proj/*.md"              # Check only matching notes
```

**Issue types detected:**

| Issue | Description | Auto-fixable |
|-------|-------------|--------------|
| `broken-links` | Links pointing to non-existent notes | No (would need to create or remove) |
| `broken-embeds` | Embeds pointing to non-existent files | No |
| `broken-heading-links` | `[[Note#Heading]]` where heading doesn't exist | No |
| `broken-block-refs` | `[[Note#^block]]` where block ID doesn't exist | No |
| `orphans` | Notes with no incoming links | No |
| `duplicate-aliases` | Same alias defined in multiple notes | No |
| `duplicate-block-ids` | Same `^block-id` used multiple times in a note | Yes (rename) |
| `empty-notes` | Notes with no content (only frontmatter or blank) | No |
| `missing-frontmatter` | Notes without YAML frontmatter | Yes (add empty) |
| `invalid-frontmatter` | Malformed YAML in frontmatter | No |
| `unresolved-aliases` | Links using aliases that don't resolve | No |

Output:
```json
{
  "issues": [
    {
      "type": "broken-links",
      "file": "proj/Vaultiel.md",
      "line": 45,
      "message": "Link to non-existent note: [[Old Design Doc]]",
      "target": "Old Design Doc.md",
      "fixable": false
    },
    {
      "type": "orphans",
      "file": "cap/random thought.md",
      "line": null,
      "message": "Note has no incoming links",
      "fixable": false
    },
    {
      "type": "duplicate-block-ids",
      "file": "proj/API.md",
      "line": 23,
      "message": "Block ID '^todo' already used on line 15",
      "fixable": true
    }
  ],
  "summary": {
    "total": 3,
    "by_type": {
      "broken-links": 1,
      "orphans": 1,
      "duplicate-block-ids": 1
    },
    "fixable": 1
  }
}
```

Flags:
- `--fix` — Automatically fix issues where possible
- `--only TYPE` — Only check specific issue type (repeatable)
- `--ignore TYPE` — Skip specific issue type (repeatable)
- `--glob PATTERN` — Check only notes matching pattern
- `--fail-on TYPE` — Exit with non-zero status if issue type found (for CI)
- `--format FORMAT` — Output format: `json` (default), `text`, `github` (GitHub Actions annotations)

---

#### `vaultiel find-orphans`
Find notes with no incoming links (shorthand for `vaultiel lint --only orphans`).

```bash
vaultiel find-orphans
vaultiel find-orphans --exclude "templates/*"    # Ignore template files
vaultiel find-orphans --exclude "log/*"          # Ignore daily logs
```

---

#### `vaultiel find-broken-links`
Find broken links in the vault (shorthand for `vaultiel lint --only broken-links`).

```bash
vaultiel find-broken-links
vaultiel find-broken-links --note "proj/Vaultiel.md"    # Check specific note
```

---

## Vaultiel Metadata Field

Notes can have a `vaultiel` field in frontmatter for Vaultiel-specific metadata:

```yaml
vaultiel:
  id: "550e8400-e29b-41d4-a716-446655440000"
  created: "2026-02-02T18:30:00Z"
```

### Purpose
- **`id`**: UUID for tracking notes across renames (useful when syncing with external systems)
- **`created`**: Creation timestamp (independent of filesystem)

### Commands

#### `vaultiel init-metadata NOTE_PATH`
Initialize vaultiel metadata for a note.

```bash
vaultiel init-metadata "proj/Vaultiel.md"
vaultiel init-metadata --glob "**/*.md"  # All notes
```

#### `vaultiel get-by-id UUID`
Find a note by its vaultiel ID.

```bash
vaultiel get-by-id "550e8400-e29b-41d4-a716-446655440000"
```

---

## Future Features

### Graph Database Export
Export vault to a format loadable into graph databases (Neo4j, etc.).

```bash
vaultiel export-graph --format neo4j-cypher > vault.cypher
vaultiel export-graph --format json-ld > vault.jsonld
```

With IDs, this enables:
- Incremental sync (diff-based updates)
- Bidirectional sync (DB → vault)
- Filtered subsets (large DB → small local vault)

---

### Templating
Templates with executable JavaScript for note creation.

```bash
vaultiel create "log/2026-02-02.md" --template "daily-log"
```

Template files would have access to:
- Vaultiel API (via TypeScript bindings)
- Date/time utilities
- User-defined functions

*Requires TypeScript bindings to be complete.*

---

### Sub-vault Support
Mount external vaults with namespace isolation.

```bash
vaultiel mount /path/to/guest-vault --as "guest"
# Notes accessible as guest/note-name
```

Vaultiel handles naming conflicts and link resolution.

---

## Caching System

For large vaults (1000+ notes), vaultiel can maintain a cache to avoid re-parsing the entire vault on every command.

### Cache Location
```
~/.cache/vaultiel/<vault-hash>/
├── index.json          # Note metadata index
├── links.json          # Link graph
├── tags.json           # Tag index
├── tasks.json          # Task index
└── meta.json           # Cache metadata (version, last update, etc.)
```

Or vault-local:
```
<vault>/.vaultiel/cache/
```

### Cache Strategy

**Index Contents:**
- Note paths and mtimes
- Frontmatter (parsed)
- Outgoing links with context
- Tags
- Block IDs
- Tasks (parsed)

**Invalidation:**
- On note mtime change → re-index that note
- On note deletion → remove from index, update incoming links
- On note creation → add to index
- Full rebuild available via `vaultiel cache rebuild`

**Consistency:**
- Before any read operation, check mtimes of relevant files
- Optionally run in "trust cache" mode for maximum speed (`--trust-cache`)

### Cache Commands

#### `vaultiel cache status`
Show cache status and statistics.

```bash
vaultiel cache status
```

Output:
```json
{
  "vault": "/Users/lukas/store/obsidian/myvault",
  "cache_path": "/Users/lukas/.cache/vaultiel/abc123/",
  "indexed_notes": 1543,
  "last_full_index": "2026-02-02T18:30:00Z",
  "stale_notes": 12,
  "cache_size_bytes": 2456789
}
```

#### `vaultiel cache rebuild`
Force a full cache rebuild.

```bash
vaultiel cache rebuild
vaultiel cache rebuild --verbose  # Show progress
```

#### `vaultiel cache clear`
Clear the cache entirely.

```bash
vaultiel cache clear
```

### Config Options

```toml
[cache]
enabled = true                    # Enable caching (default: true for vaults >500 notes)
location = "global"               # "global" (~/.cache) or "local" (.vaultiel/)
auto_threshold = 500              # Auto-enable for vaults with more notes than this
trust_mode = false                # Skip mtime checks (faster but may be stale)
```

### Design Considerations

1. **Incremental updates**: Only re-parse changed files. Use file mtime as change detector.

2. **Atomic writes**: Write cache to temp file, then rename. Prevents corruption on crash.

3. **Lock file**: Prevent concurrent cache writes from multiple vaultiel processes.

4. **Memory-mapped index**: For extremely large vaults, consider mmap for the link graph to avoid loading everything into RAM.

5. **Obsidian coexistence**: If Obsidian is running, it may also be modifying files. The mtime-based invalidation handles this, but "trust cache" mode should be used with caution.

## Design Decisions

1. **Obsidian coexistence**: No automatic detection of Obsidian running. Users can use `--no-propagate` flag on `rename` and `delete` if they want to avoid conflicts.

2. **Inline attributes scope**: Note-level only. Block-level inline attributes are out of scope for now.

3. **Caching**: The cache is rebuilt on-demand or via explicit `cache rebuild`. No daemon required — mtime checks on read operations handle staleness.

4. **Link alias updates**: When renaming a note, vaultiel updates the link target (`[[old-name]]` → `[[new-name]]`) but does NOT modify display aliases (`[[old-name|My Alias]]` → `[[new-name|My Alias]]`). The alias is user-chosen text.

5. **Nested tags**: No depth limit. Tags like `#tray/autonomy/urgent/critical` are fully supported.

6. **Task hierarchy**: Nested/indented tasks are fully supported. See [Task Hierarchy](#task-hierarchy) section.

7. **Delete behavior**: Default `delete` warns about broken links but proceeds. Use `--remove-links` to clean up references, or `--no-propagate` to skip checks entirely.

8. **Embeds vs links**: Embeds (`![[...]]`) and links (`[[...]]`) are tracked separately with an `embed` boolean field. Both can have aliases, block refs, and heading refs.

9. **Heading slugs**: Heading slugs follow Obsidian's algorithm (lowercase, spaces to hyphens, special chars removed). The `get-headings` output includes computed slugs.

10. **Orphan definition**: A note is considered an "orphan" if it has zero incoming links. Notes in certain folders (like `templates/`) can be excluded from orphan detection via `--exclude` flags.

11. **Lint philosophy**: `lint` reports issues but doesn't fix by default. Use `--fix` for auto-fixable issues. Non-fixable issues require user judgment (e.g., should a broken link be removed or should the target note be created?).

12. **Alias resolution**: When multiple notes have the same alias, `resolve` returns an error with all matches. Use `--all` to get all matches, or be more specific with folder prefixes.

13. **External links**: Standard markdown links (`[text](https://url)`) are NOT tracked in the link graph. Only wikilinks (`[[note]]`) and embeds (`![[file]]`) are tracked. External links are considered content, not structural relationships.

14. **Code blocks**: Wikilinks and other syntax inside fenced code blocks (` ``` `) or inline code (`` ` ``) are NOT parsed. They are treated as literal text. This matches Obsidian's behavior.

15. **Dataview inline fields**: Dataview uses `field:: value` syntax which looks similar to vaultiel's inline attributes `[key::value]`. Vaultiel only parses the bracketed form. Dataview-style fields without brackets are ignored by vaultiel (they're in Dataview's domain).

16. **Canvas files**: Obsidian `.canvas` files are JSON and may contain references to notes. Canvas support is out of scope for the initial implementation but could be added as a future feature.

17. **HTML comments**: HTML comments (`<!-- comment -->`) are preserved during all content operations. They are not parsed for links or other syntax.

18. **Non-existent notes**: Content operations (`set-content`, `append-content`, `prepend-content`, `replace-content`) require the note to exist and will error if it doesn't. Use `create` first, or combine: `vaultiel create "note.md" && vaultiel append-content "note.md" --content "..."`. This is intentional to prevent accidental file creation in wrong locations.

19. **Relative dates**: Date flags (`--due`, `--scheduled`, etc.) accept relative dates: `today`, `tomorrow`, `yesterday`, `+3d` (3 days from now), `-1w` (1 week ago), `next monday`, `last friday`. ISO 8601 dates (`2026-02-15`) are always accepted.

---

## References

- [Obsidian Tasks Plugin](https://publish.obsidian.md/tasks/Getting+Started/Getting+Started)
- [Obsidian URI Protocol](https://help.obsidian.md/Advanced+topics/Using+Obsidian+URI)
- Lukas's Obako system: `SECOND_BRAIN_BIBLE.md` in vault root

---

*Last updated: 2026-02-02 (v2 - added list, delete, resolve, prepend-content, replace-content, get-headings, get-section, get-embeds, info, lint; clarified edge cases for external links, code blocks, Dataview fields, canvas files, HTML comments)*
