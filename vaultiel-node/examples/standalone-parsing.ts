/**
 * Standalone Parsing
 *
 * Demonstrates how to parse markdown content without a vault.
 * Useful for processing content from APIs, databases, or other sources.
 */

import {
    parseLinks,
    parseContentTags,
    parseContentHeadings,
    parseContentBlockIds,
} from '@vaultiel/node';

function main() {
    console.log('=== Vaultiel Node.js Demo: Standalone Parsing ===\n');

    // Sample markdown content
    const content = `---
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

\`\`\`typescript
// This [[link]] should not be parsed
// Neither should this #tag
\`\`\`

But this \`[[inline code]]\` is still parsed (Obsidian behavior).

## Tasks

- [ ] Incomplete task 📅 2024-02-15 ⏫
- [x] Completed task ✅ 2024-01-10
- [ ] Task with link to [[Project]]

## Conclusion

That's all! See [[Introduction]] to go back.

#conclusion
`;

    console.log('Input content:');
    console.log('-'.repeat(40));
    console.log(content.substring(0, 500) + '...\n');

    // --- Parse Links ---
    console.log('--- Parsing Links ---');
    const links = parseLinks(content);
    console.log(`Found ${links.length} links:\n`);

    for (const link of links) {
        let linkRepr = `[[${link.target}`;
        if (link.heading) linkRepr += `#${link.heading}`;
        if (link.blockId) linkRepr += `#^${link.blockId}`;
        linkRepr += ']]';
        if (link.alias) linkRepr = `[[${link.target}|${link.alias}]]`;
        if (link.embed) linkRepr = '!' + linkRepr;

        console.log(`  Line ${link.line.toString().padStart(2)}: ${linkRepr}`);
        if (link.alias) console.log(`           Alias: ${link.alias}`);
        if (link.heading) console.log(`           Heading: ${link.heading}`);
        if (link.blockId) console.log(`           Block ID: ${link.blockId}`);
        if (link.embed) console.log('           (embedded)');
    }
    console.log();

    // --- Parse Tags ---
    console.log('--- Parsing Tags ---');
    const tags = parseContentTags(content);
    console.log(`Found ${tags.length} tags:\n`);

    for (const tag of tags) {
        console.log(`  Line ${tag.line.toString().padStart(2)}: #${tag.name}`);
    }
    console.log();

    // --- Parse Headings ---
    console.log('--- Parsing Headings ---');
    const headings = parseContentHeadings(content);
    console.log(`Found ${headings.length} headings:\n`);

    for (const h of headings) {
        const indent = '  '.repeat(h.level - 1);
        console.log(`  Line ${h.line.toString().padStart(2)}: ${indent}${'#'.repeat(h.level)} ${h.text}`);
        console.log(`           Slug: ${h.slug}`);
    }
    console.log();

    // --- Parse Block IDs ---
    console.log('--- Parsing Block IDs ---');
    const blocks = parseContentBlockIds(content);
    console.log(`Found ${blocks.length} block IDs:\n`);

    for (const block of blocks) {
        console.log(`  Line ${block.line.toString().padStart(2)}: ^${block.id}`);
        console.log(`           Type: ${block.blockType}`);
    }
    console.log();

    // --- Practical Example: Extract All References ---
    console.log('--- Practical Example: Extract All References ---');

    const references = {
        internalLinks: [] as string[],
        embeds: [] as string[],
    };

    for (const link of links) {
        if (link.embed) {
            references.embeds.push(link.target);
        } else {
            let ref = link.target;
            if (link.heading) ref += `#${link.heading}`;
            if (link.blockId) ref += `#^${link.blockId}`;
            references.internalLinks.push(ref);
        }
    }

    console.log('Internal links:', references.internalLinks);
    console.log('Embeds:', references.embeds);
    console.log();

    // --- Practical Example: Build Table of Contents ---
    console.log('--- Practical Example: Build Table of Contents ---');

    console.log('Table of Contents:');
    for (const h of headings) {
        const indent = '  '.repeat(h.level - 1);
        console.log(`${indent}- [${h.text}](#${h.slug})`);
    }
    console.log();

    // --- Practical Example: Find All Unique Tags ---
    console.log('--- Practical Example: Unique Tags ---');

    const uniqueTags = [...new Set(tags.map(t => t.name))].sort();
    console.log(`Unique tags (${uniqueTags.length}): ${JSON.stringify(uniqueTags)}`);
    console.log();

    // --- Practical Example: Parse Arbitrary Content ---
    console.log('--- Processing Arbitrary Content ---');

    const snippet = `
Just a quick note with [[Link A]] and [[Link B|alias]].
Also has #tag1 and #tag2.
`;

    const snippetLinks = parseLinks(snippet);
    const snippetTags = parseContentTags(snippet);

    console.log(`Snippet has ${snippetLinks.length} links and ${snippetTags.length} tags`);
    console.log(`Links: ${JSON.stringify(snippetLinks.map(l => l.target))}`);
    console.log(`Tags: ${JSON.stringify(snippetTags.map(t => t.name))}`);
}

main();
