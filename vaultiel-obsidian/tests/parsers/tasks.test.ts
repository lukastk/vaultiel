import { describe, it, expect } from "vitest";
import { parseTasks } from "../../src/parsers/tasks.js";
import { DEFAULT_TASK_CONFIG } from "../../src/types.js";

describe("parseTasks", () => {
  it("parses a simple task", () => {
    const tasks = parseTasks("- [ ] A simple task", "test.md");
    expect(tasks).toHaveLength(1);
    expect(tasks[0].symbol).toBe("[ ]");
    expect(tasks[0].description).toBe("A simple task");
    expect(tasks[0].indent).toBe(0);
  });

  it("parses a completed task", () => {
    const tasks = parseTasks("- [x] Completed task", "test.md");
    expect(tasks).toHaveLength(1);
    expect(tasks[0].symbol).toBe("[x]");
  });

  it("parses tasks with dates", () => {
    const tasks = parseTasks(
      "- [ ] Task with dates ⏳ 2026-02-05 📅 2026-02-10",
      "test.md",
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0].scheduled).toBe("2026-02-05");
    expect(tasks[0].due).toBe("2026-02-10");
  });

  it("parses tasks with priority", () => {
    const tasks = parseTasks("- [ ] High priority task ⏫", "test.md");
    expect(tasks).toHaveLength(1);
    expect(tasks[0].priority).toBe("high");
  });

  it("parses tasks with links", () => {
    const tasks = parseTasks(
      "- [ ] Task linking to [[Note A]] and [[Note B|alias]]",
      "test.md",
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0].links).toHaveLength(2);
    expect(tasks[0].links[0].to).toBe("Note A");
    expect(tasks[0].links[1].to).toBe("Note B");
    expect(tasks[0].links[1].alias).toBe("alias");
  });

  it("parses tasks with tags", () => {
    const tasks = parseTasks(
      "- [ ] Task with #tag1 and #project/subtag",
      "test.md",
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0].tags).toEqual(["#tag1", "#project/subtag"]);
  });

  it("parses tasks with block ID", () => {
    const tasks = parseTasks("- [ ] Task with block ID ^abc123", "test.md");
    expect(tasks).toHaveLength(1);
    expect(tasks[0].blockId).toBe("abc123");
  });

  it("parses nested tasks", () => {
    const content =
      "- [ ] Parent task\n\t- [ ] Child task\n\t\t- [ ] Grandchild task";
    const tasks = parseTasks(content, "test.md");
    expect(tasks).toHaveLength(3);
    expect(tasks[0].indent).toBe(0);
    expect(tasks[1].indent).toBe(1);
    expect(tasks[2].indent).toBe(2);
  });

  it("parses tasks with start date", () => {
    const tasks = parseTasks(
      "- [ ] Task with start 🛫 2026-03-01",
      "test.md",
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0].start).toBe("2026-03-01");
  });

  it("parses tasks with created date", () => {
    const tasks = parseTasks("- [ ] Task ➕ 2026-02-20", "test.md");
    expect(tasks).toHaveLength(1);
    expect(tasks[0].created).toBe("2026-02-20");
  });

  it("parses tasks with cancelled date", () => {
    const tasks = parseTasks(
      "- [-] Cancelled task ❌ 2026-02-25",
      "test.md",
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0].cancelled).toBe("2026-02-25");
  });

  it("parses tasks with recurrence", () => {
    const tasks = parseTasks(
      "- [ ] Recurring task 🔁 every week 📅 2026-03-01",
      "test.md",
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0].recurrence).toBe("every week");
    expect(tasks[0].due).toBe("2026-03-01");
  });

  it("parses tasks with id and depends_on", () => {
    const tasks = parseTasks(
      "- [ ] Task 🆔 abc123 ⛔ def456,ghi789 📅 2026-03-01",
      "test.md",
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0].id).toBe("abc123");
    expect(tasks[0].dependsOn).toEqual(["def456", "ghi789"]);
    expect(tasks[0].due).toBe("2026-03-01");
  });

  it("parses tasks with on_completion", () => {
    const tasks = parseTasks(
      "- [ ] Task 🏁 delete 📅 2026-03-01",
      "test.md",
    );
    expect(tasks).toHaveLength(1);
    expect(tasks[0].onCompletion).toBe("delete");
    expect(tasks[0].due).toBe("2026-03-01");
  });

  it("parses tasks with all fields", () => {
    const content =
      "- [ ] Full task 🆔 myid ⛔ dep1 ⏫ 🔁 every day 🏁 delete ➕ 2026-01-01 🛫 2026-02-01 ⏳ 2026-02-15 📅 2026-03-01 ❌ 2026-02-20 ✅ 2026-02-25";
    const tasks = parseTasks(content, "test.md");
    expect(tasks).toHaveLength(1);

    const task = tasks[0];
    expect(task.description).toBe("Full task");
    expect(task.id).toBe("myid");
    expect(task.dependsOn).toEqual(["dep1"]);
    expect(task.priority).toBe("high");
    expect(task.recurrence).toBe("every day");
    expect(task.onCompletion).toBe("delete");
    expect(task.created).toBe("2026-01-01");
    expect(task.start).toBe("2026-02-01");
    expect(task.scheduled).toBe("2026-02-15");
    expect(task.due).toBe("2026-03-01");
    expect(task.cancelled).toBe("2026-02-20");
    expect(task.done).toBe("2026-02-25");
  });

  it("sets correct file path", () => {
    const tasks = parseTasks("- [ ] Test", "notes/my-note.md");
    expect(tasks[0].file).toBe("notes/my-note.md");
  });

  it("sets correct line numbers", () => {
    const content = "Some text\n\n- [ ] Task on line 3\n\n- [ ] Task on line 5";
    const tasks = parseTasks(content, "test.md");
    expect(tasks).toHaveLength(2);
    expect(tasks[0].line).toBe(3);
    expect(tasks[1].line).toBe(5);
  });

  it("skips tasks in code blocks", () => {
    const content = "- [ ] Real task\n\n```\n- [ ] Fake task\n```";
    const tasks = parseTasks(content, "test.md");
    expect(tasks).toHaveLength(1);
    expect(tasks[0].description).toBe("Real task");
  });

  it("preserves raw line", () => {
    const raw = "- [ ] Task with ⏫ 📅 2026-03-01";
    const tasks = parseTasks(raw, "test.md");
    expect(tasks[0].raw).toBe(raw);
  });
});
