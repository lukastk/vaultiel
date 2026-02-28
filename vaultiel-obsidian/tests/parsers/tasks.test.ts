import { describe, it, expect } from "vitest";
import { parseTasks } from "../../src/parsers/tasks.js";
import type { TaskConfig } from "../../src/types.js";

/** Build an Obsidian Tasks-compatible config using the new generic format. */
function obsidianTasksConfig(): TaskConfig {
  return {
    fields: [
      { emoji: "\u{1F6EB}", fieldName: "start", valueType: { kind: "date" }, order: 1 },      // 🛫
      { emoji: "\u{2795}", fieldName: "created", valueType: { kind: "date" }, order: 2 },       // ➕
      { emoji: "\u{23F3}", fieldName: "scheduled", valueType: { kind: "date" }, order: 3 },     // ⏳
      { emoji: "\u{1F4C5}", fieldName: "due", valueType: { kind: "date" }, order: 4 },          // 📅
      { emoji: "\u{274C}", fieldName: "cancelled", valueType: { kind: "date" }, order: 5 },     // ❌
      { emoji: "\u{2705}", fieldName: "done", valueType: { kind: "date" }, order: 6 },          // ✅
      { emoji: "\u{1F194}", fieldName: "id", valueType: { kind: "string" }, order: 7 },         // 🆔
      { emoji: "\u{26D4}", fieldName: "depends_on", valueType: { kind: "text" }, order: 8 },    // ⛔
      { emoji: "\u{1F501}", fieldName: "recurrence", valueType: { kind: "text" }, order: 9 },   // 🔁
      { emoji: "\u{1F3C1}", fieldName: "on_completion", valueType: { kind: "text" }, order: 10 }, // 🏁
      { emoji: "\u{1F53A}", fieldName: "priority", valueType: { kind: "enum", value: "highest" }, order: 11 }, // 🔺
      { emoji: "\u{23EB}", fieldName: "priority", valueType: { kind: "enum", value: "high" }, order: 12 },    // ⏫
      { emoji: "\u{1F53C}", fieldName: "priority", valueType: { kind: "enum", value: "medium" }, order: 13 }, // 🔼
      { emoji: "\u{1F53D}", fieldName: "priority", valueType: { kind: "enum", value: "low" }, order: 14 },    // 🔽
      { emoji: "\u{23EC}", fieldName: "priority", valueType: { kind: "enum", value: "lowest" }, order: 15 },  // ⏬
    ],
  };
}

const config = obsidianTasksConfig();

describe("parseTasks", () => {
  it("parses a simple task", () => {
    const tasks = parseTasks("- [ ] A simple task", "test.md", config);
    expect(tasks).toHaveLength(1);
    expect(tasks[0]!.marker).toBe("-");
    expect(tasks[0]!.symbol).toBe("[ ]");
    expect(tasks[0]!.description).toBe("A simple task");
    expect(tasks[0]!.indent).toBe(0);
  });

  it("parses tasks with different markers", () => {
    const tasks = parseTasks("* [ ] Star\n+ [ ] Plus\n1. [ ] Numbered", "test.md", config);
    expect(tasks).toHaveLength(3);
    expect(tasks[0]!.marker).toBe("*");
    expect(tasks[1]!.marker).toBe("+");
    expect(tasks[2]!.marker).toBe("1.");
  });

  it("parses a completed task", () => {
    const tasks = parseTasks("- [x] Completed task", "test.md", config);
    expect(tasks).toHaveLength(1);
    expect(tasks[0]!.symbol).toBe("[x]");
  });

  it("parses tasks with dates", () => {
    const tasks = parseTasks(
      "- [ ] Task with dates ⏳ 2026-02-05 📅 2026-02-10",
      "test.md",
      config,
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0]!.metadata["scheduled"]).toBe("2026-02-05");
    expect(tasks[0]!.metadata["due"]).toBe("2026-02-10");
  });

  it("parses tasks with priority", () => {
    const tasks = parseTasks("- [ ] High priority task ⏫", "test.md", config);
    expect(tasks).toHaveLength(1);
    expect(tasks[0]!.metadata["priority"]).toBe("high");
  });

  it("parses tasks with links", () => {
    const tasks = parseTasks(
      "- [ ] Task linking to [[Note A]] and [[Note B|alias]]",
      "test.md",
      config,
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0]!.links).toHaveLength(2);
    expect(tasks[0]!.links[0]!.to).toBe("Note A");
    expect(tasks[0]!.links[1]!.to).toBe("Note B");
    expect(tasks[0]!.links[1]!.alias).toBe("alias");
  });

  it("parses tasks with tags", () => {
    const tasks = parseTasks(
      "- [ ] Task with #tag1 and #project/subtag",
      "test.md",
      config,
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0]!.tags).toEqual(["#tag1", "#project/subtag"]);
  });

  it("parses tasks with block ID", () => {
    const tasks = parseTasks("- [ ] Task with block ID ^abc123", "test.md", config);
    expect(tasks).toHaveLength(1);
    expect(tasks[0]!.blockId).toBe("abc123");
  });

  it("parses nested tasks", () => {
    const content =
      "- [ ] Parent task\n\t- [ ] Child task\n\t\t- [ ] Grandchild task";
    const tasks = parseTasks(content, "test.md", config);
    expect(tasks).toHaveLength(3);
    expect(tasks[0]!.indent).toBe(0);
    expect(tasks[1]!.indent).toBe(1);
    expect(tasks[2]!.indent).toBe(2);
  });

  it("parses tasks with start date", () => {
    const tasks = parseTasks(
      "- [ ] Task with start 🛫 2026-03-01",
      "test.md",
      config,
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0]!.metadata["start"]).toBe("2026-03-01");
  });

  it("parses tasks with created date", () => {
    const tasks = parseTasks("- [ ] Task ➕ 2026-02-20", "test.md", config);
    expect(tasks).toHaveLength(1);
    expect(tasks[0]!.metadata["created"]).toBe("2026-02-20");
  });

  it("parses tasks with cancelled date", () => {
    const tasks = parseTasks(
      "- [-] Cancelled task ❌ 2026-02-25",
      "test.md",
      config,
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0]!.metadata["cancelled"]).toBe("2026-02-25");
  });

  it("parses tasks with recurrence", () => {
    const tasks = parseTasks(
      "- [ ] Recurring task 🔁 every week 📅 2026-03-01",
      "test.md",
      config,
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0]!.metadata["recurrence"]).toBe("every week");
    expect(tasks[0]!.metadata["due"]).toBe("2026-03-01");
  });

  it("parses tasks with id and depends_on", () => {
    const tasks = parseTasks(
      "- [ ] Task 🆔 abc123 ⛔ def456,ghi789 📅 2026-03-01",
      "test.md",
      config,
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0]!.metadata["id"]).toBe("abc123");
    expect(tasks[0]!.metadata["depends_on"]).toBe("def456,ghi789");
    expect(tasks[0]!.metadata["due"]).toBe("2026-03-01");
  });

  it("parses tasks with on_completion", () => {
    const tasks = parseTasks(
      "- [ ] Task 🏁 delete 📅 2026-03-01",
      "test.md",
      config,
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0]!.metadata["on_completion"]).toBe("delete");
    expect(tasks[0]!.metadata["due"]).toBe("2026-03-01");
  });

  it("parses tasks with all fields", () => {
    const content =
      "- [ ] Full task 🆔 myid ⛔ dep1 ⏫ 🔁 every day 🏁 delete ➕ 2026-01-01 🛫 2026-02-01 ⏳ 2026-02-15 📅 2026-03-01 ❌ 2026-02-20 ✅ 2026-02-25";
    const tasks = parseTasks(content, "test.md", config);
    expect(tasks).toHaveLength(1);

    const task = tasks[0]!;
    expect(task.description).toBe("Full task");
    expect(task.metadata["id"]).toBe("myid");
    expect(task.metadata["depends_on"]).toBe("dep1");
    expect(task.metadata["priority"]).toBe("high");
    expect(task.metadata["recurrence"]).toBe("every day");
    expect(task.metadata["on_completion"]).toBe("delete");
    expect(task.metadata["created"]).toBe("2026-01-01");
    expect(task.metadata["start"]).toBe("2026-02-01");
    expect(task.metadata["scheduled"]).toBe("2026-02-15");
    expect(task.metadata["due"]).toBe("2026-03-01");
    expect(task.metadata["cancelled"]).toBe("2026-02-20");
    expect(task.metadata["done"]).toBe("2026-02-25");
  });

  it("sets correct file path", () => {
    const tasks = parseTasks("- [ ] Test", "notes/my-note.md", config);
    expect(tasks[0]!.file).toBe("notes/my-note.md");
  });

  it("sets correct line numbers", () => {
    const content = "Some text\n\n- [ ] Task on line 3\n\n- [ ] Task on line 5";
    const tasks = parseTasks(content, "test.md", config);
    expect(tasks).toHaveLength(2);
    expect(tasks[0]!.line).toBe(3);
    expect(tasks[1]!.line).toBe(5);
  });

  it("skips tasks in code blocks", () => {
    const content = "- [ ] Real task\n\n```\n- [ ] Fake task\n```";
    const tasks = parseTasks(content, "test.md", config);
    expect(tasks).toHaveLength(1);
    expect(tasks[0]!.description).toBe("Real task");
  });

  it("preserves raw line", () => {
    const raw = "- [ ] Task with ⏫ 📅 2026-03-01";
    const tasks = parseTasks(raw, "test.md", config);
    expect(tasks[0]!.raw).toBe(raw);
  });

  it("parses with empty config (no metadata extraction)", () => {
    const emptyConfig: TaskConfig = { fields: [] };
    const tasks = parseTasks(
      "- [ ] Task ⏫ 📅 2026-03-01",
      "test.md",
      emptyConfig,
    );
    expect(tasks).toHaveLength(1);
    expect(Object.keys(tasks[0]!.metadata)).toHaveLength(0);
    // Emojis remain in description since no fields are configured
    expect(tasks[0]!.description).toContain("⏫");
    expect(tasks[0]!.description).toContain("📅");
  });
});
