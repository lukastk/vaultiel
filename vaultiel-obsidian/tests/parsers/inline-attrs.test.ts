import { describe, it, expect } from "vitest";
import { parseInlineAttrs } from "../../src/parsers/inline-attrs.js";

describe("parseInlineAttrs", () => {
  it("parses a simple inline attribute", () => {
    const attrs = parseInlineAttrs("Some text [status::active] here.");
    expect(attrs).toHaveLength(1);
    expect(attrs[0].key).toBe("status");
    expect(attrs[0].value).toBe("active");
  });

  it("parses inline attribute with wikilink value", () => {
    const attrs = parseInlineAttrs("[parent::[[Other Note]]]");
    expect(attrs).toHaveLength(1);
    expect(attrs[0].key).toBe("parent");
    expect(attrs[0].value).toBe("[[Other Note]]");
  });

  it("parses multiple inline attributes", () => {
    const attrs = parseInlineAttrs("[key1::value1] some text [key2::value2]");
    expect(attrs).toHaveLength(2);
  });

  it("parses keys with hyphens", () => {
    const attrs = parseInlineAttrs("[my-key::my value]");
    expect(attrs).toHaveLength(1);
    expect(attrs[0].key).toBe("my-key");
    expect(attrs[0].value).toBe("my value");
  });

  it("handles values with spaces", () => {
    const attrs = parseInlineAttrs(
      "[description::This is a longer value with spaces]",
    );
    expect(attrs).toHaveLength(1);
    expect(attrs[0].value).toBe("This is a longer value with spaces");
  });

  it("skips attributes in code blocks", () => {
    const attrs = parseInlineAttrs("[real::attr]\n\n```\n[fake::attr]\n```");
    expect(attrs).toHaveLength(1);
    expect(attrs[0].key).toBe("real");
  });

  it("skips attributes in inline code", () => {
    const attrs = parseInlineAttrs("[real::attr] and `[fake::attr]` here.");
    expect(attrs).toHaveLength(1);
  });

  it("computes correct line numbers", () => {
    const attrs = parseInlineAttrs("[attr1::val1]\nsome text\n[attr2::val2]");
    expect(attrs).toHaveLength(2);
    expect(attrs[0].line).toBe(1);
    expect(attrs[1].line).toBe(3);
  });

  it("does not match Dataview-style bare attributes", () => {
    const attrs = parseInlineAttrs("status:: active");
    expect(attrs).toHaveLength(0);
  });
});
