import { describe, it, expect } from "vitest";
import { parseInlineProperties } from "../../src/parsers/inline-properties.js";

describe("parseInlineProperties", () => {
  it("parses a simple inline property", () => {
    const props = parseInlineProperties("Some text [status::active] here.");
    expect(props).toHaveLength(1);
    expect(props[0].key).toBe("status");
    expect(props[0].value).toBe("active");
  });

  it("parses inline property with wikilink value", () => {
    const props = parseInlineProperties("[parent::[[Other Note]]]");
    expect(props).toHaveLength(1);
    expect(props[0].key).toBe("parent");
    expect(props[0].value).toBe("[[Other Note]]");
  });

  it("parses multiple inline properties", () => {
    const props = parseInlineProperties("[key1::value1] some text [key2::value2]");
    expect(props).toHaveLength(2);
  });

  it("parses keys with hyphens", () => {
    const props = parseInlineProperties("[my-key::my value]");
    expect(props).toHaveLength(1);
    expect(props[0].key).toBe("my-key");
    expect(props[0].value).toBe("my value");
  });

  it("handles values with spaces", () => {
    const props = parseInlineProperties(
      "[description::This is a longer value with spaces]",
    );
    expect(props).toHaveLength(1);
    expect(props[0].value).toBe("This is a longer value with spaces");
  });

  it("skips properties in code blocks", () => {
    const props = parseInlineProperties("[real::prop]\n\n```\n[fake::prop]\n```");
    expect(props).toHaveLength(1);
    expect(props[0].key).toBe("real");
  });

  it("skips properties in inline code", () => {
    const props = parseInlineProperties("[real::prop] and `[fake::prop]` here.");
    expect(props).toHaveLength(1);
  });

  it("computes correct line numbers", () => {
    const props = parseInlineProperties("[prop1::val1]\nsome text\n[prop2::val2]");
    expect(props).toHaveLength(2);
    expect(props[0].line).toBe(1);
    expect(props[1].line).toBe(3);
  });

  it("does not match Dataview-style bare attributes", () => {
    const props = parseInlineProperties("status:: active");
    expect(props).toHaveLength(0);
  });
});
