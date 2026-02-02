//! CLI argument definitions using clap.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "vaultiel")]
#[command(author, version, about = "A CLI for Obsidian-style vaults", long_about = None)]
pub struct Cli {
    /// Path to the vault (overrides config default)
    #[arg(long, global = true)]
    pub vault: Option<PathBuf>,

    /// Output as JSON (default)
    #[arg(long, global = true, conflicts_with_all = ["yaml", "toml"])]
    pub json: bool,

    /// Output as YAML
    #[arg(long, global = true, conflicts_with_all = ["json", "toml"])]
    pub yaml: bool,

    /// Output as TOML
    #[arg(long, global = true, conflicts_with_all = ["json", "yaml"])]
    pub toml: bool,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Increase output verbosity (can be repeated)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn output_format(&self) -> OutputFormat {
        if self.yaml {
            OutputFormat::Yaml
        } else if self.toml {
            OutputFormat::Toml
        } else {
            OutputFormat::Json
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Json,
    Yaml,
    Toml,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List notes in the vault
    List(ListArgs),

    /// Create a new note
    Create(CreateArgs),

    /// Delete a note
    Delete(DeleteArgs),

    /// Search for notes
    Search(SearchArgs),

    /// Resolve a note name or alias to a path
    Resolve(ResolveArgs),

    /// Get note content
    #[command(name = "get-content")]
    GetContent(GetContentArgs),

    /// Set note content
    #[command(name = "set-content")]
    SetContent(SetContentArgs),

    /// Append content to a note
    #[command(name = "append-content")]
    AppendContent(AppendContentArgs),

    /// Prepend content to a note (after frontmatter)
    #[command(name = "prepend-content")]
    PrependContent(PrependContentArgs),

    /// Replace content in a note
    #[command(name = "replace-content")]
    ReplaceContent(ReplaceContentArgs),

    /// Get note frontmatter
    #[command(name = "get-frontmatter")]
    GetFrontmatter(GetFrontmatterArgs),

    /// Modify frontmatter fields
    #[command(name = "modify-frontmatter")]
    ModifyFrontmatter(ModifyFrontmatterArgs),

    /// Remove a frontmatter field
    #[command(name = "remove-frontmatter")]
    RemoveFrontmatter(RemoveFrontmatterArgs),

    // === Phase 2: Links, Tags, Blocks ===

    /// Get all links (incoming and outgoing) for a note
    #[command(name = "get-links")]
    GetLinks(GetLinksArgs),

    /// Get incoming links to a note
    #[command(name = "get-in-links")]
    GetInLinks(GetLinksArgs),

    /// Get outgoing links from a note
    #[command(name = "get-out-links")]
    GetOutLinks(GetLinksArgs),

    /// Get embeds in a note
    #[command(name = "get-embeds")]
    GetEmbeds(GetEmbedsArgs),

    /// Get tags from a note or vault
    #[command(name = "get-tags")]
    GetTags(GetTagsArgs),

    /// Get block IDs in a note
    #[command(name = "get-blocks")]
    GetBlocks(GetBlocksArgs),

    /// Get references to blocks in a note
    #[command(name = "get-block-refs")]
    GetBlockRefs(GetBlockRefsArgs),

    /// Get headings in a note
    #[command(name = "get-headings")]
    GetHeadings(GetHeadingsArgs),

    /// Get section content from a note
    #[command(name = "get-section")]
    GetSection(GetSectionArgs),

    /// Rename a note with link propagation
    Rename(RenameArgs),

    // === Phase 3: Tasks ===

    /// Get tasks from notes
    #[command(name = "get-tasks")]
    GetTasks(GetTasksArgs),

    /// Format a task string for Obsidian
    #[command(name = "format-task")]
    FormatTask(FormatTaskArgs),

    // === Phase 4: Vault Health & Info ===

    /// Display vault information and statistics
    Info(InfoArgs),

    /// Check vault health and report issues
    Lint(LintArgs),

    /// Find notes with no incoming links
    #[command(name = "find-orphans")]
    FindOrphans(FindOrphansArgs),

    /// Find broken links in the vault
    #[command(name = "find-broken-links")]
    FindBrokenLinks(FindBrokenLinksArgs),

    // === Phase 5: Caching ===

    /// Cache management commands
    Cache(CacheArgs),

    // === Phase 6: Metadata & IDs ===

    /// Initialize vaultiel metadata for notes
    #[command(name = "init-metadata")]
    InitMetadata(InitMetadataArgs),

    /// Find a note by its vaultiel ID
    #[command(name = "get-by-id")]
    GetById(GetByIdArgs),

    /// Get vaultiel metadata from a note
    #[command(name = "get-metadata")]
    GetMetadata(GetMetadataArgs),
}

// === List ===

#[derive(Parser, Debug)]
pub struct ListArgs {
    /// Filter to notes matching glob pattern
    #[arg(long)]
    pub glob: Option<String>,

    /// Filter by tag (repeatable for AND logic)
    #[arg(long)]
    pub tag: Vec<String>,

    /// Filter by frontmatter field (KEY=VALUE)
    #[arg(long)]
    pub frontmatter: Vec<String>,

    /// Only notes with outgoing links
    #[arg(long)]
    pub has_links: bool,

    /// Only notes with incoming links
    #[arg(long)]
    pub has_backlinks: bool,

    /// Only notes with no incoming links
    #[arg(long)]
    pub orphans: bool,

    /// Sort by field
    #[arg(long, value_enum, default_value = "path")]
    pub sort: SortField,

    /// Reverse sort order
    #[arg(long)]
    pub reverse: bool,

    /// Limit number of results
    #[arg(long)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum SortField {
    #[default]
    Path,
    Modified,
    Created,
    Name,
}

// === Create ===

#[derive(Parser, Debug)]
pub struct CreateArgs {
    /// Path for the new note (relative to vault)
    pub path: String,

    /// Initial frontmatter as JSON
    #[arg(long)]
    pub frontmatter: Option<String>,

    /// Initial content (below frontmatter)
    #[arg(long)]
    pub content: Option<String>,

    /// Open in Obsidian after creation
    #[arg(long)]
    pub open: bool,

    /// Show what would be created without writing
    #[arg(long)]
    pub dry_run: bool,
}

// === Delete ===

#[derive(Parser, Debug)]
pub struct DeleteArgs {
    /// Path to the note to delete
    pub path: String,

    /// Just delete the file, don't check for incoming links
    #[arg(long)]
    pub no_propagate: bool,

    /// Remove all links to this note from other files
    #[arg(long)]
    pub remove_links: bool,

    /// Skip confirmation prompt
    #[arg(long)]
    pub force: bool,

    /// Show what would be deleted without making changes
    #[arg(long)]
    pub dry_run: bool,
}

// === Search ===

#[derive(Parser, Debug)]
pub struct SearchArgs {
    /// Search query
    pub query: String,

    /// Number of results
    #[arg(long, default_value = "10")]
    pub limit: usize,

    /// Search algorithm
    #[arg(long, value_enum, default_value = "subsequence")]
    pub mode: SearchMode,

    /// Search note content (not just titles/paths)
    #[arg(long)]
    pub content: bool,

    /// Filter by frontmatter field (KEY=VALUE)
    #[arg(long)]
    pub frontmatter: Vec<String>,

    /// Filter by tag
    #[arg(long)]
    pub tag: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum SearchMode {
    #[default]
    Subsequence,
    Fuzzy,
    Exact,
    Regex,
}

// === Resolve ===

#[derive(Parser, Debug)]
pub struct ResolveArgs {
    /// Note name or alias to resolve
    pub query: String,

    /// Return all matches instead of erroring on ambiguity
    #[arg(long)]
    pub all: bool,

    /// Only match exact paths, not aliases or partial names
    #[arg(long)]
    pub strict: bool,
}

// === Content Operations ===

#[derive(Parser, Debug)]
pub struct GetContentArgs {
    /// Path to the note
    pub path: String,

    /// Include YAML frontmatter block
    #[arg(long)]
    pub include_frontmatter: bool,

    /// Include the vaultiel metadata field
    #[arg(long)]
    pub include_vaultiel_field: bool,
}

#[derive(Parser, Debug)]
pub struct SetContentArgs {
    /// Path to the note
    pub path: String,

    /// Content to set
    #[arg(long)]
    pub content: Option<String>,

    /// Read content from file
    #[arg(long)]
    pub file: Option<PathBuf>,

    /// Only replace content below frontmatter
    #[arg(long)]
    pub below_frontmatter: bool,

    /// Only replace frontmatter
    #[arg(long)]
    pub frontmatter_only: bool,

    /// Show what would change without writing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Parser, Debug)]
pub struct AppendContentArgs {
    /// Path to the note
    pub path: String,

    /// Content to append
    #[arg(long)]
    pub content: Option<String>,

    /// Read content from file
    #[arg(long)]
    pub file: Option<PathBuf>,

    /// Show what would change without writing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Parser, Debug)]
pub struct PrependContentArgs {
    /// Path to the note
    pub path: String,

    /// Content to prepend
    #[arg(long)]
    pub content: Option<String>,

    /// Read content from file
    #[arg(long)]
    pub file: Option<PathBuf>,

    /// Show what would change without writing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Parser, Debug)]
pub struct ReplaceContentArgs {
    /// Path to the note
    pub path: String,

    /// Replace section under heading
    #[arg(long)]
    pub section: Option<String>,

    /// Replace first match of regex pattern
    #[arg(long)]
    pub pattern: Option<String>,

    /// Replace all matches of regex pattern
    #[arg(long)]
    pub pattern_all: Option<String>,

    /// Replace line range (e.g., "10-15")
    #[arg(long)]
    pub lines: Option<String>,

    /// Replace block with given ID
    #[arg(long)]
    pub block: Option<String>,

    /// Replacement content
    #[arg(long)]
    pub content: Option<String>,

    /// Read replacement content from file
    #[arg(long)]
    pub file: Option<PathBuf>,

    /// Show what would change without writing
    #[arg(long)]
    pub dry_run: bool,
}

// === Frontmatter Operations ===

#[derive(Parser, Debug)]
pub struct GetFrontmatterArgs {
    /// Path to the note
    pub path: String,

    /// Output format (overrides global)
    #[arg(long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Exclude inline attributes
    #[arg(long)]
    pub no_inline: bool,

    /// Get specific key only
    #[arg(long, short)]
    pub key: Option<String>,
}

#[derive(Parser, Debug)]
pub struct ModifyFrontmatterArgs {
    /// Path to the note
    pub path: String,

    /// Key to modify
    #[arg(short, long)]
    pub key: String,

    /// Value to set
    #[arg(short, long)]
    pub value: Option<String>,

    /// Add value to list
    #[arg(long = "add")]
    pub add_value: Option<String>,

    /// Remove value from list
    #[arg(long = "remove")]
    pub remove_value: Option<String>,

    /// Show what would change without writing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Parser, Debug)]
pub struct RemoveFrontmatterArgs {
    /// Path to the note
    pub path: String,

    /// Key to remove
    #[arg(short, long)]
    pub key: String,

    /// Show what would change without writing
    #[arg(long)]
    pub dry_run: bool,
}

// === Link Operations ===

#[derive(Parser, Debug)]
pub struct GetLinksArgs {
    /// Path to the note
    pub path: String,

    /// Filter by context (supports wildcards, e.g., "frontmatter:*")
    #[arg(long)]
    pub context: Option<String>,

    /// Only show embeds
    #[arg(long)]
    pub embeds_only: bool,

    /// Exclude embeds
    #[arg(long)]
    pub no_embeds: bool,

    /// Only media embeds (images, audio, video, PDF)
    #[arg(long)]
    pub media_only: bool,
}

#[derive(Parser, Debug)]
pub struct GetEmbedsArgs {
    /// Path to the note
    pub path: String,

    /// Only media embeds (images, audio, video, PDF)
    #[arg(long)]
    pub media_only: bool,

    /// Only note embeds
    #[arg(long)]
    pub notes_only: bool,
}

// === Tag Operations ===

#[derive(Parser, Debug)]
pub struct GetTagsArgs {
    /// Path to a specific note (omit for vault-wide)
    pub path: Option<String>,

    /// Include usage counts
    #[arg(long)]
    pub with_counts: bool,

    /// Return as nested hierarchy
    #[arg(long)]
    pub nested: bool,

    /// Filter to notes matching glob pattern
    #[arg(long)]
    pub glob: Option<String>,
}

// === Block Operations ===

#[derive(Parser, Debug)]
pub struct GetBlocksArgs {
    /// Path to the note
    pub path: String,
}

#[derive(Parser, Debug)]
pub struct GetBlockRefsArgs {
    /// Path to the note
    pub path: String,
}

// === Heading Operations ===

#[derive(Parser, Debug)]
pub struct GetHeadingsArgs {
    /// Path to the note
    pub path: String,

    /// Minimum heading level (1-6)
    #[arg(long)]
    pub min_level: Option<u8>,

    /// Maximum heading level (1-6)
    #[arg(long)]
    pub max_level: Option<u8>,

    /// Return as nested hierarchy
    #[arg(long)]
    pub nested: bool,
}

#[derive(Parser, Debug)]
pub struct GetSectionArgs {
    /// Path to the note
    pub path: String,

    /// Heading to find (e.g., "## Configuration" or just "Configuration")
    pub heading: String,

    /// Find heading by slug instead of text
    #[arg(long)]
    pub by_slug: bool,

    /// Include subheadings in output (default: true)
    #[arg(long, default_value = "true")]
    pub include_subheadings: bool,

    /// Exclude subheadings
    #[arg(long)]
    pub exclude_subheadings: bool,

    /// Exclude the heading line itself
    #[arg(long)]
    pub content_only: bool,
}

// === Rename Operations ===

#[derive(Parser, Debug)]
pub struct RenameArgs {
    /// Current note path
    pub from: String,

    /// New note path
    pub to: String,

    /// Don't update links in other notes
    #[arg(long)]
    pub no_propagate: bool,

    /// Show what would change without writing
    #[arg(long)]
    pub dry_run: bool,
}

// === Task Operations ===

#[derive(Parser, Debug)]
pub struct GetTasksArgs {
    /// Filter to tasks in specific note
    #[arg(long)]
    pub note: Option<String>,

    /// Filter to tasks in notes matching glob
    #[arg(long)]
    pub glob: Option<String>,

    /// Filter by task symbol (repeatable, e.g., --symbol "[ ]" --symbol "[x]")
    #[arg(long)]
    pub symbol: Vec<String>,

    /// Due date before (exclusive)
    #[arg(long)]
    pub due_before: Option<String>,

    /// Due date after (exclusive)
    #[arg(long)]
    pub due_after: Option<String>,

    /// Due on specific date
    #[arg(long)]
    pub due_on: Option<String>,

    /// Scheduled date before (exclusive)
    #[arg(long)]
    pub scheduled_before: Option<String>,

    /// Scheduled date after (exclusive)
    #[arg(long)]
    pub scheduled_after: Option<String>,

    /// Scheduled on specific date
    #[arg(long)]
    pub scheduled_on: Option<String>,

    /// Done date before (exclusive)
    #[arg(long)]
    pub done_before: Option<String>,

    /// Done date after (exclusive)
    #[arg(long)]
    pub done_after: Option<String>,

    /// Done on specific date
    #[arg(long)]
    pub done_on: Option<String>,

    /// Filter by priority (highest, high, medium, low, lowest)
    #[arg(long)]
    pub priority: Option<String>,

    /// Filter by description text
    #[arg(long)]
    pub contains: Option<String>,

    /// Filter by custom metadata presence (repeatable)
    #[arg(long = "has")]
    pub has_metadata: Vec<String>,

    /// Filter to tasks linking to a note
    #[arg(long)]
    pub links_to: Option<String>,

    /// Filter by tag
    #[arg(long)]
    pub tag: Option<String>,

    /// Filter to tasks with block references
    #[arg(long)]
    pub has_block_ref: bool,

    /// Filter by specific block reference
    #[arg(long)]
    pub block_ref: Option<String>,

    /// Return flat list instead of hierarchy
    #[arg(long)]
    pub flat: bool,
}

#[derive(Parser, Debug)]
pub struct FormatTaskArgs {
    /// Task description
    #[arg(long)]
    pub desc: String,

    /// Task symbol (default: "[ ]")
    #[arg(long, default_value = "[ ]")]
    pub symbol: String,

    /// Due date (ISO date or relative: today, tomorrow, +3d)
    #[arg(long)]
    pub due: Option<String>,

    /// Scheduled date (ISO date or relative)
    #[arg(long)]
    pub scheduled: Option<String>,

    /// Done date (ISO date or relative)
    #[arg(long)]
    pub done: Option<String>,

    /// Priority level (highest, high, medium, low, lowest)
    #[arg(long)]
    pub priority: Option<String>,

    /// Custom metadata in KEY=VALUE format (repeatable)
    #[arg(long = "custom")]
    pub custom_metadata: Vec<String>,
}

// === Vault Health & Info ===

#[derive(Parser, Debug)]
pub struct InfoArgs {
    /// Include extended statistics
    #[arg(long)]
    pub detailed: bool,
}

#[derive(Parser, Debug)]
pub struct LintArgs {
    /// Auto-fix issues where possible
    #[arg(long)]
    pub fix: bool,

    /// Only check specific issue type (repeatable)
    #[arg(long)]
    pub only: Vec<String>,

    /// Skip specific issue type (repeatable)
    #[arg(long)]
    pub ignore: Vec<String>,

    /// Check only notes matching pattern
    #[arg(long)]
    pub glob: Option<String>,

    /// Exit non-zero if type found (repeatable, for CI)
    #[arg(long)]
    pub fail_on: Vec<String>,

    /// Output format: json, text, github
    #[arg(long, default_value = "json")]
    pub format: String,
}

#[derive(Parser, Debug)]
pub struct FindOrphansArgs {
    /// Exclude notes matching pattern (repeatable)
    #[arg(long)]
    pub exclude: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct FindBrokenLinksArgs {
    /// Check specific note only
    #[arg(long)]
    pub note: Option<String>,
}

// === Cache Operations ===

#[derive(Parser, Debug)]
pub struct CacheArgs {
    #[command(subcommand)]
    pub command: CacheCommands,
}

#[derive(Subcommand, Debug)]
pub enum CacheCommands {
    /// Show cache status and statistics
    Status,

    /// Force a full cache rebuild
    Rebuild(CacheRebuildArgs),

    /// Clear the cache entirely
    Clear,
}

#[derive(Parser, Debug)]
pub struct CacheRebuildArgs {
    /// Show progress during rebuild
    #[arg(long)]
    pub progress: bool,
}

// === Metadata Operations ===

#[derive(Parser, Debug)]
pub struct InitMetadataArgs {
    /// Path to the note (omit to use --glob)
    pub path: Option<String>,

    /// Initialize metadata for notes matching glob pattern
    #[arg(long)]
    pub glob: Option<String>,

    /// Force re-initialization even if metadata exists
    #[arg(long)]
    pub force: bool,

    /// Show what would change without writing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Parser, Debug)]
pub struct GetByIdArgs {
    /// The vaultiel UUID to search for
    pub id: String,
}

#[derive(Parser, Debug)]
pub struct GetMetadataArgs {
    /// Path to the note
    pub path: String,
}
