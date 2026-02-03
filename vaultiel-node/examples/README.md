# Vaultiel Node.js Examples

TypeScript examples demonstrating the Vaultiel Node.js bindings.

## Prerequisites

1. Install dependencies:
   ```bash
   npm install @vaultiel/node
   # or
   yarn add @vaultiel/node
   ```

2. For TypeScript examples, you'll need `ts-node` or compile to JavaScript:
   ```bash
   npm install -D typescript ts-node @types/node
   ```

## Examples

### basic-operations.ts

Demonstrates fundamental vault operations:
- Creating notes with frontmatter
- Listing and filtering notes
- Reading content and frontmatter
- Resolving note names and aliases
- Parsing links, tags, headings, and tasks

```bash
npx ts-node basic-operations.ts
```

### link-graph.ts

Demonstrates link graph analysis:
- Building the complete link graph
- Finding incoming/outgoing links
- Calculating link statistics
- Finding orphan notes
- Analyzing link contexts

```bash
npx ts-node link-graph.ts
```

### task-analysis.ts

Demonstrates task extraction and analysis:
- Extracting tasks from notes
- Filtering by status, date, priority
- Finding overdue tasks
- Analyzing task hierarchy
- Task statistics

```bash
npx ts-node task-analysis.ts
```

### standalone-parsing.ts

Demonstrates parsing without a vault:
- Parsing links from arbitrary content
- Extracting tags from strings
- Building tables of contents from headings
- Processing content from other sources

```bash
npx ts-node standalone-parsing.ts
```

## Running All Examples

```bash
# Install ts-node if not already installed
npm install -D ts-node typescript @types/node

# Run all examples
for script in *.ts; do
    echo "=== Running $script ==="
    npx ts-node "$script"
    echo
done
```

## Common Patterns

### Error Handling

```typescript
import { Vault } from '@vaultiel/node';

const vault = new Vault('/path/to/vault');

// Check before accessing
if (vault.noteExists('note.md')) {
    const content = vault.getContent('note.md');
}

// Or use try/catch
try {
    const content = vault.getContent('maybe.md');
} catch (error) {
    console.error('Error:', (error as Error).message);
}
```

### Working with Frontmatter

```typescript
// Get as JSON string and parse
const fmJson = vault.getFrontmatter('note.md');
if (fmJson) {
    const fm = JSON.parse(fmJson);
    const title = fm.title || 'Untitled';
    const tags = fm.tags || [];
}
```

### Batch Processing

```typescript
import { Vault, Task } from '@vaultiel/node';

const vault = new Vault('/path/to/vault');

// Process all notes
for (const note of vault.listNotes()) {
    const links = vault.getLinks(note);
    // ... process links
}

// Process matching notes
for (const note of vault.listNotesMatching('projects/*.md')) {
    const tasks = vault.getTasks(note);
    // ... process tasks
}
```

### Async Wrapper (for Promise-based code)

```typescript
import { Vault } from '@vaultiel/node';

async function processVault(vaultPath: string): Promise<void> {
    const vault = new Vault(vaultPath);

    // Wrap synchronous operations if needed
    const notes = vault.listNotes();

    await Promise.all(notes.map(async (note) => {
        const links = vault.getLinks(note);
        // ... async processing
    }));
}
```

### Building Link Statistics

```typescript
const incomingCounts = new Map<string, number>();

for (const note of vault.listNotes()) {
    for (const link of vault.getLinks(note)) {
        try {
            const target = vault.resolveNote(link.target);
            incomingCounts.set(target, (incomingCounts.get(target) || 0) + 1);
        } catch {
            // Broken link
        }
    }
}

const mostLinked = [...incomingCounts.entries()]
    .sort((a, b) => b[1] - a[1])
    .slice(0, 10);
```

### Finding Orphans

```typescript
const orphans: string[] = [];
for (const note of vault.listNotes()) {
    const incoming = vault.getIncomingLinks(note);
    if (incoming.length === 0) {
        orphans.push(note);
    }
}
```

### Task Filtering

```typescript
const today = new Date().toISOString().split('T')[0];

// Get all tasks
const allTasks: Task[] = [];
for (const note of vault.listNotes()) {
    allTasks.push(...vault.getTasks(note));
}

// Filter
const incomplete = allTasks.filter(t => t.symbol === '[ ]');
const dueToday = incomplete.filter(t => t.due === today);
const highPriority = incomplete.filter(t => t.priority === 'high');
const overdue = incomplete.filter(t => t.due && t.due < today);
```

## Integration Ideas

### Express.js API

```typescript
import express from 'express';
import { Vault } from '@vaultiel/node';

const app = express();
const vault = new Vault('/path/to/vault');

app.get('/api/notes', (req, res) => {
    const notes = vault.listNotes();
    res.json({ notes, count: notes.length });
});

app.get('/api/notes/:path/links', (req, res) => {
    try {
        const links = vault.getLinks(req.params.path);
        res.json({ links });
    } catch (error) {
        res.status(404).json({ error: 'Note not found' });
    }
});

app.listen(3000);
```

### Obsidian Plugin

```typescript
import { Vault } from '@vaultiel/node';
import { App, Plugin } from 'obsidian';

export default class MyPlugin extends Plugin {
    vault: Vault | null = null;

    async onload() {
        const vaultPath = (this.app.vault.adapter as any).basePath;
        this.vault = new Vault(vaultPath);

        this.addCommand({
            id: 'find-orphans',
            name: 'Find orphan notes',
            callback: () => {
                if (!this.vault) return;

                const orphans = this.vault.listNotes().filter(note => {
                    return this.vault!.getIncomingLinks(note).length === 0;
                });

                console.log('Orphan notes:', orphans);
            },
        });
    }
}
```

### CLI Tool with Commander

```typescript
import { Command } from 'commander';
import { Vault } from '@vaultiel/node';

const program = new Command();

program
    .name('my-vault-tool')
    .argument('<vault>', 'Path to vault')
    .option('-o, --orphans', 'Find orphan notes')
    .action((vaultPath, options) => {
        const vault = new Vault(vaultPath);

        if (options.orphans) {
            const orphans = vault.listNotes().filter(note =>
                vault.getIncomingLinks(note).length === 0
            );
            orphans.forEach(n => console.log(n));
        } else {
            vault.listNotes().forEach(n => console.log(n));
        }
    });

program.parse();
```

## TypeScript Configuration

For these examples, use this `tsconfig.json`:

```json
{
    "compilerOptions": {
        "target": "ES2020",
        "module": "commonjs",
        "strict": true,
        "esModuleInterop": true,
        "skipLibCheck": true,
        "forceConsistentCasingInFileNames": true,
        "outDir": "./dist"
    },
    "include": ["*.ts"]
}
```

## Cleanup

Each example creates a temporary vault. They are automatically cleaned up on restart, but you can manually remove them:

```bash
rm -rf /tmp/vaultiel-*-demo
```
