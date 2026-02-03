#!/usr/bin/env python3
"""
Task Analysis

Demonstrates how to extract, filter, and analyze tasks from a vault.
"""

import tempfile
from collections import defaultdict
from datetime import date, timedelta
from pathlib import Path
from vaultiel import Vault, Task


def create_demo_vault(vault: Vault):
    """Create a vault with various tasks."""
    today = date.today()
    tomorrow = today + timedelta(days=1)
    next_week = today + timedelta(days=7)
    yesterday = today - timedelta(days=1)

    vault.create_note(
        "Inbox.md",
        f"""---
title: Inbox
---

# Inbox

Quick capture for tasks.

- [ ] Review pull request 📅 {today} ⏫ #urgent
- [ ] Reply to emails ⏳ {today}
- [ ] Read article about Rust 🔽
- [x] Set up project ✅ {yesterday}
- [ ] Plan next sprint 📅 {next_week}
""",
    )

    vault.create_note(
        "Projects/Alpha.md",
        f"""---
title: Project Alpha
type: project
status: active
---

# Project Alpha

## Tasks

- [ ] Implement authentication 📅 {tomorrow} ⏫ ^auth-task
    - [ ] Design login flow
    - [ ] Implement OAuth
    - [ ] Write tests
- [ ] Set up CI/CD 📅 {next_week} 🔼
- [ ] Write documentation
- [x] Create repository ✅ {yesterday}
- [>] Deferred: Research alternatives

## Notes

See [[Inbox]] for quick tasks.

#project #priority
""",
    )

    vault.create_note(
        "Projects/Beta.md",
        f"""---
title: Project Beta
type: project
status: planning
---

# Project Beta

## Planning Tasks

- [ ] Define requirements 📅 {tomorrow}
- [ ] Create mockups
- [ ] Estimate timeline

#project
""",
    )

    vault.create_note(
        "Daily/{}.md".format(today),
        f"""---
title: Daily Note
date: {today}
---

# {today}

## Today's Tasks

- [ ] Morning standup ⏳ {today}
- [ ] Work on [[Projects/Alpha]] 📅 {today} ⏫
- [ ] Review [[Projects/Beta]] planning

## Notes

Focus on authentication today.

#daily
""",
    )


def analyze_tasks(tasks: list[Task]):
    """Analyze a list of tasks and print statistics."""
    if not tasks:
        print("No tasks to analyze.")
        return

    # Count by status
    status_counts = defaultdict(int)
    for task in tasks:
        status_counts[task.symbol] += 1

    print(f"Total tasks: {len(tasks)}")
    print("By status:")
    for symbol, count in sorted(status_counts.items()):
        label = {
            "[ ]": "Todo",
            "[x]": "Done",
            "[>]": "Deferred",
            "[-]": "Cancelled",
        }.get(symbol, symbol)
        print(f"  {label} ({symbol}): {count}")

    # Count by priority
    priority_counts = defaultdict(int)
    for task in tasks:
        priority_counts[task.priority or "none"] += 1

    print("By priority:")
    for priority, count in sorted(priority_counts.items()):
        print(f"  {priority}: {count}")

    # Count by file
    file_counts = defaultdict(int)
    for task in tasks:
        file_counts[task.file] += 1

    print("By file:")
    for file, count in sorted(file_counts.items(), key=lambda x: -x[1]):
        print(f"  {file}: {count}")


def main():
    vault_path = Path(tempfile.mkdtemp(prefix="vaultiel-tasks-demo-"))
    (vault_path / "Projects").mkdir()
    (vault_path / "Daily").mkdir()

    print(f"=== Vaultiel Python Demo: Task Analysis ===")
    print(f"Using vault: {vault_path}\n")

    vault = Vault(str(vault_path))
    create_demo_vault(vault)
    print("Created demo vault with tasks\n")

    today = date.today()
    tomorrow = today + timedelta(days=1)

    # --- Get All Tasks ---
    print("--- All Tasks ---")
    all_tasks = []
    for note in vault.list_notes():
        tasks = vault.get_tasks(note)
        all_tasks.extend(tasks)

    analyze_tasks(all_tasks)
    print()

    # --- Incomplete Tasks ---
    print("--- Incomplete Tasks ---")
    incomplete = [t for t in all_tasks if t.symbol == "[ ]"]
    print(f"Found {len(incomplete)} incomplete tasks:")
    for task in incomplete[:5]:
        print(f"  ○ {task.description}")
        if task.due:
            print(f"      Due: {task.due}")
    if len(incomplete) > 5:
        print(f"  ... and {len(incomplete) - 5} more")
    print()

    # --- Tasks Due Today ---
    print("--- Tasks Due Today ---")
    today_str = str(today)
    due_today = [t for t in incomplete if t.due == today_str]
    print(f"Tasks due {today_str}:")
    for task in due_today:
        priority = f" [{task.priority}]" if task.priority else ""
        print(f"  ○ {task.description}{priority}")
        print(f"      File: {task.file}")
    print()

    # --- Overdue Tasks ---
    print("--- Overdue Tasks ---")
    overdue = [
        t for t in incomplete
        if t.due and t.due < today_str
    ]
    if overdue:
        print(f"Found {len(overdue)} overdue tasks:")
        for task in overdue:
            print(f"  ⚠ {task.description} (due: {task.due})")
    else:
        print("No overdue tasks!")
    print()

    # --- High Priority Tasks ---
    print("--- High Priority Tasks ---")
    high_priority = [t for t in incomplete if t.priority == "high"]
    print(f"High priority tasks ({len(high_priority)}):")
    for task in high_priority:
        print(f"  ⏫ {task.description}")
        print(f"      File: {task.file}:{task.line}")
    print()

    # --- Tasks from Specific Project ---
    print("--- Tasks from Project Alpha ---")
    alpha_tasks = vault.get_tasks("Projects/Alpha.md")
    print(f"Tasks in Project Alpha ({len(alpha_tasks)}):")
    for task in alpha_tasks:
        status = "✓" if task.symbol == "[x]" else "○"
        indent = "  " * task.indent
        print(f"  {indent}{status} {task.description}")
    print()

    # --- Tasks with Tags ---
    print("--- Tasks with Tags ---")
    tasks_with_tags = [t for t in all_tasks if t.tags]
    print(f"Tasks containing tags ({len(tasks_with_tags)}):")
    for task in tasks_with_tags:
        tags_str = ", ".join(task.tags)
        print(f"  ○ {task.description}")
        print(f"      Tags: {tags_str}")
    print()

    # --- Tasks with Block IDs ---
    print("--- Tasks with Block IDs ---")
    tasks_with_blocks = [t for t in all_tasks if t.block_id]
    print(f"Tasks with block references ({len(tasks_with_blocks)}):")
    for task in tasks_with_blocks:
        print(f"  ○ {task.description}")
        print(f"      Block ID: ^{task.block_id}")
    print()

    # --- Scheduled vs Due ---
    print("--- Scheduled vs Due ---")
    scheduled = [t for t in incomplete if t.scheduled]
    has_due = [t for t in incomplete if t.due]
    print(f"Tasks with scheduled date: {len(scheduled)}")
    print(f"Tasks with due date: {len(has_due)}")
    print()

    # --- Task Hierarchy ---
    print("--- Task Hierarchy Example ---")
    alpha_tasks = vault.get_tasks("Projects/Alpha.md")
    for task in alpha_tasks:
        if task.indent == 0 and task.symbol == "[ ]":
            print(f"○ {task.description}")
            # Find child tasks
            children = [
                t for t in alpha_tasks
                if t.indent > 0 and t.line > task.line
            ]
            # Show first few children
            for child in children[:3]:
                if child.indent == 1:
                    print(f"    ○ {child.description}")
            break
    print()

    # --- Cleanup Info ---
    print(f"Demo complete. Vault at: {vault_path}")
    print(f"To clean up: rm -rf {vault_path}")


if __name__ == "__main__":
    main()
