/**
 * Vault class implementation using Obsidian's native APIs.
 *
 * Provides the same API surface as @vaultiel/node's Vault class,
 * but backed by Obsidian's App, MetadataCache, and Vault APIs.
 */

import {
  App,
  TFile,
  type LinkCache,
  type EmbedCache,
} from "obsidian";
import picomatch from "picomatch";

import type {
  Link,
  Tag,
  Heading,
  BlockId,
  Task,
  TaskChild,
  TaskConfig,
  VaultielMetadata,
  LinkRef,
  InlineProperty,
} from "./types.js";
import { parseLinks } from "./parsers/links.js";
import { parseTags } from "./parsers/tags.js";
import { parseHeadings, slugify } from "./parsers/headings.js";
import { parseBlockIds } from "./parsers/block-ids.js";
import { parseInlineProperties } from "./parsers/inline-properties.js";
import { parseTasks, parseTaskTrees } from "./parsers/tasks.js";
import { parseSearchQuery, evaluateNote as evaluateSearchNote } from "./parsers/search.js";
import type { SearchResult } from "./parsers/search.js";

declare const crypto: { randomUUID(): string };

/** Normalize a note path to always have .md extension. */
function normalizePath(path: string): string {
  return path.endsWith(".md") ? path : `${path}.md`;
}

/** Get a TFile from a path, or throw. */
function getFile(app: App, path: string): TFile {
  const normalized = normalizePath(path);
  const file = app.vault.getAbstractFileByPath(normalized);
  if (!(file instanceof TFile)) {
    throw new Error(`Note not found: ${normalized}`);
  }
  return file;
}

/**
 * Vault class backed by Obsidian's native APIs.
 *
 * Methods that use metadataCache are synchronous.
 * Methods that read/write file content are async.
 */
export class Vault {
  private app: App;
  private taskConfig: TaskConfig;

  constructor(app: App, taskConfig: TaskConfig) {
    this.app = app;
    this.taskConfig = taskConfig;
  }

  /** Get the vault root path. */
  get root(): string {
    return (this.app.vault.adapter as any).basePath as string;
  }

  // ==========================================================================
  // Read Operations (sync — metadataCache based)
  // ==========================================================================

  /** List all notes in the vault. */
  listNotes(): string[] {
    return this.app.vault.getMarkdownFiles().map((f) => f.path);
  }

  /** List notes matching a glob pattern. */
  listNotesMatching(pattern: string): string[] {
    const isMatch = picomatch(pattern);
    return this.app.vault
      .getMarkdownFiles()
      .filter((f) => isMatch(f.path))
      .map((f) => f.path);
  }

  /** Check if a note exists. */
  noteExists(path: string): boolean {
    const normalized = normalizePath(path);
    const file = this.app.vault.getAbstractFileByPath(normalized);
    return file instanceof TFile;
  }

  /** Resolve a note name or alias to a path. */
  resolveNote(query: string): string {
    const file = this.app.metadataCache.getFirstLinkpathDest(query, "");
    if (!file) {
      throw new Error(`Could not resolve note: ${query}`);
    }
    return file.path;
  }

  /** Get note frontmatter as JSON string. */
  getFrontmatter(path: string): string | null {
    const file = getFile(this.app, path);
    const cache = this.app.metadataCache.getFileCache(file);
    if (!cache?.frontmatter) return null;

    // Strip the "position" key that Obsidian adds internally
    const { position: _, ...fm } = cache.frontmatter;
    return JSON.stringify(fm);
  }

  /** Parse links from a note (using metadataCache). */
  getLinks(path: string): Link[] {
    const file = getFile(this.app, path);
    const cache = this.app.metadataCache.getFileCache(file);
    if (!cache) return [];

    const results: Link[] = [];

    // Regular links
    if (cache.links) {
      for (const link of cache.links) {
        results.push(mapLinkCache(link, false));
      }
    }

    // Embeds
    if (cache.embeds) {
      for (const embed of cache.embeds) {
        results.push(mapEmbedCache(embed));
      }
    }

    return results;
  }

  /** Parse tags from a note (using metadataCache). */
  getTags(path: string): Tag[] {
    const file = getFile(this.app, path);
    const cache = this.app.metadataCache.getFileCache(file);
    if (!cache?.tags) return [];

    return cache.tags.map((t) => ({
      name: t.tag,
      line: t.position.start.line + 1, // Obsidian uses 0-indexed
    }));
  }

  /** Parse headings from a note (using metadataCache + slug computation). */
  getHeadings(path: string): Heading[] {
    const file = getFile(this.app, path);
    const cache = this.app.metadataCache.getFileCache(file);
    if (!cache?.headings) return [];

    // We need to compute slugs ourselves since Obsidian doesn't provide them
    const slugCounts = new Map<string, number>();

    return cache.headings.map((h) => {
      const baseSlug = slugify(h.heading);
      const count = (slugCounts.get(baseSlug) ?? 0) + 1;
      slugCounts.set(baseSlug, count);
      const slug = count === 1 ? baseSlug : `${baseSlug}-${count - 1}`;

      return {
        text: h.heading,
        level: h.level,
        line: h.position.start.line + 1,
        slug,
      };
    });
  }

  /** Parse block IDs from a note (using metadataCache). */
  getBlockIds(path: string): BlockId[] {
    const file = getFile(this.app, path);
    const cache = this.app.metadataCache.getFileCache(file);
    if (!cache?.blocks) return [];

    return Object.entries(cache.blocks).map(([id, block]) => ({
      id,
      line: block.position.start.line + 1,
      blockType: "paragraph", // Obsidian's BlockCache doesn't include type; could compute from content
    }));
  }

  /** Get incoming links to a note. */
  getIncomingLinks(path: string): LinkRef[] {
    const file = getFile(this.app, path);
    // getBacklinksForFile returns a SearchResult-like object
    const backlinks = (this.app.metadataCache as any).getBacklinksForFile(file);
    if (!backlinks?.data) return [];

    const refs: LinkRef[] = [];
    const data: Map<string, LinkCache[]> = backlinks.data;

    for (const [sourcePath, linkCaches] of data) {
      for (const lc of linkCaches) {
        if (!lc?.position) continue; // skip entries without position data
        refs.push({
          from: sourcePath,
          line: lc.position.start.line + 1,
          context: lc.displayText || lc.link,
          alias: lc.displayText !== lc.link ? lc.displayText : undefined,
          heading: undefined, // Would need to parse from lc.link
          blockId: undefined,
          embed: false, // Backlinks API doesn't distinguish
        });
      }
    }

    return refs;
  }

  /** Get outgoing links from a note. */
  getOutgoingLinks(path: string): LinkRef[] {
    const file = getFile(this.app, path);
    const cache = this.app.metadataCache.getFileCache(file);
    if (!cache) return [];

    const refs: LinkRef[] = [];
    const resolved = this.app.metadataCache.resolvedLinks[file.path] ?? {};

    if (cache.links) {
      for (const lc of cache.links) {
        // Try to find the resolved target
        const targetPath = this.app.metadataCache.getFirstLinkpathDest(lc.link, file.path)?.path;
        refs.push({
          from: file.path,
          line: lc.position.start.line + 1,
          context: lc.displayText || lc.link,
          alias: lc.displayText !== lc.link ? lc.displayText : undefined,
          heading: undefined,
          blockId: undefined,
          embed: false,
        });
      }
    }

    if (cache.embeds) {
      for (const ec of cache.embeds) {
        refs.push({
          from: file.path,
          line: ec.position.start.line + 1,
          context: ec.displayText || ec.link,
          alias: ec.displayText !== ec.link ? ec.displayText : undefined,
          heading: undefined,
          blockId: undefined,
          embed: true,
        });
      }
    }

    return refs;
  }

  /** Get vaultiel metadata from a note. */
  getVaultielMetadata(path: string): VaultielMetadata | null {
    const fmStr = this.getFrontmatter(path);
    if (!fmStr) return null;

    const fm = JSON.parse(fmStr);
    const id = fm["vaultiel-id"];
    const created = fm["vaultiel-created"];
    if (!id || !created) return null;

    return { id, created };
  }

  // ==========================================================================
  // Read Operations (async — file I/O)
  // ==========================================================================

  /** Get note content (full content including frontmatter). */
  async getContent(path: string): Promise<string> {
    const file = getFile(this.app, path);
    return this.app.vault.cachedRead(file);
  }

  /** Get note body (content without frontmatter). */
  async getBody(path: string): Promise<string> {
    const file = getFile(this.app, path);
    const content = await this.app.vault.cachedRead(file);
    const cache = this.app.metadataCache.getFileCache(file);

    if (cache?.frontmatterPosition) {
      const endLine = cache.frontmatterPosition.end.line;
      const lines = content.split("\n");
      // frontmatterPosition.end.line is the closing --- line (0-indexed)
      return lines.slice(endLine + 1).join("\n");
    }

    return content;
  }

  /** Parse tasks from a note. */
  async getTasks(path: string, linksTo?: string): Promise<Task[]> {
    const file = getFile(this.app, path);
    const content = await this.app.vault.cachedRead(file);

    let tasks = parseTasks(content, file.path, this.taskConfig);

    if (linksTo) {
      const targetNormalized = linksTo.replace(/\.md$/, "").toLowerCase();
      tasks = tasks.filter((t) =>
        t.links.some(
          (link) => link.to.replace(/\.md$/, "").toLowerCase() === targetNormalized,
        ),
      );
    }

    return tasks;
  }

  /** Parse task trees from a note, including non-task list items as children. */
  async getTaskTrees(path: string): Promise<TaskChild[]> {
    const file = getFile(this.app, path);
    const content = await this.app.vault.cachedRead(file);
    return parseTaskTrees(content, file.path, this.taskConfig);
  }

  /** Inspect a note — returns full JSON representation. */
  async inspect(path: string): Promise<string> {
    const file = getFile(this.app, path);
    const content = await this.app.vault.cachedRead(file);

    const fmStr = this.getFrontmatter(path);
    const frontmatter = fmStr ? JSON.parse(fmStr) : null;
    const inlineProperties = parseInlineProperties(content);
    const headings = this.getHeadings(path);
    const tasks = parseTasks(content, file.path, this.taskConfig);
    const links = this.getLinks(path);
    const tags = this.getTags(path);
    const blockIds = this.getBlockIds(path);

    const result = {
      path: file.path,
      name: file.basename,
      frontmatter,
      inline_properties: inlineProperties,
      headings,
      tasks,
      links: { outgoing: links },
      tags,
      block_ids: blockIds,
      stats: {
        lines: content.split("\n").length,
        words: content.split(/\s+/).filter(Boolean).length,
        size_bytes: file.stat.size,
      },
    };

    return JSON.stringify(result);
  }

  // ==========================================================================
  // Write Operations (async)
  // ==========================================================================

  /** Create a new note. */
  async createNote(path: string, content: string): Promise<void> {
    const normalized = normalizePath(path);

    // Create parent folders if needed
    const dir = normalized.substring(0, normalized.lastIndexOf("/"));
    if (dir) {
      const folder = this.app.vault.getAbstractFileByPath(dir);
      if (!folder) {
        await this.app.vault.createFolder(dir);
      }
    }

    await this.app.vault.create(normalized, content);
  }

  /** Delete a note (moves to system trash). */
  async deleteNote(path: string): Promise<void> {
    const file = getFile(this.app, path);
    await this.app.vault.trash(file, false);
  }

  /** Rename a note (without link propagation). */
  async renameNote(from: string, to: string): Promise<void> {
    const file = getFile(this.app, from);
    const normalizedTo = normalizePath(to);
    await this.app.vault.rename(file, normalizedTo);
  }

  /** Set the body content of a note (preserves frontmatter). */
  async setContent(path: string, newBody: string): Promise<void> {
    const file = getFile(this.app, path);
    const cache = this.app.metadataCache.getFileCache(file);

    await this.app.vault.process(file, (data) => {
      if (cache?.frontmatterPosition) {
        const lines = data.split("\n");
        const endLine = cache.frontmatterPosition.end.line;
        const fmLines = lines.slice(0, endLine + 1);
        return fmLines.join("\n") + "\n" + newBody;
      }
      return newBody;
    });
  }

  /** Set the raw content of a note (replaces everything including frontmatter). */
  async setRawContent(path: string, content: string): Promise<void> {
    const file = getFile(this.app, path);
    await this.app.vault.process(file, () => content);
  }

  /** Modify a frontmatter field. */
  async modifyFrontmatter(
    path: string,
    key: string,
    value: unknown,
  ): Promise<void> {
    const file = getFile(this.app, path);
    await this.app.fileManager.processFrontMatter(file, (fm) => {
      fm[key] = value;
    });
  }

  /** Append content to a note. */
  async appendContent(path: string, content: string): Promise<void> {
    const file = getFile(this.app, path);
    await this.app.vault.process(file, (data) => data + content);
  }

  /** Replace first occurrence of pattern in note content. */
  async replaceContent(
    path: string,
    pattern: string,
    replacement: string,
  ): Promise<void> {
    const file = getFile(this.app, path);
    await this.app.vault.process(file, (data) => {
      const idx = data.indexOf(pattern);
      if (idx === -1) return data;
      return data.slice(0, idx) + replacement + data.slice(idx + pattern.length);
    });
  }

  /** Change the task checkbox symbol on a specific line (1-indexed). */
  async setTaskSymbol(
    path: string,
    line: number,
    newSymbol: string,
  ): Promise<void> {
    if ([...newSymbol].length !== 1) {
      throw new Error(
        `newSymbol must be a single character, got ${[...newSymbol].length} characters`,
      );
    }

    const file = getFile(this.app, path);
    await this.app.vault.process(file, (data) => {
      const lines = data.split("\n");
      const idx = line - 1; // Convert 1-indexed to 0-indexed

      if (idx < 0 || idx >= lines.length) {
        throw new Error(
          `Line ${line} is out of range (note has ${lines.length} lines)`,
        );
      }

      const targetLine = lines[idx]!;
      const taskRe = /^(\s*(?:[-*+]|\d+\.) \[).\](.*)$/;
      if (!taskRe.test(targetLine)) {
        throw new Error(`Line ${line} is not a task: "${targetLine}"`);
      }

      lines[idx] = targetLine.replace(taskRe, `$1${newSymbol}]$2`);
      return lines.join("\n");
    });
  }

  // ==========================================================================
  // Property Operations — Scope-Specific
  // ==========================================================================

  /** Remove a frontmatter key. */
  async removeFrontmatterKey(path: string, key: string): Promise<void> {
    const file = getFile(this.app, path);
    await this.app.fileManager.processFrontMatter(file, (fm) => {
      delete fm[key];
    });
  }

  /** Append a value to a frontmatter key's list. */
  async appendFrontmatterValue(
    path: string,
    key: string,
    value: unknown,
  ): Promise<void> {
    const file = getFile(this.app, path);
    await this.app.fileManager.processFrontMatter(file, (fm) => {
      const existing = fm[key];
      if (existing === undefined) {
        fm[key] = [value];
      } else if (Array.isArray(existing)) {
        existing.push(value);
      } else {
        fm[key] = [existing, value];
      }
    });
  }

  /** Rename a frontmatter key (atomic single processFrontMatter call). */
  async renameFrontmatterKey(
    path: string,
    oldKey: string,
    newKey: string,
  ): Promise<void> {
    const file = getFile(this.app, path);
    await this.app.fileManager.processFrontMatter(file, (fm) => {
      if (oldKey in fm) {
        fm[newKey] = fm[oldKey];
        delete fm[oldKey];
      }
    });
  }

  /** Get inline properties from a note. */
  async getInlineProperties(path: string): Promise<InlineProperty[]> {
    const file = getFile(this.app, path);
    const content = await this.app.vault.cachedRead(file);
    return parseInlineProperties(content);
  }

  /** Set an inline property's value. */
  async setInlineProperty(
    path: string,
    key: string,
    newValue: string,
    index?: number,
  ): Promise<void> {
    const file = getFile(this.app, path);
    await this.app.vault.process(file, (data) => {
      const props = parseInlineProperties(data);
      const matching = props.filter((p) => p.key === key);

      let target: InlineProperty;
      if (index !== undefined) {
        const t = props[index];
        if (!t) throw new Error(`Inline property index ${index} out of range (note has ${props.length} inline properties)`);
        target = t;
      } else {
        if (matching.length === 0) throw new Error(`No inline property found with key "${key}"`);
        if (matching.length > 1) throw new Error(`Multiple inline properties with key "${key}" — specify an index`);
        target = matching[0]!;
      }

      const lines = data.split("\n");
      const lineIdx = target.line - 1;
      const line = lines[lineIdx]!;
      const escaped = target.value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      const keyEscaped = key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      const re = new RegExp(`\\[${keyEscaped}::${escaped}\\]`);
      lines[lineIdx] = line.replace(re, `[${key}::${newValue}]`);
      return lines.join("\n");
    });
  }

  /** Remove an inline property. */
  async removeInlineProperty(
    path: string,
    key?: string,
    index?: number,
  ): Promise<void> {
    const file = getFile(this.app, path);
    await this.app.vault.process(file, (data) => {
      const props = parseInlineProperties(data);

      let target: InlineProperty;
      if (index !== undefined) {
        const t = props[index];
        if (!t) throw new Error(`Inline property index ${index} out of range (note has ${props.length} inline properties)`);
        target = t;
      } else if (key !== undefined) {
        const matching = props.filter((p) => p.key === key);
        if (matching.length === 0) throw new Error(`No inline property found with key "${key}"`);
        if (matching.length > 1) throw new Error(`Multiple inline properties with key "${key}" — specify an index`);
        target = matching[0]!;
      } else {
        throw new Error("Must specify either key or index to remove an inline property");
      }

      const lines = data.split("\n");
      const lineIdx = target.line - 1;
      const line = lines[lineIdx]!;
      const escaped = target.value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      const keyEscaped = target.key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      const re = new RegExp(`\\[${keyEscaped}::${escaped}\\]`);
      lines[lineIdx] = line.replace(re, "");
      return lines.join("\n");
    });
  }

  /** Rename all inline properties with oldKey to newKey. */
  async renameInlineProperty(
    path: string,
    oldKey: string,
    newKey: string,
  ): Promise<void> {
    const file = getFile(this.app, path);
    await this.app.vault.process(file, (data) => {
      const props = parseInlineProperties(data);
      const matching = props.filter((p) => p.key === oldKey);
      if (matching.length === 0) return data;

      const lines = data.split("\n");
      // Process in reverse order so positions remain valid
      const sorted = [...matching].sort((a, b) => b.line - a.line);
      for (const prop of sorted) {
        const lineIdx = prop.line - 1;
        const line = lines[lineIdx]!;
        const escaped = prop.value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
        const keyEscaped = oldKey.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
        const re = new RegExp(`\\[${keyEscaped}::${escaped}\\]`);
        lines[lineIdx] = line.replace(re, `[${newKey}::${prop.value}]`);
      }
      return lines.join("\n");
    });
  }

  // ==========================================================================
  // Property-Agnostic Operations
  // ==========================================================================

  /** Get merged properties (frontmatter + inline). Frontmatter takes precedence. */
  async getProperties(path: string): Promise<Record<string, unknown>> {
    const fmStr = this.getFrontmatter(path);
    const fm: Record<string, unknown> = fmStr ? JSON.parse(fmStr) : {};
    const inlineProps = await this.getInlineProperties(path);

    const merged: Record<string, unknown> = { ...fm };
    for (const prop of inlineProps) {
      if (prop.key in merged) continue; // frontmatter takes precedence
      const existing = merged[prop.key];
      if (existing !== undefined) {
        merged[prop.key] = Array.isArray(existing)
          ? [...existing, prop.value]
          : [existing, prop.value];
      } else {
        merged[prop.key] = prop.value;
      }
    }

    return merged;
  }

  /** Get a single property by key (frontmatter checked first, then inline). */
  async getProperty(
    path: string,
    key: string,
  ): Promise<unknown | undefined> {
    const fmStr = this.getFrontmatter(path);
    if (fmStr) {
      const fm = JSON.parse(fmStr) as Record<string, unknown>;
      if (key in fm) return fm[key];
    }

    const inlineProps = await this.getInlineProperties(path);
    const matching = inlineProps.filter((p) => p.key === key);
    if (matching.length === 0) return undefined;
    if (matching.length === 1) return matching[0]!.value;
    return matching.map((p) => p.value);
  }

  /** Set a property with auto-detection. */
  async setProperty(
    path: string,
    key: string,
    value: unknown,
    scope?: string,
    index?: number,
  ): Promise<void> {
    const resolvedScope = scope ?? "auto";

    if (resolvedScope === "frontmatter") {
      await this.modifyFrontmatter(path, key, value);
      return;
    }

    if (resolvedScope === "inline") {
      const strValue = typeof value === "string" ? value : JSON.stringify(value);
      await this.setInlineProperty(path, key, strValue, index);
      return;
    }

    if (resolvedScope === "both") {
      throw new Error("Cannot use 'both' scope for setProperty (ambiguous intent)");
    }

    // Auto-detect
    const fmStr = this.getFrontmatter(path);
    const fm: Record<string, unknown> = fmStr ? JSON.parse(fmStr) : {};
    const inlineProps = await this.getInlineProperties(path);
    const inFm = key in fm;
    const inInline = inlineProps.some((p) => p.key === key);

    if (inFm && inInline) {
      throw new Error(`Property "${key}" exists in both frontmatter and inline — specify a scope`);
    }

    if (inFm) {
      await this.modifyFrontmatter(path, key, value);
    } else if (inInline) {
      const strValue = typeof value === "string" ? value : JSON.stringify(value);
      await this.setInlineProperty(path, key, strValue, index);
    } else {
      // New key — default to frontmatter
      await this.modifyFrontmatter(path, key, value);
    }
  }

  /** Remove a property. Auto/Both: remove from all locations. */
  async removeProperty(
    path: string,
    key: string,
    scope?: string,
    index?: number,
  ): Promise<void> {
    const resolvedScope = scope ?? "auto";

    if (resolvedScope === "frontmatter") {
      await this.removeFrontmatterKey(path, key);
      return;
    }

    if (resolvedScope === "inline") {
      await this.removeInlineProperty(path, key, index);
      return;
    }

    // Auto or Both: remove from all locations
    const fmStr = this.getFrontmatter(path);
    if (fmStr) {
      const fm = JSON.parse(fmStr) as Record<string, unknown>;
      if (key in fm) {
        await this.removeFrontmatterKey(path, key);
      }
    }

    // Remove all inline occurrences (reverse order)
    const inlineProps = await this.getInlineProperties(path);
    const matchingIndices = inlineProps
      .map((p, i) => ({ key: p.key, index: i }))
      .filter((p) => p.key === key)
      .map((p) => p.index);
    for (let i = matchingIndices.length - 1; i >= 0; i--) {
      await this.removeInlineProperty(path, undefined, matchingIndices[i]);
    }
  }

  /** Rename a property key. Auto/Both: rename in all locations. */
  async renameProperty(
    path: string,
    oldKey: string,
    newKey: string,
    scope?: string,
  ): Promise<void> {
    const resolvedScope = scope ?? "auto";

    if (resolvedScope === "frontmatter") {
      await this.renameFrontmatterKey(path, oldKey, newKey);
      return;
    }

    if (resolvedScope === "inline") {
      await this.renameInlineProperty(path, oldKey, newKey);
      return;
    }

    // Auto or Both: rename in all locations
    await this.renameFrontmatterKey(path, oldKey, newKey);
    await this.renameInlineProperty(path, oldKey, newKey);
  }

  // ==========================================================================
  // Metadata Operations (async)
  // ==========================================================================

  /** Initialize vaultiel metadata for a note. */
  async initMetadata(
    path: string,
    force: boolean,
  ): Promise<VaultielMetadata | null> {
    const existing = this.getVaultielMetadata(path);
    if (existing && !force) return null;

    const id = crypto.randomUUID();
    const created = new Date().toISOString();

    const file = getFile(this.app, path);
    await this.app.fileManager.processFrontMatter(file, (fm) => {
      fm["vaultiel-id"] = id;
      fm["vaultiel-created"] = created;
    });

    return { id, created };
  }

  /** Find a note by its vaultiel ID. */
  async findById(id: string): Promise<string | null> {
    const files = this.app.vault.getMarkdownFiles();

    for (const file of files) {
      const cache = this.app.metadataCache.getFileCache(file);
      if (cache?.frontmatter?.["vaultiel-id"] === id) {
        return file.path;
      }
    }

    return null;
  }

  /** Search notes by query string. */
  async search(queryStr: string): Promise<SearchResult[]> {
    const query = parseSearchQuery(queryStr);
    return this.searchStructured(query);
  }

  /** Search notes by a pre-built SearchQuery AST (skips parsing). */
  async searchStructured(query: import("./parsers/search.js").SearchQuery): Promise<SearchResult[]> {
    const files = this.app.vault.getMarkdownFiles();
    const results: SearchResult[] = [];

    for (const file of files) {
      const content = await this.app.vault.cachedRead(file);
      const matches = evaluateSearchNote(file.path, content, query);
      if (matches.length > 0) {
        results.push({ path: file.path, matches });
      }
    }

    results.sort((a, b) => a.path.localeCompare(b.path));
    return results;
  }
}

// =============================================================================
// Helper functions for mapping Obsidian cache types to vaultiel types
// =============================================================================

function mapLinkCache(lc: LinkCache, embed: boolean): Link {
  const parts = parseObsidianLink(lc.link);
  return {
    target: parts.target,
    alias: lc.displayText !== lc.link ? lc.displayText : undefined,
    heading: parts.heading,
    blockId: parts.blockId,
    embed,
    line: lc.position.start.line + 1,
  };
}

function mapEmbedCache(ec: EmbedCache): Link {
  const parts = parseObsidianLink(ec.link);
  return {
    target: parts.target,
    alias: ec.displayText !== ec.link ? ec.displayText : undefined,
    heading: parts.heading,
    blockId: parts.blockId,
    embed: true,
    line: ec.position.start.line + 1,
  };
}

/** Parse an Obsidian link string ("Note#heading" or "Note#^blockid") into parts. */
function parseObsidianLink(link: string): {
  target: string;
  heading?: string;
  blockId?: string;
} {
  // Check for block reference first: target#^blockid
  const blockIdx = link.indexOf("#^");
  if (blockIdx !== -1) {
    return {
      target: link.slice(0, blockIdx),
      blockId: link.slice(blockIdx + 2),
    };
  }

  // Check for heading reference: target#heading
  const headingIdx = link.indexOf("#");
  if (headingIdx !== -1) {
    return {
      target: link.slice(0, headingIdx),
      heading: link.slice(headingIdx + 1),
    };
  }

  return { target: link };
}
