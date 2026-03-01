#!/usr/bin/env node

/**
 * Standalone CLI for vaultiel vault operations.
 *
 * Usage:
 *   vaultiel --vault <path> <subcommand> [args]
 *   vaultiel --vault <path> --help
 *   vaultiel --vault <path> <subcommand> --help
 */

import { resolve } from "node:path";
import { JsVault } from "@vaultiel/node";
import { vaultSubcommands } from "./vault-subcommands.js";
import { formatSubcommandList } from "./parser.js";
import { dispatchVaultSubcommand } from "./dispatch.js";

const CLI_NAME = "vaultiel";
const args = process.argv.slice(2);

// Handle --help before requiring --vault
if (args.length === 0 || (args.includes("--help") && !args.includes("--vault"))) {
  console.log(formatSubcommandList(vaultSubcommands, CLI_NAME));
  process.exit(0);
}

// Parse --vault
const vaultIdx = args.indexOf("--vault");
if (vaultIdx === -1 || vaultIdx + 1 >= args.length) {
  console.error(`Usage: ${CLI_NAME} --vault <path> <subcommand> [args]`);
  console.error(`\nRun '${CLI_NAME} --help' for a list of subcommands.`);
  process.exit(1);
}
const vaultPath = resolve(args[vaultIdx + 1]!);
const remaining = [...args.slice(0, vaultIdx), ...args.slice(vaultIdx + 2)];

// Top-level --help with --vault
if (remaining.length === 0 || (remaining.length === 1 && remaining[0] === "--help")) {
  console.log(formatSubcommandList(vaultSubcommands, CLI_NAME));
  process.exit(0);
}

// Create vault (no task config for standalone CLI)
const vault = new JsVault(vaultPath);

// Dispatch subcommand
const subcmdName = remaining[0]!;
const subcmdArgs = remaining.slice(1);
dispatchVaultSubcommand(vault, vaultSubcommands, subcmdName, subcmdArgs, CLI_NAME);
