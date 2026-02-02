# Phase 3: Tasks - Implementation Plan

## Overview

Implement full Obsidian Tasks plugin compatibility with task extraction, filtering, hierarchy tracking, and formatting.

## Architecture

### New Modules

1. **`parser/task.rs`** - Task parsing logic
   - Parse task markers (`- [ ]`, `- [x]`, `- [>]`, etc.)
   - Extract Obsidian Tasks metadata (due, scheduled, done dates)
   - Extract priority markers
   - Extract custom metadata fields
   - Extract links, tags, and block IDs within tasks
   - Track indentation levels

2. **`cli/tasks.rs`** - CLI commands
   - `get-tasks` with filtering
   - `format-task` for generating task strings

### Types

```rust
/// A parsed task from the vault.
pub struct Task {
    pub location: TaskLocation,
    pub raw: String,
    pub symbol: String,
    pub description: String,
    pub indent: usize,
    pub parent_line: Option<usize>,
    pub scheduled: Option<NaiveDate>,
    pub due: Option<NaiveDate>,
    pub done: Option<NaiveDate>,
    pub priority: Option<Priority>,
    pub custom: HashMap<String, String>,
    pub links: Vec<TaskLink>,
    pub tags: Vec<String>,
    pub block_id: Option<String>,
}

pub struct TaskLocation {
    pub file: PathBuf,
    pub line: usize,
}

pub struct TaskLink {
    pub to: String,
    pub alias: Option<String>,
}

pub enum Priority {
    Highest,
    High,
    Medium,
    Low,
    Lowest,
}
```

### Task Metadata Symbols (configurable)

Default Obsidian Tasks symbols:
- Due: `📅`
- Scheduled: `⏳`
- Done: `✅`
- Priority Highest: `🔺`
- Priority High: `⏫`
- Priority Medium: `🔼`
- Priority Low: `🔽`
- Priority Lowest: `⏬`

## Implementation Order

1. **Task Parser Core**
   - Parse task line pattern: `^(\s*)- \[(.)\] (.*)$`
   - Extract symbol from checkbox
   - Parse description (everything after `] `)

2. **Metadata Extraction**
   - Extract dates with emoji prefixes
   - Extract priority markers
   - Extract custom metadata from config
   - Parse order: custom → standard (per spec)

3. **Content Extraction**
   - Extract wikilinks from description
   - Extract tags from description
   - Extract block ID (always at end)

4. **Hierarchy Building**
   - Track indentation (tabs or 4 spaces)
   - Build parent/child relationships
   - Support nested output (tree) and flat output

5. **Filtering**
   - By note/glob pattern
   - By symbol
   - By date ranges (due, scheduled, done)
   - By priority
   - By text content
   - By custom metadata presence
   - By linked note
   - By tag
   - By block reference

6. **Format Command**
   - Generate task strings
   - Support relative dates (today, tomorrow, +3d)
   - Apply configured symbols

## CLI Arguments

```
get-tasks [OPTIONS]

Options:
  --note <PATH>           Filter to tasks in specific note
  --glob <PATTERN>        Filter to tasks in notes matching glob
  --symbol <SYMBOL>       Filter by task marker (repeatable)
  --due-before <DATE>     Due date before (exclusive)
  --due-after <DATE>      Due date after (exclusive)
  --due-on <DATE>         Due on specific date
  --scheduled-before <DATE>
  --scheduled-after <DATE>
  --scheduled-on <DATE>
  --done-before <DATE>
  --done-after <DATE>
  --done-on <DATE>
  --priority <LEVEL>      Filter by priority
  --contains <TEXT>       Filter by description text
  --has <KEY>             Filter by custom metadata presence
  --links-to <PATH>       Filter to tasks linking to note
  --tag <TAG>             Filter by tag
  --has-block-ref         Filter to tasks with block refs
  --block-ref <ID>        Filter by specific block ref
  --flat                  Return flat list instead of hierarchy

format-task [OPTIONS]
  --desc <TEXT>           Task description (required)
  --symbol <SYMBOL>       Task symbol (default: "[ ]")
  --due <DATE>            Due date
  --scheduled <DATE>      Scheduled date
  --done <DATE>           Done date
  --priority <LEVEL>      Priority level
  --<custom-key> <VALUE>  Custom metadata (from config)
```

## Date Parsing

Support formats:
- ISO dates: `2026-02-10`
- Relative: `today`, `tomorrow`, `yesterday`
- Offsets: `+3d`, `-1w`, `+2m` (days, weeks, months)

## Output Format

### Hierarchical (default)
```json
{
  "tasks": [
    {
      "location": {"file": "path.md", "line": 10},
      "raw": "- [ ] Parent task",
      "symbol": "[ ]",
      "description": "Parent task",
      "indent": 0,
      "parent_line": null,
      "children": [...],
      "scheduled": null,
      "due": null,
      "done": null,
      "priority": null,
      "custom": {},
      "links": [],
      "tags": [],
      "block_id": null
    }
  ]
}
```

### Flat (--flat)
Same structure but no `children` field, all tasks at top level.

## Test Fixtures

Create `fixtures/tasks/` with:
- Simple tasks
- Nested tasks (hierarchy)
- Tasks with dates
- Tasks with priorities
- Tasks with links and tags
- Tasks with custom metadata
- Tasks with block IDs
