use std::path::PathBuf;
use rayon::prelude::*;
use vaultiel::{Note, Result, Vault};

pub fn list(vault: &Vault, pattern: Option<&str>) -> Result<()> {
    let notes = match pattern {
        Some(p) => vault.list_notes_matching(p)?,
        None => vault.list_notes()?,
    };
    for note in notes {
        println!("{}", note.display());
    }
    Ok(())
}

pub fn exists(vault: &Vault, note: &str) -> Result<()> {
    let path = vault.normalize_note_path(note);
    if vault.note_exists(&path) {
        println!("true");
    } else {
        println!("false");
        std::process::exit(1);
    }
    Ok(())
}

pub fn resolve(vault: &Vault, query: &str) -> Result<()> {
    let path = vault.resolve_note(query)?;
    println!("{}", path.display());
    Ok(())
}

pub fn content(vault: &Vault, note: &str) -> Result<()> {
    let path = PathBuf::from(note);
    let n = vault.load_note(&path)?;
    print!("{}", n.full_content());
    Ok(())
}

pub fn body(vault: &Vault, note: &str) -> Result<()> {
    let path = PathBuf::from(note);
    let n = vault.load_note(&path)?;
    print!("{}", n.body());
    Ok(())
}

pub fn frontmatter(vault: &Vault, note: &str) -> Result<()> {
    let path = PathBuf::from(note);
    let n = vault.load_note(&path)?;
    match n.frontmatter()? {
        Some(fm) => println!("{}", serde_json::to_string_pretty(&fm).unwrap()),
        None => println!("null"),
    }
    Ok(())
}

pub fn inspect(vault: &Vault, note: &str) -> Result<()> {
    let path = PathBuf::from(note);
    let n = vault.load_note(&path)?;

    let fm = n.frontmatter()?;
    let links = n.links();
    let tags = n.tags();
    let headings = n.headings();
    let block_ids = n.block_ids();
    let inline_props = n.inline_properties();
    let info = vault.note_info(&path)?;

    let result = serde_json::json!({
        "path": note,
        "name": n.name(),
        "frontmatter": fm,
        "links": links,
        "tags": tags,
        "headings": headings,
        "block_ids": block_ids,
        "inline_properties": inline_props,
        "info": info,
    });

    println!("{}", serde_json::to_string_pretty(&result).unwrap());
    Ok(())
}

pub fn properties(vault: &Vault, note: &str, inline: bool, frontmatter: bool) -> Result<()> {
    let path = PathBuf::from(note);
    let n = vault.load_note(&path)?;

    if inline {
        let props = n.inline_properties();
        println!("{}", serde_json::to_string_pretty(&props).unwrap());
    } else if frontmatter {
        match n.frontmatter()? {
            Some(fm) => println!("{}", serde_json::to_string_pretty(&fm).unwrap()),
            None => println!("null"),
        }
    } else {
        let props = n.get_properties()?;
        println!("{}", serde_json::to_string_pretty(&props).unwrap());
    }
    Ok(())
}

pub fn property(vault: &Vault, note: &str, key: &str, inline: bool, frontmatter: bool) -> Result<()> {
    let path = PathBuf::from(note);
    let n = vault.load_note(&path)?;

    if inline {
        let props = n.inline_properties();
        let val = props.iter().find(|p| p.key == key);
        match val {
            Some(p) => println!("{}", p.value),
            None => {
                eprintln!("Property not found: {key}");
                std::process::exit(1);
            }
        }
    } else if frontmatter {
        match n.frontmatter()? {
            Some(fm) => {
                if let Some(val) = fm.get(key) {
                    println!("{}", serde_json::to_string_pretty(val).unwrap());
                } else {
                    eprintln!("Property not found: {key}");
                    std::process::exit(1);
                }
            }
            None => {
                eprintln!("No frontmatter");
                std::process::exit(1);
            }
        }
    } else {
        match n.get_property(key)? {
            Some(val) => println!("{}", serde_json::to_string_pretty(&val).unwrap()),
            None => {
                eprintln!("Property not found: {key}");
                std::process::exit(1);
            }
        }
    }
    Ok(())
}

pub fn search(vault: &Vault, query: &str) -> Result<()> {
    let results = vault.search_query_string(query)?;
    println!("{}", serde_json::to_string_pretty(&results).unwrap());
    Ok(())
}

pub fn all_frontmatter(
    vault: &Vault,
    pattern: Option<&str>,
    has_key: Option<&str>,
    where_filter: Option<&str>,
) -> Result<()> {
    let notes = match pattern {
        Some(p) => vault.list_notes_matching(p)?,
        None => vault.list_notes()?,
    };

    // Parse where filter: "key=value"
    let where_kv = where_filter.and_then(|w| {
        let mut parts = w.splitn(2, '=');
        let k = parts.next()?;
        let v = parts.next()?;
        Some((k.to_string(), v.to_string()))
    });

    let vault_root = &vault.root;

    let results: Vec<String> = notes.par_iter()
        .filter_map(|path| {
            let n = match Note::load_frontmatter_only(vault_root, path) {
                Ok(n) => n,
                Err(_) => return None,
            };
            let fm = match n.frontmatter() {
                Ok(Some(fm)) => fm,
                _ => return None,
            };

            // Filter: has-key
            if let Some(key) = has_key {
                if fm.get(key).is_none() {
                    return None;
                }
            }

            // Filter: where key=value
            if let Some((ref k, ref v)) = where_kv {
                match fm.get(k) {
                    Some(val) => {
                        let val_str = match val {
                            serde_yaml::Value::String(s) => s.clone(),
                            other => serde_json::to_string(other).unwrap_or_default(),
                        };
                        if val_str != *v {
                            return None;
                        }
                    }
                    None => return None,
                }
            }

            let entry = serde_json::json!({
                "path": path.display().to_string(),
                "frontmatter": fm,
            });
            Some(serde_json::to_string(&entry).unwrap())
        })
        .collect();

    for line in results {
        println!("{}", line);
    }
    Ok(())
}
