#!/bin/bash
# Task Management with Vaultiel CLI
# This script demonstrates task extraction and filtering.

set -e

VAULT="${VAULT:-/tmp/vaultiel-tasks-demo}"
TODAY=$(date +%Y-%m-%d)
TOMORROW=$(date -v+1d +%Y-%m-%d 2>/dev/null || date -d "+1 day" +%Y-%m-%d)
NEXT_WEEK=$(date -v+7d +%Y-%m-%d 2>/dev/null || date -d "+7 days" +%Y-%m-%d)

echo "=== Vaultiel CLI Demo: Task Management ==="
echo "Using vault: $VAULT"
echo "Today: $TODAY"
echo

mkdir -p "$VAULT"

# Create notes with tasks
vaultiel --vault "$VAULT" create "Inbox.md" \
    --content "# Inbox

Capture tasks here for processing.

- [ ] Review pull request for [[Project Alpha]] 📅 $TODAY ⏫ #urgent
- [ ] Schedule meeting with team ⏳ $TOMORROW
- [ ] Read documentation 🔽
- [x] Set up development environment ✅ 2024-01-10

## Later
- [ ] Learn new framework 📅 $NEXT_WEEK
- [ ] Write blog post"

vaultiel --vault "$VAULT" create "Project Alpha.md" \
    --frontmatter '{"type": "project", "status": "active"}' \
    --content "# Project Alpha

## Tasks

- [ ] Implement feature A 📅 $TODAY ⏫ ^task-a
	- [ ] Write tests
	- [ ] Update documentation
- [ ] Implement feature B 📅 $TOMORROW 🔼
	- [ ] Design API
	- [ ] Code review
- [x] Initial setup ✅ 2024-01-08
- [>] Deferred task (moved to backlog)
- [-] Cancelled task

## Notes

See [[Inbox]] for incoming tasks.

#project #coding"

vaultiel --vault "$VAULT" create "Weekly Review.md" \
    --content "# Weekly Review

## This Week's Focus

- [ ] Complete [[Project Alpha]] feature A 📅 $TODAY ⏫
- [ ] Respond to emails ⏳ $TODAY
- [ ] 1:1 with manager ⏳ $TOMORROW

## Recurring

- [ ] Check metrics dashboard
- [ ] Update project status

#review #weekly"

echo "Created demo vault with tasks"
echo

# --- All Tasks ---
echo "--- All Tasks in Vault ---"
vaultiel --vault "$VAULT" get-tasks | jq 'length'
echo "total tasks found"
echo

# --- Incomplete Tasks ---
echo "--- Incomplete Tasks ([ ]) ---"
vaultiel --vault "$VAULT" get-tasks --symbol "[ ]" | jq '.[].description'
echo

# --- Tasks Due Today ---
echo "--- Tasks Due Today ---"
vaultiel --vault "$VAULT" get-tasks --due-on "$TODAY" | jq '.[] | {file: .file, description: .description, priority: .priority}'
echo

# --- High Priority Tasks ---
echo "--- High Priority Tasks ---"
vaultiel --vault "$VAULT" get-tasks --priority high | jq '.[] | {file: .file, description: .description}'
echo

# --- Tasks in Specific Note ---
echo "--- Tasks in Project Alpha ---"
vaultiel --vault "$VAULT" get-tasks --note "Project Alpha.md" | jq '.[] | {description: .description, symbol: .symbol}'
echo

# --- Tasks with Block References ---
echo "--- Tasks with Block IDs ---"
vaultiel --vault "$VAULT" get-tasks --has-block-ref | jq '.[] | {description: .description, block_id: .block_id}'
echo

# --- Tasks Linking to a Note ---
echo "--- Tasks Linking to Project Alpha ---"
vaultiel --vault "$VAULT" get-tasks --links-to "Project Alpha.md" | jq '.[] | {file: .file, description: .description}'
echo

# --- Scheduled Tasks ---
echo "--- Tasks Scheduled for Today ---"
vaultiel --vault "$VAULT" get-tasks --scheduled-on "$TODAY" | jq '.[] | {description: .description}'
echo

# --- Flat vs Hierarchical ---
echo "--- Hierarchical Task View (default) ---"
vaultiel --vault "$VAULT" get-tasks --note "Project Alpha.md" | jq '.[0] | {description: .description, children: [.children[].description]}'
echo

echo "--- Flat Task View ---"
vaultiel --vault "$VAULT" get-tasks --note "Project Alpha.md" --flat | jq '.[] | {description: .description, indent: .indent}'
echo

# --- Format a New Task ---
echo "--- Format a New Task ---"
vaultiel --vault "$VAULT" format-task \
    --desc "Complete quarterly report" \
    --due "$NEXT_WEEK" \
    --priority high \
    --scheduled "$TOMORROW"
echo

echo
echo "Demo vault created at: $VAULT"
echo "To clean up, run: rm -rf $VAULT"
