/**
 * Task Analysis
 *
 * Demonstrates how to extract, filter, and analyze tasks from a vault.
 */

import { Vault, Task } from '@vaultiel/node';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';

function formatDate(date: Date): string {
    return date.toISOString().split('T')[0];
}

function createDemoVault(vault: Vault): void {
    const today = new Date();
    const tomorrow = new Date(today.getTime() + 24 * 60 * 60 * 1000);
    const nextWeek = new Date(today.getTime() + 7 * 24 * 60 * 60 * 1000);
    const yesterday = new Date(today.getTime() - 24 * 60 * 60 * 1000);

    vault.createNote(
        'Inbox.md',
        `---
title: Inbox
---

# Inbox

Quick capture for tasks.

- [ ] Review pull request 📅 ${formatDate(today)} ⏫ #urgent
- [ ] Reply to emails ⏳ ${formatDate(today)}
- [ ] Read article about TypeScript 🔽
- [x] Set up project ✅ ${formatDate(yesterday)}
- [ ] Plan next sprint 📅 ${formatDate(nextWeek)}
`
    );

    // Create Projects directory
    fs.mkdirSync(path.join(vault.root, 'Projects'), { recursive: true });

    vault.createNote(
        'Projects/Alpha.md',
        `---
title: Project Alpha
type: project
status: active
---

# Project Alpha

## Tasks

- [ ] Implement authentication 📅 ${formatDate(tomorrow)} ⏫ ^auth-task
    - [ ] Design login flow
    - [ ] Implement OAuth
    - [ ] Write tests
- [ ] Set up CI/CD 📅 ${formatDate(nextWeek)} 🔼
- [ ] Write documentation
- [x] Create repository ✅ ${formatDate(yesterday)}
- [>] Deferred: Research alternatives

## Notes

See [[Inbox]] for quick tasks.

#project #priority
`
    );

    vault.createNote(
        'Projects/Beta.md',
        `---
title: Project Beta
type: project
status: planning
---

# Project Beta

## Planning Tasks

- [ ] Define requirements 📅 ${formatDate(tomorrow)}
- [ ] Create mockups
- [ ] Estimate timeline

#project
`
    );
}

function analyzeTasks(tasks: Task[]): void {
    if (tasks.length === 0) {
        console.log('No tasks to analyze.');
        return;
    }

    // Count by status
    const statusCounts = new Map<string, number>();
    for (const task of tasks) {
        statusCounts.set(task.symbol, (statusCounts.get(task.symbol) || 0) + 1);
    }

    console.log(`Total tasks: ${tasks.length}`);
    console.log('By status:');
    const statusLabels: Record<string, string> = {
        '[ ]': 'Todo',
        '[x]': 'Done',
        '[>]': 'Deferred',
        '[-]': 'Cancelled',
    };
    for (const [symbol, count] of Array.from(statusCounts.entries()).sort()) {
        const label = statusLabels[symbol] || symbol;
        console.log(`  ${label} (${symbol}): ${count}`);
    }

    // Count by priority
    const priorityCounts = new Map<string, number>();
    for (const task of tasks) {
        const priority = task.priority || 'none';
        priorityCounts.set(priority, (priorityCounts.get(priority) || 0) + 1);
    }

    console.log('By priority:');
    for (const [priority, count] of Array.from(priorityCounts.entries()).sort()) {
        console.log(`  ${priority}: ${count}`);
    }

    // Count by file
    const fileCounts = new Map<string, number>();
    for (const task of tasks) {
        fileCounts.set(task.file, (fileCounts.get(task.file) || 0) + 1);
    }

    console.log('By file:');
    const sortedFiles = Array.from(fileCounts.entries()).sort((a, b) => b[1] - a[1]);
    for (const [file, count] of sortedFiles) {
        console.log(`  ${file}: ${count}`);
    }
}

function main() {
    const vaultPath = fs.mkdtempSync(path.join(os.tmpdir(), 'vaultiel-tasks-demo-'));
    console.log('=== Vaultiel Node.js Demo: Task Analysis ===');
    console.log(`Using vault: ${vaultPath}\n`);

    const vault = new Vault(vaultPath);
    createDemoVault(vault);
    console.log('Created demo vault with tasks\n');

    const today = formatDate(new Date());
    const tomorrow = formatDate(new Date(Date.now() + 24 * 60 * 60 * 1000));

    // --- Get All Tasks ---
    console.log('--- All Tasks ---');
    const allTasks: Task[] = [];
    for (const note of vault.listNotes()) {
        const tasks = vault.getTasks(note);
        allTasks.push(...tasks);
    }

    analyzeTasks(allTasks);
    console.log();

    // --- Incomplete Tasks ---
    console.log('--- Incomplete Tasks ---');
    const incomplete = allTasks.filter(t => t.symbol === '[ ]');
    console.log(`Found ${incomplete.length} incomplete tasks:`);
    incomplete.slice(0, 5).forEach(task => {
        console.log(`  ○ ${task.description}`);
        if (task.due) console.log(`      Due: ${task.due}`);
    });
    if (incomplete.length > 5) {
        console.log(`  ... and ${incomplete.length - 5} more`);
    }
    console.log();

    // --- Tasks Due Today ---
    console.log('--- Tasks Due Today ---');
    const dueToday = incomplete.filter(t => t.due === today);
    console.log(`Tasks due ${today}:`);
    dueToday.forEach(task => {
        const priority = task.priority ? ` [${task.priority}]` : '';
        console.log(`  ○ ${task.description}${priority}`);
        console.log(`      File: ${task.file}`);
    });
    console.log();

    // --- Overdue Tasks ---
    console.log('--- Overdue Tasks ---');
    const overdue = incomplete.filter(t => t.due && t.due < today);
    if (overdue.length > 0) {
        console.log(`Found ${overdue.length} overdue tasks:`);
        overdue.forEach(task => {
            console.log(`  ⚠ ${task.description} (due: ${task.due})`);
        });
    } else {
        console.log('No overdue tasks!');
    }
    console.log();

    // --- High Priority Tasks ---
    console.log('--- High Priority Tasks ---');
    const highPriority = incomplete.filter(t => t.priority === 'high');
    console.log(`High priority tasks (${highPriority.length}):`);
    highPriority.forEach(task => {
        console.log(`  ⏫ ${task.description}`);
        console.log(`      File: ${task.file}:${task.line}`);
    });
    console.log();

    // --- Tasks from Specific Project ---
    console.log('--- Tasks from Project Alpha ---');
    const alphaTasks = vault.getTasks('Projects/Alpha.md');
    console.log(`Tasks in Project Alpha (${alphaTasks.length}):`);
    alphaTasks.forEach(task => {
        const status = task.symbol === '[x]' ? '✓' : '○';
        const indent = '  '.repeat(task.indent);
        console.log(`  ${indent}${status} ${task.description}`);
    });
    console.log();

    // --- Tasks with Tags ---
    console.log('--- Tasks with Tags ---');
    const tasksWithTags = allTasks.filter(t => t.tags.length > 0);
    console.log(`Tasks containing tags (${tasksWithTags.length}):`);
    tasksWithTags.forEach(task => {
        const tagsStr = task.tags.join(', ');
        console.log(`  ○ ${task.description}`);
        console.log(`      Tags: ${tagsStr}`);
    });
    console.log();

    // --- Tasks with Block IDs ---
    console.log('--- Tasks with Block IDs ---');
    const tasksWithBlocks = allTasks.filter(t => t.blockId);
    console.log(`Tasks with block references (${tasksWithBlocks.length}):`);
    tasksWithBlocks.forEach(task => {
        console.log(`  ○ ${task.description}`);
        console.log(`      Block ID: ^${task.blockId}`);
    });
    console.log();

    // --- Scheduled vs Due ---
    console.log('--- Scheduled vs Due ---');
    const scheduled = incomplete.filter(t => t.scheduled);
    const hasDue = incomplete.filter(t => t.due);
    console.log(`Tasks with scheduled date: ${scheduled.length}`);
    console.log(`Tasks with due date: ${hasDue.length}`);
    console.log();

    // --- Cleanup Info ---
    console.log(`Demo complete. Vault at: ${vaultPath}`);
    console.log(`To clean up: rm -rf ${vaultPath}`);
}

main();
