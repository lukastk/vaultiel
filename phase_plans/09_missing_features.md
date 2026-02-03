# Phase 9: Missing Features Implementation

This plan covers features that were skipped during initial implementation of phases 1-8.

## Priority Order

1. **High Priority** - Core functionality gaps
2. **Medium Priority** - Quality of life improvements
3. **Low Priority** - Advanced features that can wait

---

## High Priority

### 1. Search Tag Filtering (Phase 2)

Add tag filtering to the `search` command.

**Files to modify:**
- `vaultiel-rs/src/cli/args.rs` - Add `--tag`, `--tag-any`, `--no-tag` flags to SearchArgs
- `vaultiel-rs/src/cli/search.rs` - Implement tag filtering logic

**Flags:**
- `--tag <TAG>` - Only notes with this tag (repeatable, AND logic)
- `--tag-any <TAG>` - Notes with any of these tags (repeatable, OR logic)
- `--no-tag <TAG>` - Exclude notes with this tag (repeatable)

**Implementation:**
1. Parse tags from each matching note
2. Apply tag filters before returning results
3. Tag matching should be case-insensitive and support nested tags (e.g., `--tag project` matches `#project/foo`)

---

### 2. Lint Auto-fix (Phase 4)

Add `--fix` flag to automatically fix certain issues.

**Files to modify:**
- `vaultiel-rs/src/cli/args.rs` - Already has `--fix` flag, need to implement
- `vaultiel-rs/src/cli/lint.rs` - Add fix logic for each fixable issue type
- `vaultiel-rs/src/health/mod.rs` - Add fix functions

**Fixable issues:**
- `empty_note` - Delete the file (with confirmation)
- `missing_frontmatter` - Add empty frontmatter block `---\n---\n`
- `duplicate_block_id` - Rename duplicates with suffix (`^id` → `^id-1`, `^id-2`)

**Non-fixable issues (report only):**
- `broken_link` - Can't know what the user intended
- `broken_embed` - Same reason
- `orphan` - User decision required
- `invalid_frontmatter` - Requires manual YAML fix

**Implementation:**
1. Add `Fixable` trait or method to health check results
2. For each fixable issue, implement a fix function
3. In `--fix` mode, apply fixes and report what was changed
4. Respect `--dry-run` to show what would be fixed

---

## Medium Priority

### 3. Cache Integration (Phase 5)

Make caching transparent for read operations.

**Files to modify:**
- `vaultiel-rs/src/vault.rs` - Add cache-aware methods
- `vaultiel-rs/src/cache/index.rs` - Add query methods
- Various CLI commands - Use cache when available

**Implementation:**
1. Add `--trust-cache` global flag to skip mtime checks
2. Add `cache.auto_threshold` config check on vault open
3. Create `CachedVault` wrapper or add cache field to `Vault`
4. For commands like `get-links`, `get-tags`, `get-tasks` - use cached data when available

**Order:**
1. First: Add `--trust-cache` flag
2. Then: Integrate cache into `list`, `get-links`, `get-tags`, `get-tasks`
3. Finally: Auto-enable for large vaults

---

### 4. Vaultiel Metadata Preservation (Phase 6)

Ensure vaultiel field is preserved during operations.

**Files to modify:**
- `vaultiel-rs/src/cli/content.rs` - Preserve vaultiel field in set-content
- `vaultiel-rs/src/cli/frontmatter.rs` - Preserve vaultiel field in modify/remove
- `vaultiel-rs/src/cli/rename.rs` - Ensure ID stable across renames

**Implementation:**
1. When modifying frontmatter, always preserve `vaultiel` key
2. Add `--include-vaultiel-field` flag to `get-content` (currently excluded by default)
3. Document that vaultiel.id remains stable across renames (already true since it's in the file)

---

### 5. Rename Frontmatter Key (Phase 2)

Add command to rename a frontmatter key across multiple notes.

**Files to modify:**
- `vaultiel-rs/src/cli/args.rs` - Add `RenameFrontmatterArgs`
- `vaultiel-rs/src/cli/frontmatter.rs` - Add `rename_frontmatter` function
- `vaultiel-rs/src/main.rs` - Add command handler

**Command:**
```
vaultiel rename-frontmatter --from <OLD_KEY> --to <NEW_KEY> [--glob <PATTERN>] [--dry-run]
```

**Implementation:**
1. Find all notes (or those matching glob)
2. For each note with the old key, rename it to new key
3. Report changes made

---

## Low Priority

### 6. Documentation and Examples (Phase 7)

Create documentation for TypeScript and Python bindings.

**Files to create:**
- `vaultiel-node/examples/` - Example scripts
- `vaultiel-py/examples/` - Example scripts
- Update READMEs with more examples

**Examples to include:**
- Basic vault operations (list, read, create)
- Link graph traversal
- Task querying
- Frontmatter manipulation

---

### 7. Incremental Graph Export (Phase 8)

Add diff-based export for graph databases.

**Files to modify:**
- `vaultiel-rs/src/export/mod.rs` - Add incremental export support
- `vaultiel-rs/src/cli/export.rs` - Add `--since` flag

**Implementation:**
1. Track last export timestamp
2. Only export nodes/relationships for changed notes
3. Generate DELETE statements for removed notes/links

---

## Implementation Order

1. Search tag filtering (quick win, commonly needed)
2. Lint auto-fix (useful for automation)
3. Rename frontmatter key (completes Phase 2)
4. Vaultiel metadata preservation (completes Phase 6)
5. Cache integration (performance improvement)
6. Documentation (can be done incrementally)
7. Incremental export (low priority)

---

## Not Planned

The following Phase 8 features are deferred indefinitely:
- **Templating** - Complex feature, unclear requirements
- **Sub-vault Support** - Significant architectural changes needed

These can be revisited based on user demand.
