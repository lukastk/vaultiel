use std::path::PathBuf;
use vaultiel::{Result, Vault};
use vaultiel::metadata;

pub fn init_metadata(vault: &Vault, note: &str, force: bool) -> Result<()> {
    let path = PathBuf::from(note);
    let result = metadata::init_metadata(vault, &path, force)?;
    if let Some(md) = result {
        println!("{}", serde_json::to_string_pretty(&md).unwrap());
    }
    Ok(())
}

pub fn metadata_cmd(vault: &Vault, note: &str) -> Result<()> {
    let path = PathBuf::from(note);
    let md = metadata::get_metadata(vault, &path)?;
    match md {
        Some(m) => println!("{}", serde_json::to_string_pretty(&m).unwrap()),
        None => println!("null"),
    }
    Ok(())
}

pub fn find_by_id(vault: &Vault, id: &str) -> Result<()> {
    let result = metadata::find_by_id(vault, id)?;
    match result {
        Some(path) => println!("{}", path.display()),
        None => {
            eprintln!("Not found");
            std::process::exit(1);
        }
    }
    Ok(())
}
