/**
 * Search query parser and matcher for vaultiel-obsidian.
 *
 * TypeScript port of the Rust search module (vaultiel-rs/src/search/).
 * Types mirror the Rust types exactly for JSON interop.
 */

import { parseTags } from "./tags.js";
import { parseHeadings } from "./headings.js";
import { parseInlineProperties } from "./inline-properties.js";

// ============================================================================
// Types (mirror Rust search::types exactly)
// ============================================================================

export type SearchQuery =
  | { type: "field"; field: FieldPredicate }
  | { type: "and"; children: SearchQuery[] }
  | { type: "or"; children: SearchQuery[] }
  | { type: "not"; child: SearchQuery };

export type FieldPredicate =
  | { field: "path"; matcher: StringMatcher }
  | { field: "filename"; matcher: StringMatcher }
  | { field: "tag"; value: string }
  | { field: "content"; matcher: StringMatcher }
  | { field: "section"; query: SearchQuery }
  | { field: "line"; query: SearchQuery }
  | { field: "property"; key: string; op: PropertyOp; value?: string };

export type StringMatcher =
  | { kind: "contains"; value: string }
  | { kind: "exact"; value: string }
  | { kind: "regex"; pattern: string };

export type PropertyOp = "exists" | "eq" | "not_eq" | "lt" | "gt" | "lte" | "gte";

export interface SearchMatch {
  field: string;
  line?: number;
  text?: string;
}

export interface SearchResult {
  path: string;
  matches: SearchMatch[];
}

// ============================================================================
// Tokenizer
// ============================================================================

type Token =
  | { type: "word"; value: string }
  | { type: "quoted_string"; value: string }
  | { type: "regex_literal"; value: string }
  | { type: "field_prefix"; value: string }
  | { type: "open_paren" }
  | { type: "close_paren" }
  | { type: "or" }
  | { type: "not" }
  | { type: "comparison_op"; value: string };

const KNOWN_FIELDS = ["path", "filename", "tag", "content", "section", "line", "property"];

function isWordChar(ch: string): boolean {
  return /[\w.\/-]/.test(ch);
}

function tokenize(input: string): Token[] {
  const tokens: Token[] = [];
  const chars = [...input];
  const len = chars.length;
  let i = 0;

  while (i < len) {
    const ch = chars[i]!;

    if (/\s/.test(ch)) { i++; continue; }

    if (ch === "(") { tokens.push({ type: "open_paren" }); i++; continue; }
    if (ch === ")") { tokens.push({ type: "close_paren" }); i++; continue; }

    if (ch === "-" && i + 1 < len && !/\s/.test(chars[i + 1]!)) {
      tokens.push({ type: "not" }); i++; continue;
    }

    if (ch === '"') {
      i++;
      let s = "";
      while (i < len && chars[i] !== '"') {
        if (chars[i] === "\\" && i + 1 < len) { i++; s += chars[i]; }
        else { s += chars[i]; }
        i++;
      }
      if (i < len) i++;
      tokens.push({ type: "quoted_string", value: s });
      continue;
    }

    if (ch === "/") {
      i++;
      let pattern = "";
      while (i < len && chars[i] !== "/") {
        if (chars[i] === "\\" && i + 1 < len) { pattern += chars[i]!; i++; pattern += chars[i]!; }
        else { pattern += chars[i]!; }
        i++;
      }
      if (i < len) i++;
      tokens.push({ type: "regex_literal", value: pattern });
      continue;
    }

    if (ch === "!" && i + 1 < len && chars[i + 1] === "=") {
      tokens.push({ type: "comparison_op", value: "!=" }); i += 2; continue;
    }
    if (ch === "<" && i + 1 < len && chars[i + 1] === "=") {
      tokens.push({ type: "comparison_op", value: "<=" }); i += 2; continue;
    }
    if (ch === ">" && i + 1 < len && chars[i + 1] === "=") {
      tokens.push({ type: "comparison_op", value: ">=" }); i += 2; continue;
    }
    if (ch === "<") { tokens.push({ type: "comparison_op", value: "<" }); i++; continue; }
    if (ch === ">") { tokens.push({ type: "comparison_op", value: ">" }); i++; continue; }
    if (ch === "=") { tokens.push({ type: "comparison_op", value: "=" }); i++; continue; }

    if (isWordChar(ch)) {
      const start = i;
      while (i < len && isWordChar(chars[i]!)) i++;
      const word = chars.slice(start, i).join("");

      if (word === "OR") { tokens.push({ type: "or" }); continue; }

      if (i < len && chars[i] === ":") {
        const lower = word.toLowerCase();
        if (KNOWN_FIELDS.includes(lower)) {
          i++;
          tokens.push({ type: "field_prefix", value: lower });
          continue;
        }
      }

      tokens.push({ type: "word", value: word });
      continue;
    }

    i++;
  }

  return tokens;
}

// ============================================================================
// Parser
// ============================================================================

class Parser {
  private tokens: Token[];
  private pos = 0;

  constructor(tokens: Token[]) {
    this.tokens = tokens;
  }

  private peek(): Token | undefined {
    return this.tokens[this.pos];
  }

  private advance(): Token | undefined {
    return this.tokens[this.pos++];
  }

  private expect(type: string): void {
    const tok = this.advance();
    if (!tok || tok.type !== type) {
      throw new Error(`Expected ${type}, got ${tok ? tok.type : "end of input"}`);
    }
  }

  parseQuery(): SearchQuery {
    return this.parseOrExpr();
  }

  private parseOrExpr(): SearchQuery {
    const children = [this.parseAndExpr()];
    while (this.peek()?.type === "or") {
      this.advance();
      children.push(this.parseAndExpr());
    }
    return children.length === 1 ? children[0]! : { type: "or", children };
  }

  private parseAndExpr(): SearchQuery {
    const children = [this.parseUnaryExpr()];
    while (this.peek() && this.peek()!.type !== "or" && this.peek()!.type !== "close_paren") {
      children.push(this.parseUnaryExpr());
    }
    return children.length === 1 ? children[0]! : { type: "and", children };
  }

  private parseUnaryExpr(): SearchQuery {
    if (this.peek()?.type === "not") {
      this.advance();
      const child = this.parseAtom();
      return { type: "not", child };
    }
    return this.parseAtom();
  }

  private parseAtom(): SearchQuery {
    const tok = this.peek();
    if (!tok) throw new Error("Unexpected end of input");

    if (tok.type === "field_prefix") return this.parseFieldExpr();
    if (tok.type === "open_paren") {
      this.advance();
      const q = this.parseQuery();
      this.expect("close_paren");
      return q;
    }
    if (tok.type === "word" || tok.type === "quoted_string" || tok.type === "regex_literal") {
      const matcher = this.parseStringMatcher();
      return { type: "field", field: { field: "content", matcher } };
    }

    throw new Error(`Unexpected token: ${tok.type}`);
  }

  private parseFieldExpr(): SearchQuery {
    const tok = this.advance()!;
    if (tok.type !== "field_prefix") throw new Error("Expected field prefix");
    const fieldName = (tok as { type: "field_prefix"; value: string }).value;

    switch (fieldName) {
      case "property": return this.parsePropertyExpr();
      case "tag": return this.parseTagExpr();
      case "section": return this.parseScopingExpr("section");
      case "line": return this.parseScopingExpr("line");
      case "path": {
        const matcher = this.parseStringMatcher();
        return { type: "field", field: { field: "path", matcher } };
      }
      case "filename": {
        const matcher = this.parseStringMatcher();
        return { type: "field", field: { field: "filename", matcher } };
      }
      case "content": {
        const matcher = this.parseStringMatcher();
        return { type: "field", field: { field: "content", matcher } };
      }
      default: throw new Error(`Unknown field: ${fieldName}`);
    }
  }

  private parsePropertyExpr(): SearchQuery {
    const keyTok = this.advance();
    if (!keyTok || (keyTok.type !== "word" && keyTok.type !== "quoted_string")) {
      throw new Error("Expected property key");
    }
    const key = (keyTok as { value: string }).value;

    if (this.peek()?.type === "comparison_op") {
      const opTok = this.advance()! as { type: "comparison_op"; value: string };
      const opMap: Record<string, PropertyOp> = {
        "=": "eq", "!=": "not_eq", "<": "lt", ">": "gt", "<=": "lte", ">=": "gte",
      };
      const op = opMap[opTok.value]!;

      const valTok = this.advance();
      if (!valTok || (valTok.type !== "word" && valTok.type !== "quoted_string")) {
        throw new Error("Expected property value");
      }
      const value = (valTok as { value: string }).value;

      return { type: "field", field: { field: "property", key, op, value } };
    }

    return { type: "field", field: { field: "property", key, op: "exists" } };
  }

  private parseTagExpr(): SearchQuery {
    if (this.peek()?.type === "open_paren") {
      this.advance();
      const children: SearchQuery[] = [];
      const firstTok = this.advance();
      if (!firstTok) throw new Error("Expected tag value");
      children.push({ type: "field", field: { field: "tag", value: (firstTok as { value: string }).value } });

      while (this.peek()?.type === "or") {
        this.advance();
        const nextTok = this.advance();
        if (!nextTok) throw new Error("Expected tag value after OR");
        children.push({ type: "field", field: { field: "tag", value: (nextTok as { value: string }).value } });
      }
      this.expect("close_paren");
      return children.length === 1 ? children[0]! : { type: "or", children };
    }

    const tok = this.advance();
    if (!tok) throw new Error("Expected tag value");
    return { type: "field", field: { field: "tag", value: (tok as { value: string }).value } };
  }

  private parseScopingExpr(scope: "section" | "line"): SearchQuery {
    let subQuery: SearchQuery;
    if (this.peek()?.type === "open_paren") {
      this.advance();
      subQuery = this.parseQuery();
      this.expect("close_paren");
    } else {
      const matcher = this.parseStringMatcher();
      subQuery = { type: "field", field: { field: "content", matcher } };
    }
    return { type: "field", field: { field: scope, query: subQuery } };
  }

  private parseStringMatcher(): StringMatcher {
    const tok = this.advance();
    if (!tok) throw new Error("Expected string value");
    switch (tok.type) {
      case "quoted_string": return { kind: "exact", value: (tok as { value: string }).value };
      case "regex_literal": return { kind: "regex", pattern: (tok as { value: string }).value };
      case "word": return { kind: "contains", value: (tok as { value: string }).value };
      default: throw new Error(`Expected string, got ${tok.type}`);
    }
  }
}

// ============================================================================
// Public parser API
// ============================================================================

export function parseSearchQuery(input: string): SearchQuery {
  const trimmed = input.trim();
  if (!trimmed) throw new Error("Empty search query");

  const tokens = tokenize(trimmed);
  if (tokens.length === 0) throw new Error("Empty search query");

  const parser = new Parser(tokens);
  return parser.parseQuery();
}

// ============================================================================
// Matcher
// ============================================================================

function matchesString(haystack: string, matcher: StringMatcher, caseSensitive: boolean): boolean {
  switch (matcher.kind) {
    case "contains":
      return caseSensitive
        ? haystack.includes(matcher.value)
        : haystack.toLowerCase().includes(matcher.value.toLowerCase());
    case "exact":
      return caseSensitive
        ? haystack === matcher.value
        : haystack.toLowerCase() === matcher.value.toLowerCase();
    case "regex": {
      const re = new RegExp(matcher.pattern, caseSensitive ? "" : "i");
      return re.test(haystack);
    }
  }
}

function splitFrontmatter(content: string): { body: string; bodyStartLine: number } {
  if (!content.startsWith("---\n") && !content.startsWith("---\r\n")) {
    return { body: content, bodyStartLine: 1 };
  }
  const endIdx = content.indexOf("\n---", 4);
  if (endIdx === -1) return { body: content, bodyStartLine: 1 };

  // Find the end of the closing ---
  let afterEnd = endIdx + 4; // skip \n---
  if (content[afterEnd] === "\r") afterEnd++;
  if (content[afterEnd] === "\n") afterEnd++;

  const fmLines = content.slice(0, afterEnd).split("\n").length;
  return { body: content.slice(afterEnd), bodyStartLine: fmLines };
}

function getFrontmatter(content: string): Record<string, unknown> | null {
  if (!content.startsWith("---\n") && !content.startsWith("---\r\n")) return null;
  const endIdx = content.indexOf("\n---", 4);
  if (endIdx === -1) return null;

  const yamlStr = content.slice(4, endIdx);
  // Simple YAML parser for common frontmatter
  const result: Record<string, unknown> = {};
  let currentKey = "";
  let inArray = false;
  let arrayValues: string[] = [];

  for (const line of yamlStr.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed) continue;

    if (inArray) {
      if (trimmed.startsWith("- ")) {
        arrayValues.push(trimmed.slice(2).trim());
        continue;
      } else {
        result[currentKey] = arrayValues;
        inArray = false;
        arrayValues = [];
      }
    }

    const colonIdx = line.indexOf(":");
    if (colonIdx === -1) continue;
    const key = line.slice(0, colonIdx).trim();
    const value = line.slice(colonIdx + 1).trim();

    if (!value) {
      // Could be start of an array
      currentKey = key;
      inArray = true;
      arrayValues = [];
      continue;
    }

    // Parse value
    if (value === "true") result[key] = true;
    else if (value === "false") result[key] = false;
    else if (value === "null") result[key] = null;
    else if (/^-?\d+$/.test(value)) result[key] = parseInt(value, 10);
    else if (/^-?\d+\.\d+$/.test(value)) result[key] = parseFloat(value);
    else result[key] = value;
  }

  if (inArray) {
    result[currentKey] = arrayValues;
  }

  return Object.keys(result).length > 0 ? result : null;
}

export function evaluateNote(path: string, content: string, query: SearchQuery): SearchMatch[] {
  try {
    return evaluateInner(path, content, query);
  } catch {
    return [];
  }
}

function evaluateInner(path: string, content: string, query: SearchQuery): SearchMatch[] {
  switch (query.type) {
    case "field":
      return evaluateField(path, content, query.field);
    case "and": {
      const all: SearchMatch[] = [];
      for (const child of query.children) {
        const m = evaluateInner(path, content, child);
        if (m.length === 0) return [];
        all.push(...m);
      }
      return all;
    }
    case "or": {
      for (const child of query.children) {
        const m = evaluateInner(path, content, child);
        if (m.length > 0) return m;
      }
      return [];
    }
    case "not": {
      const m = evaluateInner(path, content, query.child);
      return m.length === 0 ? [{ field: "not" }] : [];
    }
  }
}

function evaluateField(path: string, content: string, pred: FieldPredicate): SearchMatch[] {
  switch (pred.field) {
    case "path":
      return matchesString(path, pred.matcher, true)
        ? [{ field: "path", text: path }]
        : [];
    case "filename": {
      const name = path.replace(/\.md$/, "").split("/").pop() ?? path;
      return matchesString(name, pred.matcher, true)
        ? [{ field: "filename", text: name }]
        : [];
    }
    case "tag":
      return evaluateTag(content, pred.value);
    case "content":
      return evaluateContent(content, pred.matcher);
    case "section":
      return evaluateSection(path, content, pred.query);
    case "line":
      return evaluateLine(path, content, pred.query);
    case "property":
      return evaluateProperty(content, pred.key, pred.op, pred.value);
  }
}

function evaluateTag(content: string, value: string): SearchMatch[] {
  const valueLower = value.toLowerCase().replace(/^#/, "");

  const tags = parseTags(content);
  for (const tag of tags) {
    const tagStripped = tag.name.replace(/^#/, "").toLowerCase();
    if (tagStripped === valueLower) {
      return [{ field: "tag", line: tag.line, text: tag.name }];
    }
  }

  const fm = getFrontmatter(content);
  if (fm?.tags && Array.isArray(fm.tags)) {
    for (const t of fm.tags) {
      const s = String(t).replace(/^#/, "").toLowerCase();
      if (s === valueLower) {
        return [{ field: "tag", line: 1, text: String(t) }];
      }
    }
  }

  return [];
}

function evaluateContent(content: string, matcher: StringMatcher): SearchMatch[] {
  const { body, bodyStartLine } = splitFrontmatter(content);
  const matches: SearchMatch[] = [];

  const lines = body.split("\n");
  for (let i = 0; i < lines.length; i++) {
    if (matchesString(lines[i]!, matcher, false)) {
      matches.push({ field: "content", line: bodyStartLine + i, text: lines[i] });
    }
  }

  return matches;
}

function evaluateSection(path: string, content: string, subQuery: SearchQuery): SearchMatch[] {
  const { body, bodyStartLine } = splitFrontmatter(content);
  const headings = parseHeadings(content);
  const bodyLines = body.split("\n");

  const headingBodyIndices = headings
    .filter(h => h.line >= bodyStartLine)
    .map(h => h.line - bodyStartLine);

  const sections: [number, number][] = [];
  if (headingBodyIndices.length === 0) {
    sections.push([0, bodyLines.length]);
  } else {
    if (headingBodyIndices[0]! > 0) sections.push([0, headingBodyIndices[0]!]);
    for (let i = 0; i < headingBodyIndices.length; i++) {
      const start = headingBodyIndices[i]!;
      const end = i + 1 < headingBodyIndices.length ? headingBodyIndices[i + 1]! : bodyLines.length;
      sections.push([start, end]);
    }
  }

  for (const [start, end] of sections) {
    const sectionText = bodyLines.slice(start, end).join("\n");
    const m = evaluateInner(path, sectionText, subQuery);
    if (m.length > 0) {
      return m.map(match => ({
        ...match,
        field: `section:${match.field}`,
        line: match.line !== undefined ? match.line + bodyStartLine + start - 1 : undefined,
      }));
    }
  }

  return [];
}

function evaluateLine(path: string, content: string, subQuery: SearchQuery): SearchMatch[] {
  const { body, bodyStartLine } = splitFrontmatter(content);

  for (const [i, line] of body.split("\n").entries()) {
    const m = evaluateInner(path, line, subQuery);
    if (m.length > 0) {
      return m.map(match => ({
        ...match,
        field: `line:${match.field}`,
        line: bodyStartLine + i,
      }));
    }
  }

  return [];
}

function evaluateProperty(
  content: string,
  key: string,
  op: PropertyOp,
  expectedValue: string | undefined,
): SearchMatch[] {
  const found: { value: string; line?: number }[] = [];

  const fm = getFrontmatter(content);
  if (fm && key in fm) {
    const v = fm[key];
    found.push({ value: String(v), line: 1 });
  }

  const inlineProps = parseInlineProperties(content);
  for (const prop of inlineProps) {
    if (prop.key === key) {
      found.push({ value: prop.value, line: prop.line });
    }
  }

  if (found.length === 0) return [];

  if (op === "exists") {
    return [{ field: `property:${key}`, line: found[0]!.line, text: found[0]!.value }];
  }

  const expected = expectedValue ?? "";
  for (const { value, line } of found) {
    if (compareValues(value, expected, op)) {
      return [{ field: `property:${key}`, line, text: value }];
    }
  }

  return [];
}

function compareValues(actual: string, expected: string, op: PropertyOp): boolean {
  const aNum = Number(actual);
  const bNum = Number(expected);
  if (!isNaN(aNum) && !isNaN(bNum)) {
    switch (op) {
      case "eq": return Math.abs(aNum - bNum) < Number.EPSILON;
      case "not_eq": return Math.abs(aNum - bNum) >= Number.EPSILON;
      case "lt": return aNum < bNum;
      case "gt": return aNum > bNum;
      case "lte": return aNum <= bNum;
      case "gte": return aNum >= bNum;
      case "exists": return true;
    }
  }

  switch (op) {
    case "eq": return actual.toLowerCase() === expected.toLowerCase();
    case "not_eq": return actual.toLowerCase() !== expected.toLowerCase();
    case "lt": return actual < expected;
    case "gt": return actual > expected;
    case "lte": return actual <= expected;
    case "gte": return actual >= expected;
    case "exists": return true;
  }
}
