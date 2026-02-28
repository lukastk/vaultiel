import { describe, it, expect } from "vitest";
import { parseTaskTrees, formatTaskTree } from "../../src/parsers/tasks.js";
import type { TaskConfig, TaskChild } from "../../src/types.js";

const emptyConfig: TaskConfig = { fields: [] };

describe("parseTaskTrees", () => {
  it("parses a single task", () => {
    const trees = parseTaskTrees("- [ ] A task", "test.md", emptyConfig);
    expect(trees).toHaveLength(1);
    expect(trees[0]!.type).toBe("task");
    if (trees[0]!.type === "task") {
      expect(trees[0]!.description).toBe("A task");
      expect(trees[0]!.marker).toBe("-");
      expect(trees[0]!.children).toHaveLength(0);
    }
  });

  it("parses tasks with different markers", () => {
    const content = "* [ ] Star task\n+ [ ] Plus task\n1. [ ] Numbered task";
    const trees = parseTaskTrees(content, "test.md", emptyConfig);
    expect(trees).toHaveLength(3);
    if (trees[0]!.type === "task") expect(trees[0]!.marker).toBe("*");
    if (trees[1]!.type === "task") expect(trees[1]!.marker).toBe("+");
    if (trees[2]!.type === "task") expect(trees[2]!.marker).toBe("1.");
  });

  it("parses subtasks as children", () => {
    const content = "- [ ] Parent\n    - [ ] Child";
    const trees = parseTaskTrees(content, "test.md", emptyConfig);
    expect(trees).toHaveLength(1);
    if (trees[0]!.type === "task") {
      expect(trees[0]!.children).toHaveLength(1);
      expect(trees[0]!.children[0]!.type).toBe("task");
      if (trees[0]!.children[0]!.type === "task") {
        expect(trees[0]!.children[0]!.description).toBe("Child");
      }
    }
  });

  it("parses text items under tasks", () => {
    const content = "- [ ] My task\n    - A bullet point\n    * Another point";
    const trees = parseTaskTrees(content, "test.md", emptyConfig);
    expect(trees).toHaveLength(1);
    if (trees[0]!.type === "task") {
      expect(trees[0]!.children).toHaveLength(2);
      const c0 = trees[0]!.children[0]!;
      const c1 = trees[0]!.children[1]!;
      expect(c0.type).toBe("text");
      if (c0.type === "text") {
        expect(c0.content).toBe("A bullet point");
        expect(c0.marker).toBe("-");
      }
      expect(c1.type).toBe("text");
      if (c1.type === "text") {
        expect(c1.content).toBe("Another point");
        expect(c1.marker).toBe("*");
      }
    }
  });

  it("ignores top-level text items", () => {
    const content = "- Just a bullet\n- [ ] Real task";
    const trees = parseTaskTrees(content, "test.md", emptyConfig);
    expect(trees).toHaveLength(1);
    expect(trees[0]!.type).toBe("task");
  });

  it("parses the full example from the plan", () => {
    const content = [
      "- [ ] My task",
      "    - [ ] My subtask",
      "        - My subtask's bullet point",
      "        * My subtask's other bullet point",
      "    * My task's bullet point",
      "    1. My task's other bullet point",
      "    2. [ ] My other subtask",
    ].join("\n");

    const trees = parseTaskTrees(content, "test.md", emptyConfig);
    expect(trees).toHaveLength(1);

    const root = trees[0]!;
    expect(root.type).toBe("task");
    if (root.type !== "task") return;

    expect(root.description).toBe("My task");
    expect(root.marker).toBe("-");
    expect(root.children).toHaveLength(4);

    // subtask
    const c0 = root.children[0]!;
    expect(c0.type).toBe("task");
    if (c0.type === "task") {
      expect(c0.description).toBe("My subtask");
      expect(c0.children).toHaveLength(2);
      expect(c0.children[0]!.type).toBe("text");
      expect(c0.children[1]!.type).toBe("text");
      if (c0.children[0]!.type === "text") {
        expect(c0.children[0]!.content).toBe("My subtask's bullet point");
        expect(c0.children[0]!.marker).toBe("-");
      }
      if (c0.children[1]!.type === "text") {
        expect(c0.children[1]!.content).toBe("My subtask's other bullet point");
        expect(c0.children[1]!.marker).toBe("*");
      }
    }

    // bullet
    const c1 = root.children[1]!;
    expect(c1.type).toBe("text");
    if (c1.type === "text") {
      expect(c1.content).toBe("My task's bullet point");
      expect(c1.marker).toBe("*");
    }

    // numbered
    const c2 = root.children[2]!;
    expect(c2.type).toBe("text");
    if (c2.type === "text") {
      expect(c2.content).toBe("My task's other bullet point");
      expect(c2.marker).toBe("1.");
    }

    // numbered subtask
    const c3 = root.children[3]!;
    expect(c3.type).toBe("task");
    if (c3.type === "task") {
      expect(c3.description).toBe("My other subtask");
      expect(c3.marker).toBe("2.");
    }
  });

  it("skips tasks in code blocks", () => {
    const content = "- [ ] Real\n```\n- [ ] Fake\n```";
    const trees = parseTaskTrees(content, "test.md", emptyConfig);
    expect(trees).toHaveLength(1);
    if (trees[0]!.type === "task") {
      expect(trees[0]!.description).toBe("Real");
    }
  });

  it("handles text items with block IDs", () => {
    const content = "- [ ] Task\n    - A note ^abc123";
    const trees = parseTaskTrees(content, "test.md", emptyConfig);
    if (trees[0]!.type === "task") {
      const child = trees[0]!.children[0]!;
      if (child.type === "text") {
        expect(child.content).toBe("A note");
        expect(child.blockId).toBe("abc123");
      }
    }
  });
});

describe("formatTaskTree", () => {
  it("formats a single task", () => {
    const tree: TaskChild[] = [{
      type: "task",
      file: "test.md",
      line: 1,
      raw: "- [ ] My task",
      marker: "-",
      symbol: "[ ]",
      description: "My task",
      indent: 0,
      metadata: {},
      links: [],
      tags: [],
      children: [],
    }];
    expect(formatTaskTree(tree)).toBe("- [ ] My task");
  });

  it("formats nested tasks and text", () => {
    const tree: TaskChild[] = [{
      type: "task",
      file: "test.md",
      line: 1,
      raw: "",
      marker: "-",
      symbol: "[ ]",
      description: "Parent",
      indent: 0,
      metadata: {},
      links: [],
      tags: [],
      children: [
        {
          type: "task",
          file: "test.md",
          line: 2,
          raw: "",
          marker: "-",
          symbol: "[x]",
          description: "Child",
          indent: 1,
          metadata: {},
          links: [],
          tags: [],
          children: [],
        },
        {
          type: "text",
          file: "test.md",
          line: 3,
          raw: "",
          content: "A note",
          marker: "*",
          indent: 1,
          children: [],
        },
      ],
    }];
    expect(formatTaskTree(tree)).toBe("- [ ] Parent\n    - [x] Child\n    * A note");
  });

  it("uses custom indent string", () => {
    const tree: TaskChild[] = [{
      type: "task",
      file: "test.md",
      line: 1,
      raw: "",
      marker: "-",
      symbol: "[ ]",
      description: "Task",
      indent: 0,
      metadata: {},
      links: [],
      tags: [],
      children: [{
        type: "text",
        file: "test.md",
        line: 2,
        raw: "",
        content: "Note",
        marker: "-",
        indent: 1,
        children: [],
      }],
    }];
    expect(formatTaskTree(tree, "\t")).toBe("- [ ] Task\n\t- Note");
  });
});
