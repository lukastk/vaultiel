#!/usr/bin/env python3
"""
Basic Vaultiel Operations

Demonstrates fundamental vault operations using the Python bindings.
"""

import os
import tempfile
from pathlib import Path
from vaultiel import Vault


def main():
    # Create a temporary vault for demonstration
    vault_path = Path(tempfile.mkdtemp(prefix="vaultiel-demo-"))
    print(f"=== Vaultiel Python Demo: Basic Operations ===")
    print(f"Using vault: {vault_path}\n")

    # Initialize vault
    vault = Vault(str(vault_path))
    print(f"Vault root: {vault.root}\n")

    # --- Creating Notes ---
    print("--- Creating Notes ---")

    vault.create_note(
        "Welcome.md",
        """---
title: Welcome
tags:
  - demo
  - intro
---

# Welcome to Vaultiel

This is a demo vault created with the Python bindings.

## Features

- Fast markdown parsing
- Link graph traversal
- Task extraction

See [[Getting Started]] for more information.
""",
    )
    print("Created Welcome.md")

    vault.create_note(
        "Getting Started.md",
        """---
title: Getting Started
aliases:
  - quickstart
  - tutorial
---

# Getting Started

Welcome to the tutorial! Check out [[Welcome]] if you haven't already.

## Installation

```bash
pip install vaultiel
```

## Next Steps

- Explore the API
- Build something cool!

#tutorial #beginner
""",
    )
    print("Created Getting Started.md")

    vault.create_note(
        "Project Notes.md",
        """---
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
""",
    )
    print("Created Project Notes.md")
    print()

    # --- Listing Notes ---
    print("--- Listing Notes ---")
    notes = vault.list_notes()
    print(f"All notes ({len(notes)}):")
    for note in notes:
        print(f"  - {note}")
    print()

    # List with glob pattern
    print("Notes matching 'Project*':")
    matching = vault.list_notes_matching("Project*")
    for note in matching:
        print(f"  - {note}")
    print()

    # --- Reading Content ---
    print("--- Reading Content ---")

    # Full content
    content = vault.get_content("Welcome.md")
    print("Full content of Welcome.md:")
    print(content[:200] + "...\n")

    # Body only (without frontmatter)
    body = vault.get_body("Welcome.md")
    print("Body of Welcome.md:")
    print(body[:150] + "...\n")

    # --- Frontmatter ---
    print("--- Frontmatter ---")

    # As dict (most convenient)
    fm = vault.get_frontmatter_dict("Getting Started.md")
    if fm:
        print("Frontmatter of Getting Started.md:")
        print(f"  Title: {fm.get('title')}")
        print(f"  Aliases: {fm.get('aliases')}")
    print()

    # --- Note Resolution ---
    print("--- Note Resolution ---")

    # Resolve by name
    path = vault.resolve_note("Welcome")
    print(f"'Welcome' resolves to: {path}")

    # Resolve by alias
    path = vault.resolve_note("quickstart")
    print(f"'quickstart' resolves to: {path}")

    # Check existence
    exists = vault.note_exists("Welcome.md")
    print(f"Welcome.md exists: {exists}")

    missing = vault.note_exists("NonExistent.md")
    print(f"NonExistent.md exists: {missing}")
    print()

    # --- Parsing ---
    print("--- Parsing Links and Tags ---")

    links = vault.get_links("Project Notes.md")
    print(f"Links in Project Notes.md ({len(links)}):")
    for link in links:
        print(f"  - [[{link.target}]] at line {link.line}")

    tags = vault.get_tags("Project Notes.md")
    print(f"Tags in Project Notes.md ({len(tags)}):")
    for tag in tags:
        print(f"  - #{tag.name} at line {tag.line}")
    print()

    # --- Headings ---
    print("--- Headings ---")

    headings = vault.get_headings("Welcome.md")
    print("Headings in Welcome.md:")
    for h in headings:
        indent = "  " * (h.level - 1)
        print(f"{indent}{'#' * h.level} {h.text} (line {h.line})")
    print()

    # --- Tasks ---
    print("--- Tasks ---")

    tasks = vault.get_tasks("Project Notes.md")
    print(f"Tasks in Project Notes.md ({len(tasks)}):")
    for task in tasks:
        status = "✓" if task.symbol == "[x]" else "○"
        print(f"  {status} {task.description}")
        if task.due:
            print(f"      Due: {task.due}")
        if task.priority:
            print(f"      Priority: {task.priority}")
    print()

    # --- Cleanup Info ---
    print("--- Demo Complete ---")
    print(f"Vault created at: {vault_path}")
    print(f"To clean up: rm -rf {vault_path}")


if __name__ == "__main__":
    main()
