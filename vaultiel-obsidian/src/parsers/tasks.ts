/**
 * Task parsing with generic emoji metadata extraction.
 *
 * Ported from vaultiel-rs/src/parser/task.rs
 */

import type { Task, TaskLink, TaskConfig, EmojiFieldDef } from "../types.js";
import { findCodeBlockRanges } from "./code-block.js";

// Task line: optional indent, "- [symbol] ", then the rest
const TASK_REGEX = /^(\s*)- \[(.)\] (.*)$/;

// ISO date: YYYY-MM-DD
const DATE_REGEX = /\d{4}-\d{2}-\d{2}/;

// Block ID at end of line
const BLOCK_ID_REGEX = /\s+\^([a-zA-Z0-9_-]+)\s*$/;

// Tags in task description
const TAG_REGEX = /#[a-zA-Z_][a-zA-Z0-9_/-]*/g;

// Wikilinks in task description (simplified, for extracting task links)
const WIKILINK_REGEX = /\[\[([^\]\|#]+)(?:#[^\]\|]*)?\|?([^\]]*)\]\]/g;

// Number at start of string
const NUMBER_REGEX = /^-?\d+(?:\.\d+)?/;

/** Count indentation level (tabs or 4 spaces = 1 level). */
function countIndent(s: string): number {
  let spaces = 0;
  let tabs = 0;
  for (const c of s) {
    if (c === "\t") tabs++;
    else if (c === " ") spaces++;
    else break;
  }
  return tabs + Math.floor(spaces / 4);
}

/** Extract block ID from end of text. Returns [remaining, blockId]. */
function extractBlockId(text: string): [string, string | undefined] {
  const match = BLOCK_ID_REGEX.exec(text);
  if (match) {
    const blockId = match[1];
    const without = text.replace(BLOCK_ID_REGEX, "").trimEnd();
    return [without, blockId];
  }
  return [text, undefined];
}

/** Get sorted fields from config (by order ascending). */
function sortedFields(config: TaskConfig): EmojiFieldDef[] {
  return [...config.fields].sort((a, b) => a.order - b.order);
}

/** Get all emoji strings from config. */
function allEmojis(config: TaskConfig): string[] {
  return config.fields.map((f) => f.emoji);
}

/** Remove an emoji and optional surrounding whitespace from a string at a given position. */
function removeEmoji(text: string, pos: number, emojiLen: number): string {
  return text.slice(0, pos) + text.slice(pos + emojiLen);
}

/** Extract a date value after an emoji. Returns [value, newText] or undefined. */
function extractDateAfter(
  text: string,
  pos: number,
  emojiLen: number,
): [string, string] | undefined {
  const after = text.slice(pos + emojiLen);
  const trimmed = after.trimStart();
  const dateMatch = DATE_REGEX.exec(trimmed);
  if (!dateMatch) return undefined;

  const value = dateMatch[0]!;
  const trimLen = after.length - trimmed.length;
  const newText =
    text.slice(0, pos) +
    text.slice(pos + emojiLen + trimLen + dateMatch.index + value.length);
  return [value, newText];
}

/** Extract a single word/token after an emoji. Returns [value, newText] or undefined. */
function extractWordAfter(
  text: string,
  pos: number,
  emojiLen: number,
): [string, string] | undefined {
  const after = text.slice(pos + emojiLen);
  const trimmed = after.trimStart();
  const wordMatch = /^\S+/.exec(trimmed);
  if (!wordMatch) return undefined;

  const value = wordMatch[0]!;
  const trimLen = after.length - trimmed.length;
  const newText =
    text.slice(0, pos) +
    text.slice(pos + emojiLen + trimLen + value.length);
  return [value, newText];
}

/** Extract a number after an emoji. Returns [value, newText] or undefined. */
function extractNumberAfter(
  text: string,
  pos: number,
  emojiLen: number,
): [string, string] | undefined {
  const after = text.slice(pos + emojiLen);
  const trimmed = after.trimStart();
  const numMatch = NUMBER_REGEX.exec(trimmed);
  if (!numMatch) return undefined;

  const value = numMatch[0]!;
  const trimLen = after.length - trimmed.length;
  const newText =
    text.slice(0, pos) +
    text.slice(pos + emojiLen + trimLen + value.length);
  return [value, newText];
}

/** Extract text until the next registered emoji. Returns [value, newText] or undefined. */
function extractTextUntilNextEmoji(
  text: string,
  pos: number,
  emojiLen: number,
  emojis: string[],
): [string, string] | undefined {
  const after = text.slice(pos + emojiLen);
  const trimmed = after.trimStart();

  // Find end: next registered emoji or end of string
  let valueEnd = trimmed.length;
  for (let i = 0; i < trimmed.length; ) {
    const remaining = trimmed.slice(i);
    // Check if any registered emoji starts here
    if (emojis.some((e) => remaining.startsWith(e))) {
      valueEnd = i;
      break;
    }
    const cp = trimmed.codePointAt(i)!;
    i += cp > 0xffff ? 2 : 1;
  }

  const value = trimmed.slice(0, valueEnd).trim();
  if (!value) return undefined;

  const trimLen = after.length - trimmed.length;
  const newText =
    text.slice(0, pos) +
    text.slice(pos + emojiLen + trimLen + valueEnd);
  return [value, newText];
}

/** Extract metadata from task text. Returns [description, metadata]. */
function extractMetadata(
  text: string,
  config: TaskConfig,
): [string, Record<string, string>] {
  let remaining = text;
  const metadata: Record<string, string> = {};
  const emojis = allEmojis(config);

  for (const field of sortedFields(config)) {
    const pos = remaining.indexOf(field.emoji);
    if (pos === -1) continue;

    const emojiLen = field.emoji.length;

    switch (field.valueType.kind) {
      case "date": {
        const result = extractDateAfter(remaining, pos, emojiLen);
        if (result) {
          metadata[field.fieldName] = result[0];
          remaining = result[1];
        }
        break;
      }
      case "string": {
        const result = extractWordAfter(remaining, pos, emojiLen);
        if (result) {
          metadata[field.fieldName] = result[0];
          remaining = result[1];
        }
        break;
      }
      case "text": {
        const result = extractTextUntilNextEmoji(remaining, pos, emojiLen, emojis);
        if (result) {
          metadata[field.fieldName] = result[0];
          remaining = result[1];
        }
        break;
      }
      case "number": {
        const result = extractNumberAfter(remaining, pos, emojiLen);
        if (result) {
          metadata[field.fieldName] = result[0];
          remaining = result[1];
        }
        break;
      }
      case "flag": {
        metadata[field.fieldName] = field.valueType.value;
        remaining = removeEmoji(remaining, pos, emojiLen);
        break;
      }
      case "enum": {
        metadata[field.fieldName] = field.valueType.value;
        remaining = removeEmoji(remaining, pos, emojiLen);
        break;
      }
    }
  }

  // Clean up whitespace
  const description = remaining.split(/\s+/).filter(Boolean).join(" ");
  return [description, metadata];
}

/** Extract wikilinks from task description. */
function extractTaskLinks(description: string): TaskLink[] {
  const links: TaskLink[] = [];
  WIKILINK_REGEX.lastIndex = 0;
  let m: RegExpExecArray | null;
  while ((m = WIKILINK_REGEX.exec(description)) !== null) {
    const to = (m[1] || "").trim();
    const alias = m[2] ? m[2].trim() || undefined : undefined;
    links.push({ to, alias });
  }
  return links;
}

/** Extract tags from task description. */
function extractTaskTags(description: string): string[] {
  TAG_REGEX.lastIndex = 0;
  const tags: string[] = [];
  let m: RegExpExecArray | null;
  while ((m = TAG_REGEX.exec(description)) !== null) {
    tags.push(m[0]);
  }
  return tags;
}

/** Parse all tasks from content. */
export function parseTasks(
  content: string,
  filePath: string,
  config: TaskConfig,
): Task[] {
  const lines = content.split("\n");
  const codeRanges = findCodeBlockRanges(content);
  const tasks: Task[] = [];
  const parentStack: Array<{ indent: number; line: number }> = [];

  for (let lineIdx = 0; lineIdx < lines.length; lineIdx++) {
    const lineNum = lineIdx + 1;
    const line = lines[lineIdx]!;

    // Skip lines inside code blocks
    if (codeRanges.some((r) => lineNum >= r.startLine && lineNum <= r.endLine)) continue;

    const match = TASK_REGEX.exec(line);
    if (!match) {
      // Non-task line: trim parent stack
      const lineIndent = countIndent(line);
      while (parentStack.length > 0 && parentStack[parentStack.length - 1]!.indent >= lineIndent) {
        parentStack.pop();
      }
      continue;
    }

    const indentStr = match[1]!;
    const symbol = `[${match[2]}]`;
    const rest = match[3]!;
    const indent = countIndent(indentStr);

    // Extract block ID
    const [restWithoutBlock, blockId] = extractBlockId(rest);

    // Extract metadata
    const [description, metadata] = extractMetadata(restWithoutBlock, config);

    // Extract links and tags from description
    const links = extractTaskLinks(description);
    const tags = extractTaskTags(description);

    // Determine parent
    while (parentStack.length > 0 && parentStack[parentStack.length - 1]!.indent >= indent) {
      parentStack.pop();
    }

    parentStack.push({ indent, line: lineNum });

    tasks.push({
      file: filePath,
      line: lineNum,
      raw: line,
      symbol,
      description,
      indent,
      metadata,
      links,
      tags,
      blockId,
    });
  }

  return tasks;
}
