# Phase 4: Vault Health & Info - Implementation Plan

## Overview

Implement vault statistics, health checks, and linting capabilities.

## Architecture

### New Modules

1. **`cli/info.rs`** - Vault info command
   - Basic stats (note count, link count, etc.)
   - Detailed stats (by folder, top tags, top linked)

2. **`cli/lint.rs`** - Lint command and issue detection
   - Issue type definitions
   - Detection logic for each issue type
   - Auto-fix capabilities
   - Output formatting (JSON, text, GitHub Actions)

3. **`health/mod.rs`** - Health check infrastructure
   - Issue type enum
   - Issue struct
   - Check runners

### Issue Types

```rust
pub enum IssueType {
    BrokenLinks,
    BrokenEmbeds,
    BrokenHeadingLinks,
    BrokenBlockRefs,
    Orphans,
    DuplicateAliases,
    DuplicateBlockIds,
    EmptyNotes,
    MissingFrontmatter,
    InvalidFrontmatter,
}

pub struct Issue {
    pub issue_type: IssueType,
    pub file: PathBuf,
    pub line: Option<usize>,
    pub message: String,
    pub target: Option<String>,
    pub fixable: bool,
}
```

## Implementation Order

1. **Info Command**
   - Basic stats collection
   - Detailed stats with --detailed flag

2. **Health Check Infrastructure**
   - Issue type definitions
   - Issue struct and output format

3. **Issue Detectors**
   - Broken links (note doesn't exist)
   - Broken embeds (file doesn't exist)
   - Broken heading links (heading doesn't exist)
   - Broken block refs (block ID doesn't exist)
   - Orphans (no incoming links)
   - Duplicate aliases (same alias in multiple notes)
   - Duplicate block IDs (same ID twice in one note)
   - Empty notes (no content)
   - Missing frontmatter
   - Invalid frontmatter (YAML parse error)

4. **Lint Command**
   - Run all checks
   - Filter by --only / --ignore
   - Scope with --glob
   - --fail-on for CI
   - --format for output format

5. **Auto-fix**
   - Missing frontmatter (add empty)
   - Duplicate block IDs (rename)

6. **Shorthand Commands**
   - find-orphans
   - find-broken-links

## CLI Arguments

```
info [OPTIONS]
  --detailed        Include extended statistics

lint [OPTIONS]
  --fix             Auto-fix issues where possible
  --only <TYPE>     Only check specific type (repeatable)
  --ignore <TYPE>   Skip specific type (repeatable)
  --glob <PATTERN>  Check only matching notes
  --fail-on <TYPE>  Exit non-zero if type found (repeatable)
  --format <FMT>    Output format: json, text, github

find-orphans [OPTIONS]
  --exclude <PATTERN>  Exclude notes matching pattern (repeatable)

find-broken-links [OPTIONS]
  --note <PATH>     Check specific note only
```

## Output Formats

### Info Basic
```json
{
  "vault_path": "/path/to/vault",
  "note_count": 1543,
  "total_size_bytes": 15234567,
  "link_count": 8921,
  "tag_count": 234,
  "task_count": 456,
  "orphan_count": 23,
  "broken_link_count": 5
}
```

### Lint Output
```json
{
  "issues": [...],
  "summary": {
    "total": 3,
    "by_type": {...},
    "fixable": 1
  }
}
```

### GitHub Actions Format
```
::error file=proj/Vaultiel.md,line=45::Link to non-existent note: [[Old Design Doc]]
::warning file=cap/random.md::Note has no incoming links
```
