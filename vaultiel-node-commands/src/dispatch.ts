import type { JsVault } from "@vaultiel/node";
import type { CLISubcommand } from "./types.js";
import { parseSubcommandArgs, formatSubcommandHelp, formatSubcommandList } from "./parser.js";

/**
 * Dispatch a vault subcommand: lookup, parse args, execute, print output.
 */
export function dispatchVaultSubcommand(
  vault: JsVault,
  subcmds: CLISubcommand[],
  subcmdName: string,
  argv: string[],
  cliName: string,
): void {
  const subcmd = subcmds.find((s) => s.name === subcmdName);
  if (!subcmd) {
    console.error(`Unknown subcommand: ${subcmdName}`);
    console.error("");
    console.error(formatSubcommandList(subcmds, cliName));
    process.exit(1);
  }

  // Handle --help for this subcommand
  if (argv.includes("--help")) {
    console.log(formatSubcommandHelp(subcmd));
    return;
  }

  let args: Record<string, unknown>;
  try {
    args = parseSubcommandArgs(argv, subcmd);
  } catch (err) {
    console.error(String(err instanceof Error ? err.message : err));
    console.error("");
    console.log(formatSubcommandHelp(subcmd));
    process.exit(1);
  }

  let output;
  try {
    output = subcmd.execute(vault, args);
  } catch (err) {
    console.error(String(err instanceof Error ? err.message : err));
    process.exit(1);
  }

  if (output.data !== undefined) {
    console.log(JSON.stringify(output.data, null, 2));
  } else if (output.text !== undefined) {
    console.log(output.text);
  }

  if (output.exitCode !== 0) {
    process.exit(output.exitCode);
  }
}
