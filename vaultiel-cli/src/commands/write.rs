use std::path::PathBuf;
use vaultiel::{Result, Vault};

pub fn create(vault: &Vault, note: &str, content: &str) -> Result<()> {
    let path = PathBuf::from(note);
    vault.create_note(&path, content)?;
    Ok(())
}

pub fn delete(vault: &Vault, note: &str) -> Result<()> {
    let path = PathBuf::from(note);
    vault.delete_note(&path)?;
    Ok(())
}

pub fn rename(vault: &Vault, from: &str, to: &str) -> Result<()> {
    vault.rename_note(&PathBuf::from(from), &PathBuf::from(to))?;
    Ok(())
}

pub fn set_content(vault: &Vault, note: &str, content: &str) -> Result<()> {
    let path = PathBuf::from(note);
    let n = vault.load_note(&path)?;
    let updated = n.with_body(content);
    vault.save_note(&updated)?;
    Ok(())
}

pub fn set_raw_content(vault: &Vault, note: &str, content: &str) -> Result<()> {
    let path = PathBuf::from(note);
    vault.set_raw_content(&path, content)?;
    Ok(())
}

pub fn modify_frontmatter(vault: &Vault, note: &str, key: &str, value: &str, append: bool) -> Result<()> {
    let path = PathBuf::from(note);
    let n = vault.load_note(&path)?;
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(value)
        .unwrap_or_else(|_| serde_yaml::Value::String(value.to_string()));

    let updated = if append {
        n.append_frontmatter_value(key, &yaml_value)?
    } else {
        let mut fm = n.frontmatter()?.unwrap_or_else(|| serde_yaml::Value::Mapping(Default::default()));
        if let Some(map) = fm.as_mapping_mut() {
            map.insert(serde_yaml::Value::String(key.to_string()), yaml_value);
        }
        n.with_frontmatter(&fm)?
    };
    vault.save_note(&updated)?;
    Ok(())
}

pub fn append(vault: &Vault, note: &str, content: &str) -> Result<()> {
    let path = PathBuf::from(note);
    let n = vault.load_note(&path)?;
    let updated = n.append(content);
    vault.save_note(&updated)?;
    Ok(())
}

pub fn replace(vault: &Vault, note: &str, pattern: &str, replacement: &str) -> Result<()> {
    let path = PathBuf::from(note);
    let n = vault.load_note(&path)?;
    let new_content = n.full_content().replace(pattern, replacement);
    let updated = n.with_content(new_content);
    vault.save_note(&updated)?;
    Ok(())
}

pub fn set_task_symbol(vault: &Vault, note: &str, line: usize, symbol: char) -> Result<()> {
    let path = PathBuf::from(note);
    let n = vault.load_note(&path)?;
    let updated = n.set_task_symbol(line, symbol)?;
    vault.save_note(&updated)?;
    Ok(())
}

pub fn remove_frontmatter(vault: &Vault, note: &str, key: &str) -> Result<()> {
    let path = PathBuf::from(note);
    let n = vault.load_note(&path)?;
    let updated = n.remove_frontmatter_key(key)?;
    vault.save_note(&updated)?;
    Ok(())
}
