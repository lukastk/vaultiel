/**
 * Node.js bindings for Vaultiel - A library for Obsidian-style vaults.
 *
 * @module @vaultiel/node
 */

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
  priority?: string;
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

/**
 * Represents an Obsidian-style vault.
 */
export class Vault {
  /**
   * Open a vault at the specified path.
   * @param path - Path to the vault root directory.
   */
  constructor(path: string);

  /** Get the vault root path. */
  readonly root: string;

  /**
   * List all notes in the vault.
   * @returns Array of relative note paths.
   */
  listNotes(): string[];

  /**
   * List notes matching a glob pattern.
   * @param pattern - Glob pattern (e.g., "proj/*.md").
   * @returns Array of matching note paths.
   */
  listNotesMatching(pattern: string): string[];

  /**
   * Check if a note exists.
   * @param path - Note path (with or without .md extension).
   */
  noteExists(path: string): boolean;

  /**
   * Get note content.
   * @param path - Note path.
   * @returns Full note content including frontmatter.
   */
  getContent(path: string): string;

  /**
   * Get note body (content without frontmatter).
   * @param path - Note path.
   */
  getBody(path: string): string;

  /**
   * Get note frontmatter as JSON string.
   * @param path - Note path.
   * @returns JSON string or null if no frontmatter.
   */
  getFrontmatter(path: string): string | null;

  /**
   * Create a new note.
   * @param path - Note path.
   * @param content - Note content.
   */
  createNote(path: string, content: string): void;

  /**
   * Delete a note.
   * @param path - Note path.
   */
  deleteNote(path: string): void;

  /**
   * Rename a note (without link propagation).
   * @param from - Current path.
   * @param to - New path.
   */
  renameNote(from: string, to: string): void;

  /**
   * Resolve a note name or alias to a path.
   * @param query - Note name, alias, or partial path.
   * @returns Resolved note path.
   */
  resolveNote(query: string): string;

  /**
   * Parse links from a note.
   * @param path - Note path.
   */
  getLinks(path: string): Link[];

  /**
   * Parse tags from a note.
   * @param path - Note path.
   */
  getTags(path: string): Tag[];

  /**
   * Parse headings from a note.
   * @param path - Note path.
   */
  getHeadings(path: string): Heading[];

  /**
   * Parse block IDs from a note.
   * @param path - Note path.
   */
  getBlockIds(path: string): BlockId[];

  /**
   * Parse tasks from a note.
   * @param path - Note path.
   */
  getTasks(path: string): Task[];

  /**
   * Get incoming links to a note.
   * @param path - Note path.
   */
  getIncomingLinks(path: string): LinkRef[];

  /**
   * Get outgoing links from a note.
   * @param path - Note path.
   */
  getOutgoingLinks(path: string): LinkRef[];

  /**
   * Initialize vaultiel metadata for a note.
   * @param path - Note path.
   * @param force - Replace existing metadata if true.
   * @returns New metadata or null if skipped.
   */
  initMetadata(path: string, force: boolean): VaultielMetadata | null;

  /**
   * Get vaultiel metadata from a note.
   * @param path - Note path.
   */
  getVaultielMetadata(path: string): VaultielMetadata | null;

  /**
   * Find a note by its vaultiel ID.
   * @param id - UUID to search for.
   * @returns Note path or null if not found.
   */
  findById(id: string): string | null;
}

/**
 * Parse links from markdown content.
 * @param content - Markdown content.
 */
export function parseLinks(content: string): Link[];

/**
 * Parse tags from markdown content.
 * @param content - Markdown content.
 */
export function parseContentTags(content: string): Tag[];

/**
 * Parse headings from markdown content.
 * @param content - Markdown content.
 */
export function parseContentHeadings(content: string): Heading[];

/**
 * Parse block IDs from markdown content.
 * @param content - Markdown content.
 */
export function parseContentBlockIds(content: string): BlockId[];
