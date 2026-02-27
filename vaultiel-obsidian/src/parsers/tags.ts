/**
 * Tag parsing (#tag and #tag/subtag).
 *
 * Ported from vaultiel-rs/src/parser/tag.rs
 */

import type { Tag } from "../types.js";
import { findCodeBlockRanges, isInCodeBlock } from "./code-block.js";

// Tag pattern: # followed by letter/underscore, then word chars, hyphens, and slashes.
// Must not be preceded by a word character or &.
const TAG = /(?:^|[^\w&])#([a-zA-Z_][\w/-]*)/g;

/** Check if a position is inside a wikilink. */
function isInWikilink(content: string, pos: number): boolean {
  const before = content.slice(0, pos);
  const after = content.slice(pos);

  const lastOpen = before.lastIndexOf("[[");
  const lastClose = before.lastIndexOf("]]");

  if (lastOpen !== -1 && lastClose !== -1) {
    if (lastOpen > lastClose) {
      return after.includes("]]");
    }
    return false;
  }
  if (lastOpen !== -1 && lastClose === -1) {
    return after.includes("]]");
  }
  return false;
}

/** Parse all tags from content. */
export function parseTags(content: string): Tag[] {
  const codeRanges = findCodeBlockRanges(content);
  const tags: Tag[] = [];

  TAG.lastIndex = 0;
  let cap: RegExpExecArray | null;
  while ((cap = TAG.exec(content)) !== null) {
    const tagMatch = cap[1];
    // The # is just before the captured group
    const hashPos = cap.index + cap[0].indexOf("#");
    const end = hashPos + 1 + tagMatch.length;

    // Check that tag isn't followed by more word chars or /
    if (end < content.length) {
      const nextChar = content[end];
      if (/[\w/]/.test(nextChar)) continue;
    }

    // Skip if inside code block
    if (isInCodeBlock(hashPos, codeRanges)) continue;

    // Skip if inside a wikilink
    if (isInWikilink(content, hashPos)) continue;

    const tagName = `#${tagMatch}`;

    // Calculate line number (1-indexed)
    let line = 1;
    for (let i = 0; i < hashPos; i++) {
      if (content[i] === "\n") line++;
    }

    tags.push({ name: tagName, line });
  }

  return tags;
}
