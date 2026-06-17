import { describe, it, expect } from "vitest";
import * as YAML from "yaml";
import { editFrontmatterText, dedupeYamlKeys, type YamlCodec } from "../src/frontmatter-edit.js";

// Obsidian's parseYaml/stringifyYaml are eemeli `yaml` v2 (uniqueKeys: true → throws
// on duplicate keys). Using the same lib here matches production behavior.
const codec: YamlCodec = {
  parse: (s) => YAML.parse(s),
  stringify: (o) => YAML.stringify(o),
};

/** Apply a multi-key patch (undefined = leave untouched). */
function patch(data: string, p: Record<string, unknown>): string {
  return editFrontmatterText(
    data,
    (fm) => {
      let changed = false;
      for (const [k, v] of Object.entries(p)) {
        if (v === undefined) continue;
        fm[k] = v;
        changed = true;
      }
      return changed;
    },
    codec,
  );
}

/** A no-op edit (mutate returns false). */
function noop(data: string): string {
  return editFrontmatterText(data, () => false, codec);
}

/** Parse the frontmatter mapping out of a note's text (test oracle). */
function readFm(data: string): Record<string, unknown> {
  const m = /^﻿?---[ \t]*\r?\n([\s\S]*?)\r?\n(?:---|\.\.\.)[ \t]*(?:\r?\n|$)/.exec(data);
  return m ? (YAML.parse(m[1]!) ?? {}) : {};
}

describe("editFrontmatterText", () => {
  // ---- Duplicate-key healing (D1) ----
  describe("duplicate-key healing", () => {
    it("heals duplicate scalar key (last wins), no throw", () => {
      const out = patch("---\na: 1\na: 2\n---\nbody", { b: 3 });
      const fm = readFm(out);
      expect(fm).toEqual({ a: 2, b: 3 });
      expect((out.match(/^a:/gm) || []).length).toBe(1);
    });

    it("heals a duplicated nested block (the real corruption shape)", () => {
      const corrupt = [
        "---",
        "ticket-id: cc1",
        "sesh-ticket-data:",
        "  machine: mymain",
        "  status: active",
        "  closedAt: 0",
        "  machine: mymain",
        "  status: active",
        "  closedAt: 3521",
        "---",
        "the body",
      ].join("\n");
      // A no-op edit still heals because the block was duplicated.
      const out = noop(corrupt);
      const fm = readFm(out);
      expect(fm["ticket-id"]).toBe("cc1");
      expect((fm["sesh-ticket-data"] as any).closedAt).toBe(3521); // last-wins within the kept block
      expect(out.endsWith("the body")).toBe(true);
      // And it must be valid YAML now (no throw on re-parse).
      expect(() => YAML.parse(readFmBlock(out))).not.toThrow();
    });

    it("heals while patching a different key", () => {
      const out = patch("---\nx: 1\nx: 9\ny: 2\n---\n", { z: 3 });
      expect(readFm(out)).toEqual({ x: 9, y: 2, z: 3 });
    });
  });

  // ---- Boundary detection (D7) ----
  describe("boundary detection", () => {
    it("a body line of --- stays in the body", () => {
      const out = patch("---\na: 1\n---\nintro\n---\nmore", { b: 2 });
      expect(readFm(out)).toEqual({ a: 1, b: 2 });
      expect(out).toContain("intro\n---\nmore");
    });

    it("a fenced YAML example in the body is untouched", () => {
      const body = "text\n```yaml\nk: v\n---\n```\nend";
      const out = patch(`---\na: 1\n---\n${body}`, { b: 2 });
      expect(out.endsWith(body)).toBe(true);
    });

    it("a leftover second --- block in the body is preserved verbatim", () => {
      const out = patch("---\na: 1\n---\n\n---\nstale: block\n---\n", { b: 2 });
      expect(readFm(out)).toEqual({ a: 1, b: 2 });
      expect(out).toContain("\n---\nstale: block\n---\n");
    });

    it("no frontmatter at all → block created, body byte-identical after it", () => {
      const out = patch("just a body\nline two", { a: 1 });
      expect(readFm(out)).toEqual({ a: 1 });
      expect(out.endsWith("just a body\nline two")).toBe(true);
      expect(out.startsWith("---\n")).toBe(true);
    });

    it("empty block → key inserted", () => {
      const out = patch("---\n---\nbody", { a: 1 });
      expect(readFm(out)).toEqual({ a: 1 });
      expect(out.endsWith("body")).toBe(true);
    });

    it("opening --- not at byte 0 → treated as no frontmatter", () => {
      const out = patch("\n---\na: 1\n---\nbody", { b: 2 });
      // Leading blank line means no frontmatter block; a new one is prepended.
      expect(readFm(out)).toEqual({ b: 2 });
      expect(out).toContain("\n---\na: 1\n---\nbody");
    });

    it("opened but never closed → treated as no frontmatter (no data loss)", () => {
      const out = patch("---\na: 1\nno close here", { b: 2 });
      expect(out).toContain("---\na: 1\nno close here");
      expect(readFm(out)).toEqual({ b: 2 });
    });
  });

  // ---- Body byte-preservation (D7/D8) ----
  describe("byte preservation", () => {
    it("preserves CRLF body verbatim", () => {
      const out = patch("---\na: 1\n---\r\nline1\r\nline2", { b: 2 });
      expect(out.endsWith("line1\r\nline2")).toBe(true);
    });

    it("file with trailing newline keeps exactly one; without gains none", () => {
      const withNl = patch("---\na: 1\n---\nbody\n", { b: 2 });
      expect(withNl.endsWith("body\n")).toBe(true);
      expect(withNl.endsWith("body\n\n")).toBe(false);
      const without = patch("---\na: 1\n---\nbody", { b: 2 });
      expect(without.endsWith("body")).toBe(true);
    });

    it("BOM before --- is preserved and does NOT cause block prepend", () => {
      const out = patch("﻿---\na: 1\n---\nbody", { b: 2 });
      expect(out.charCodeAt(0)).toBe(0xfeff);
      expect(readFm(out)).toEqual({ a: 1, b: 2 });
      // exactly 2 fences (one open + one close); a BOM-induced prepend would make 4.
      expect((out.replace(/^﻿/, "").match(/(^|\n)---[ \t]*(\r?\n|$)/g) || []).length).toBe(2);
    });

    it("CRLF frontmatter block round-trips with CRLF", () => {
      const out = patch("---\r\na: 1\r\n---\r\nbody", { b: 2 });
      expect(readFm(out)).toEqual({ a: 1, b: 2 });
      expect(out).toContain("\r\n");
    });
  });

  // ---- Value-shape round-trips (CHANGE 2 / sesh-ticket-data) ----
  describe("value shapes", () => {
    const seshData = {
      found: true,
      machine: "mymain",
      status: "active",
      closedAt: 0,
      syncedAt: 1700000000,
      thread: { id: "x", name: "n", parent: "p" },
      promptHash: "abc",
    };

    it("SeshTicketData round-trips with types preserved", () => {
      const out = patch("---\nticket-id: cc1\n---\nbody", { "sesh-ticket-data": seshData });
      const fm = readFm(out);
      expect(fm["sesh-ticket-data"]).toEqual(seshData);
      const sd = fm["sesh-ticket-data"] as any;
      expect(typeof sd.closedAt).toBe("number");
      expect(typeof sd.syncedAt).toBe("number");
      expect(typeof sd.found).toBe("boolean");
      expect(typeof sd.thread).toBe("object");
    });

    it("a no-op write after setting is byte-stable (no churn loop)", () => {
      const once = patch("---\nticket-id: cc1\n---\nbody", { "sesh-ticket-data": seshData });
      const twice = noop(once);
      expect(twice).toBe(once);
    });

    it("timestamps emit unquoted and re-read as numbers", () => {
      const out = patch("---\n---\n", { closedAt: 0, syncedAt: 1700000000 });
      expect(out).toMatch(/syncedAt: 1700000000/);
      expect(readFm(out)).toMatchObject({ closedAt: 0, syncedAt: 1700000000 });
    });
  });

  // ---- Null / delete semantics ----
  describe("null/undefined", () => {
    it("setting a value to null clears it (re-reads as null)", () => {
      const out = editFrontmatterText("---\na: 1\nb: 2\n---\n", (fm) => { fm["a"] = null; return true; }, codec);
      expect(readFm(out)).toEqual({ a: null, b: 2 });
    });

    it("undefined in a patch leaves the key untouched", () => {
      const out = patch("---\na: 1\nb: 2\n---\n", { a: undefined, c: 3 });
      expect(readFm(out)).toEqual({ a: 1, b: 2, c: 3 });
    });
  });

  // ---- Quoting / special chars ----
  describe("special chars", () => {
    it("value with a colon round-trips as a string (not a nested map)", () => {
      const out = patch("---\n---\n", { k: "a: b" });
      expect(readFm(out)).toEqual({ k: "a: b" });
    });
    it("numeric-looking and boolean-looking strings stay strings", () => {
      const out = patch("---\n---\n", { a: "007", b: "true", c: "null" });
      expect(readFm(out)).toEqual({ a: "007", b: "true", c: "null" });
    });
    it("unicode / emoji values round-trip", () => {
      const out = patch("---\n---\n", { a: "🎫 票", b: "日本語" });
      expect(readFm(out)).toEqual({ a: "🎫 票", b: "日本語" });
    });
    it("leading special chars round-trip", () => {
      const out = patch("---\n---\n", { a: "# h", b: "- x", c: "@y", d: " spaced " });
      expect(readFm(out)).toEqual({ a: "# h", b: "- x", c: "@y", d: " spaced " });
    });
  });

  // ---- Order / position ----
  describe("order/position", () => {
    it("updating an existing key changes value in place (does not move to end)", () => {
      const out = patch("---\na: 1\nb: 2\nc: 3\n---\n", { b: 9 });
      const keys = Object.keys(readFm(out));
      expect(keys).toEqual(["a", "b", "c"]);
      expect(readFm(out)).toEqual({ a: 1, b: 9, c: 3 });
    });
    it("adding a new key appends", () => {
      const out = patch("---\na: 1\nb: 2\n---\n", { z: 3 });
      expect(Object.keys(readFm(out))).toEqual(["a", "b", "z"]);
    });
  });

  // ---- Idempotency ----
  it("applying the same patch twice is byte-identical the second time", () => {
    const one = patch("---\na: 1\n---\nbody\n", { b: 2 });
    const two = patch(one, { b: 2 });
    expect(two).toBe(one);
  });

  // ---- Round-trip fidelity baseline ----
  it("a representative real block is byte-stable on a no-op", () => {
    const note = [
      "---",
      "notetype: tkt",
      "createdat: 2026-06-17T07:52:19.255Z",
      "stage: live",
      "parents:",
      '  - "[[Create sesh V2 (again)]]"',
      "DOC: false",
      "ticket-id: cc1d7ef5",
      "sesh-ticket-data:",
      "  found: true",
      "  machine: mymain",
      "  status: active",
      "  closedAt: 0",
      "  thread:",
      "    id: b7570793",
      '    name: mysetup - sesh',
      '    parent: ""',
      "---",
      "",
      "the body",
    ].join("\n");
    // First normalize via one no-op (the block was authored by stringifyYaml-equivalent),
    // then assert a second no-op is byte-identical.
    const normalized = editFrontmatterText(note, (fm) => { fm["__t"] = 1; return true; }, codec);
    const reverted = editFrontmatterText(normalized, (fm) => { delete fm["__t"]; return true; }, codec);
    const again = noop(reverted);
    expect(again).toBe(reverted);
    expect(reverted.endsWith("\nthe body")).toBe(true);
  });
});

describe("dedupeYamlKeys", () => {
  it("is identity for a block with no duplicates", () => {
    const b = "a: 1\nb:\n  c: 2\nd: 3";
    expect(dedupeYamlKeys(b)).toBe(b);
  });
  it("keeps last value at first-seen position", () => {
    expect(dedupeYamlKeys("a: 1\nb: 2\na: 9")).toBe("a: 9\nb: 2");
  });
  it("keeps the last full nested block", () => {
    const b = "k:\n  x: 1\nk:\n  y: 2";
    expect(dedupeYamlKeys(b)).toBe("k:\n  y: 2");
  });
  it("dedupes duplicate keys NESTED inside a mapping (the real corruption shape)", () => {
    const b = ["sd:", "  machine: mymain", "  status: active", "  closedAt: 0", "  machine: mymain", "  status: active", "  closedAt: 3521"].join("\n");
    const out = dedupeYamlKeys(b);
    expect(() => YAML.parse(out)).not.toThrow();
    expect(YAML.parse(out)).toEqual({ sd: { machine: "mymain", status: "active", closedAt: 3521 } });
  });
  it("preserves sequences untouched", () => {
    const b = "parents:\n  - a\n  - b\nx: 1";
    expect(dedupeYamlKeys(b)).toBe(b);
  });
});

/** Helper: extract the raw frontmatter block text for a re-parse assertion. */
function readFmBlock(data: string): string {
  const m = /^﻿?---[ \t]*\r?\n([\s\S]*?)\r?\n(?:---|\.\.\.)[ \t]*(?:\r?\n|$)/.exec(data);
  return m ? m[1]! : "";
}
