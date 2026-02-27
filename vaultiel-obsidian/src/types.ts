/** Note information. */
export interface NoteInfo {
  path: string;
  name: string;
  modified?: string;
  created?: string;
  sizeBytes: number;
}

/** A wikilink or embed. */
export interface Link {
  target: string;
  alias?: string;
  heading?: string;
  blockId?: string;
  embed: boolean;
  line: number;
}

/** A tag found in content. */
export interface Tag {
  name: string;
  line: number;
}

/** A heading found in content. */
export interface Heading {
  text: string;
  level: number;
  line: number;
  slug: string;
}

/** A block ID found in content. */
export interface BlockId {
  id: string;
  line: number;
  blockType: string;
}

/** A link found within a task's description. */
export interface TaskLink {
  to: string;
  alias?: string;
}

/** A task found in content. */
export interface Task {
  file: string;
  line: number;
  raw: string;
  symbol: string;
  description: string;
  indent: number;
  scheduled?: string;
  due?: string;
  done?: string;
  start?: string;
  created?: string;
  cancelled?: string;
  recurrence?: string;
  onCompletion?: string;
  id?: string;
  dependsOn: string[];
  priority?: string;
  links: TaskLink[];
  tags: string[];
  blockId?: string;
}

/** Vaultiel metadata for a note. */
export interface VaultielMetadata {
  id: string;
  created: string;
}

/** A link reference in the vault graph. */
export interface LinkRef {
  from: string;
  line: number;
  context: string;
  alias?: string;
  heading?: string;
  blockId?: string;
  embed: boolean;
}

/** An inline attribute ([key::value]). */
export interface InlineAttr {
  key: string;
  value: string;
  line: number;
}

/** Task emoji configuration. */
export interface TaskConfig {
  due: string;
  scheduled: string;
  done: string;
  start: string;
  created: string;
  cancelled: string;
  recurrence: string;
  onCompletion: string;
  dependsOn: string;
  id: string;
  priorityHighest: string;
  priorityHigh: string;
  priorityMedium: string;
  priorityLow: string;
  priorityLowest: string;
  customMetadata: Record<string, string>;
}

/** Default task configuration matching Obsidian Tasks plugin. */
export const DEFAULT_TASK_CONFIG: TaskConfig = {
  due: "\u{1F4C5}",           // 📅
  scheduled: "\u{23F3}",       // ⏳
  done: "\u{2705}",            // ✅
  start: "\u{1F6EB}",          // 🛫
  created: "\u{2795}",         // ➕
  cancelled: "\u{274C}",       // ❌
  recurrence: "\u{1F501}",     // 🔁
  onCompletion: "\u{1F3C1}",   // 🏁
  dependsOn: "\u{26D4}",       // ⛔
  id: "\u{1F194}",             // 🆔
  priorityHighest: "\u{1F53A}", // 🔺
  priorityHigh: "\u{23EB}",    // ⏫
  priorityMedium: "\u{1F53C}", // 🔼
  priorityLow: "\u{1F53D}",   // 🔽
  priorityLowest: "\u{23EC}",  // ⏬
  customMetadata: {},
};
