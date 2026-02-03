#!/usr/bin/env python3
"""
Standalone Parsing

Demonstrates how to parse markdown content without a vault.
Useful for processing content from other sources (APIs, databases, etc.)
"""

from vaultiel import (
    parse_links,
    parse_content_tags,
    parse_content_headings,
    parse_content_block_ids,
)


def main():
    print("=== Vaultiel Python Demo: Standalone Parsing ===\n")

    # Sample markdown content
    content = """---
title: Sample Document
tags:
  - sample
  - demo
---

# Introduction

Welcome to this sample document. It demonstrates various Obsidian features.

## Links and Embeds

Here are some wikilinks:
- Simple link: [[Other Note]]
- Link with alias: [[Long Note Title|Short Name]]
- Link to heading: [[Other Note#Specific Section]]
- Link to block: [[Other Note#^block-123]]

And some embeds:
![[Embedded Note]]
![[image.png|400]]

## Tags and Blocks

This section has #inline-tag and also #nested/tag/here.

Here's an important paragraph. ^important-block

And a list item with a block ID:
- Key point ^key-point

## Code Example

Code blocks should be ignored:

```python
# This [[link]] should not be parsed
# Neither should this #tag
```

But this `[[inline code]]` is still parsed (Obsidian behavior).

## Tasks

- [ ] Incomplete task 📅 2024-02-15 ⏫
- [x] Completed task ✅ 2024-01-10
- [ ] Task with link to [[Project]]

## Conclusion

That's all! See [[Introduction]] to go back.

#conclusion
"""

    print("Input content:")
    print("-" * 40)
    print(content[:500] + "...\n")

    # --- Parse Links ---
    print("--- Parsing Links ---")
    links = parse_links(content)
    print(f"Found {len(links)} links:\n")

    for link in links:
        link_repr = f"[[{link.target}"
        if link.heading:
            link_repr += f"#{link.heading}"
        if link.block_id:
            link_repr += f"#^{link.block_id}"
        link_repr += "]]"
        if link.alias:
            link_repr = f"[[{link.target}|{link.alias}]]"
        if link.embed:
            link_repr = "!" + link_repr

        print(f"  Line {link.line:2d}: {link_repr}")
        if link.alias:
            print(f"           Alias: {link.alias}")
        if link.heading:
            print(f"           Heading: {link.heading}")
        if link.block_id:
            print(f"           Block ID: {link.block_id}")
        if link.embed:
            print(f"           (embedded)")
    print()

    # --- Parse Tags ---
    print("--- Parsing Tags ---")
    tags = parse_content_tags(content)
    print(f"Found {len(tags)} tags:\n")

    for tag in tags:
        print(f"  Line {tag.line:2d}: #{tag.name}")
    print()

    # --- Parse Headings ---
    print("--- Parsing Headings ---")
    headings = parse_content_headings(content)
    print(f"Found {len(headings)} headings:\n")

    for h in headings:
        indent = "  " * (h.level - 1)
        print(f"  Line {h.line:2d}: {indent}{'#' * h.level} {h.text}")
        print(f"           Slug: {h.slug}")
    print()

    # --- Parse Block IDs ---
    print("--- Parsing Block IDs ---")
    blocks = parse_content_block_ids(content)
    print(f"Found {len(blocks)} block IDs:\n")

    for block in blocks:
        print(f"  Line {block.line:2d}: ^{block.id}")
        print(f"           Type: {block.block_type}")
    print()

    # --- Practical Example: Extract All References ---
    print("--- Practical Example: Extract All References ---")

    references = {
        "internal_links": [],
        "embeds": [],
        "external_references": [],
    }

    for link in links:
        if link.embed:
            references["embeds"].append(link.target)
        else:
            ref = link.target
            if link.heading:
                ref += f"#{link.heading}"
            if link.block_id:
                ref += f"#^{link.block_id}"
            references["internal_links"].append(ref)

    print("Internal links:", references["internal_links"])
    print("Embeds:", references["embeds"])
    print()

    # --- Practical Example: Build Table of Contents ---
    print("--- Practical Example: Build Table of Contents ---")

    print("Table of Contents:")
    for h in headings:
        indent = "  " * (h.level - 1)
        print(f"{indent}- [{h.text}](#{h.slug})")
    print()

    # --- Practical Example: Find All Unique Tags ---
    print("--- Practical Example: Unique Tags ---")

    unique_tags = sorted(set(tag.name for tag in tags))
    print(f"Unique tags ({len(unique_tags)}): {unique_tags}")
    print()

    # --- Practical Example: Parse Content from String ---
    print("--- Processing Arbitrary Content ---")

    snippet = """
Just a quick note with [[Link A]] and [[Link B|alias]].
Also has #tag1 and #tag2.
"""

    snippet_links = parse_links(snippet)
    snippet_tags = parse_content_tags(snippet)

    print(f"Snippet has {len(snippet_links)} links and {len(snippet_tags)} tags")
    print(f"Links: {[l.target for l in snippet_links]}")
    print(f"Tags: {[t.name for t in snippet_tags]}")


if __name__ == "__main__":
    main()
