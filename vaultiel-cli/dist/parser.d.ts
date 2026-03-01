import type { CLISubcommand } from "./types.js";
/**
 * Parse CLI argv into a Record of named arguments for a subcommand.
 * Positional args are matched in order, options are matched by --flag.
 */
export declare function parseSubcommandArgs(argv: string[], subcmd: CLISubcommand): Record<string, unknown>;
/** Format help text for a single subcommand. */
export declare function formatSubcommandHelp(subcmd: CLISubcommand): string;
/** Format a grouped listing of all subcommands. */
export declare function formatSubcommandList(subcmds: CLISubcommand[], cliName: string): string;
//# sourceMappingURL=parser.d.ts.map