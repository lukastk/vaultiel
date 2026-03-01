export const vaultSubcommands = [
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
            const pattern = args["pattern"];
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
            const exists = vault.noteExists(args["note"]);
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
            const path = vault.resolveNote(args["query"]);
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
            const content = vault.getContent(args["note"]);
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
            const body = vault.getBody(args["note"]);
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
            const json = vault.getFrontmatter(args["note"]);
            if (json === null) {
                return { text: "null", exitCode: 0 };
            }
            const parsed = JSON.parse(json);
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
            const json = vault.inspect(args["note"]);
            const parsed = JSON.parse(json);
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
            return { data: vault.getLinks(args["note"]), exitCode: 0 };
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
            return { data: vault.getTags(args["note"]), exitCode: 0 };
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
            return { data: vault.getHeadings(args["note"]), exitCode: 0 };
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
            return { data: vault.getBlockIds(args["note"]), exitCode: 0 };
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
            const linksTo = args["linksTo"];
            return {
                data: vault.getTasks(args["note"], linksTo ?? null),
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
            const json = vault.getTaskTrees(args["note"]);
            const parsed = JSON.parse(json);
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
                data: vault.getIncomingLinks(args["note"]),
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
                data: vault.getOutgoingLinks(args["note"]),
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
            const content = args["content"];
            vault.createNote(args["note"], content);
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
            vault.deleteNote(args["note"]);
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
            vault.renameNote(args["from"], args["to"]);
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
            vault.setContent(args["note"], args["content"]);
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
            vault.setRawContent(args["note"], args["content"]);
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
            vault.modifyFrontmatter(args["note"], args["key"], args["value"]);
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
            vault.appendContent(args["note"], args["content"]);
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
            vault.replaceContent(args["note"], args["pattern"], args["replacement"]);
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
            const line = args["line"];
            const symbol = args["symbol"];
            if (line === undefined || symbol === undefined) {
                throw new Error("--line and --symbol are required");
            }
            vault.setTaskSymbol(args["note"], line, symbol);
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
            const result = vault.initMetadata(args["note"], args["force"]);
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
            const result = vault.getVaultielMetadata(args["note"]);
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
            const path = vault.findById(args["id"]);
            if (path === null) {
                return { text: "Not found", exitCode: 1 };
            }
            return { text: path, exitCode: 0 };
        },
    },
];
//# sourceMappingURL=vault-subcommands.js.map