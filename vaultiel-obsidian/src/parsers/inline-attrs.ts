/**
 * Inline attribute parsing ([key::value]).
 *
 * Ported from vaultiel-rs/src/parser/inline_attr.rs
 */

import type { InlineAttr } from "../types.js";
import { findCodeBlockRanges, isInCodeBlock } from "./code-block.js";

// Inline attribute: [key::value]
// Key: word chars and hyphens
// Value: allows ]] inside (for wikilinks like [[Note]])
const INLINE_ATTR = /\[([\w-]+)::([^\]]*(?:\]\][^\]]*)*)\]/g;

/** Parse all inline attributes from content. */
export function parseInlineAttrs(content: string): InlineAttr[] {
  const codeRanges = findCodeBlockRanges(content);
  const attrs: InlineAttr[] = [];

  INLINE_ATTR.lastIndex = 0;
  let cap: RegExpExecArray | null;
  while ((cap = INLINE_ATTR.exec(content)) !== null) {
    const start = cap.index;

    // Skip if inside code block
    if (isInCodeBlock(start, codeRanges)) continue;

    const key = cap[1];
    const value = cap[2].trim();

    // Calculate line number (1-indexed)
    let line = 1;
    for (let i = 0; i < start; i++) {
      if (content[i] === "\n") line++;
    }

    attrs.push({ key, value, line });
  }

  return attrs;
}
