/**
 * Obsidian Tasks plugin integration helpers.
 *
 * These are NOT part of the VaultAdapter interface — they wrap the
 * Obsidian Tasks plugin API for operations like toggling task status
 * and task modification that require the plugin.
 */

import { App, Notice, TFile } from "obsidian";

/** Get the Obsidian Tasks plugin instance, or null if not installed. */
function getTasksPlugin(app: App): any | null {
  const plugin = (app as any).plugins?.plugins?.["obsidian-tasks-plugin"];
  if (!plugin) return null;
  return plugin;
}

/** Check if the Obsidian Tasks plugin is available. */
export function isObsidianTasksAvailable(app: App): boolean {
  return getTasksPlugin(app) !== null;
}

/** Get all tasks from the Obsidian Tasks plugin. */
export function getObsidianTasks(app: App): any[] {
  const plugin = getTasksPlugin(app);
  if (!plugin) {
    new Notice("Obsidian Tasks plugin is not installed or enabled.");
    return [];
  }

  if (typeof plugin.getTasks === "function") {
    return plugin.getTasks();
  }

  return [];
}

/**
 * Get tasks from a specific file via the Obsidian Tasks plugin.
 *
 * @param app Obsidian App instance
 * @param path Note path to filter tasks from
 */
export function getObsidianTasksForFile(app: App, path: string): any[] {
  const allTasks = getObsidianTasks(app);
  const normalizedPath = path.endsWith(".md") ? path : `${path}.md`;
  return allTasks.filter(
    (t: any) => t.path === normalizedPath || t.taskLocation?.path === normalizedPath,
  );
}

/**
 * Toggle a task's done status using the Obsidian Tasks plugin API.
 *
 * @param app Obsidian App instance
 * @param file The file containing the task
 * @param line The 1-indexed line number of the task
 */
export async function toggleTask(
  app: App,
  file: TFile,
  line: number,
): Promise<void> {
  const plugin = getTasksPlugin(app);
  if (!plugin) {
    new Notice("Obsidian Tasks plugin is not installed or enabled.");
    return;
  }

  const api = plugin.apiV1;
  if (!api || typeof api.executeToggleTaskDoneCommand !== "function") {
    // Fall back to manual toggle via file content
    await app.vault.process(file, (data) => {
      const lines = data.split("\n");
      const idx = line - 1;
      if (idx >= 0 && idx < lines.length) {
        const taskLine = lines[idx];
        if (taskLine.includes("- [ ] ")) {
          lines[idx] = taskLine.replace("- [ ] ", "- [x] ");
        } else if (taskLine.includes("- [x] ")) {
          lines[idx] = taskLine.replace("- [x] ", "- [ ] ");
        }
      }
      return lines.join("\n");
    });
    return;
  }

  // Use the plugin API
  await api.executeToggleTaskDoneCommand(file, line - 1);
}

/**
 * Modify a task's markdown text using file manipulation.
 *
 * @param app Obsidian App instance
 * @param file The file containing the task
 * @param line The 1-indexed line number of the task
 * @param newMarkdown The new markdown text for the task line
 */
export async function modifyTaskMarkdown(
  app: App,
  file: TFile,
  line: number,
  newMarkdown: string,
): Promise<void> {
  await app.vault.process(file, (data) => {
    const lines = data.split("\n");
    const idx = line - 1;
    if (idx >= 0 && idx < lines.length) {
      lines[idx] = newMarkdown;
    }
    return lines.join("\n");
  });
}
