//! Inspect command — full JSON note representation.

use crate::cli::output::Output;
use crate::config::TaskConfig;
use crate::error::{ExitCode, Result};
use crate::graph::LinkGraph;
use crate::parser::task::parse_tasks;
use crate::types::{BlockId, Heading, InlineAttr, Link, Tag, Task};
use crate::vault::Vault;
use serde::Serialize;
use serde_json::Value as JsonValue;

/// Output for the inspect command.
#[derive(Debug, Serialize)]
pub struct InspectOutput {
    pub path: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontmatter: Option<JsonValue>,
    pub inline_attrs: Vec<InlineAttr>,
    pub headings: Vec<Heading>,
    pub tasks: Vec<Task>,
    pub links: InspectLinks,
    pub tags: Vec<Tag>,
    pub block_ids: Vec<BlockId>,
    pub stats: InspectStats,
}

/// Link information in inspect output.
#[derive(Debug, Serialize)]
pub struct InspectLinks {
    pub outgoing: Vec<Link>,
    pub incoming: Vec<IncomingLink>,
}

/// Simplified incoming link for inspect output.
#[derive(Debug, Serialize)]
pub struct IncomingLink {
    pub from: String,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Note statistics.
#[derive(Debug, Serialize)]
pub struct InspectStats {
    pub lines: usize,
    pub words: usize,
    pub size_bytes: u64,
}

/// Run the inspect command.
pub fn run(
    vault: &Vault,
    path: &str,
    no_content: bool,
    include_incoming: bool,
    output: &Output,
) -> Result<ExitCode> {
    let note_path = vault.resolve_note(path)?;
    let note = vault.load_note(&note_path)?;
    let info = vault.note_info(&note_path)?;

    // Parse frontmatter as JSON value
    let frontmatter = if let Some(yaml) = note.frontmatter()? {
        // Convert serde_yaml::Value to serde_json::Value
        let json_str = serde_json::to_string(&yaml).unwrap_or_default();
        serde_json::from_str(&json_str).ok()
    } else {
        None
    };

    // Parse tasks
    let task_config: TaskConfig = (&vault.config.tasks).into();
    let tasks = parse_tasks(&note.content, &note_path, &task_config);

    // Get outgoing links
    let outgoing = note.links();

    // Get incoming links (only when requested — building the link graph is expensive)
    let incoming: Vec<IncomingLink> = if include_incoming {
        let graph = LinkGraph::build(vault)?;
        let incoming_raw = graph.get_incoming(&note_path);
        incoming_raw
            .into_iter()
            .map(|l| IncomingLink {
                from: l.from.to_string_lossy().to_string(),
                line: l.link.line,
                context: Some(l.context.as_string()),
            })
            .collect()
    } else {
        Vec::new()
    };

    // Collect tags, headings, block_ids, inline_attrs
    let tags = note.tags();
    let headings = note.headings();
    let block_ids = note.block_ids();
    let inline_attrs = note.inline_attrs();

    // Stats
    let content = if no_content { note.body() } else { &note.content };
    let lines = content.lines().count();
    let words = content.split_whitespace().count();
    let size_bytes = info.size_bytes.unwrap_or(0);

    let result = InspectOutput {
        path: note_path.to_string_lossy().to_string(),
        name: note.name().to_string(),
        folder: note.folder().map(|f| f.to_string_lossy().to_string()),
        frontmatter,
        inline_attrs,
        headings,
        tasks,
        links: InspectLinks { outgoing, incoming },
        tags,
        block_ids,
        stats: InspectStats {
            lines,
            words,
            size_bytes,
        },
    };

    output.print(&result)?;
    Ok(ExitCode::Success)
}
