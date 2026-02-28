/**
 * @vaultiel/obsidian — Vaultiel vault API backed by Obsidian's native APIs.
 *
 * This package provides the same Vault API as @vaultiel/node, but implemented
 * using Obsidian's App, MetadataCache, and Vault APIs instead of the Rust core.
 *
 * Code written against the VaultAdapter interface runs unchanged in both
 * CLI (via @vaultiel/node) and Obsidian plugin (via @vaultiel/obsidian) contexts.
 *
 * @module @vaultiel/obsidian
 */

// Vault class
export { Vault } from "./vault.js";

// Types
export type {
  NoteInfo,
  Link,
  Tag,
  Heading,
  BlockId,
  TaskLink,
  Task,
  TaskNode,
  TaskTextItem,
  TaskChild,
  VaultielMetadata,
  LinkRef,
  InlineAttr,
  TaskConfig,
  EmojiFieldDef,
  EmojiValueType,
} from "./types.js";

// Standalone parse functions (work on raw markdown strings, no vault context)
export { parseLinks } from "./parsers/links.js";
export { parseTags } from "./parsers/tags.js";
export { parseHeadings, slugify } from "./parsers/headings.js";
export { parseBlockIds } from "./parsers/block-ids.js";
export { parseInlineAttrs } from "./parsers/inline-attrs.js";
export { parseTasks, parseTaskTrees, formatTaskTree } from "./parsers/tasks.js";

// Obsidian Tasks plugin helpers (not part of VaultAdapter)
export {
  isObsidianTasksAvailable,
  getObsidianTasks,
  getObsidianTasksForFile,
  toggleTask,
  modifyTaskMarkdown,
} from "./obsidian-tasks.js";
