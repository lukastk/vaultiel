import type { JsVault } from "@vaultiel/node";
import type { CLISubcommand } from "./types.js";
/**
 * Dispatch a vault subcommand: lookup, parse args, execute, print output.
 */
export declare function dispatchVaultSubcommand(vault: JsVault, subcmds: CLISubcommand[], subcmdName: string, argv: string[], cliName: string): void;
//# sourceMappingURL=dispatch.d.ts.map