# Vaultiel Node.js Bindings

Node.js/TypeScript bindings for Vaultiel - a library for programmatically interacting with Obsidian-style vaults.

Built with [napi-rs](https://napi.rs/) for high performance with full TypeScript support.

## Installation

```bash
npm install @vaultiel/node
# or
yarn add @vaultiel/node
# or
pnpm add @vaultiel/node
```

## Quick Start

```typescript
import { Vault } from '@vaultiel/node';

// Open a vault
const vault = new Vault('/path/to/your/vault');

// List all notes
const notes = vault.listNotes();
console.log(`Found ${notes.length} notes`);

// Get note content
const content = vault.getContent('my-note.md');
console.log(content);

// Get links from a note
const links = vault.getLinks('my-note.md');
links.forEach(link => {
    console.log(`Link to ${link.target} at line ${link.line}`);
});

// Get incoming links (backlinks)
const backlinks = vault.getIncomingLinks('my-note.md');
console.log(`${backlinks.length} notes link to this note`);
```

## Features

- **Fast**: Built on Rust for high performance
- **Full Obsidian compatibility**: Parses wikilinks, embeds, tags, block IDs, and tasks
- **Link graph**: Query incoming and outgoing links with context metadata
- **TypeScript native**: Full type definitions included
- **Cross-platform**: macOS, Linux, and Windows support

## API Reference

### Vault Class

The main entry point for vault operations.

```typescript
import { Vault } from '@vaultiel/node';

const vault = new Vault('/path/to/vault');
```

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `root` | `string` | Vault root directory path |

#### Note Operations

```typescript
// List all notes
const notes: string[] = vault.listNotes();

// List notes matching glob pattern
const projNotes: string[] = vault.listNotesMatching('proj/*.md');

// Check if note exists
const exists: boolean = vault.noteExists('my-note.md');

// Get full content (including frontmatter)
const content: string = vault.getContent('my-note.md');

// Get body only (without frontmatter)
const body: string = vault.getBody('my-note.md');

// Get frontmatter as JSON string
const fmJson: string | null = vault.getFrontmatter('my-note.md');
if (fmJson) {
    const frontmatter = JSON.parse(fmJson);
    console.log(frontmatter.title);
}

// Create a note
vault.createNote('new-note.md', '---\ntitle: New Note\n---\n\nContent here.');

// Delete a note
vault.deleteNote('old-note.md');

// Rename a note (no link propagation)
vault.renameNote('old-name.md', 'new-name.md');

// Resolve name/alias to path
const path: string = vault.resolveNote('My Note'); // or alias
```

#### Parsing

```typescript
import { Vault, Link, Tag, Heading, BlockId, Task } from '@vaultiel/node';

const vault = new Vault('/path/to/vault');

// Get links from a note
const links: Link[] = vault.getLinks('my-note.md');
links.forEach(link => {
    console.log(`Link to ${link.target} at line ${link.line}`);
    if (link.alias) console.log(`  Alias: ${link.alias}`);
    if (link.heading) console.log(`  Heading: #${link.heading}`);
    if (link.blockId) console.log(`  Block: ^${link.blockId}`);
    if (link.embed) console.log('  (embedded)');
});

// Get tags from a note
const tags: Tag[] = vault.getTags('my-note.md');
tags.forEach(tag => {
    console.log(`#${tag.name} at line ${tag.line}`);
});

// Get headings from a note
const headings: Heading[] = vault.getHeadings('my-note.md');
headings.forEach(h => {
    console.log(`${'#'.repeat(h.level)} ${h.text} (slug: ${h.slug})`);
});

// Get block IDs from a note
const blocks: BlockId[] = vault.getBlockIds('my-note.md');
blocks.forEach(block => {
    console.log(`^${block.id} (${block.blockType}) at line ${block.line}`);
});

// Get tasks from a note
const tasks: Task[] = vault.getTasks('my-note.md');
tasks.forEach(task => {
    console.log(`[${task.symbol}] ${task.description}`);
    if (task.due) console.log(`  Due: ${task.due}`);
    if (task.scheduled) console.log(`  Scheduled: ${task.scheduled}`);
    if (task.priority) console.log(`  Priority: ${task.priority}`);
    task.tags.forEach(tag => console.log(`  Tag: ${tag}`));
});
```

#### Link Graph

```typescript
import { Vault, LinkRef } from '@vaultiel/node';

const vault = new Vault('/path/to/vault');

// Get incoming links (backlinks)
const incoming: LinkRef[] = vault.getIncomingLinks('my-note.md');
incoming.forEach(ref => {
    console.log(`Linked from ${ref.from} at line ${ref.line}`);
    console.log(`  Context: ${ref.context}`); // body, frontmatter:key, task, etc.
});

// Get outgoing links
const outgoing: LinkRef[] = vault.getOutgoingLinks('my-note.md');
```

#### Metadata

```typescript
import { Vault, VaultielMetadata } from '@vaultiel/node';

const vault = new Vault('/path/to/vault');

// Initialize vaultiel metadata (UUID + timestamp)
const metadata: VaultielMetadata | null = vault.initMetadata('my-note.md', false);
if (metadata) {
    console.log(`ID: ${metadata.id}`);
    console.log(`Created: ${metadata.created}`);
}

// Initialize with force (overwrite existing)
vault.initMetadata('my-note.md', true);

// Get existing metadata
const existing = vault.getVaultielMetadata('my-note.md');

// Find note by UUID
const path: string | null = vault.findById('550e8400-e29b-41d4-a716-446655440000');
```

### Standalone Parsing Functions

Parse content without a vault:

```typescript
import {
    parseLinks,
    parseContentTags,
    parseContentHeadings,
    parseContentBlockIds,
} from '@vaultiel/node';

const content = `
# My Note

This links to [[Other Note]] and [[Another|with alias]].

Has #tags and #nested/tags too.

Important block ^my-block
`;

const links = parseLinks(content);
const tags = parseContentTags(content);
const headings = parseContentHeadings(content);
const blocks = parseContentBlockIds(content);
```

### Type Definitions

#### NoteInfo

```typescript
interface NoteInfo {
    path: string;
    name: string;
    modified?: string;
    created?: string;
    sizeBytes: number;
}
```

#### Link

```typescript
interface Link {
    target: string;       // Target path or name
    alias?: string;       // Display alias [[target|alias]]
    heading?: string;     // Heading reference [[note#heading]]
    blockId?: string;     // Block reference [[note#^block]]
    embed: boolean;       // True for ![[embeds]]
    line: number;         // Line number (1-indexed)
}
```

#### Tag

```typescript
interface Tag {
    name: string;         // Tag name (without #)
    line: number;         // Line number
}
```

#### Heading

```typescript
interface Heading {
    text: string;         // Heading text
    level: number;        // 1-6
    line: number;         // Line number
    slug: string;         // URL slug (lowercase, hyphens)
}
```

#### BlockId

```typescript
interface BlockId {
    id: string;           // Block ID (without ^)
    line: number;         // Line number
    blockType: string;    // paragraph, list-item, etc.
}
```

#### Task

```typescript
interface Task {
    file: string;         // Source file path
    line: number;         // Line number
    raw: string;          // Raw task line
    symbol: string;       // Task marker: [ ], [x], [>], etc.
    description: string;  // Task text
    indent: number;       // Indentation level
    scheduled?: string;   // Scheduled date (YYYY-MM-DD)
    due?: string;         // Due date
    done?: string;        // Completion date
    priority?: string;    // Priority level
    tags: string[];       // Tags in task
    blockId?: string;     // Block ID on task
}
```

#### LinkRef

```typescript
interface LinkRef {
    from: string;         // Source note path
    line: number;         // Line number
    context: string;      // Where link appears (body, frontmatter:key, task)
    alias?: string;       // Link alias
    heading?: string;     // Heading reference
    blockId?: string;     // Block reference
    embed: boolean;       // True for embeds
}
```

#### VaultielMetadata

```typescript
interface VaultielMetadata {
    id: string;           // UUID
    created: string;      // ISO 8601 timestamp
}
```

## Examples

### Find Orphan Notes

```typescript
import { Vault } from '@vaultiel/node';

const vault = new Vault('/path/to/vault');

const notes = vault.listNotes();
const orphans: string[] = [];

for (const note of notes) {
    const incoming = vault.getIncomingLinks(note);
    if (incoming.length === 0) {
        orphans.push(note);
    }
}

console.log(`Found ${orphans.length} orphan notes:`);
orphans.forEach(orphan => console.log(`  - ${orphan}`));
```

### Extract All Tasks

```typescript
import { Vault, Task } from '@vaultiel/node';

const vault = new Vault('/path/to/vault');

const allTasks: Task[] = [];
for (const note of vault.listNotes()) {
    const tasks = vault.getTasks(note);
    allTasks.push(...tasks);
}

// Filter incomplete tasks
const incomplete = allTasks.filter(t => t.symbol === '[ ]');

// Filter by due date
const today = new Date().toISOString().split('T')[0];
const dueToday = incomplete.filter(t => t.due === today);

console.log(`Tasks due today: ${dueToday.length}`);
dueToday.forEach(task => {
    console.log(`  [${task.symbol}] ${task.description}`);
    console.log(`    File: ${task.file}:${task.line}`);
});
```

### Build Link Graph

```typescript
import { Vault } from '@vaultiel/node';

const vault = new Vault('/path/to/vault');

// Build adjacency list
const graph = new Map<string, string[]>();
for (const note of vault.listNotes()) {
    const links = vault.getLinks(note);
    const targets: string[] = [];

    for (const link of links) {
        try {
            const targetPath = vault.resolveNote(link.target);
            targets.push(targetPath);
        } catch {
            // Broken link
        }
    }

    graph.set(note, targets);
}

// Find most linked notes
const incomingCounts = new Map<string, number>();
for (const [source, targets] of graph) {
    for (const target of targets) {
        incomingCounts.set(target, (incomingCounts.get(target) || 0) + 1);
    }
}

const top10 = [...incomingCounts.entries()]
    .sort((a, b) => b[1] - a[1])
    .slice(0, 10);

console.log('Top 10 most linked notes:');
top10.forEach(([note, count]) => {
    console.log(`  ${count.toString().padStart(3)} links: ${note}`);
});
```

### Process Frontmatter

```typescript
import { Vault } from '@vaultiel/node';

const vault = new Vault('/path/to/vault');

interface ProjectNote {
    path: string;
    title: string;
    status: string;
}

const projectNotes: ProjectNote[] = [];

for (const note of vault.listNotes()) {
    const fmJson = vault.getFrontmatter(note);
    if (fmJson) {
        const fm = JSON.parse(fmJson);
        if (fm.type === 'project') {
            projectNotes.push({
                path: note,
                title: fm.title || note,
                status: fm.status || 'unknown',
            });
        }
    }
}

console.log(JSON.stringify(projectNotes, null, 2));
```

### Obsidian Plugin Integration

```typescript
import { Vault, parseLinks } from '@vaultiel/node';
import { App, Plugin } from 'obsidian';

export default class MyPlugin extends Plugin {
    vault: Vault | null = null;

    async onload() {
        // Initialize vaultiel with the Obsidian vault path
        const vaultPath = (this.app.vault.adapter as any).basePath;
        this.vault = new Vault(vaultPath);

        // Add command to find orphans
        this.addCommand({
            id: 'find-orphans',
            name: 'Find orphan notes',
            callback: () => {
                if (!this.vault) return;

                const notes = this.vault.listNotes();
                const orphans = notes.filter(note => {
                    const incoming = this.vault!.getIncomingLinks(note);
                    return incoming.length === 0;
                });

                console.log('Orphan notes:', orphans);
            },
        });
    }
}
```

## Error Handling

Most methods throw on failure:

```typescript
import { Vault } from '@vaultiel/node';

const vault = new Vault('/path/to/vault');

try {
    const content = vault.getContent('nonexistent.md');
} catch (error) {
    console.error('Error:', error.message);
}

// Check before accessing
if (vault.noteExists('maybe.md')) {
    const content = vault.getContent('maybe.md');
}
```

## Building from Source

Requires Rust (1.70+) and Node.js (18+):

```bash
# Install dependencies
cd vaultiel-node
npm install

# Build debug
npm run build:debug

# Build release
npm run build

# Run tests
npm test
```

## Platform Support

Pre-built binaries are available for:

- macOS (x64, ARM64)
- Linux (x64, ARM64)
- Windows (x64)

If a pre-built binary isn't available for your platform, the package will attempt to build from source (requires Rust).

## Comparison with CLI

The Node.js bindings provide a subset of CLI functionality focused on reading and querying:

| Feature | CLI | Node.js |
|---------|-----|---------|
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
