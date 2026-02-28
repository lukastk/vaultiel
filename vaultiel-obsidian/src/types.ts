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
  metadata: Record<string, string>;
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

/** Value type for an emoji metadata field. */
export type EmojiValueType =
  | { kind: "date" }
  | { kind: "string" }
  | { kind: "text" }
  | { kind: "number" }
  | { kind: "flag"; value: string }
  | { kind: "enum"; value: string };

/** Definition of an emoji metadata field for tasks. */
export interface EmojiFieldDef {
  emoji: string;
  fieldName: string;
  valueType: EmojiValueType;
  order: number;
}

/** Task emoji configuration. */
export interface TaskConfig {
  fields: EmojiFieldDef[];
}
