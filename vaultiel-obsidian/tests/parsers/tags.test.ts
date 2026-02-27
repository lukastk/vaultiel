import { describe, it, expect } from "vitest";
import { parseTags } from "../../src/parsers/tags.js";

describe("parseTags", () => {
  it("parses a simple tag", () => {
    const tags = parseTags("Some text #rust here.");
    expect(tags).toHaveLength(1);
    expect(tags[0].name).toBe("#rust");
  });

  it("parses nested tags", () => {
    const tags = parseTags("#tray/autonomy/urgent");
    expect(tags).toHaveLength(1);
    expect(tags[0].name).toBe("#tray/autonomy/urgent");
  });

  it("parses multiple tags", () => {
    const tags = parseTags("Tags: #rust #cli #obsidian");
    expect(tags).toHaveLength(3);
  });

  it("parses tags with hyphens", () => {
    const tags = parseTags("#my-tag");
    expect(tags).toHaveLength(1);
    expect(tags[0].name).toBe("#my-tag");
  });

  it("parses tags with underscores", () => {
    const tags = parseTags("#my_tag");
    expect(tags).toHaveLength(1);
    expect(tags[0].name).toBe("#my_tag");
  });

  it("parses tags starting with underscore", () => {
    const tags = parseTags("#_private");
    expect(tags).toHaveLength(1);
    expect(tags[0].name).toBe("#_private");
  });

  it("rejects numeric-only tags", () => {
    const tags = parseTags("Issue #123 is fixed.");
    expect(tags).toHaveLength(0);
  });

  it("rejects HTML entities", () => {
    const tags = parseTags("Use &nbsp; for space.");
    expect(tags).toHaveLength(0);
  });

  it("rejects headings as tags", () => {
    const tags = parseTags("# Heading\n## Subheading");
    expect(tags).toHaveLength(0);
  });

  it("skips tags in code blocks", () => {
    const tags = parseTags("Real #tag\n\n```\n#fake-tag\n```");
    expect(tags).toHaveLength(1);
    expect(tags[0].name).toBe("#tag");
  });

  it("skips tags in inline code", () => {
    const tags = parseTags("Real #tag and `#fake-tag` here.");
    expect(tags).toHaveLength(1);
    expect(tags[0].name).toBe("#tag");
  });

  it("computes correct line numbers", () => {
    const tags = parseTags("#tag1\nsome text\n#tag2");
    expect(tags).toHaveLength(2);
    expect(tags[0].line).toBe(1);
    expect(tags[1].line).toBe(3);
  });

  it("skips tags inside wikilinks", () => {
    const tags = parseTags("Real #tag and [[Note#heading]] here.");
    expect(tags).toHaveLength(1);
    expect(tags[0].name).toBe("#tag");
  });
});
