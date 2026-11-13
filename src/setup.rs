use std::path::Path;

use crate::config::{SEPARATOR, UTF8_BOM, machine_name, parse_line};

pub fn ignore_add(disk: &Path, pattern: &str) -> Result<(), String> {
    if !disk.is_dir() {
        return Err(format!("{} is not a valid path", disk.display()));
    }
    let ignore_path = disk.join(".syncignore");
    let content = open_file(&ignore_path).map_err(|err| {
        format!(
            "Error while opening file {} : {}",
            ignore_path.display(),
            err
        )
    })?;
    let content = add_pattern(&content, pattern)?;
    std::fs::write(&ignore_path, content).map_err(|err| {
        format!(
            "Error while writing file {} : {}",
            ignore_path.display(),
            err
        )
    })?;
    println!("Added {} to {}", pattern.trim(), ignore_path.display());
    Ok(())
}

pub fn ignore_remove(disk: &Path, pattern: &str) -> Result<(), String> {
    if !disk.is_dir() {
        return Err(format!("{} is not a valid path", disk.display()));
    }
    let ignore_path = disk.join(".syncignore");
    let content = open_file(&ignore_path).map_err(|err| {
        format!(
            "Error while opening file {} : {}",
            ignore_path.display(),
            err
        )
    })?;
    let content = remove_pattern(&content, pattern)?;
    std::fs::write(&ignore_path, content).map_err(|err| {
        format!(
            "Error while writing file {} : {}",
            ignore_path.display(),
            err
        )
    })?;
    println!("Removed {} from {}", pattern.trim(), ignore_path.display());
    Ok(())
}

pub fn folder_add(
    disk: &Path,
    machine: &str,
    disk_folder: &Path,
    machine_folder: &Path,
) -> Result<(), String> {
    if !disk.is_dir() {
        return Err(format!("{} is not a valid path", disk.display()));
    }
    let sync_path = disk.join(".filesync");
    let machine_folder = if machine_name()
        .unwrap_or("".to_string())
        .eq_ignore_ascii_case(machine)
    {
        let machine_local = std::path::absolute(machine_folder).map_err(|err| {
            format!(
                "Unable to resolve folder {} : {}",
                machine_folder.display(),
                err
            )
        })?;
        if !machine_local.is_dir() {
            return Err(format!("Folder {} does not exist", machine_local.display()));
        }
        machine_local
    } else {
        machine_folder.to_path_buf()
    };
    let content = open_file(&sync_path)
        .map_err(|err| format!("Error while opening file {} : {}", sync_path.display(), err))?;
    let content = add_folder(&content, machine, disk_folder, machine_folder.as_path())?;
    std::fs::write(&sync_path, content)
        .map_err(|err| format!("Error while writing file {} : {}", sync_path.display(), err))?;
    println!(
        "Added {} -> {} for machine {} to {}",
        disk_folder.display(),
        machine_folder.display(),
        machine,
        sync_path.display()
    );
    Ok(())
}

pub fn folder_remove(disk: &Path, machine: &str, disk_folder: &Path) -> Result<(), String> {
    if !disk.is_dir() {
        return Err(format!("{} is not a valid path", disk.display()));
    }
    let sync_path = disk.join(".filesync");
    let content = open_file(&sync_path)
        .map_err(|err| format!("Error while opening file {} : {}", sync_path.display(), err))?;
    let content = remove_folder(&content, machine, disk_folder).map_err(|err| err.to_string())?;
    std::fs::write(&sync_path, content)
        .map_err(|err| format!("Error while writing file {} : {}", sync_path.display(), err))?;
    println!(
        "Removed {} for machine {} from {}",
        disk_folder.display(),
        machine,
        sync_path.display()
    );
    Ok(())
}

fn open_file(file_path: &Path) -> std::io::Result<String> {
    match std::fs::read_to_string(file_path) {
        Ok(text) => Ok(text.trim_start_matches(UTF8_BOM).to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok("".to_string()),
        Err(e) => Err(e),
    }
}

fn add_pattern(current: &str, motif: &str) -> Result<String, String> {
    let motif = motif.trim();
    if motif.is_empty() {
        return Err("Cannot add empty motif".to_string());
    }
    if motif.starts_with('#') {
        return Err("Cannot add commentary".to_string());
    }
    let mut pattern_check = ignore::gitignore::GitignoreBuilder::new(".");
    pattern_check
        .add_line(None, motif)
        .map_err(|err| err.to_string())?;
    if current.lines().any(|line| line.trim() == motif) {
        return Err(format!("Motif {} already in .syncignore", motif));
    }

    let current = current.to_string();

    Ok(append_line(current, motif))
}

fn remove_pattern(current: &str, motif: &str) -> Result<String, String> {
    let motif = motif.trim();
    if motif.is_empty() {
        return Err("Cannot remove empty motif".to_string());
    }
    if motif.starts_with('#') {
        return Err("Cannot remove commentary".to_string());
    }
    let prec_line_count = current.lines().count();
    let current: Vec<&str> = current
        .lines()
        .filter(|line| !(line.trim() == motif))
        .collect();
    if current.len() == prec_line_count {
        return Err(format!("Unable to remove {} from .syncignore", motif));
    }
    let mut current = current.join("\n");
    current.push('\n');
    Ok(current)
}

fn add_folder(
    content: &str,
    machine: &str,
    disk_folder: &Path,
    machine_folder: &Path,
) -> Result<String, String> {
    let line = format!(
        "{} {} {} {} {}",
        machine,
        SEPARATOR,
        disk_folder.to_str().ok_or("Disk path is not UTF-8")?,
        SEPARATOR,
        machine_folder.to_str().ok_or("Machine path is not UTF-8")?
    );
    parse_line(&line)?;
    let content = content.to_string();
    if content
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with("#"))
        .any(|line| parse_line(line).is_ok_and(|c| c.matches(machine, disk_folder)))
    {
        return Err(format!("{} already exist in the .filesync", line));
    }

    Ok(append_line(content, &line))
}

fn remove_folder(content: &str, machine: &str, disk_path: &Path) -> Result<String, String> {
    let prec_line_count = content.lines().count();
    let content: Vec<&str> = content
        .lines()
        .filter(|line| filter_sync_config(line, machine, disk_path))
        .collect();
    if content.len() == prec_line_count {
        return Err(format!(
            "Unable to remove path {} for machine {} from .filesync",
            disk_path.display(),
            machine
        ));
    }
    let mut content = content.join("\n");
    content.push('\n');
    Ok(content)
}

fn filter_sync_config(line: &str, machine: &str, disk_path: &Path) -> bool {
    if line.is_empty() || line.starts_with("#") {
        return true;
    }
    match parse_line(line) {
        Ok(config) => !config.matches(machine, disk_path),
        Err(_) => true,
    }
}

fn append_line(mut content: String, line: &str) -> String {
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(line);
    content.push('\n');
    content
}

#[cfg(test)]
mod tests;
