import { describe, it, expect } from "vitest";
import { parseLinks } from "../../src/parsers/links.js";

describe("parseLinks", () => {
  it("parses a simple link", () => {
    const links = parseLinks("See [[My Note]] for details.");
    expect(links).toHaveLength(1);
    expect(links[0].target).toBe("My Note");
    expect(links[0].alias).toBeUndefined();
    expect(links[0].embed).toBe(false);
  });

  it("parses a link with alias", () => {
    const links = parseLinks("See [[My Note|the note]] for details.");
    expect(links).toHaveLength(1);
    expect(links[0].target).toBe("My Note");
    expect(links[0].alias).toBe("the note");
  });

  it("parses a link with heading", () => {
    const links = parseLinks("See [[My Note#Section]] for details.");
    expect(links).toHaveLength(1);
    expect(links[0].target).toBe("My Note");
    expect(links[0].heading).toBe("Section");
  });

  it("parses a link with block ref", () => {
    const links = parseLinks("See [[My Note#^abc123]] for details.");
    expect(links).toHaveLength(1);
    expect(links[0].target).toBe("My Note");
    expect(links[0].blockId).toBe("abc123");
  });

  it("parses a link with heading and alias", () => {
    const links = parseLinks("[[Note#Section|alias]]");
    expect(links).toHaveLength(1);
    expect(links[0].target).toBe("Note");
    expect(links[0].heading).toBe("Section");
    expect(links[0].alias).toBe("alias");
  });

  it("parses embeds", () => {
    const links = parseLinks("![[image.png]]");
    expect(links).toHaveLength(1);
    expect(links[0].embed).toBe(true);
    expect(links[0].target).toBe("image.png");
  });

  it("parses note embeds", () => {
    const links = parseLinks("![[Other Note]]");
    expect(links).toHaveLength(1);
    expect(links[0].embed).toBe(true);
    expect(links[0].target).toBe("Other Note");
  });

  it("parses embed with heading", () => {
    const links = parseLinks("![[Note#Section]]");
    expect(links).toHaveLength(1);
    expect(links[0].heading).toBe("Section");
  });

  it("parses multiple links", () => {
    const content = "See [[Note A]] and [[Note B|B]] and ![[image.png]].";
    const allLinks = parseLinks(content);
    expect(allLinks).toHaveLength(3);

    const regularLinks = allLinks.filter((l) => !l.embed);
    expect(regularLinks).toHaveLength(2);

    const embeds = allLinks.filter((l) => l.embed);
    expect(embeds).toHaveLength(1);
  });

  it("skips links in code blocks", () => {
    const content = "See [[real link]]\n\n```\n[[fake link]]\n```\n\nMore text";
    const links = parseLinks(content);
    expect(links).toHaveLength(1);
    expect(links[0].target).toBe("real link");
  });

  it("skips links in inline code", () => {
    const content = "See [[real link]] and `[[fake link]]` here.";
    const links = parseLinks(content);
    expect(links).toHaveLength(1);
    expect(links[0].target).toBe("real link");
  });

  it("computes correct line numbers", () => {
    const content = "Line 1\n[[Link on line 2]]\nLine 3\n[[Link on line 4]]";
    const links = parseLinks(content);
    expect(links).toHaveLength(2);
    expect(links[0].line).toBe(2);
    expect(links[1].line).toBe(4);
  });

  it("handles paths with slashes", () => {
    const links = parseLinks("[[folder/subfolder/note]]");
    expect(links).toHaveLength(1);
    expect(links[0].target).toBe("folder/subfolder/note");
  });
});
