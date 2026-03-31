use std::io::{self, Read};
use std::process;

use clap::{Parser, Subcommand};
use vaultiel::Vault;

mod commands;

#[derive(Parser)]
#[command(name = "vaultiel", about = "Fast CLI for Obsidian-style vault operations")]
struct Cli {
    /// Path to the vault root directory
    #[arg(long)]
    vault: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // --- Read ---
    /// List notes in the vault
    List {
        /// Glob pattern to filter notes
        #[arg(long)]
        pattern: Option<String>,
    },
    /// Check if a note exists
    Exists {
        /// Note path
        note: String,
    },
    /// Resolve a note query to a path
    Resolve {
        /// Note query (path, name, or alias)
        query: String,
    },
    /// Get full content of a note
    Content {
        /// Note path
        note: String,
    },
    /// Get body of a note (without frontmatter)
    Body {
        /// Note path
        note: String,
    },
    /// Get frontmatter as JSON
    Frontmatter {
        /// Note path
        note: String,
    },
    /// Full note inspection (frontmatter, links, tags, tasks, headings, stats)
    Inspect {
        /// Note path
        note: String,
    },
    /// Get properties (frontmatter + inline merged)
    Properties {
        /// Note path
        note: String,
        /// Only inline properties
        #[arg(long)]
        inline: bool,
        /// Only frontmatter properties
        #[arg(long)]
        frontmatter: bool,
    },
    /// Get a single property value
    Property {
        /// Note path
        note: String,
        /// Property key
        key: String,
        /// Only inline
        #[arg(long)]
        inline: bool,
        /// Only frontmatter
        #[arg(long)]
        frontmatter: bool,
    },
    /// Search notes
    Search {
        /// Search query
        query: String,
    },
    /// Bulk frontmatter dump as JSONL (one JSON object per line)
    AllFrontmatter {
        /// Glob pattern to filter notes
        #[arg(long)]
        pattern: Option<String>,
        /// Only include notes that have this frontmatter key
        #[arg(long)]
        has_key: Option<String>,
        /// Filter: key=value (only include notes where frontmatter key equals value)
        #[arg(long, name = "KEY=VALUE")]
        r#where: Option<String>,
    },

    // --- Parse ---
    /// Get links from a note
    Links {
        /// Note path
        note: String,
    },
    /// Get tags from a note
    Tags {
        /// Note path
        note: String,
    },
    /// Get headings from a note
    Headings {
        /// Note path
        note: String,
    },
    /// Get block IDs from a note
    BlockIds {
        /// Note path
        note: String,
    },
    /// Get tasks from a note
    Tasks {
        /// Note path
        note: String,
        /// Filter tasks linking to a target
        #[arg(long)]
        links_to: Option<String>,
    },
    /// Get hierarchical task trees from a note
    TaskTrees {
        /// Note path
        note: String,
    },

    // --- Graph ---
    /// Get incoming links to a note
    IncomingLinks {
        /// Note path
        note: String,
    },
    /// Get outgoing links from a note
    OutgoingLinks {
        /// Note path
        note: String,
    },

    // --- Write ---
    /// Create a new note
    Create {
        /// Note path
        note: String,
        /// Content (use "-" to read from stdin)
        content: String,
    },
    /// Delete a note
    Delete {
        /// Note path
        note: String,
    },
    /// Rename a note
    Rename {
        /// Source path
        from: String,
        /// Destination path
        to: String,
    },
    /// Set note body content
    SetContent {
        /// Note path
        note: String,
        /// New content (use "-" to read from stdin)
        content: String,
    },
    /// Set raw note content (including frontmatter)
    SetRawContent {
        /// Note path
        note: String,
        /// New content (use "-" to read from stdin)
        content: String,
    },
    /// Modify a frontmatter key
    ModifyFrontmatter {
        /// Note path
        note: String,
        /// Frontmatter key
        key: String,
        /// Value (parsed as YAML)
        value: String,
        /// Append to list instead of replacing
        #[arg(long)]
        append: bool,
    },
    /// Append content to a note
    Append {
        /// Note path
        note: String,
        /// Content to append (use "-" to read from stdin)
        content: String,
    },
    /// Replace text in a note
    Replace {
        /// Note path
        note: String,
        /// Pattern to find
        pattern: String,
        /// Replacement text
        replacement: String,
    },
    /// Set a task's checkbox symbol
    SetTaskSymbol {
        /// Note path
        note: String,
        /// Line number (1-indexed)
        #[arg(long)]
        line: usize,
        /// New symbol character
        #[arg(long)]
        symbol: char,
    },
    /// Remove a frontmatter key
    RemoveFrontmatter {
        /// Note path
        note: String,
        /// Key to remove
        key: String,
    },

    // --- Metadata ---
    /// Initialize vaultiel metadata on a note
    InitMetadata {
        /// Note path
        note: String,
        /// Overwrite existing metadata
        #[arg(long)]
        force: bool,
    },
    /// Get vaultiel metadata from a note
    Metadata {
        /// Note path
        note: String,
    },
    /// Find a note by its vaultiel ID
    FindById {
        /// Vaultiel note ID
        id: String,
    },
}

/// Read content from stdin if the value is "-", otherwise return as-is.
fn resolve_content(value: &str) -> String {
    if value == "-" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).expect("Failed to read from stdin");
        buf
    } else {
        value.to_string()
    }
}

fn main() {
    let cli = Cli::parse();

    let vault = match Vault::new(&cli.vault) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error opening vault: {e}");
            process::exit(1);
        }
    };

    let result = match cli.command {
        // Read
        Commands::List { pattern } => commands::read::list(&vault, pattern.as_deref()),
        Commands::Exists { note } => commands::read::exists(&vault, &note),
        Commands::Resolve { query } => commands::read::resolve(&vault, &query),
        Commands::Content { note } => commands::read::content(&vault, &note),
        Commands::Body { note } => commands::read::body(&vault, &note),
        Commands::Frontmatter { note } => commands::read::frontmatter(&vault, &note),
        Commands::Inspect { note } => commands::read::inspect(&vault, &note),
        Commands::Properties { note, inline, frontmatter } => {
            commands::read::properties(&vault, &note, inline, frontmatter)
        }
        Commands::Property { note, key, inline, frontmatter } => {
            commands::read::property(&vault, &note, &key, inline, frontmatter)
        }
        Commands::Search { query } => commands::read::search(&vault, &query),
        Commands::AllFrontmatter { pattern, has_key, r#where } => {
            commands::read::all_frontmatter(&vault, pattern.as_deref(), has_key.as_deref(), r#where.as_deref())
        }

        // Parse
        Commands::Links { note } => commands::parse::links(&vault, &note),
        Commands::Tags { note } => commands::parse::tags(&vault, &note),
        Commands::Headings { note } => commands::parse::headings(&vault, &note),
        Commands::BlockIds { note } => commands::parse::block_ids(&vault, &note),
        Commands::Tasks { note, links_to } => {
            commands::parse::tasks(&vault, &note, links_to.as_deref())
        }
        Commands::TaskTrees { note } => commands::parse::task_trees(&vault, &note),

        // Graph
        Commands::IncomingLinks { note } => commands::graph::incoming_links(&vault, &note),
        Commands::OutgoingLinks { note } => commands::graph::outgoing_links(&vault, &note),

        // Write
        Commands::Create { note, content } => {
            commands::write::create(&vault, &note, &resolve_content(&content))
        }
        Commands::Delete { note } => commands::write::delete(&vault, &note),
        Commands::Rename { from, to } => commands::write::rename(&vault, &from, &to),
        Commands::SetContent { note, content } => {
            commands::write::set_content(&vault, &note, &resolve_content(&content))
        }
        Commands::SetRawContent { note, content } => {
            commands::write::set_raw_content(&vault, &note, &resolve_content(&content))
        }
        Commands::ModifyFrontmatter { note, key, value, append } => {
            commands::write::modify_frontmatter(&vault, &note, &key, &value, append)
        }
        Commands::Append { note, content } => {
            commands::write::append(&vault, &note, &resolve_content(&content))
        }
        Commands::Replace { note, pattern, replacement } => {
            commands::write::replace(&vault, &note, &pattern, &replacement)
        }
        Commands::SetTaskSymbol { note, line, symbol } => {
            commands::write::set_task_symbol(&vault, &note, line, symbol)
        }
        Commands::RemoveFrontmatter { note, key } => {
            commands::write::remove_frontmatter(&vault, &note, &key)
        }

        // Metadata
        Commands::InitMetadata { note, force } => commands::meta::init_metadata(&vault, &note, force),
        Commands::Metadata { note } => commands::meta::metadata_cmd(&vault, &note),
        Commands::FindById { id } => commands::meta::find_by_id(&vault, &id),
    };

    if let Err(e) = result {
        eprintln!("{e}");
        process::exit(1);
    }
}
