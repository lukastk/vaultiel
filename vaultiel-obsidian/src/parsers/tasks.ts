/**
 * Task parsing for Obsidian Tasks plugin compatibility.
 *
 * Ported from vaultiel-rs/src/parser/task.rs
 */

import type { Task, TaskLink, TaskConfig } from "../types.js";
import { DEFAULT_TASK_CONFIG } from "../types.js";
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

/** Check if character is likely the start of an emoji used in task metadata. */
function isEmojiStart(c: string): boolean {
  const code = c.codePointAt(0);
  if (code === undefined) return false;
  return (
    (code >= 0x1f100 && code <= 0x1f1ff) || // Enclosed Alphanumeric (🆔)
    (code >= 0x1f300 && code <= 0x1f9ff) || // Misc Symbols, Emoticons
    (code >= 0x2600 && code <= 0x26ff) || // Misc Symbols (⛔)
    (code >= 0x2700 && code <= 0x27bf) || // Dingbats (➕, ❌, ✅)
    (code >= 0x231a && code <= 0x231b) || // Watch, Hourglass
    (code >= 0x23e9 && code <= 0x23f3) // ⏳, ⏫, ⏬, ⏲
  );
}

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

/** Extract a date field: find emoji, extract ISO date after it, remove from string. */
function extractDateField(
  remaining: { value: string },
  emoji: string,
): string | undefined {
  const pos = remaining.value.indexOf(emoji);
  if (pos === -1) return undefined;

  const after = remaining.value.slice(pos + emoji.length);
  const trimmed = after.trimStart();
  const dateMatch = DATE_REGEX.exec(trimmed);
  if (!dateMatch) return undefined;

  const value = dateMatch[0];
  const trimLen = after.length - trimmed.length;
  remaining.value =
    remaining.value.slice(0, pos) +
    remaining.value.slice(pos + emoji.length + trimLen + dateMatch.index + value.length);
  return value;
}

/** Extract a text field: find emoji, extract text until next emoji or end. */
function extractTextField(
  remaining: { value: string },
  emoji: string,
): string | undefined {
  const pos = remaining.value.indexOf(emoji);
  if (pos === -1) return undefined;

  const after = remaining.value.slice(pos + emoji.length);
  const trimmed = after.trimStart();

  // Find end: next emoji or end of string
  let valueEnd = trimmed.length;
  for (let i = 0; i < trimmed.length; ) {
    const cp = trimmed.codePointAt(i)!;
    const char = String.fromCodePoint(cp);
    if (isEmojiStart(char)) {
      valueEnd = i;
      break;
    }
    i += char.length;
  }

  const value = trimmed.slice(0, valueEnd).trim();
  if (!value) return undefined;

  const trimLen = after.length - trimmed.length;
  remaining.value =
    remaining.value.slice(0, pos) +
    remaining.value.slice(pos + emoji.length + trimLen + valueEnd);
  return value;
}

interface TaskMetadata {
  scheduled?: string;
  due?: string;
  done?: string;
  start?: string;
  created?: string;
  cancelled?: string;
  recurrence?: string;
  onCompletion?: string;
  id?: string;
  dependsOn: string[];
  priority?: string;
}

/** Extract metadata from task text. Returns [description, metadata]. */
function extractMetadata(
  text: string,
  config: TaskConfig,
): [string, TaskMetadata] {
  const remaining = { value: text };
  const metadata: TaskMetadata = { dependsOn: [] };

  // Extract custom metadata first
  for (const [key, emoji] of Object.entries(config.customMetadata)) {
    const pos = remaining.value.indexOf(emoji);
    if (pos === -1) continue;

    const after = remaining.value.slice(pos + emoji.length);
    const trimmed = after.trimStart();
    const trimLen = after.length - trimmed.length;

    let valueEnd = trimmed.length;
    for (let i = 0; i < trimmed.length; ) {
      const cp = trimmed.codePointAt(i)!;
      const char = String.fromCodePoint(cp);
      if (isEmojiStart(char)) {
        valueEnd = i;
        break;
      }
      i += char.length;
    }

    const rawValue = trimmed.slice(0, valueEnd).trim();
    if (rawValue) {
      // Check if it's a date
      const dateMatch = DATE_REGEX.exec(rawValue);
      const _value = dateMatch ? dateMatch[0] : rawValue.split(/\s/)[0];

      remaining.value =
        remaining.value.slice(0, pos) +
        remaining.value.slice(pos + emoji.length + trimLen + valueEnd);
    }
  }

  // Extract text-based fields
  metadata.id = extractTextField(remaining, config.id);

  const rawDeps = extractTextField(remaining, config.dependsOn);
  if (rawDeps) {
    metadata.dependsOn = rawDeps
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean);
  }

  metadata.recurrence = extractTextField(remaining, config.recurrence);
  metadata.onCompletion = extractTextField(remaining, config.onCompletion);

  // Extract date fields
  metadata.start = extractDateField(remaining, config.start);
  metadata.created = extractDateField(remaining, config.created);
  metadata.scheduled = extractDateField(remaining, config.scheduled);
  metadata.due = extractDateField(remaining, config.due);
  metadata.cancelled = extractDateField(remaining, config.cancelled);
  metadata.done = extractDateField(remaining, config.done);

  // Extract priority (check highest to lowest)
  if (remaining.value.includes(config.priorityHighest)) {
    metadata.priority = "highest";
    remaining.value = remaining.value.replace(config.priorityHighest, "");
  } else if (remaining.value.includes(config.priorityHigh)) {
    metadata.priority = "high";
    remaining.value = remaining.value.replace(config.priorityHigh, "");
  } else if (remaining.value.includes(config.priorityMedium)) {
    metadata.priority = "medium";
    remaining.value = remaining.value.replace(config.priorityMedium, "");
  } else if (remaining.value.includes(config.priorityLow)) {
    metadata.priority = "low";
    remaining.value = remaining.value.replace(config.priorityLow, "");
  } else if (remaining.value.includes(config.priorityLowest)) {
    metadata.priority = "lowest";
    remaining.value = remaining.value.replace(config.priorityLowest, "");
  }

  // Clean up whitespace
  const description = remaining.value.split(/\s+/).filter(Boolean).join(" ");

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
  config: TaskConfig = DEFAULT_TASK_CONFIG,
): Task[] {
  const lines = content.split("\n");
  const codeRanges = findCodeBlockRanges(content);
  const tasks: Task[] = [];
  const parentStack: Array<{ indent: number; line: number }> = [];

  for (let lineIdx = 0; lineIdx < lines.length; lineIdx++) {
    const lineNum = lineIdx + 1;
    const line = lines[lineIdx];

    // Skip lines inside code blocks
    if (codeRanges.some((r) => lineNum >= r.startLine && lineNum <= r.endLine)) continue;

    const match = TASK_REGEX.exec(line);
    if (!match) {
      // Non-task line: trim parent stack
      const lineIndent = countIndent(line);
      while (parentStack.length > 0 && parentStack[parentStack.length - 1].indent >= lineIndent) {
        parentStack.pop();
      }
      continue;
    }

    const indentStr = match[1];
    const symbol = `[${match[2]}]`;
    const rest = match[3];
    const indent = countIndent(indentStr);

    // Extract block ID
    const [restWithoutBlock, blockId] = extractBlockId(rest);

    // Extract metadata
    const [description, metadata] = extractMetadata(restWithoutBlock, config);

    // Extract links and tags from description
    const links = extractTaskLinks(description);
    const tags = extractTaskTags(description);

    // Determine parent
    while (parentStack.length > 0 && parentStack[parentStack.length - 1].indent >= indent) {
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
      scheduled: metadata.scheduled,
      due: metadata.due,
      done: metadata.done,
      start: metadata.start,
      created: metadata.created,
      cancelled: metadata.cancelled,
      recurrence: metadata.recurrence,
      onCompletion: metadata.onCompletion,
      id: metadata.id,
      dependsOn: metadata.dependsOn,
      priority: metadata.priority,
      links,
      tags,
      blockId,
    });
  }

  return tasks;
}
