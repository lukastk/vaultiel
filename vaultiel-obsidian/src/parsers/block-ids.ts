/**
 * Block ID parsing (^block-id).
 *
 * Ported from vaultiel-rs/src/parser/block_id.rs
 */

import type { BlockId } from "../types.js";
import { findCodeBlockRanges, isLineInFencedCodeBlock } from "./code-block.js";

// Block ID at end of line: whitespace, ^, then alphanumeric/hyphen/underscore
const BLOCK_ID = /\s+\^([a-zA-Z0-9_-]+)\s*$/;

/** Determine block type from line content. */
function determineBlockType(line: string): string {
  const trimmed = line.trimStart();

  if (trimmed.startsWith("#")) return "heading";
  if (trimmed.startsWith("- ") || trimmed.startsWith("* ") || trimmed.startsWith("+ "))
    return "listitem";

  // Numbered list
  const numMatch = trimmed.match(/^(\d+)\. /);
  if (numMatch) return "listitem";

  if (trimmed.startsWith(">")) return "blockquote";
  if (trimmed.startsWith("|") && trimmed.endsWith("|")) return "table";
  if (trimmed.startsWith("```") || trimmed.startsWith("~~~")) return "codeblock";

  return "paragraph";
}

/** Parse all block IDs from content. */
export function parseBlockIds(content: string): BlockId[] {
  const codeRanges = findCodeBlockRanges(content);
  const blockIds: BlockId[] = [];

  const lines = content.split("\n");
  for (let lineIdx = 0; lineIdx < lines.length; lineIdx++) {
    const lineNum = lineIdx + 1;
    const line = lines[lineIdx];

    // Skip lines inside fenced code blocks
    if (isLineInFencedCodeBlock(lineNum, codeRanges)) continue;

    const match = BLOCK_ID.exec(line);
    if (!match) continue;

    const id = match[1];
    const blockType = determineBlockType(line);

    blockIds.push({ id, line: lineNum, blockType });
  }

  return blockIds;
}
