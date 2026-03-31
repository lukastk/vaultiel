export type {
  CLISubcommand,
  CLIOutput,
  SubcommandArg,
  SubcommandOption,
} from "./types.js";

export { vaultSubcommands } from "./vault-subcommands.js";
export { dispatchVaultSubcommand } from "./dispatch.js";
export {
  parseSubcommandArgs,
  formatSubcommandHelp,
  formatSubcommandList,
} from "./parser.js";
