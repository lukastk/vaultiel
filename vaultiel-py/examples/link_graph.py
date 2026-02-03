#!/usr/bin/env python3
"""
Link Graph Analysis

Demonstrates how to analyze the link structure of a vault.
"""

import tempfile
from collections import defaultdict
from pathlib import Path
from vaultiel import Vault


def create_demo_vault(vault: Vault):
    """Create an interconnected demo vault."""

    vault.create_note(
        "Hub.md",
        """---
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
""",
    )

    vault.create_note(
        "Spoke A.md",
        """---
title: Spoke A
---

# Spoke A

## Summary

This is Spoke A, connected to [[Hub]].

Also links to [[Spoke B]].

^summary

#spoke
""",
    )

    vault.create_note(
        "Spoke B.md",
        """---
title: Spoke B
links:
  - "[[Hub]]"
---

# Spoke B

Connected to [[Hub]] and [[Spoke A]].

#spoke
""",
    )

    vault.create_note(
        "Spoke C.md",
        """---
title: Spoke C
---

# Spoke C

Only connected to [[Hub]].

#spoke
""",
    )

    vault.create_note(
        "Orphan.md",
        """---
title: Orphan Note
---

# Orphan Note

This note has no incoming links. It's isolated.

#orphan
""",
    )


def main():
    vault_path = Path(tempfile.mkdtemp(prefix="vaultiel-graph-demo-"))
    print(f"=== Vaultiel Python Demo: Link Graph ===")
    print(f"Using vault: {vault_path}\n")

    vault = Vault(str(vault_path))
    create_demo_vault(vault)
    print("Created demo vault with interconnected notes\n")

    # --- Outgoing Links ---
    print("--- Outgoing Links from Hub.md ---")
    outgoing = vault.get_outgoing_links("Hub.md")
    for ref in outgoing:
        link_type = "embed" if ref.embed else "link"
        print(f"  [{link_type}] -> {ref.from_note} (line {ref.line}, context: {ref.context})")
    print()

    # Note: In outgoing links, 'from_note' is actually the target
    # This matches the LinkRef structure used internally

    # --- Incoming Links (Backlinks) ---
    print("--- Incoming Links to Hub.md (Backlinks) ---")
    incoming = vault.get_incoming_links("Hub.md")
    print(f"Hub.md has {len(incoming)} incoming links:")
    for ref in incoming:
        print(f"  <- {ref.from_note} (line {ref.line}, context: {ref.context})")
    print()

    # --- Build Complete Link Graph ---
    print("--- Building Complete Link Graph ---")

    # Adjacency list: note -> [notes it links to]
    graph = defaultdict(list)
    all_notes = vault.list_notes()

    for note in all_notes:
        links = vault.get_links(note)
        for link in links:
            if not link.embed:  # Only count regular links, not embeds
                try:
                    target = vault.resolve_note(link.target)
                    graph[note].append(target)
                except RuntimeError:
                    pass  # Broken link

    print("Link graph (source -> targets):")
    for source, targets in sorted(graph.items()):
        targets_str = ", ".join(targets) if targets else "(none)"
        print(f"  {source} -> {targets_str}")
    print()

    # --- Calculate Incoming Link Counts ---
    print("--- Incoming Link Counts ---")
    incoming_counts = defaultdict(int)
    for source, targets in graph.items():
        for target in targets:
            incoming_counts[target] += 1

    # Sort by count (most linked first)
    sorted_counts = sorted(incoming_counts.items(), key=lambda x: -x[1])
    print("Notes ranked by incoming links:")
    for note, count in sorted_counts:
        print(f"  {count:2d} links -> {note}")

    # Find notes with no incoming links
    no_incoming = [n for n in all_notes if incoming_counts[n] == 0]
    print(f"\nNotes with no incoming links (orphans): {no_incoming}")
    print()

    # --- Find Orphans ---
    print("--- Finding Orphans (Alternative Method) ---")
    orphans = []
    for note in all_notes:
        incoming = vault.get_incoming_links(note)
        if len(incoming) == 0:
            orphans.append(note)

    print(f"Orphan notes: {orphans}")
    print()

    # --- Analyze Link Contexts ---
    print("--- Link Context Analysis ---")
    context_counts = defaultdict(int)

    for note in all_notes:
        incoming = vault.get_incoming_links(note)
        for ref in incoming:
            # Simplify context (e.g., "frontmatter:links" -> "frontmatter")
            ctx = ref.context.split(":")[0] if ":" in ref.context else ref.context
            context_counts[ctx] += 1

    print("Links by context:")
    for ctx, count in sorted(context_counts.items(), key=lambda x: -x[1]):
        print(f"  {ctx}: {count}")
    print()

    # --- Find Notes That Link to Multiple Others ---
    print("--- Most Connected Notes (by outgoing links) ---")
    outgoing_counts = {note: len(targets) for note, targets in graph.items()}
    sorted_outgoing = sorted(outgoing_counts.items(), key=lambda x: -x[1])

    for note, count in sorted_outgoing[:5]:
        print(f"  {count:2d} outgoing links from {note}")
    print()

    # --- Cleanup Info ---
    print(f"Demo complete. Vault at: {vault_path}")
    print(f"To clean up: rm -rf {vault_path}")


if __name__ == "__main__":
    main()
