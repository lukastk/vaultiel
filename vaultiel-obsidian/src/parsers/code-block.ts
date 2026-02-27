/**
 * Code block detection for skipping parsing inside code.
 *
 * Ported from vaultiel-rs/src/parser/code_block.rs
 */

export interface CodeBlockRange {
  /** Start byte offset (inclusive). */
  start: number;
  /** End byte offset (exclusive). */
  end: number;
  /** Line number where the code block starts (1-indexed). */
  startLine: number;
  /** Line number where the code block ends (1-indexed). */
  endLine: number;
  /** Whether this is a fenced code block (vs inline code). */
  isFenced: boolean;
}

const FENCE_OPEN = /^(`{3,}|~{3,})/gm;
const INLINE_CODE_SINGLE = /`[^`\n]+`/g;
const INLINE_CODE_DOUBLE = /``(?:[^`]|`[^`])*``/g;

function countNewlines(s: string): number {
  let count = 0;
  for (let i = 0; i < s.length; i++) {
    if (s[i] === "\n") count++;
  }
  return count;
}

/** Find all code block and inline code ranges in content. */
export function findCodeBlockRanges(content: string): CodeBlockRange[] {
  const ranges: CodeBlockRange[] = [];

  // Find fenced code blocks
  let pos = 0;
  while (pos < content.length) {
    FENCE_OPEN.lastIndex = pos;
    const openMatch = FENCE_OPEN.exec(content);
    if (!openMatch) break;

    const absStart = openMatch.index;
    // Only match if at actual start of line (regex ^ handles this in multiline mode)
    const fenceStr = openMatch[1];
    const fenceChar = fenceStr[0];
    const fenceLen = fenceStr.length;

    // Find end of opening line
    const lineEndIdx = content.indexOf("\n", absStart);
    const lineEnd = lineEndIdx === -1 ? content.length : lineEndIdx + 1;

    // Look for matching closing fence
    let searchPos = lineEnd;
    let foundClose = false;

    while (searchPos < content.length) {
      const newlinePos = content.indexOf("\n", searchPos);
      if (newlinePos === -1) {
        // Check the last line
        const lastLine = content.slice(searchPos);
        const trimmed = lastLine.trim();
        if (
          trimmed.length >= fenceLen &&
          [...trimmed].every((c) => c === fenceChar)
        ) {
          const absEnd = content.length;
          const startLine = countNewlines(content.slice(0, absStart)) + 1;
          const endLine = countNewlines(content.slice(0, absEnd)) + 1;
          ranges.push({ start: absStart, end: absEnd, startLine, endLine, isFenced: true });
          pos = absEnd;
          foundClose = true;
        }
        break;
      }

      const nextLineStart = newlinePos + 1;
      if (nextLineStart < content.length) {
        // Find end of this line
        const nextNewline = content.indexOf("\n", nextLineStart);
        const closeLineEnd = nextNewline === -1 ? content.length : nextNewline;
        const closeLine = content.slice(nextLineStart, closeLineEnd);
        const trimmed = closeLine.trim();

        if (
          trimmed.length >= fenceLen &&
          [...trimmed].every((c) => c === fenceChar)
        ) {
          const absEnd = closeLineEnd;
          const startLine = countNewlines(content.slice(0, absStart)) + 1;
          const endLine = countNewlines(content.slice(0, absEnd)) + 1;
          ranges.push({ start: absStart, end: absEnd, startLine, endLine, isFenced: true });
          pos = absEnd;
          foundClose = true;
          break;
        }
      }
      searchPos = newlinePos + 1;
    }

    if (!foundClose) {
      pos = lineEnd;
    }
  }

  // Find inline code with double backticks (not inside fenced blocks)
  INLINE_CODE_DOUBLE.lastIndex = 0;
  let m: RegExpExecArray | null;
  while ((m = INLINE_CODE_DOUBLE.exec(content)) !== null) {
    const start = m.index;
    const end = start + m[0].length;
    if (ranges.some((r) => r.isFenced && start >= r.start && end <= r.end)) continue;
    const startLine = countNewlines(content.slice(0, start)) + 1;
    const endLine = countNewlines(content.slice(0, end)) + 1;
    ranges.push({ start, end, startLine, endLine, isFenced: false });
  }

  // Find inline code with single backticks (not overlapping existing ranges)
  INLINE_CODE_SINGLE.lastIndex = 0;
  while ((m = INLINE_CODE_SINGLE.exec(content)) !== null) {
    const start = m.index;
    const end = start + m[0].length;
    if (
      ranges.some(
        (r) => (start >= r.start && start < r.end) || (end > r.start && end <= r.end),
      )
    ) {
      continue;
    }
    const startLine = countNewlines(content.slice(0, start)) + 1;
    const endLine = countNewlines(content.slice(0, end)) + 1;
    ranges.push({ start, end, startLine, endLine, isFenced: false });
  }

  ranges.sort((a, b) => a.start - b.start);
  return ranges;
}

/** Check if a byte offset is inside any code block. */
export function isInCodeBlock(offset: number, ranges: CodeBlockRange[]): boolean {
  return ranges.some((r) => offset >= r.start && offset < r.end);
}

/** Check if a line number is inside any fenced code block. */
export function isLineInFencedCodeBlock(line: number, ranges: CodeBlockRange[]): boolean {
  return ranges.some((r) => r.isFenced && line >= r.startLine && line <= r.endLine);
}
