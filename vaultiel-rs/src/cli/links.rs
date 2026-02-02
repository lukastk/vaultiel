//! Link-related CLI commands.

use crate::cli::output::Output;
use crate::error::{ExitCode, Result};
use crate::graph::resolution::{get_media_type, is_media_target};
use crate::graph::{IncomingLink, LinkGraph, LinkInfo};
use crate::types::LinkContext;
use crate::vault::Vault;
use serde::Serialize;
use std::path::PathBuf;

/// Output for get-links command.
#[derive(Debug, Serialize)]
pub struct LinksOutput {
    pub incoming: Vec<IncomingLinkOutput>,
    pub outgoing: Vec<OutgoingLinkOutput>,
}

/// Output format for an incoming link.
#[derive(Debug, Serialize)]
pub struct IncomingLinkOutput {
    pub from: PathBuf,
    pub line: usize,
    pub context: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
    pub embed: bool,
}

/// Output format for an outgoing link.
#[derive(Debug, Serialize)]
pub struct OutgoingLinkOutput {
    pub to: String,
    pub line: usize,
    pub context: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
    pub embed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_path: Option<PathBuf>,
}

/// Output for get-embeds command.
#[derive(Debug, Serialize)]
pub struct EmbedsOutput {
    pub embeds: Vec<EmbedOutput>,
}

/// Output format for an embed.
#[derive(Debug, Serialize)]
pub struct EmbedOutput {
    pub target: String,
    pub line: usize,
    #[serde(rename = "type")]
    pub embed_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    pub context: String,
}

fn format_context(ctx: &LinkContext) -> String {
    ctx.as_string()
}

fn incoming_to_output(link: &IncomingLink) -> IncomingLinkOutput {
    IncomingLinkOutput {
        from: link.from.clone(),
        line: link.link.line,
        context: format_context(&link.context),
        alias: link.link.alias.clone(),
        heading: link.link.heading.clone(),
        block_id: link.link.block_id.clone(),
        embed: link.link.embed,
    }
}

fn outgoing_to_output(link: &LinkInfo) -> OutgoingLinkOutput {
    OutgoingLinkOutput {
        to: link.link.target.clone(),
        line: link.link.line,
        context: format_context(&link.context),
        alias: link.link.alias.clone(),
        heading: link.link.heading.clone(),
        block_id: link.link.block_id.clone(),
        embed: link.link.embed,
        resolved_path: link.resolved_path.clone(),
    }
}

/// Filter options for link queries.
#[derive(Debug, Default)]
pub struct LinkFilter {
    pub context: Option<String>,
    pub embeds_only: bool,
    pub no_embeds: bool,
    pub media_only: bool,
    pub notes_only: bool,
}

impl LinkFilter {
    pub fn matches_outgoing(&self, link: &LinkInfo) -> bool {
        // Embed filtering
        if self.embeds_only && !link.link.embed {
            return false;
        }
        if self.no_embeds && link.link.embed {
            return false;
        }

        // Media filtering
        if self.media_only {
            if !link.link.embed || !is_media_target(&link.link.target) {
                return false;
            }
        }
        if self.notes_only {
            if !link.link.embed || is_media_target(&link.link.target) {
                return false;
            }
        }

        // Context filtering
        if let Some(ref pattern) = self.context {
            let ctx = format_context(&link.context);
            if !context_matches(&ctx, pattern) {
                return false;
            }
        }

        true
    }

    pub fn matches_incoming(&self, link: &IncomingLink) -> bool {
        // Embed filtering
        if self.embeds_only && !link.link.embed {
            return false;
        }
        if self.no_embeds && link.link.embed {
            return false;
        }

        // Context filtering
        if let Some(ref pattern) = self.context {
            let ctx = format_context(&link.context);
            if !context_matches(&ctx, pattern) {
                return false;
            }
        }

        true
    }
}

/// Check if a context matches a pattern (supports wildcards).
fn context_matches(context: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        context.starts_with(prefix)
    } else if pattern.starts_with('*') {
        let suffix = &pattern[1..];
        context.ends_with(suffix)
    } else {
        context == pattern
    }
}

/// Execute get-links command.
pub fn get_links(
    vault: &Vault,
    path: &str,
    filter: LinkFilter,
    output: &Output,
) -> Result<ExitCode> {
    let note_path = vault.resolve_note(path)?;
    let graph = LinkGraph::build(vault)?;

    let incoming: Vec<_> = graph
        .get_incoming(&note_path)
        .into_iter()
        .filter(|l| filter.matches_incoming(l))
        .map(incoming_to_output)
        .collect();

    let outgoing: Vec<_> = graph
        .get_outgoing(&note_path)
        .into_iter()
        .filter(|l| filter.matches_outgoing(l))
        .map(outgoing_to_output)
        .collect();

    let result = LinksOutput { incoming, outgoing };
    output.print(&result)?;

    Ok(ExitCode::Success)
}

/// Execute get-in-links command.
pub fn get_in_links(
    vault: &Vault,
    path: &str,
    filter: LinkFilter,
    output: &Output,
) -> Result<ExitCode> {
    let note_path = vault.resolve_note(path)?;
    let graph = LinkGraph::build(vault)?;

    let incoming: Vec<_> = graph
        .get_incoming(&note_path)
        .into_iter()
        .filter(|l| filter.matches_incoming(l))
        .map(incoming_to_output)
        .collect();

    #[derive(Serialize)]
    struct IncomingOutput {
        incoming: Vec<IncomingLinkOutput>,
    }

    let result = IncomingOutput { incoming };
    output.print(&result)?;

    Ok(ExitCode::Success)
}

/// Execute get-out-links command.
pub fn get_out_links(
    vault: &Vault,
    path: &str,
    filter: LinkFilter,
    output: &Output,
) -> Result<ExitCode> {
    let note_path = vault.resolve_note(path)?;
    let graph = LinkGraph::build(vault)?;

    let outgoing: Vec<_> = graph
        .get_outgoing(&note_path)
        .into_iter()
        .filter(|l| filter.matches_outgoing(l))
        .map(outgoing_to_output)
        .collect();

    #[derive(Serialize)]
    struct OutgoingOutput {
        outgoing: Vec<OutgoingLinkOutput>,
    }

    let result = OutgoingOutput { outgoing };
    output.print(&result)?;

    Ok(ExitCode::Success)
}

/// Execute get-embeds command.
pub fn get_embeds(
    vault: &Vault,
    path: &str,
    media_only: bool,
    notes_only: bool,
    output: &Output,
) -> Result<ExitCode> {
    let note_path = vault.resolve_note(path)?;
    let graph = LinkGraph::build(vault)?;

    let embeds: Vec<_> = graph
        .get_outgoing(&note_path)
        .into_iter()
        .filter(|l| l.link.embed)
        .filter(|l| {
            if media_only {
                is_media_target(&l.link.target)
            } else if notes_only {
                !is_media_target(&l.link.target)
            } else {
                true
            }
        })
        .map(|l| {
            let embed_type = if is_media_target(&l.link.target) {
                get_media_type(&l.link.target).unwrap_or("media").to_string()
            } else {
                "note".to_string()
            };

            // Extract size from alias if it looks like dimensions
            let size = l.link.alias.as_ref().and_then(|a| {
                if a.chars().all(|c| c.is_ascii_digit() || c == 'x') {
                    Some(a.clone())
                } else {
                    None
                }
            });

            EmbedOutput {
                target: l.link.target.clone(),
                line: l.link.line,
                embed_type,
                heading: l.link.heading.clone(),
                block_id: l.link.block_id.clone(),
                size,
                context: format_context(&l.context),
            }
        })
        .collect();

    let result = EmbedsOutput { embeds };
    output.print(&result)?;

    Ok(ExitCode::Success)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_matches() {
        assert!(context_matches("body", "body"));
        assert!(context_matches("body", "*"));
        assert!(context_matches("frontmatter:parent", "frontmatter:*"));
        assert!(context_matches("frontmatter:parent", "frontmatter:parent"));
        assert!(!context_matches("body", "frontmatter:*"));
        assert!(!context_matches("frontmatter:links", "frontmatter:parent"));
    }
}
