# Phase 2: Links, Tags & Blocks

## Overview

This phase builds the graph structure and traversal capabilities. We'll create an in-memory link graph that tracks relationships between notes, then implement commands to query links, tags, blocks, and headings. The crown jewel is the `rename` command with link propagation.

## Architecture Decisions

### Link Graph Structure

The link graph needs to efficiently answer:
1. What notes does note X link to? (outgoing)
2. What notes link to note X? (incoming)
3. What is the context of each link? (body, frontmatter, inline attr, task)

```rust
pub struct LinkGraph {
    /// Map from note path to its parsed links
    outgoing: HashMap<PathBuf, Vec<LinkInfo>>,
    /// Map from target path to notes that link to it
    incoming: HashMap<PathBuf, Vec<IncomingLink>>,
}

pub struct LinkInfo {
    pub link: Link,           // From types.rs (target, alias, heading, block_id, embed)
    pub context: LinkContext, // From types.rs (Body, FrontmatterScalar, etc.)
    pub resolved_path: Option<PathBuf>, // Resolved target path (if exists)
}

pub struct IncomingLink {
    pub from: PathBuf,        // Source note
    pub link: Link,           // The link itself
    pub context: LinkContext, // Where in the source note
}
```

### Module Structure

```
vaultiel-rs/src/
├── graph/
│   ├── mod.rs
│   ├── link_graph.rs    # LinkGraph struct and building
│   └── resolution.rs    # Link target resolution logic
├── cli/
│   ├── links.rs         # get-links, get-in-links, get-out-links, get-embeds
│   ├── tags.rs          # get-tags
│   ├── blocks.rs        # get-blocks, get-block-refs
│   ├── headings.rs      # get-headings, get-section
│   └── rename.rs        # rename, rename-frontmatter
```

### Link Resolution Strategy

Obsidian resolves links in this order:
1. Exact path match (if path contains `/`)
2. Exact filename match (case-insensitive on macOS/Windows)
3. Alias match from frontmatter

We'll implement this in `graph/resolution.rs`.

## Implementation Order

### Step 1: Link Graph Infrastructure
1. Create `graph/mod.rs` and `graph/link_graph.rs`
2. Implement `LinkGraph::build(vault: &Vault)` that:
   - Iterates all notes
   - Parses links from body content
   - Parses links from frontmatter values (wikilinks in strings)
   - Tracks context for each link
3. Implement link resolution (path matching, alias lookup)
4. Build incoming link index

### Step 2: Link Commands
1. `get-out-links` - Query outgoing from graph (simplest)
2. `get-in-links` - Query incoming from graph
3. `get-links` - Combined view with filtering
4. `get-embeds` - Filter for embed=true, add media type detection

### Step 3: Tag Commands
1. `get-tags` for specific note - Already have parser, add CLI
2. `get-tags` vault-wide - Iterate all notes, aggregate
3. `--with-counts` - Count occurrences
4. `--nested` - Build hierarchy from tag paths

### Step 4: Block Commands
1. `get-blocks` - List block IDs in note (have parser)
2. `get-block-refs` - Find links with block_id targeting this note

### Step 5: Heading Commands
1. `get-headings` flat - Already have parser
2. `get-headings --nested` - Build tree structure
3. `get-section` - Extract content between headings

### Step 6: Rename with Propagation
1. `rename --no-propagate` - Simple file move
2. `rename` with propagation:
   - Build link graph
   - Find all incoming links
   - Update each source file
   - Handle aliases in links
   - Handle heading/block references
3. `--dry-run` mode

### Step 7: Tag Filtering for Search
1. Add `--tag`, `--tag-any`, `--no-tag` to `search` command

## Key Design Decisions

### 1. Graph Building Strategy
- Build graph on-demand (not cached yet - that's Phase 5)
- Parse all notes when any graph query is needed
- Keep graph in memory for duration of command

### 2. Link Context Detection
- Body: Default for links not in frontmatter/inline attrs
- Frontmatter: Parse YAML values for wikilink patterns
- Inline: Already tracked by inline_attr parser
- Task: Detect if link is inside a task line (starts with `- [ ]` etc.)

### 3. Frontmatter Link Parsing
Frontmatter can contain wikilinks in string values:
```yaml
parent: "[[Other Note]]"
related:
  - "[[Note A]]"
  - "[[Note B]]"
```
Need to scan string values for wikilink patterns.

### 4. Section Extraction
For `get-section`, a section includes:
- The heading line
- All content until the next heading of same or higher level
- Optionally includes subheadings (default: yes)

## Testing Strategy

### Unit Tests
- Link graph building with various link types
- Link resolution (exact path, filename, alias)
- Incoming link indexing
- Tag aggregation and hierarchy building
- Section extraction edge cases

### Integration Tests (add to fixtures)
- `fixtures/links/` already has good link variety
- Add notes with frontmatter links for context testing
- Test rename propagation with multiple incoming links

## Open Questions (To Resolve)

1. **Case sensitivity for link resolution?**
   - Decision: Case-insensitive on macOS (matches Obsidian), configurable later

2. **How to handle broken links in graph?**
   - Decision: Include them with `resolved_path: None`, let commands filter

3. **Rename: what if target path exists?**
   - Decision: Error with exit code, suggest --force (don't implement --force yet)

## Success Criteria

Phase 2 is complete when:
- [ ] Link graph builds correctly from vault
- [ ] All link commands work (get-links, get-in-links, get-out-links, get-embeds)
- [ ] Tag commands work with counts and nested output
- [ ] Block commands work (get-blocks, get-block-refs)
- [ ] Heading commands work with nested output
- [ ] get-section extracts correct content
- [ ] rename propagates link updates correctly
- [ ] --dry-run shows accurate preview
- [ ] Integration tests cover all commands
