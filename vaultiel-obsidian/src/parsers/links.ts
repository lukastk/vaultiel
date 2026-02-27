/**
 * Wikilink and embed parsing.
 *
 * Ported from vaultiel-rs/src/parser/wikilink.rs
 */

import type { Link } from "../types.js";
import { findCodeBlockRanges, isInCodeBlock } from "./code-block.js";

// Wikilink pattern: [[target]] or [[target|alias]] or [[target#heading]] or [[target#^block]]
// Groups: (1) ! for embed, (2) target path, (3) block ref, (4) heading ref, (5) alias
const WIKILINK =
  /(!?)\[\[([^\]\|#]+)(?:#\^([a-zA-Z0-9_-]+))?(?:#([^\]\|]+))?(?:\|([^\]]+))?\]\]/g;

/** Parse all wikilinks and embeds from content. */
export function parseLinks(content: string): Link[] {
  const codeRanges = findCodeBlockRanges(content);
  const links: Link[] = [];

  WIKILINK.lastIndex = 0;
  let cap: RegExpExecArray | null;
  while ((cap = WIKILINK.exec(content)) !== null) {
    const start = cap.index;

    // Skip if inside code block
    if (isInCodeBlock(start, codeRanges)) continue;

    const isEmbed = cap[1] === "!";
    const target = (cap[2] || "").trim();
    const blockId = cap[3] || undefined;
    const heading = cap[4] || undefined;
    const alias = cap[5] || undefined;

    // Calculate line number (1-indexed)
    let line = 1;
    for (let i = 0; i < start; i++) {
      if (content[i] === "\n") line++;
    }

    links.push({
      target,
      alias,
      heading,
      blockId,
      embed: isEmbed,
      line,
    });
  }

  return links;
}
