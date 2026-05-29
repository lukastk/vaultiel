import type { JsVault } from "@vaultiel/node";
import type { CLISubcommand } from "./types.js";
import { parseSubcommandArgs, formatSubcommandHelp, formatSubcommandList } from "./parser.js";

/** Read all of stdin as a UTF-8 string. */
async function readStdin(): Promise<string> {
  const chunks: Buffer[] = [];
  for await (const chunk of process.stdin) chunks.push(chunk as Buffer);
  return Buffer.concat(chunks).toString("utf-8");
}

/**
 * For any argument flagged with `stdin: true` whose value is "-", replace the
 * value with the contents of stdin. Stdin is read at most once and reused.
 */
async function resolveStdinArgs(
  subcmd: CLISubcommand,
  args: Record<string, unknown>,
): Promise<void> {
  const stdinArgs = subcmd.args.filter((a) => a.stdin && args[a.name] === "-");
  if (stdinArgs.length === 0) return;
  const content = await readStdin();
  for (const arg of stdinArgs) {
    args[arg.name] = content;
  }
}

/**
 * Dispatch a vault subcommand: lookup, parse args, execute, print output.
 */
export async function dispatchVaultSubcommand(
  vault: JsVault,
  subcmds: CLISubcommand[],
  subcmdName: string,
  argv: string[],
  cliName: string,
): Promise<void> {
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

  await resolveStdinArgs(subcmd, args);

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
