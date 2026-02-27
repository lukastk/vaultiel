import { describe, it, expect } from "vitest";
import { parseHeadings, slugify } from "../../src/parsers/headings.js";

describe("slugify", () => {
  it("lowercases and replaces spaces with hyphens", () => {
    expect(slugify("Hello World")).toBe("hello-world");
  });

  it("handles abbreviations", () => {
    expect(slugify("API Design")).toBe("api-design");
  });

  it("strips special characters", () => {
    expect(slugify("What's New?")).toBe("whats-new");
  });

  it("strips C++ style characters", () => {
    expect(slugify("C++ Programming")).toBe("c-programming");
  });

  it("trims leading/trailing spaces", () => {
    expect(slugify("  Spaced  ")).toBe("spaced");
  });

  it("preserves underscores", () => {
    expect(slugify("Under_score")).toBe("under_score");
  });

  it("collapses multiple spaces", () => {
    expect(slugify("Multiple   Spaces")).toBe("multiple-spaces");
  });
});

describe("parseHeadings", () => {
  it("parses simple headings", () => {
    const content = "# Heading 1\n\nSome text\n\n## Heading 2";
    const headings = parseHeadings(content);
    expect(headings).toHaveLength(2);
    expect(headings[0].text).toBe("Heading 1");
    expect(headings[0].level).toBe(1);
    expect(headings[0].line).toBe(1);
    expect(headings[1].text).toBe("Heading 2");
    expect(headings[1].level).toBe(2);
    expect(headings[1].line).toBe(5);
  });

  it("strips block IDs from heading text", () => {
    const headings = parseHeadings("# Heading ^block-id");
    expect(headings).toHaveLength(1);
    expect(headings[0].text).toBe("Heading");
  });

  it("generates unique slugs for duplicates", () => {
    const content = "# Test\n\n## Test\n\n### Test";
    const headings = parseHeadings(content);
    expect(headings).toHaveLength(3);
    expect(headings[0].slug).toBe("test");
    expect(headings[1].slug).toBe("test-1");
    expect(headings[2].slug).toBe("test-2");
  });

  it("skips headings in code blocks", () => {
    const content = "# Real Heading\n\n```\n# Not a heading\n```";
    const headings = parseHeadings(content);
    expect(headings).toHaveLength(1);
    expect(headings[0].text).toBe("Real Heading");
  });

  it("handles all heading levels", () => {
    const content = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6";
    const headings = parseHeadings(content);
    expect(headings).toHaveLength(6);
    for (let i = 0; i < 6; i++) {
      expect(headings[i].level).toBe(i + 1);
    }
  });

  it("ignores non-line-start hashes", () => {
    const content = "text # not a heading\n# Real heading";
    const headings = parseHeadings(content);
    expect(headings).toHaveLength(1);
    expect(headings[0].text).toBe("Real heading");
  });
});
