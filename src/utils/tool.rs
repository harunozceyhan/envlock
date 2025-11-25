use anyhow::{Context, Ok, Result};
use colored::Colorize;
use std::collections::HashMap;

pub fn ensure_folders_of_path(file_path: &str) -> Result<()> {
    let mut parts: Vec<&str> = file_path.split('/').collect();
    parts.pop(); // Remove last
    let folder_path = parts.join("/");

    std::fs::create_dir_all(&folder_path)
        .with_context(|| format!("Failed to create directory {}", folder_path))?;

    Ok(())
}

pub fn parse_env(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    map
}

fn get_changes(
    current: &HashMap<String, String>,
    locked: &HashMap<String, String>,
) -> (Vec<String>, Vec<String>, Vec<(String, String, String)>) {
    let mut additions = Vec::new();
    let mut removals = Vec::new();
    let mut changes = Vec::new();

    for (k, v) in current {
        match locked.get(k) {
            None => additions.push(k.clone()),
            Some(old_v) if old_v != v => changes.push((k.clone(), old_v.clone(), v.clone())),
            _ => {}
        }
    }

    for k in locked.keys() {
        if !current.contains_key(k) {
            removals.push(k.clone());
        }
    }

    (additions, removals, changes)
}

pub fn print_diff(current: &HashMap<String, String>, locked: &HashMap<String, String>) {
    let (additions, removals, changes) = get_changes(current, locked);

    if additions.is_empty() && removals.is_empty() && changes.is_empty() {
        println!("✅ No differences between .env and encrypted env.");
        return;
    }

    println!("Differences:");

    for k in additions {
        println!("{} {} {}", "+".green(), k.green(), "added".green());
    }
    for k in removals {
        println!("{} {} {}", "-".red(), k.red(), "removed".red());
    }
    for (k, old_v, new_v) in changes {
        println!(
            "{} {} {} {} {} {}",
            "~".cyan(),
            "changed:".cyan(),
            k.cyan(),
            old_v.cyan(),
            "→".cyan(),
            new_v.cyan()
        );
    }
}
