import type { CLISubcommand } from "./types.js";

export const vaultSubcommands: CLISubcommand[] = [
  // ── Read ──────────────────────────────────────────────────

  {
    name: "list",
    description: "List notes in the vault",
    group: "Read",
    args: [],
    options: [
      {
        name: "pattern",
        flag: "--pattern",
        description: "Glob pattern to filter notes",
        type: "string",
      },
    ],
    execute(vault, args) {
      const pattern = args["pattern"] as string | undefined;
      const notes = pattern
        ? vault.listNotesMatching(pattern)
        : vault.listNotes();
      return { text: notes.join("\n"), exitCode: 0 };
    },
  },

  {
    name: "validate",
    description: "Validate frontmatter parsing",
    group: "Read",
    args: [
      { name: "note", description: "Note path (optional — validates all if omitted)", required: false },
    ],
    options: [
      {
        name: "pattern",
        flag: "--pattern",
        description: "Glob pattern to filter notes",
        type: "string",
      },
    ],
    execute(vault, args) {
      const note = args["note"] as string | undefined;
      const pattern = args["pattern"] as string | undefined;

      let paths: string[];
      if (note) {
        paths = [note];
      } else if (pattern) {
        paths = vault.listNotesMatching(pattern);
      } else {
        paths = vault.listNotes();
      }

      const errors: string[] = [];
      for (const path of paths) {
        try {
          vault.getFrontmatter(path);
        } catch (err) {
          const msg = err instanceof Error ? err.message : String(err);
          errors.push(`${path}: ${msg}`);
        }
      }

      if (errors.length === 0) {
        return { text: `All ${paths.length} notes valid`, exitCode: 0 };
      }

      return { text: errors.join("\n"), exitCode: 1 };
    },
  },

  {
    name: "exists",
    description: "Check if a note exists",
    group: "Read",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      const exists = vault.noteExists(args["note"] as string);
      return { text: String(exists), exitCode: exists ? 0 : 1 };
    },
  },

  {
    name: "resolve",
    description: "Resolve a note name or alias to a path",
    group: "Read",
    args: [
      { name: "query", description: "Note name or alias", required: true },
    ],
    options: [],
    execute(vault, args) {
      const path = vault.resolveNote(args["query"] as string);
      return { text: path, exitCode: 0 };
    },
  },

  {
    name: "content",
    description: "Get full content of a note (including frontmatter)",
    group: "Read",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      const content = vault.getContent(args["note"] as string);
      return { text: content, exitCode: 0 };
    },
  },

  {
    name: "body",
    description: "Get body of a note (without frontmatter)",
    group: "Read",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      const body = vault.getBody(args["note"] as string);
      return { text: body, exitCode: 0 };
    },
  },

  {
    name: "frontmatter",
    description: "Get frontmatter of a note as JSON",
    group: "Read",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      const json = vault.getFrontmatter(args["note"] as string);
      if (json === null) {
        return { text: "null", exitCode: 0 };
      }
      const parsed: unknown = JSON.parse(json);
      return { data: parsed, exitCode: 0 };
    },
  },

  {
    name: "inspect",
    description: "Full inspection of a note (JSON)",
    group: "Read",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      const json = vault.inspect(args["note"] as string);
      const parsed: unknown = JSON.parse(json);
      return { data: parsed, exitCode: 0 };
    },
  },

  {
    name: "inline-properties",
    description: "List inline properties of a note",
    group: "Read",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      const props = vault.getInlineProperties(args["note"] as string);
      return { data: props, exitCode: 0 };
    },
  },

  {
    name: "properties",
    description: "Get all properties of a note (frontmatter + inline)",
    group: "Read",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [
      {
        name: "inline",
        flag: "--inline",
        description: "Only show inline properties",
        type: "boolean",
        default: false,
      },
      {
        name: "frontmatter",
        flag: "--frontmatter",
        description: "Only show frontmatter properties",
        type: "boolean",
        default: false,
      },
    ],
    execute(vault, args) {
      const note = args["note"] as string;
      const onlyInline = args["inline"] as boolean;
      const onlyFrontmatter = args["frontmatter"] as boolean;

      if (onlyInline) {
        const props = vault.getInlineProperties(note);
        return { data: props, exitCode: 0 };
      }

      if (onlyFrontmatter) {
        const json = vault.getFrontmatter(note);
        if (json === null) return { data: {}, exitCode: 0 };
        return { data: JSON.parse(json) as unknown, exitCode: 0 };
      }

      // Merge: frontmatter + inline
      const fmJson = vault.getFrontmatter(note);
      const fm = fmJson ? JSON.parse(fmJson) as Record<string, unknown> : {};
      const inlineProps = vault.getInlineProperties(note);

      const merged: Record<string, unknown> = { ...fm };
      for (const prop of inlineProps) {
        if (prop.key in merged) {
          const existing = merged[prop.key];
          merged[prop.key] = Array.isArray(existing)
            ? [...existing, prop.value]
            : [existing, prop.value];
        } else {
          merged[prop.key] = prop.value;
        }
      }

      return { data: merged, exitCode: 0 };
    },
  },

  {
    name: "property",
    description: "Get a single property value by key",
    group: "Read",
    args: [
      { name: "note", description: "Note path", required: true },
      { name: "key", description: "Property key", required: true },
    ],
    options: [
      {
        name: "inline",
        flag: "--inline",
        description: "Only search inline properties",
        type: "boolean",
        default: false,
      },
      {
        name: "frontmatter",
        flag: "--frontmatter",
        description: "Only search frontmatter",
        type: "boolean",
        default: false,
      },
    ],
    execute(vault, args) {
      const note = args["note"] as string;
      const key = args["key"] as string;
      const onlyInline = args["inline"] as boolean;
      const onlyFrontmatter = args["frontmatter"] as boolean;

      const values: unknown[] = [];

      if (!onlyInline) {
        const fmJson = vault.getFrontmatter(note);
        if (fmJson) {
          const fm = JSON.parse(fmJson) as Record<string, unknown>;
          if (key in fm) values.push(fm[key]);
        }
      }

      if (!onlyFrontmatter) {
        const inlineProps = vault.getInlineProperties(note);
        for (const prop of inlineProps) {
          if (prop.key === key) values.push(prop.value);
        }
      }

      if (values.length === 0) {
        return { exitCode: 1 };
      }

      const result = values.length === 1 ? values[0] : values;
      return { data: result, exitCode: 0 };
    },
  },

  {
    name: "search",
    description: "Search notes by query",
    group: "Read",
    args: [
      { name: "query", description: "Search query string", required: true },
    ],
    options: [
      {
        name: "json",
        flag: "--json",
        description: "Output full match details as JSON",
        type: "boolean",
        default: false,
      },
      {
        name: "limit",
        flag: "--limit",
        description: "Maximum number of results",
        type: "number",
      },
    ],
    execute(vault, args) {
      const query = args["query"] as string;
      const jsonOutput = args["json"] as boolean;
      const limit = args["limit"] as number | undefined;

      let results = vault.search(query);

      if (limit !== undefined && limit > 0) {
        results = results.slice(0, limit);
      }

      if (results.length === 0) {
        return { text: "No matches found", exitCode: 1 };
      }

      if (jsonOutput) {
        return { data: results, exitCode: 0 };
      }

      // Default: one path per line with match summary
      const lines: string[] = [];
      for (const result of results) {
        const matchCount = result.matches.length;
        const fields = [...new Set(result.matches.map((m: { field: string }) => m.field))];
        lines.push(`${result.path}  (${matchCount} match${matchCount !== 1 ? "es" : ""}: ${fields.join(", ")})`);
      }
      return { text: lines.join("\n"), exitCode: 0 };
    },
  },

  // ── Parse ─────────────────────────────────────────────────

  {
    name: "links",
    description: "Parse links from a note",
    group: "Parse",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      return { data: vault.getLinks(args["note"] as string), exitCode: 0 };
    },
  },

  {
    name: "tags",
    description: "Parse tags from a note",
    group: "Parse",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      return { data: vault.getTags(args["note"] as string), exitCode: 0 };
    },
  },

  {
    name: "headings",
    description: "Parse headings from a note",
    group: "Parse",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      return { data: vault.getHeadings(args["note"] as string), exitCode: 0 };
    },
  },

  {
    name: "block-ids",
    description: "Parse block IDs from a note",
    group: "Parse",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      return { data: vault.getBlockIds(args["note"] as string), exitCode: 0 };
    },
  },

  {
    name: "tasks",
    description: "Parse tasks from a note",
    group: "Parse",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [
      {
        name: "linksTo",
        flag: "--links-to",
        description: "Filter to tasks linking to this target",
        type: "string",
      },
    ],
    execute(vault, args) {
      const linksTo = args["linksTo"] as string | undefined;
      return {
        data: vault.getTasks(args["note"] as string, linksTo ?? null),
        exitCode: 0,
      };
    },
  },

  {
    name: "task-trees",
    description: "Parse task trees from a note (hierarchical)",
    group: "Parse",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      const json = vault.getTaskTrees(args["note"] as string);
      const parsed: unknown = JSON.parse(json);
      return { data: parsed, exitCode: 0 };
    },
  },

  // ── Graph ─────────────────────────────────────────────────

  {
    name: "incoming-links",
    description: "Get incoming links to a note",
    group: "Graph",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      return {
        data: vault.getIncomingLinks(args["note"] as string),
        exitCode: 0,
      };
    },
  },

  {
    name: "outgoing-links",
    description: "Get outgoing links from a note",
    group: "Graph",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      return {
        data: vault.getOutgoingLinks(args["note"] as string),
        exitCode: 0,
      };
    },
  },

  // ── Write ─────────────────────────────────────────────────

  {
    name: "create",
    description: "Create a new note",
    group: "Write",
    args: [
      { name: "note", description: "Note path", required: true },
      { name: "content", description: 'Content (use "-" for stdin)', required: true },
    ],
    options: [],
    execute(vault, args) {
      const content = args["content"] as string;
      vault.createNote(args["note"] as string, content);
      return { exitCode: 0 };
    },
  },

  {
    name: "delete",
    description: "Delete a note",
    group: "Write",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      vault.deleteNote(args["note"] as string);
      return { exitCode: 0 };
    },
  },

  {
    name: "rename",
    description: "Rename a note",
    group: "Write",
    args: [
      { name: "from", description: "Current note path", required: true },
      { name: "to", description: "New note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      vault.renameNote(args["from"] as string, args["to"] as string);
      return { exitCode: 0 };
    },
  },

  {
    name: "set-content",
    description: "Set note body (preserves frontmatter)",
    group: "Write",
    args: [
      { name: "note", description: "Note path", required: true },
      { name: "content", description: 'New body content (use "-" for stdin)', required: true },
    ],
    options: [],
    execute(vault, args) {
      vault.setContent(args["note"] as string, args["content"] as string);
      return { exitCode: 0 };
    },
  },

  {
    name: "set-raw-content",
    description: "Set full content of a note (replaces everything)",
    group: "Write",
    args: [
      { name: "note", description: "Note path", required: true },
      { name: "content", description: 'Full content (use "-" for stdin)', required: true },
    ],
    options: [],
    execute(vault, args) {
      vault.setRawContent(args["note"] as string, args["content"] as string);
      return { exitCode: 0 };
    },
  },

  {
    name: "modify-frontmatter",
    description: "Modify a frontmatter field",
    group: "Write",
    args: [
      { name: "note", description: "Note path", required: true },
      { name: "key", description: "Frontmatter key", required: true },
      { name: "value", description: "Value (parsed as YAML)", required: true },
    ],
    options: [
      {
        name: "append",
        flag: "--append",
        description: "Append value to list instead of replacing",
        type: "boolean",
        default: false,
      },
    ],
    execute(vault, args) {
      const note = args["note"] as string;
      const key = args["key"] as string;
      const value = args["value"] as string;
      if (args["append"] as boolean) {
        vault.appendFrontmatterValue(note, key, value);
      } else {
        vault.modifyFrontmatter(note, key, value);
      }
      return { exitCode: 0 };
    },
  },

  {
    name: "append",
    description: "Append content to a note",
    group: "Write",
    args: [
      { name: "note", description: "Note path", required: true },
      { name: "content", description: 'Content to append (use "-" for stdin)', required: true },
    ],
    options: [],
    execute(vault, args) {
      vault.appendContent(args["note"] as string, args["content"] as string);
      return { exitCode: 0 };
    },
  },

  {
    name: "replace",
    description: "Replace first occurrence of pattern in a note",
    group: "Write",
    args: [
      { name: "note", description: "Note path", required: true },
      { name: "pattern", description: "Pattern to find", required: true },
      { name: "replacement", description: "Replacement text", required: true },
    ],
    options: [],
    execute(vault, args) {
      vault.replaceContent(
        args["note"] as string,
        args["pattern"] as string,
        args["replacement"] as string,
      );
      return { exitCode: 0 };
    },
  },

  {
    name: "set-task-symbol",
    description: "Change the checkbox symbol of a task",
    group: "Write",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [
      {
        name: "line",
        flag: "--line",
        description: "Line number (1-indexed)",
        type: "number",
      },
      {
        name: "symbol",
        flag: "--symbol",
        description: "New symbol (single character)",
        type: "string",
      },
    ],
    execute(vault, args) {
      const line = args["line"] as number | undefined;
      const symbol = args["symbol"] as string | undefined;
      if (line === undefined || symbol === undefined) {
        throw new Error("--line and --symbol are required");
      }
      vault.setTaskSymbol(args["note"] as string, line, symbol);
      return { exitCode: 0 };
    },
  },

  {
    name: "remove-frontmatter",
    description: "Remove a frontmatter key",
    group: "Write",
    args: [
      { name: "note", description: "Note path", required: true },
      { name: "key", description: "Frontmatter key to remove", required: true },
    ],
    options: [],
    execute(vault, args) {
      vault.removeFrontmatterKey(args["note"] as string, args["key"] as string);
      return { exitCode: 0 };
    },
  },

  {
    name: "set-property",
    description: "Set a property value (frontmatter or inline)",
    group: "Write",
    args: [
      { name: "note", description: "Note path", required: true },
      { name: "key", description: "Property key", required: true },
      { name: "value", description: "Property value", required: true },
    ],
    options: [
      {
        name: "inline",
        flag: "--inline",
        description: "Target inline properties only",
        type: "boolean",
        default: false,
      },
      {
        name: "frontmatter",
        flag: "--frontmatter",
        description: "Target frontmatter only",
        type: "boolean",
        default: false,
      },
      {
        name: "index",
        flag: "--index",
        description: "Index of inline property to modify",
        type: "number",
      },
      {
        name: "append",
        flag: "--append",
        description: "Append to frontmatter list instead of replacing",
        type: "boolean",
        default: false,
      },
    ],
    execute(vault, args) {
      const note = args["note"] as string;
      const key = args["key"] as string;
      const value = args["value"] as string;
      const onlyInline = args["inline"] as boolean;
      const onlyFrontmatter = args["frontmatter"] as boolean;
      const index = args["index"] as number | undefined;
      const append = args["append"] as boolean;

      if (onlyInline) {
        if (append) throw new Error("--append is not allowed with --inline");
        vault.setInlineProperty(note, key, value, index ?? null);
        return { exitCode: 0 };
      }

      if (onlyFrontmatter) {
        if (append) {
          vault.appendFrontmatterValue(note, key, value);
        } else {
          vault.modifyFrontmatter(note, key, value);
        }
        return { exitCode: 0 };
      }

      // Auto-detect: check where the key exists
      const fmJson = vault.getFrontmatter(note);
      const fm = fmJson ? JSON.parse(fmJson) as Record<string, unknown> : {};
      const inlineProps = vault.getInlineProperties(note);
      const inFm = key in fm;
      const inInline = inlineProps.some(p => p.key === key);

      if (append) {
        vault.appendFrontmatterValue(note, key, value);
        return { exitCode: 0 };
      }

      if (inFm && !inInline) {
        vault.modifyFrontmatter(note, key, value);
        return { exitCode: 0 };
      }

      if (inInline && !inFm) {
        vault.setInlineProperty(note, key, value, index ?? null);
        return { exitCode: 0 };
      }

      if (inFm && inInline) {
        throw new Error(
          `Property "${key}" exists in both frontmatter and inline. Use --frontmatter or --inline to specify.`,
        );
      }

      // Key doesn't exist anywhere — default to frontmatter
      vault.modifyFrontmatter(note, key, value);
      return { exitCode: 0 };
    },
  },

  {
    name: "remove-property",
    description: "Remove a property (frontmatter and/or inline)",
    group: "Write",
    args: [
      { name: "note", description: "Note path", required: true },
      { name: "key", description: "Property key to remove", required: true },
    ],
    options: [
      {
        name: "inline",
        flag: "--inline",
        description: "Only remove from inline properties",
        type: "boolean",
        default: false,
      },
      {
        name: "frontmatter",
        flag: "--frontmatter",
        description: "Only remove from frontmatter",
        type: "boolean",
        default: false,
      },
      {
        name: "index",
        flag: "--index",
        description: "Index of inline property to remove",
        type: "number",
      },
    ],
    execute(vault, args) {
      const note = args["note"] as string;
      const key = args["key"] as string;
      const onlyInline = args["inline"] as boolean;
      const onlyFrontmatter = args["frontmatter"] as boolean;
      const index = args["index"] as number | undefined;

      if (onlyInline) {
        vault.removeInlineProperty(note, key, index ?? null);
        return { exitCode: 0 };
      }

      if (onlyFrontmatter) {
        vault.removeFrontmatterKey(note, key);
        return { exitCode: 0 };
      }

      // No flags: remove from both
      const fmJson = vault.getFrontmatter(note);
      if (fmJson) {
        const fm = JSON.parse(fmJson) as Record<string, unknown>;
        if (key in fm) {
          vault.removeFrontmatterKey(note, key);
        }
      }

      const inlineProps = vault.getInlineProperties(note);
      const matching = inlineProps.filter(p => p.key === key);
      // Remove inline properties in reverse order so indices stay valid
      for (let i = matching.length - 1; i >= 0; i--) {
        vault.removeInlineProperty(note, key, null);
      }

      return { exitCode: 0 };
    },
  },

  {
    name: "rename-property",
    description: "Rename a property key (frontmatter and/or inline)",
    group: "Write",
    args: [
      { name: "note", description: "Note path", required: true },
      { name: "from-key", description: "Current property key", required: true },
      { name: "to-key", description: "New property key", required: true },
    ],
    options: [
      {
        name: "inline",
        flag: "--inline",
        description: "Only rename in inline properties",
        type: "boolean",
        default: false,
      },
      {
        name: "frontmatter",
        flag: "--frontmatter",
        description: "Only rename in frontmatter",
        type: "boolean",
        default: false,
      },
    ],
    execute(vault, args) {
      const note = args["note"] as string;
      const fromKey = args["from-key"] as string;
      const toKey = args["to-key"] as string;
      const onlyInline = args["inline"] as boolean;
      const onlyFrontmatter = args["frontmatter"] as boolean;

      if (onlyInline) {
        vault.renameInlineProperty(note, fromKey, toKey);
        return { exitCode: 0 };
      }

      if (onlyFrontmatter) {
        vault.renameFrontmatterKey(note, fromKey, toKey);
        return { exitCode: 0 };
      }

      // No flags: rename in both
      const fmJson = vault.getFrontmatter(note);
      if (fmJson) {
        const fm = JSON.parse(fmJson) as Record<string, unknown>;
        if (fromKey in fm) {
          vault.renameFrontmatterKey(note, fromKey, toKey);
        }
      }

      vault.renameInlineProperty(note, fromKey, toKey);
      return { exitCode: 0 };
    },
  },

  // ── Metadata ──────────────────────────────────────────────

  {
    name: "init-metadata",
    description: "Initialize vaultiel metadata for a note",
    group: "Metadata",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [
      {
        name: "force",
        flag: "--force",
        description: "Overwrite existing metadata",
        type: "boolean",
        default: false,
      },
    ],
    execute(vault, args) {
      const result = vault.initMetadata(
        args["note"] as string,
        args["force"] as boolean,
      );
      return { data: result, exitCode: 0 };
    },
  },

  {
    name: "metadata",
    description: "Get vaultiel metadata from a note",
    group: "Metadata",
    args: [
      { name: "note", description: "Note path", required: true },
    ],
    options: [],
    execute(vault, args) {
      const result = vault.getVaultielMetadata(args["note"] as string);
      return { data: result, exitCode: 0 };
    },
  },

  {
    name: "find-by-id",
    description: "Find a note by its vaultiel ID",
    group: "Metadata",
    args: [
      { name: "id", description: "Vaultiel note ID", required: true },
    ],
    options: [],
    execute(vault, args) {
      const path = vault.findById(args["id"] as string);
      if (path === null) {
        return { text: "Not found", exitCode: 1 };
      }
      return { text: path, exitCode: 0 };
    },
  },
];
