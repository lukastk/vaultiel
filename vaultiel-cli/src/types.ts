import type { JsVault } from "@vaultiel/node";

export interface SubcommandArg {
  name: string;
  description: string;
  required: boolean;
}

export interface SubcommandOption {
  name: string;
  flag: string;
  description: string;
  type: "string" | "number" | "boolean";
  default?: unknown;
}

export interface CLISubcommand {
  name: string;
  description: string;
  group: string;
  args: SubcommandArg[];
  options: SubcommandOption[];
  execute(vault: JsVault, args: Record<string, unknown>): CLIOutput;
}

export interface CLIOutput {
  data?: unknown;
  text?: string;
  exitCode: number;
}
