/**
 * Position-independent frontmatter editing for Obsidian notes.
 *
 * Background: `app.fileManager.processFrontMatter` splices new YAML using the
 * metadataCache's frontmatter `position`, which lags after a write — so two
 * writes in close succession can splice at a STALE position and DUPLICATE keys,
 * producing invalid YAML. This helper instead operates on the RAW current file
 * text (read inside `app.vault.process`), finding the block by an anchored scan,
 * so there is no cached position to go stale.
 *
 * Two hard requirements, both learned the hard way:
 *  1. Obsidian's `parseYaml` (eemeli/yaml, uniqueKeys:true) THROWS on duplicate
 *     keys — i.e. on exactly the already-corrupted notes. So we DE-DUPLICATE the
 *     top-level keys (last-wins) of the raw block BEFORE parsing. This makes the
 *     editor self-healing rather than crashing on corrupt input.
 *  2. On any parse failure we cannot resolve, we THROW (the caller's
 *     `app.vault.process` aborts, leaving the file untouched) — we NEVER fall
 *     back to a fresh block, which would silently drop every other frontmatter
 *     key (ticket-id, parents, stage, …).
 *
 * The body is preserved BYTE-FOR-BYTE (sliced at offsets, never split/joined),
 * including its end-of-line convention, a trailing-newline (or its absence), and
 * any `---` lines that appear inside it.
 */

export interface YamlCodec {
  /** Parse a YAML mapping document into an object. Throws on malformed YAML. */
  parse: (s: string) => unknown;
  /** Serialize an object to a YAML mapping document (trailing newline). */
  stringify: (o: unknown) => string;
}

interface BlockInfo {
  /** offset where the YAML content starts (just after the opening `---` line) */
  contentStart: number;
  /** offset where the closing fence line starts */
  contentEnd: number;
  /** offset where the body starts (just after the closing fence line + its EOL) */
  bodyStart: number;
  /** EOL used by the opening fence line ("\n" or "\r\n") */
  eol: string;
  /** whether the closing fence line had a trailing newline */
  closeHadEol: boolean;
}

/** Locate a leading frontmatter block by an anchored raw scan (no cached position). */
function findFrontmatter(text: string): BlockInfo | null {
  // The opening fence must be the very first line: "---" + optional trailing
  // spaces + EOL. (Not "----", not "--- x".)
  const open = /^---[ \t]*(\r?\n)/.exec(text);
  if (!open) return null;
  const eol = open[1]!;
  const contentStart = open[0].length;

  // Scan line-by-line for the FIRST closing fence ("---" or "..." on its own line).
  let i = contentStart;
  while (i <= text.length) {
    const nl = text.indexOf("\n", i);
    const lineEnd = nl === -1 ? text.length : nl;
    let line = text.slice(i, lineEnd);
    if (line.endsWith("\r")) line = line.slice(0, -1);
    if (line === "---" || line === "...") {
      return {
        contentStart,
        contentEnd: i,
        bodyStart: nl === -1 ? text.length : nl + 1,
        eol,
        closeHadEol: nl !== -1,
      };
    }
    if (nl === -1) break;
    i = nl + 1;
  }
  return null; // opened but never closed → treat as no frontmatter (see editFrontmatterText)
}

const indentOf = (line: string): number => line.length - line.replace(/^ +/, "").length;

/** Is `line` a mapping-key line (not blank, comment, or sequence item)? Returns the key, else null. */
function keyOfLine(line: string): string | null {
  const t = line.trim();
  if (t === "" || t.startsWith("#") || t.startsWith("-")) return null;
  // A key line has `key:` followed by EOL or a space (value). Avoid matching
  // `http://...` style values (those are on the value side of a key already).
  const m = /^( *)([^:\s][^:]*):(\s|$)/.exec(line);
  return m ? m[2]!.trim() : null;
}

/**
 * De-duplicate duplicate mapping keys at EVERY nesting level of a raw YAML block
 * (last value wins, kept at the first-seen position). Operates purely on text so
 * it can run BEFORE Obsidian's strict (throwing) parse — making the editor
 * self-healing on the real corruption shape (duplicated keys both at top level
 * and inside nested mappings like `sesh-ticket-data`). Sequences and scalars are
 * preserved untouched.
 */
export function dedupeYamlKeys(block: string): string {
  const lines = block.split("\n");
  return dedupeScope(lines).join("\n");
}

function dedupeScope(lines: string[]): string[] {
  // Base indent = indent of the first key line in this scope.
  let baseIndent = -1;
  for (const line of lines) {
    if (keyOfLine(line) !== null) { baseIndent = indentOf(line); break; }
  }
  if (baseIndent === -1) return lines; // no keys here (sequence/scalar/blank) — leave as-is

  // Segment into groups: each starts at a key line AT baseIndent; deeper/blank
  // lines belong to the current group's value (recursed into).
  const groups: { key: string | null; head: string; body: string[] }[] = [];
  for (const line of lines) {
    const k = keyOfLine(line);
    if (k !== null && indentOf(line) === baseIndent) {
      groups.push({ key: k, head: line, body: [] });
    } else if (groups.length) {
      groups[groups.length - 1]!.body.push(line);
    } else {
      groups.push({ key: null, head: line, body: [] }); // leading cruft above first key
    }
  }

  // Last value wins; emit each key once at its first-seen position; recurse bodies.
  const last = new Map<string, { head: string; body: string[] }>();
  for (const g of groups) if (g.key !== null) last.set(g.key, { head: g.head, body: g.body });
  const emitted = new Set<string>();
  const out: string[] = [];
  for (const g of groups) {
    if (g.key === null) { out.push(g.head, ...g.body); continue; }
    if (emitted.has(g.key)) continue;
    emitted.add(g.key);
    const kept = last.get(g.key)!;
    out.push(kept.head, ...dedupeScope(kept.body));
  }
  return out;
}

/**
 * Apply `mutate` to a note's frontmatter and return the new file text (or the
 * ORIGINAL text byte-for-byte if `mutate` returns false → "no change"). Designed
 * to run inside `app.vault.process(file, data => editFrontmatterText(...))`.
 *
 * `mutate` receives the freshly-parsed (and de-duplicated) frontmatter object,
 * mutates it in place, and returns whether anything changed.
 */
export function editFrontmatterText(
  data: string,
  mutate: (fm: Record<string, unknown>) => boolean,
  codec: YamlCodec,
): string {
  const BOM = String.fromCharCode(0xfeff);
  const hasBom = data.charCodeAt(0) === 0xfeff;
  const bom = hasBom ? BOM : "";
  const text = hasBom ? data.slice(1) : data;

  const info = findFrontmatter(text);

  // --- No (well-formed) frontmatter block: create one, body preserved verbatim. ---
  if (!info) {
    const fm: Record<string, unknown> = {};
    if (!mutate(fm)) return data;
    const eol = text.includes("\r\n") ? "\r\n" : "\n";
    const block = reEol(codec.stringify(fm), eol);
    const sep = text.length === 0 || text.startsWith("\n") || text.startsWith("\r\n") ? "" : eol;
    return `${bom}---${eol}${block}---${eol}${sep}${text}`;
  }

  // --- Existing block: de-dupe → parse (throws on unresolvable malformed YAML). ---
  const rawBlock = text.slice(info.contentStart, info.contentEnd);
  const deduped = dedupeYamlKeys(rawBlock);
  const parsed = deduped.trim() === "" ? {} : codec.parse(deduped);
  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
    // Frontmatter is conventionally a mapping; anything else is malformed. Fail
    // loud rather than silently discard the block.
    throw new Error("frontmatter is not a YAML mapping");
  }
  const fm = parsed as Record<string, unknown>;

  const changed = mutate(fm);
  const wasDuplicated = deduped !== rawBlock;
  // Heal a corrupted (duplicate-key) block even on a no-op mutate; otherwise a
  // no-op preserves the file byte-for-byte.
  if (!changed && !wasDuplicated) return data;

  const block = reEol(codec.stringify(fm), info.eol);
  const body = text.slice(info.bodyStart);
  const close = `---${info.closeHadEol ? info.eol : ""}`;
  return `${bom}---${info.eol}${block}${close}${body}`;
}

/** Re-apply a file's EOL convention to a stringifyYaml result (which uses \n). */
function reEol(yaml: string, eol: string): string {
  return eol === "\n" ? yaml : yaml.replace(/\n/g, eol);
}
