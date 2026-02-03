#!/bin/bash
# Basic Vaultiel CLI Operations
# This script demonstrates fundamental note operations.

set -e

VAULT="${VAULT:-/tmp/vaultiel-demo}"

echo "=== Vaultiel CLI Demo: Basic Operations ==="
echo "Using vault: $VAULT"
echo

# Create a demo vault
mkdir -p "$VAULT"

# --- Creating Notes ---
echo "--- Creating Notes ---"

vaultiel --vault "$VAULT" create "Welcome.md" \
    --frontmatter '{"title": "Welcome", "tags": ["demo", "intro"]}' \
    --content "# Welcome to Vaultiel

This is a demo vault created by the Vaultiel CLI.

## Features

- Fast markdown parsing
- Link graph traversal
- Task extraction
- And much more!

See [[Getting Started]] for more information."

echo "Created Welcome.md"

vaultiel --vault "$VAULT" create "Getting Started.md" \
    --frontmatter '{"title": "Getting Started", "aliases": ["quickstart", "tutorial"]}' \
    --content "# Getting Started

Welcome to the tutorial! Check out [[Welcome]] if you haven't already.

## Installation

Install vaultiel via cargo:

\`\`\`bash
cargo install vaultiel
\`\`\`

## Next Steps

- Read the [[Documentation]]
- Explore the [[Examples]]

#tutorial #beginner"

echo "Created Getting Started.md"

vaultiel --vault "$VAULT" create "Documentation.md" \
    --content "# Documentation

Full documentation for Vaultiel.

## Commands

See the README for a complete list of commands.

#reference"

echo "Created Documentation.md"

# --- Listing Notes ---
echo
echo "--- Listing Notes ---"

echo "All notes:"
vaultiel --vault "$VAULT" list | jq -r '.notes[].path'

echo
echo "Notes with #tutorial tag:"
vaultiel --vault "$VAULT" list --tag tutorial | jq -r '.notes[].path'

# --- Reading Content ---
echo
echo "--- Reading Content ---"

echo "Content of Welcome.md (body only):"
vaultiel --vault "$VAULT" get-content "Welcome.md" | head -5
echo "..."

echo
echo "Frontmatter of Getting Started.md:"
vaultiel --vault "$VAULT" get-frontmatter "Getting Started.md"

# --- Resolving Notes ---
echo
echo "--- Resolving Notes ---"

echo "Resolving 'quickstart' (alias):"
vaultiel --vault "$VAULT" resolve "quickstart"

echo
echo "Resolving 'Welcome':"
vaultiel --vault "$VAULT" resolve "Welcome"

# --- Searching ---
echo
echo "--- Searching ---"

echo "Search for 'started':"
vaultiel --vault "$VAULT" search "started" --limit 5

echo
echo "Search in content for 'cargo':"
vaultiel --vault "$VAULT" search "cargo" --content --limit 5

# --- Modifying Content ---
echo
echo "--- Modifying Content ---"

echo "Appending to Documentation.md..."
vaultiel --vault "$VAULT" append-content "Documentation.md" \
    --content "

## API Reference

See the generated API docs for detailed information.

^api-section"

echo "Content after append:"
vaultiel --vault "$VAULT" get-content "Documentation.md"

# --- Cleanup ---
echo
echo "--- Cleanup ---"
echo "Demo vault created at: $VAULT"
echo "To clean up, run: rm -rf $VAULT"
