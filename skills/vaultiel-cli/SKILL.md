---
name: vaultiel-cli
description: Use the vaultiel CLI in this repository to inspect and modify Obsidian-style vault notes (list/resolve/content/frontmatter/links/tasks/metadata and write operations). Use when the user asks to operate on a vault via the vaultiel command line rather than writing Rust code.
---

# Vaultiel CLI Skill

Use this skill when the user wants to **use Vaultiel CLI commands**, not implement Vaultiel internals.

This skill reflects the current CLI defined in:
- `vaultiel-cli/src/main.rs`
- `vaultiel-cli/src/commands/*.rs`

## How to run from this repo

Preferred (from workspace root):

```bash
cd /path/to/vaultiel
cargo run -p vaultiel-cli -- --vault /absolute/path/to/vault <command> ...
```

If `vaultiel` is already installed and on PATH:

```bash
vaultiel --vault /absolute/path/to/vault <command> ...
```

Important: in this CLI, `--vault` is required (no implicit default vault in `main.rs`).

## Safety model

Read-only commands are generally safe to run without confirmation:
- `list`, `exists`, `resolve`, `content`, `body`, `frontmatter`, `inspect`
- `properties`, `property`, `search`, `all-frontmatter`
- `links`, `tags`, `headings`, `block-ids`, `tasks`, `task-trees`
- `incoming-links`, `outgoing-links`, `metadata`, `find-by-id`

Ask before mutating commands:
- `create`, `delete`, `rename`
- `set-content`, `set-raw-content`, `append`, `replace`
- `modify-frontmatter`, `remove-frontmatter`
- `set-task-symbol`
- `init-metadata` (writes vaultiel metadata)

There are no built-in dry-run flags in this current CLI implementation, so be explicit before writes.

## Command map (current CLI)

### Read

```bash
vaultiel --vault "$VAULT" list
vaultiel --vault "$VAULT" list --pattern "mod/*.md"
vaultiel --vault "$VAULT" exists "mod/My Note.md"
vaultiel --vault "$VAULT" resolve "My Note"
vaultiel --vault "$VAULT" content "mod/My Note.md"
vaultiel --vault "$VAULT" body "mod/My Note.md"
vaultiel --vault "$VAULT" frontmatter "mod/My Note.md"
vaultiel --vault "$VAULT" inspect "mod/My Note.md"
```

Properties:

```bash
vaultiel --vault "$VAULT" properties "mod/My Note.md"
vaultiel --vault "$VAULT" properties "mod/My Note.md" --frontmatter
vaultiel --vault "$VAULT" properties "mod/My Note.md" --inline
vaultiel --vault "$VAULT" property "mod/My Note.md" status
```

Search and bulk frontmatter:

```bash
vaultiel --vault "$VAULT" search "active module"
vaultiel --vault "$VAULT" all-frontmatter --pattern "*.md"
vaultiel --vault "$VAULT" all-frontmatter --has-key notetype
vaultiel --vault "$VAULT" all-frontmatter --where notetype=mod
```

`all-frontmatter` emits JSONL (one JSON object per line), useful with `jq`.

### Parse

```bash
vaultiel --vault "$VAULT" links "mod/My Note.md"
vaultiel --vault "$VAULT" tags "mod/My Note.md"
vaultiel --vault "$VAULT" headings "mod/My Note.md"
vaultiel --vault "$VAULT" block-ids "mod/My Note.md"
vaultiel --vault "$VAULT" tasks "mod/My Note.md"
vaultiel --vault "$VAULT" tasks "mod/My Note.md" --links-to "mod/Target.md"
vaultiel --vault "$VAULT" task-trees "mod/My Note.md"
```

### Graph

```bash
vaultiel --vault "$VAULT" incoming-links "mod/My Note.md"
vaultiel --vault "$VAULT" outgoing-links "mod/My Note.md"
```

### Write

```bash
vaultiel --vault "$VAULT" create "scratch/Test.md" "# Title"
vaultiel --vault "$VAULT" create "scratch/Test.md" - < body.md
vaultiel --vault "$VAULT" delete "scratch/Test.md"
vaultiel --vault "$VAULT" rename "scratch/Old.md" "scratch/New.md"

vaultiel --vault "$VAULT" set-content "mod/My Note.md" "new body"
vaultiel --vault "$VAULT" set-raw-content "mod/My Note.md" "---\nstatus: active\n---\nbody"
vaultiel --vault "$VAULT" append "mod/My Note.md" "\n- added line"
vaultiel --vault "$VAULT" replace "mod/My Note.md" "old" "new"

vaultiel --vault "$VAULT" modify-frontmatter "mod/My Note.md" status '"active"'
vaultiel --vault "$VAULT" modify-frontmatter "mod/My Note.md" tags '"new-tag"' --append
vaultiel --vault "$VAULT" remove-frontmatter "mod/My Note.md" obsolete_key

vaultiel --vault "$VAULT" set-task-symbol "mod/My Note.md" --line 42 --symbol x
```

For `create`, `set-content`, `set-raw-content`, and `append`, pass `-` as content to read from stdin.

### Metadata

```bash
vaultiel --vault "$VAULT" init-metadata "mod/My Note.md"
vaultiel --vault "$VAULT" init-metadata "mod/My Note.md" --force
vaultiel --vault "$VAULT" metadata "mod/My Note.md"
vaultiel --vault "$VAULT" find-by-id "<vaultiel-id>"
```

## Practical workflow

1. Start with `list` / `resolve` to identify exact paths.
2. Use `frontmatter`, `properties`, `tasks`, `links`, or `inspect` to gather context.
3. For batch discovery, use `all-frontmatter` + `jq`.
4. Confirm before any write command.
5. Re-read with `content`/`inspect` after writes.

## Reference files in this repo

From this skill directory, repository root is `../..`.

- `../../README.md`
- `../../vaultiel-cli/src/main.rs`
- `../../vaultiel-cli/src/commands/read.rs`
- `../../vaultiel-cli/src/commands/parse.rs`
- `../../vaultiel-cli/src/commands/graph.rs`
- `../../vaultiel-cli/src/commands/write.rs`
- `../../vaultiel-cli/src/commands/meta.rs`
