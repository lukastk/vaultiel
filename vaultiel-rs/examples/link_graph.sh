#!/bin/bash
# Link Graph Operations with Vaultiel CLI
# This script demonstrates how to work with links and the vault graph.

set -e

VAULT="${VAULT:-/tmp/vaultiel-links-demo}"

echo "=== Vaultiel CLI Demo: Link Graph ==="
echo "Using vault: $VAULT"
echo

# Create demo vault with interconnected notes
mkdir -p "$VAULT/projects" "$VAULT/people" "$VAULT/meetings"

# Create some interconnected notes
vaultiel --vault "$VAULT" create "projects/Project Alpha.md" \
    --frontmatter '{"status": "active", "lead": "[[people/Alice]]"}' \
    --content "# Project Alpha

A cutting-edge project led by [[people/Alice]].

## Team
- [[people/Alice]] - Lead
- [[people/Bob]] - Developer
- [[people/Charlie]] - Designer

## Meetings
- [[meetings/2024-01-15 Kickoff]]
- [[meetings/2024-01-22 Sprint Review]]

## Related
See also [[projects/Project Beta]] for the follow-up project.

#project #active"

vaultiel --vault "$VAULT" create "projects/Project Beta.md" \
    --frontmatter '{"status": "planning", "lead": "[[people/Bob]]"}' \
    --content "# Project Beta

Follow-up to [[projects/Project Alpha]].

## Team
- [[people/Bob]] - Lead
- [[people/Charlie]] - Developer

#project #planning"

vaultiel --vault "$VAULT" create "people/Alice.md" \
    --frontmatter '{"role": "Engineering Lead"}' \
    --content "# Alice

Engineering lead for [[projects/Project Alpha]].

#person #engineering"

vaultiel --vault "$VAULT" create "people/Bob.md" \
    --frontmatter '{"role": "Senior Developer"}' \
    --content "# Bob

Developer on [[projects/Project Alpha]], leading [[projects/Project Beta]].

#person #engineering"

vaultiel --vault "$VAULT" create "people/Charlie.md" \
    --frontmatter '{"role": "Designer"}' \
    --content "# Charlie

Designer working on multiple projects.

#person #design"

vaultiel --vault "$VAULT" create "meetings/2024-01-15 Kickoff.md" \
    --content "# Project Alpha Kickoff

Attendees: [[people/Alice]], [[people/Bob]], [[people/Charlie]]

## Notes
- Discussed project scope
- Set initial timeline

Related: [[projects/Project Alpha]]

#meeting"

vaultiel --vault "$VAULT" create "meetings/2024-01-22 Sprint Review.md" \
    --content "# Sprint Review

Attendees: [[people/Alice]], [[people/Bob]]

## Progress
- Completed initial setup
- Started core development

Related: [[projects/Project Alpha]]

#meeting"

echo "Created demo vault with interconnected notes"
echo

# --- Outgoing Links ---
echo "--- Outgoing Links from Project Alpha ---"
vaultiel --vault "$VAULT" get-out-links "projects/Project Alpha.md" | jq '.'

# --- Incoming Links (Backlinks) ---
echo
echo "--- Incoming Links to Alice ---"
vaultiel --vault "$VAULT" get-in-links "people/Alice.md" | jq '.'

# --- All Links ---
echo
echo "--- All Links for Bob (both directions) ---"
vaultiel --vault "$VAULT" get-links "people/Bob.md" | jq '.'

# --- Filter by Context ---
echo
echo "--- Links from Frontmatter Only (Project Alpha) ---"
vaultiel --vault "$VAULT" get-out-links "projects/Project Alpha.md" --context "frontmatter:*" | jq '.'

# --- Find Orphans ---
echo
echo "--- Orphan Notes (no incoming links) ---"
vaultiel --vault "$VAULT" find-orphans | jq '.'

# --- Broken Links ---
echo
echo "--- Check for Broken Links ---"
vaultiel --vault "$VAULT" find-broken-links | jq '.'

# --- Vault Info ---
echo
echo "--- Vault Statistics ---"
vaultiel --vault "$VAULT" info --detailed | jq '.'

echo
echo "Demo vault created at: $VAULT"
echo "To clean up, run: rm -rf $VAULT"
