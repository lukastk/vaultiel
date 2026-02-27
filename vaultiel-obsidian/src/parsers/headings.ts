/**
 * Heading parsing and slug generation.
 *
 * Ported from vaultiel-rs/src/parser/heading.rs
 */

import type { Heading } from "../types.js";
import { findCodeBlockRanges, isLineInFencedCodeBlock } from "./code-block.js";

// ATX-style heading: # through ######, with optional trailing ^block-id
const HEADING = /^(#{1,6})\s+(.+?)(?:\s+\^[a-zA-Z0-9_-]+)?\s*$/;

/**
 * Generate a URL-safe slug from heading text.
 *
 * Follows Obsidian's algorithm: normalize unicode, lowercase,
 * spaces to hyphens, strip special chars, collapse multiple hyphens.
 */
export function slugify(text: string): string {
  // Normalize unicode (NFC)
  const normalized = text.normalize("NFC");

  let slug = "";
  let lastWasHyphen = false;

  for (const c of normalized) {
    if (/[a-zA-Z0-9]/.test(c)) {
      slug += c.toLowerCase();
      lastWasHyphen = false;
    } else if (c === "-" || c === "_") {
      if (!lastWasHyphen && slug.length > 0) {
        slug += c;
        lastWasHyphen = c === "-";
      }
    } else if (/\s/.test(c)) {
      if (!lastWasHyphen && slug.length > 0) {
        slug += "-";
        lastWasHyphen = true;
      }
    }
    // Other characters are stripped
  }

  // Remove trailing hyphens
  while (slug.endsWith("-")) {
    slug = slug.slice(0, -1);
  }

  return slug;
}

/** Parse all headings from content. */
export function parseHeadings(content: string): Heading[] {
  const codeRanges = findCodeBlockRanges(content);
  const headings: Heading[] = [];
  const slugCounts = new Map<string, number>();

  const lines = content.split("\n");
  for (let lineIdx = 0; lineIdx < lines.length; lineIdx++) {
    const lineNum = lineIdx + 1;
    const line = lines[lineIdx];

    // Skip lines inside fenced code blocks
    if (isLineInFencedCodeBlock(lineNum, codeRanges)) continue;

    const match = HEADING.exec(line);
    if (!match) continue;

    const level = match[1].length;
    const text = match[2].trim();

    // Generate unique slug
    const baseSlug = slugify(text);
    const count = (slugCounts.get(baseSlug) ?? 0) + 1;
    slugCounts.set(baseSlug, count);
    const slug = count === 1 ? baseSlug : `${baseSlug}-${count - 1}`;

    headings.push({ text, level, line: lineNum, slug });
  }

  return headings;
}
