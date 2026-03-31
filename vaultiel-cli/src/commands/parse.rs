use std::path::PathBuf;
use vaultiel::{Result, TaskConfig, Vault};

pub fn links(vault: &Vault, note: &str) -> Result<()> {
    let n = vault.load_note(&PathBuf::from(note))?;
    println!("{}", serde_json::to_string_pretty(&n.links()).unwrap());
    Ok(())
}

pub fn tags(vault: &Vault, note: &str) -> Result<()> {
    let n = vault.load_note(&PathBuf::from(note))?;
    println!("{}", serde_json::to_string_pretty(&n.tags()).unwrap());
    Ok(())
}

pub fn headings(vault: &Vault, note: &str) -> Result<()> {
    let n = vault.load_note(&PathBuf::from(note))?;
    println!("{}", serde_json::to_string_pretty(&n.headings()).unwrap());
    Ok(())
}

pub fn block_ids(vault: &Vault, note: &str) -> Result<()> {
    let n = vault.load_note(&PathBuf::from(note))?;
    println!("{}", serde_json::to_string_pretty(&n.block_ids()).unwrap());
    Ok(())
}

pub fn tasks(vault: &Vault, note: &str, links_to: Option<&str>) -> Result<()> {
    let path = PathBuf::from(note);
    let n = vault.load_note(&path)?;
    let config = TaskConfig::empty();
    let all_tasks = vaultiel::parser::parse_tasks(n.body(), &path, &config);

    let filtered: Vec<_> = match links_to {
        Some(target) => all_tasks
            .into_iter()
            .filter(|t| t.links.iter().any(|l| l.to == target))
            .collect(),
        None => all_tasks,
    };

    println!("{}", serde_json::to_string_pretty(&filtered).unwrap());
    Ok(())
}

pub fn task_trees(vault: &Vault, note: &str) -> Result<()> {
    let path = PathBuf::from(note);
    let n = vault.load_note(&path)?;
    let config = TaskConfig::empty();
    let trees = vaultiel::parser::parse_task_trees(n.body(), &path, &config);
    println!("{}", serde_json::to_string_pretty(&trees).unwrap());
    Ok(())
}
