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
