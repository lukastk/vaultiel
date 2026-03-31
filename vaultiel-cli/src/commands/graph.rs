use std::path::PathBuf;
use vaultiel::{Result, Vault};
use vaultiel::graph::LinkGraph;

pub fn incoming_links(vault: &Vault, note: &str) -> Result<()> {
    let graph = LinkGraph::build(vault)?;
    let path = PathBuf::from(note);
    let links = graph.get_incoming(&path);
    println!("{}", serde_json::to_string_pretty(&links).unwrap());
    Ok(())
}

pub fn outgoing_links(vault: &Vault, note: &str) -> Result<()> {
    let graph = LinkGraph::build(vault)?;
    let path = PathBuf::from(note);
    let links = graph.get_outgoing(&path);
    println!("{}", serde_json::to_string_pretty(&links).unwrap());
    Ok(())
}
