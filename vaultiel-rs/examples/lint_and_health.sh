#!/bin/bash
# Vault Health and Linting with Vaultiel CLI
# This script demonstrates how to check vault health and fix issues.

set -e

VAULT="${VAULT:-/tmp/vaultiel-lint-demo}"

echo "=== Vaultiel CLI Demo: Linting & Health ==="
echo "Using vault: $VAULT"
echo

mkdir -p "$VAULT"

# Create notes with various issues
echo "Creating notes with intentional issues..."

# Note with broken links
vaultiel --vault "$VAULT" create "Has Broken Links.md" \
    --content "# Has Broken Links

This note links to [[Non Existent Note]] which doesn't exist.

Also links to [[Another Missing]] and embeds ![[missing-image.png]].

And a broken heading link: [[Valid Note#Missing Heading]]

#test"

# Valid note (target for heading link test)
vaultiel --vault "$VAULT" create "Valid Note.md" \
    --content "# Valid Note

## Existing Heading

This heading exists.

Links back to [[Has Broken Links]].

#test"

# Orphan note (no incoming links)
vaultiel --vault "$VAULT" create "Orphan Note.md" \
    --content "# Orphan Note

This note has no incoming links. It's isolated.

#orphan"

# Note with duplicate block IDs
vaultiel --vault "$VAULT" create "Duplicate Blocks.md" \
    --content "# Duplicate Blocks

First block ^myblock

Some content here.

Second block with same ID ^myblock

This is a problem!

#test"

# Empty note
vaultiel --vault "$VAULT" create "Empty Note.md" \
    --content ""

# Note without frontmatter (body only, but file exists)
cat > "$VAULT/No Frontmatter.md" << 'EOF'
# No Frontmatter

This note has no YAML frontmatter block.

Just plain markdown content.
EOF

# Note with invalid frontmatter
cat > "$VAULT/Invalid Frontmatter.md" << 'EOF'
---
title: "Unclosed string
tags: [missing, bracket
---

# Invalid Frontmatter

The YAML above is malformed.
EOF

echo "Created notes with various issues"
echo

# --- Run Full Lint ---
echo "--- Full Lint Check ---"
vaultiel --vault "$VAULT" lint 2>/dev/null | jq '.'
echo

# --- Check Specific Issue Types ---
echo "--- Broken Links Only ---"
vaultiel --vault "$VAULT" lint --only broken-links 2>/dev/null | jq '.issues'
echo

echo "--- Orphan Notes Only ---"
vaultiel --vault "$VAULT" lint --only orphans 2>/dev/null | jq '.issues'
echo

echo "--- Duplicate Block IDs ---"
vaultiel --vault "$VAULT" lint --only duplicate-block-ids 2>/dev/null | jq '.issues'
echo

# --- Ignore Certain Issues ---
echo "--- Lint Ignoring Orphans ---"
vaultiel --vault "$VAULT" lint --ignore orphans 2>/dev/null | jq '.summary'
echo

# --- Scope to Specific Files ---
echo "--- Lint Only Test Files ---"
vaultiel --vault "$VAULT" lint --glob "*Broken*.md" 2>/dev/null | jq '.issues'
echo

# --- Shorthand Commands ---
echo "--- Find Orphans (shorthand) ---"
vaultiel --vault "$VAULT" find-orphans | jq '.'
echo

echo "--- Find Broken Links (shorthand) ---"
vaultiel --vault "$VAULT" find-broken-links | jq '.'
echo

# --- Auto-fix ---
echo "--- Auto-fix (dry run) ---"
# Note: --fix with --dry-run would show what would be fixed
# For this demo, we'll show the fixable issues
vaultiel --vault "$VAULT" lint 2>/dev/null | jq '.issues | map(select(.fixable == true))'
echo

echo "--- Apply Auto-fix ---"
vaultiel --vault "$VAULT" lint --fix 2>/dev/null | jq '.summary'
echo

echo "--- Verify After Fix ---"
vaultiel --vault "$VAULT" lint --only duplicate-block-ids --only missing-frontmatter 2>/dev/null | jq '.summary'
echo

# --- CI Mode ---
echo "--- CI Mode (GitHub Actions format) ---"
vaultiel --vault "$VAULT" lint --format github 2>/dev/null || true
echo

# --- Fail on Specific Issues ---
echo
echo "--- Fail on Broken Links (for CI) ---"
if vaultiel --vault "$VAULT" lint --fail-on broken-links 2>/dev/null; then
    echo "No broken links found"
else
    echo "Exit code: $? (broken links found)"
fi
echo

# --- Vault Info ---
echo "--- Vault Statistics ---"
vaultiel --vault "$VAULT" info | jq '.'
echo

echo "Demo vault created at: $VAULT"
echo "To clean up, run: rm -rf $VAULT"
