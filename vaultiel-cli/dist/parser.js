/**
 * Parse CLI argv into a Record of named arguments for a subcommand.
 * Positional args are matched in order, options are matched by --flag.
 */
export function parseSubcommandArgs(argv, subcmd) {
    const result = {};
    // Set defaults for options
    for (const opt of subcmd.options) {
        if (opt.default !== undefined) {
            result[opt.name] = opt.default;
        }
    }
    // Build flag lookup
    const optionsByFlag = new Map();
    for (const opt of subcmd.options) {
        optionsByFlag.set(opt.flag, opt);
    }
    // Parse argv
    const positionals = [];
    let i = 0;
    while (i < argv.length) {
        const arg = argv[i];
        const opt = optionsByFlag.get(arg);
        if (opt) {
            if (opt.type === "boolean") {
                result[opt.name] = true;
                i++;
            }
            else {
                i++;
                const value = argv[i];
                if (value === undefined) {
                    throw new Error(`Option ${arg} requires a value`);
                }
                result[opt.name] = opt.type === "number" ? Number(value) : value;
                i++;
            }
        }
        else if (arg.startsWith("--")) {
            throw new Error(`Unknown option: ${arg}`);
        }
        else {
            positionals.push(arg);
            i++;
        }
    }
    // Assign positionals to args
    for (let j = 0; j < subcmd.args.length; j++) {
        const argDef = subcmd.args[j];
        const value = positionals[j];
        if (value !== undefined) {
            result[argDef.name] = value;
        }
        else if (argDef.required) {
            throw new Error(`Missing required argument: <${argDef.name}>`);
        }
    }
    return result;
}
/** Format help text for a single subcommand. */
export function formatSubcommandHelp(subcmd) {
    const lines = [];
    lines.push(`${subcmd.name} — ${subcmd.description}`);
    lines.push("");
    // Usage line
    const argParts = subcmd.args.map((a) => a.required ? `<${a.name}>` : `[${a.name}]`);
    const optParts = subcmd.options.map((o) => {
        if (o.type === "boolean")
            return `[${o.flag}]`;
        return `[${o.flag} <${o.name}>]`;
    });
    lines.push(`Usage: ${subcmd.name} ${[...argParts, ...optParts].join(" ")}`);
    if (subcmd.args.length > 0) {
        lines.push("");
        lines.push("Arguments:");
        for (const a of subcmd.args) {
            const req = a.required ? "(required)" : "(optional)";
            lines.push(`  ${a.name}  ${a.description} ${req}`);
        }
    }
    if (subcmd.options.length > 0) {
        lines.push("");
        lines.push("Options:");
        for (const o of subcmd.options) {
            const def = o.default !== undefined ? ` (default: ${String(o.default)})` : "";
            lines.push(`  ${o.flag}  ${o.description}${def}`);
        }
    }
    return lines.join("\n");
}
/** Format a grouped listing of all subcommands. */
export function formatSubcommandList(subcmds, cliName) {
    const lines = [];
    lines.push(`Usage: ${cliName} --vault <path> <subcommand> [args]`);
    lines.push("");
    // Group by group name
    const groups = new Map();
    for (const s of subcmds) {
        let group = groups.get(s.group);
        if (!group) {
            group = [];
            groups.set(s.group, group);
        }
        group.push(s);
    }
    // Find max name width for alignment
    const maxName = Math.max(...subcmds.map((s) => s.name.length));
    for (const [groupName, cmds] of groups) {
        lines.push(`${groupName}:`);
        for (const cmd of cmds) {
            lines.push(`  ${cmd.name.padEnd(maxName + 2)}${cmd.description}`);
        }
        lines.push("");
    }
    return lines.join("\n");
}
//# sourceMappingURL=parser.js.map