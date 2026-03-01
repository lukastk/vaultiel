/**
 * Inline property parsing ([key::value]).
 *
 * Ported from vaultiel-rs/src/parser/inline_property.rs
 */

import type { InlineProperty } from "../types.js";
import { findCodeBlockRanges, isInCodeBlock } from "./code-block.js";

// Inline property: [key::value]
// Key: word chars and hyphens
// Value: allows ]] inside (for wikilinks like [[Note]])
const INLINE_PROPERTY = /\[([\w-]+)::([^\]]*(?:\]\][^\]]*)*)\]/g;

/** Parse all inline properties from content. */
export function parseInlineProperties(content: string): InlineProperty[] {
  const codeRanges = findCodeBlockRanges(content);
  const props: InlineProperty[] = [];

  INLINE_PROPERTY.lastIndex = 0;
  let cap: RegExpExecArray | null;
  while ((cap = INLINE_PROPERTY.exec(content)) !== null) {
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

    props.push({ key, value, line });
  }

  return props;
}
