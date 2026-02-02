//! CLI commands for graph export.

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

use crate::cli::args::ExportFormat;
use crate::error::Result;
use crate::export::{export_cypher, export_jsonld, CypherOptions, JsonLdOptions};
use crate::Vault;

/// Export vault graph to the specified format.
pub fn export_graph(
    vault: &Vault,
    format: ExportFormat,
    output: Option<PathBuf>,
    include_tags: bool,
    include_headings: bool,
    include_frontmatter: bool,
    use_merge: bool,
    base_uri: Option<String>,
    pretty: bool,
) -> Result<()> {
    // Set up output writer
    let mut writer: Box<dyn Write> = match output {
        Some(path) => Box::new(BufWriter::new(File::create(&path)?)),
        None => Box::new(BufWriter::new(io::stdout())),
    };

    match format {
        ExportFormat::Cypher => {
            let options = CypherOptions {
                include_tags,
                include_headings,
                include_frontmatter,
                use_merge,
            };

            let stats = export_cypher(vault, &mut writer, &options)?;
            eprintln!(
                "Exported {} notes, {} links, {} tags, {} headings",
                stats.notes_created,
                stats.links_created,
                stats.tags_created,
                stats.headings_created
            );
        }
        ExportFormat::JsonLd => {
            let options = JsonLdOptions {
                include_tags,
                include_headings,
                include_frontmatter,
                base_uri,
                pretty,
            };

            let stats = export_jsonld(vault, &mut writer, &options)?;
            eprintln!(
                "Exported {} notes, {} links",
                stats.notes_created,
                stats.links_created
            );
        }
    }

    // Ensure output is flushed
    writer.flush()?;

    Ok(())
}
