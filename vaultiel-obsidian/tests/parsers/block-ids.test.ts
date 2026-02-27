import { describe, it, expect } from "vitest";
import { parseBlockIds } from "../../src/parsers/block-ids.js";

describe("parseBlockIds", () => {
  it("parses a simple block ID", () => {
    const blocks = parseBlockIds("Some paragraph text ^abc123");
    expect(blocks).toHaveLength(1);
    expect(blocks[0].id).toBe("abc123");
    expect(blocks[0].line).toBe(1);
    expect(blocks[0].blockType).toBe("paragraph");
  });

  it("detects list item block type", () => {
    const blocks = parseBlockIds("- List item ^list-id");
    expect(blocks).toHaveLength(1);
    expect(blocks[0].id).toBe("list-id");
    expect(blocks[0].blockType).toBe("listitem");
  });

  it("detects heading block type", () => {
    const blocks = parseBlockIds("# Heading ^head-id");
    expect(blocks).toHaveLength(1);
    expect(blocks[0].id).toBe("head-id");
    expect(blocks[0].blockType).toBe("heading");
  });

  it("detects blockquote block type", () => {
    const blocks = parseBlockIds("> Quote text ^quote-id");
    expect(blocks).toHaveLength(1);
    expect(blocks[0].blockType).toBe("blockquote");
  });

  it("parses multiple block IDs", () => {
    const content = "Para 1 ^id1\n\nPara 2 ^id2\n\n- Item ^id3";
    const blocks = parseBlockIds(content);
    expect(blocks).toHaveLength(3);
    expect(blocks[0].line).toBe(1);
    expect(blocks[1].line).toBe(3);
    expect(blocks[2].line).toBe(5);
  });

  it("skips block IDs in code blocks", () => {
    const content = "Real paragraph ^real-id\n\n```\nCode ^fake-id\n```";
    const blocks = parseBlockIds(content);
    expect(blocks).toHaveLength(1);
    expect(blocks[0].id).toBe("real-id");
  });

  it("requires block ID at end of line", () => {
    const blocks = parseBlockIds("Some ^id text continues");
    expect(blocks).toHaveLength(0);
  });

  it("handles trailing whitespace", () => {
    const blocks = parseBlockIds("Some text ^id   ");
    expect(blocks).toHaveLength(1);
    expect(blocks[0].id).toBe("id");
  });

  it("detects numbered list block type", () => {
    const blocks = parseBlockIds("1. First item ^item1");
    expect(blocks).toHaveLength(1);
    expect(blocks[0].blockType).toBe("listitem");
  });
});
