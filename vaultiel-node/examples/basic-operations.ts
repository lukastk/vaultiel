/**
 * Basic Vaultiel Operations
 *
 * Demonstrates fundamental vault operations using the Node.js bindings.
 */

import { Vault } from '@vaultiel/node';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';

function main() {
    // Create a temporary vault for demonstration
    const vaultPath = fs.mkdtempSync(path.join(os.tmpdir(), 'vaultiel-demo-'));
    console.log('=== Vaultiel Node.js Demo: Basic Operations ===');
    console.log(`Using vault: ${vaultPath}\n`);

    // Initialize vault
    const vault = new Vault(vaultPath);
    console.log(`Vault root: ${vault.root}\n`);

    // --- Creating Notes ---
    console.log('--- Creating Notes ---');

    vault.createNote(
        'Welcome.md',
        `---
title: Welcome
tags:
  - demo
  - intro
---

# Welcome to Vaultiel

This is a demo vault created with the Node.js bindings.

## Features

- Fast markdown parsing
- Link graph traversal
- Task extraction

See [[Getting Started]] for more information.
`
    );
    console.log('Created Welcome.md');

    vault.createNote(
        'Getting Started.md',
        `---
title: Getting Started
aliases:
  - quickstart
  - tutorial
---

# Getting Started

Welcome to the tutorial! Check out [[Welcome]] if you haven't already.

## Installation

\`\`\`bash
npm install @vaultiel/node
\`\`\`

## Next Steps

- Explore the API
- Build something cool!

#tutorial #beginner
`
    );
    console.log('Created Getting Started.md');

    vault.createNote(
        'Project Notes.md',
        `---
title: Project Notes
type: project
status: active
---

# Project Notes

## Tasks

- [ ] Implement feature A 📅 2024-02-15 ⏫
- [ ] Write documentation
- [x] Initial setup ✅ 2024-01-10

## Links

Related to [[Welcome]] and [[Getting Started]].

#project
`
    );
    console.log('Created Project Notes.md');
    console.log();

    // --- Listing Notes ---
    console.log('--- Listing Notes ---');
    const notes = vault.listNotes();
    console.log(`All notes (${notes.length}):`);
    notes.forEach(note => console.log(`  - ${note}`));
    console.log();

    // List with glob pattern
    console.log("Notes matching 'Project*':");
    const matching = vault.listNotesMatching('Project*');
    matching.forEach(note => console.log(`  - ${note}`));
    console.log();

    // --- Reading Content ---
    console.log('--- Reading Content ---');

    // Full content
    const content = vault.getContent('Welcome.md');
    console.log('Full content of Welcome.md:');
    console.log(content.substring(0, 200) + '...\n');

    // Body only (without frontmatter)
    const body = vault.getBody('Welcome.md');
    console.log('Body of Welcome.md:');
    console.log(body.substring(0, 150) + '...\n');

    // --- Frontmatter ---
    console.log('--- Frontmatter ---');

    const fmJson = vault.getFrontmatter('Getting Started.md');
    if (fmJson) {
        const fm = JSON.parse(fmJson);
        console.log('Frontmatter of Getting Started.md:');
        console.log(`  Title: ${fm.title}`);
        console.log(`  Aliases: ${JSON.stringify(fm.aliases)}`);
    }
    console.log();

    // --- Note Resolution ---
    console.log('--- Note Resolution ---');

    // Resolve by name
    let resolvedPath = vault.resolveNote('Welcome');
    console.log(`'Welcome' resolves to: ${resolvedPath}`);

    // Resolve by alias
    resolvedPath = vault.resolveNote('quickstart');
    console.log(`'quickstart' resolves to: ${resolvedPath}`);

    // Check existence
    const exists = vault.noteExists('Welcome.md');
    console.log(`Welcome.md exists: ${exists}`);

    const missing = vault.noteExists('NonExistent.md');
    console.log(`NonExistent.md exists: ${missing}`);
    console.log();

    // --- Parsing ---
    console.log('--- Parsing Links and Tags ---');

    const links = vault.getLinks('Project Notes.md');
    console.log(`Links in Project Notes.md (${links.length}):`);
    links.forEach(link => {
        console.log(`  - [[${link.target}]] at line ${link.line}`);
    });

    const tags = vault.getTags('Project Notes.md');
    console.log(`Tags in Project Notes.md (${tags.length}):`);
    tags.forEach(tag => {
        console.log(`  - #${tag.name} at line ${tag.line}`);
    });
    console.log();

    // --- Headings ---
    console.log('--- Headings ---');

    const headings = vault.getHeadings('Welcome.md');
    console.log('Headings in Welcome.md:');
    headings.forEach(h => {
        const indent = '  '.repeat(h.level - 1);
        console.log(`${indent}${'#'.repeat(h.level)} ${h.text} (line ${h.line})`);
    });
    console.log();

    // --- Tasks ---
    console.log('--- Tasks ---');

    const tasks = vault.getTasks('Project Notes.md');
    console.log(`Tasks in Project Notes.md (${tasks.length}):`);
    tasks.forEach(task => {
        const status = task.symbol === '[x]' ? '✓' : '○';
        console.log(`  ${status} ${task.description}`);
        if (task.due) console.log(`      Due: ${task.due}`);
        if (task.priority) console.log(`      Priority: ${task.priority}`);
    });
    console.log();

    // --- Cleanup Info ---
    console.log('--- Demo Complete ---');
    console.log(`Vault created at: ${vaultPath}`);
    console.log(`To clean up: rm -rf ${vaultPath}`);
}

main();
