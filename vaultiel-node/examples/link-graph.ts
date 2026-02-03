/**
 * Link Graph Analysis
 *
 * Demonstrates how to analyze the link structure of a vault.
 */

import { Vault, LinkRef } from '@vaultiel/node';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';

function createDemoVault(vault: Vault): void {
    vault.createNote(
        'Hub.md',
        `---
title: Hub Note
type: hub
---

# Hub Note

This is a central hub that links to many notes.

## Connected Notes

- [[Spoke A]]
- [[Spoke B]]
- [[Spoke C]]

## Embedded Content

![[Spoke A#Summary]]

#hub
`
    );

    vault.createNote(
        'Spoke A.md',
        `---
title: Spoke A
---

# Spoke A

## Summary

This is Spoke A, connected to [[Hub]].

Also links to [[Spoke B]].

^summary

#spoke
`
    );

    vault.createNote(
        'Spoke B.md',
        `---
title: Spoke B
links:
  - "[[Hub]]"
---

# Spoke B

Connected to [[Hub]] and [[Spoke A]].

#spoke
`
    );

    vault.createNote(
        'Spoke C.md',
        `---
title: Spoke C
---

# Spoke C

Only connected to [[Hub]].

#spoke
`
    );

    vault.createNote(
        'Orphan.md',
        `---
title: Orphan Note
---

# Orphan Note

This note has no incoming links. It's isolated.

#orphan
`
    );
}

function main() {
    const vaultPath = fs.mkdtempSync(path.join(os.tmpdir(), 'vaultiel-graph-demo-'));
    console.log('=== Vaultiel Node.js Demo: Link Graph ===');
    console.log(`Using vault: ${vaultPath}\n`);

    const vault = new Vault(vaultPath);
    createDemoVault(vault);
    console.log('Created demo vault with interconnected notes\n');

    // --- Outgoing Links ---
    console.log('--- Outgoing Links from Hub.md ---');
    const outgoing = vault.getOutgoingLinks('Hub.md');
    outgoing.forEach(ref => {
        const linkType = ref.embed ? 'embed' : 'link';
        console.log(`  [${linkType}] -> ${ref.from} (line ${ref.line}, context: ${ref.context})`);
    });
    console.log();

    // --- Incoming Links (Backlinks) ---
    console.log('--- Incoming Links to Hub.md (Backlinks) ---');
    const incoming = vault.getIncomingLinks('Hub.md');
    console.log(`Hub.md has ${incoming.length} incoming links:`);
    incoming.forEach(ref => {
        console.log(`  <- ${ref.from} (line ${ref.line}, context: ${ref.context})`);
    });
    console.log();

    // --- Build Complete Link Graph ---
    console.log('--- Building Complete Link Graph ---');

    const graph = new Map<string, string[]>();
    const allNotes = vault.listNotes();

    for (const note of allNotes) {
        const links = vault.getLinks(note);
        const targets: string[] = [];

        for (const link of links) {
            if (!link.embed) {
                try {
                    const target = vault.resolveNote(link.target);
                    targets.push(target);
                } catch {
                    // Broken link
                }
            }
        }

        graph.set(note, targets);
    }

    console.log('Link graph (source -> targets):');
    for (const [source, targets] of Array.from(graph.entries()).sort()) {
        const targetsStr = targets.length > 0 ? targets.join(', ') : '(none)';
        console.log(`  ${source} -> ${targetsStr}`);
    }
    console.log();

    // --- Calculate Incoming Link Counts ---
    console.log('--- Incoming Link Counts ---');
    const incomingCounts = new Map<string, number>();

    // Initialize all notes with 0
    for (const note of allNotes) {
        incomingCounts.set(note, 0);
    }

    // Count incoming links
    for (const targets of graph.values()) {
        for (const target of targets) {
            incomingCounts.set(target, (incomingCounts.get(target) || 0) + 1);
        }
    }

    // Sort by count
    const sortedCounts = Array.from(incomingCounts.entries()).sort((a, b) => b[1] - a[1]);

    console.log('Notes ranked by incoming links:');
    for (const [note, count] of sortedCounts) {
        console.log(`  ${count.toString().padStart(2)} links -> ${note}`);
    }

    // Find orphans
    const orphans = sortedCounts.filter(([, count]) => count === 0).map(([note]) => note);
    console.log(`\nNotes with no incoming links (orphans): ${JSON.stringify(orphans)}`);
    console.log();

    // --- Find Orphans (Alternative Method) ---
    console.log('--- Finding Orphans (Alternative Method) ---');
    const orphanNotes: string[] = [];
    for (const note of allNotes) {
        const noteIncoming = vault.getIncomingLinks(note);
        if (noteIncoming.length === 0) {
            orphanNotes.push(note);
        }
    }
    console.log(`Orphan notes: ${JSON.stringify(orphanNotes)}`);
    console.log();

    // --- Analyze Link Contexts ---
    console.log('--- Link Context Analysis ---');
    const contextCounts = new Map<string, number>();

    for (const note of allNotes) {
        const noteIncoming = vault.getIncomingLinks(note);
        for (const ref of noteIncoming) {
            const ctx = ref.context.includes(':') ? ref.context.split(':')[0] : ref.context;
            contextCounts.set(ctx, (contextCounts.get(ctx) || 0) + 1);
        }
    }

    console.log('Links by context:');
    const sortedContexts = Array.from(contextCounts.entries()).sort((a, b) => b[1] - a[1]);
    for (const [ctx, count] of sortedContexts) {
        console.log(`  ${ctx}: ${count}`);
    }
    console.log();

    // --- Most Connected Notes ---
    console.log('--- Most Connected Notes (by outgoing links) ---');
    const outgoingCounts = Array.from(graph.entries())
        .map(([note, targets]) => ({ note, count: targets.length }))
        .sort((a, b) => b.count - a.count);

    for (const { note, count } of outgoingCounts.slice(0, 5)) {
        console.log(`  ${count.toString().padStart(2)} outgoing links from ${note}`);
    }
    console.log();

    // --- Cleanup Info ---
    console.log(`Demo complete. Vault at: ${vaultPath}`);
    console.log(`To clean up: rm -rf ${vaultPath}`);
}

main();
