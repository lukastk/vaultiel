//! Block-related CLI commands.

use crate::cli::output::Output;
use crate::error::{ExitCode, Result};
use crate::graph::LinkGraph;
use crate::parser::parse_block_ids;
use crate::vault::Vault;
use serde::Serialize;
use std::path::PathBuf;

/// Output for get-blocks command.
#[derive(Debug, Serialize)]
pub struct BlocksOutput {
    pub blocks: Vec<BlockOutput>,
}

/// A block ID in a note.
#[derive(Debug, Serialize)]
pub struct BlockOutput {
    pub id: String,
    pub line: usize,
    #[serde(rename = "type")]
    pub block_type: String,
}

/// Output for get-block-refs command.
#[derive(Debug, Serialize)]
pub struct BlockRefsOutput {
    pub refs: Vec<BlockRefOutput>,
}

/// A reference to a block.
#[derive(Debug, Serialize)]
pub struct BlockRefOutput {
    pub block_id: String,
    pub from: PathBuf,
    pub line: usize,
    pub context: String,
}

/// Get all block IDs in a note.
pub fn get_blocks(vault: &Vault, path: &str, output: &Output) -> Result<ExitCode> {
    let note_path = vault.resolve_note(path)?;
    let note = vault.load_note(&note_path)?;

    let blocks = parse_block_ids(&note.content);

    let block_outputs: Vec<_> = blocks
        .iter()
        .map(|b| BlockOutput {
            id: b.id.clone(),
            line: b.line,
            block_type: format!("{:?}", b.block_type).to_lowercase().replace('\"', ""),
        })
        .collect();

    let result = BlocksOutput {
        blocks: block_outputs,
    };
    output.print(&result)?;

    Ok(ExitCode::Success)
}

/// Get all references to blocks in a note.
pub fn get_block_refs(vault: &Vault, path: &str, output: &Output) -> Result<ExitCode> {
    let note_path = vault.resolve_note(path)?;
    let note = vault.load_note(&note_path)?;

    // Get all block IDs in this note
    let blocks = parse_block_ids(&note.content);
    let block_ids: Vec<_> = blocks.iter().map(|b| b.id.clone()).collect();

    // Build link graph to find references
    let graph = LinkGraph::build(vault)?;

    // Find all incoming links that reference blocks in this note
    let incoming = graph.get_incoming(&note_path);

    let refs: Vec<_> = incoming
        .iter()
        .filter(|link| {
            link.link
                .block_id
                .as_ref()
                .map(|bid| block_ids.contains(bid))
                .unwrap_or(false)
        })
        .map(|link| BlockRefOutput {
            block_id: link.link.block_id.clone().unwrap_or_default(),
            from: link.from.clone(),
            line: link.link.line,
            context: link.context.as_string(),
        })
        .collect();

    let result = BlockRefsOutput { refs };
    output.print(&result)?;

    Ok(ExitCode::Success)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BlockType;

    #[test]
    fn test_block_type_formatting() {
        let block = crate::types::BlockId {
            id: "test".to_string(),
            line: 1,
            block_type: BlockType::Paragraph,
        };

        let output = BlockOutput {
            id: block.id.clone(),
            line: block.line,
            block_type: format!("{:?}", block.block_type)
                .to_lowercase()
                .replace('\"', ""),
        };

        assert_eq!(output.block_type, "paragraph");
    }
}
