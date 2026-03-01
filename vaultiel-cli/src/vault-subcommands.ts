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
    options: [],
    execute(vault, args) {
      vault.modifyFrontmatter(
        args["note"] as string,
        args["key"] as string,
        args["value"] as string,
      );
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
