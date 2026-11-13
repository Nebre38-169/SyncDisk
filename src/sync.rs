use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use crate::scan::FileMap;

const TOLERANCE: Duration = Duration::from_secs(60);

#[derive(Debug, PartialEq)]
enum Action {
    Create,
    Update,
    UpToDate,
    Skip { gap: Duration },
}

pub struct SyncOption {
    pub(crate) verbose: bool,
    pub(crate) dry_run: bool,
    pub(crate) yes: bool,
}

fn file_decision(source_date: SystemTime, destination_date: Option<SystemTime>) -> Action {
    let Some(destination_date) = destination_date else {
        return Action::Create;
    };
    match source_date.duration_since(destination_date) {
        Ok(gap) if gap >= TOLERANCE => Action::Update,
        Ok(_) => Action::UpToDate,
        Err(e) if e.duration() > TOLERANCE => Action::Skip { gap: e.duration() },
        Err(_) => Action::UpToDate,
    }
}

fn action_plan(source_map: &FileMap, destination_map: &FileMap) -> Vec<(PathBuf, Action)> {
    let mut plan = Vec::new();
    for (path, date) in source_map {
        let dest_date = destination_map.get(path).copied();
        let action = file_decision(*date, dest_date);
        plan.push((path.clone(), action));
    }
    plan
}

fn format_duration(gap: Duration) -> String {
    let sec = gap.as_secs();
    match sec {
        0..3600 => format!("{}m", sec / 60),
        3600..86400 => format!("{}h", sec / 3600),
        _ => format!("{} days", sec / 86400),
    }
}

fn copy_file(source_path: &Path, dest_path: &Path) -> std::io::Result<()> {
    let date = std::fs::metadata(source_path)?.modified()?;
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?
    };
    std::fs::copy(source_path, dest_path)?;
    let file = OpenOptions::new().write(true).open(dest_path)?;
    file.set_modified(date)?;
    Ok(())
}

fn confirm(question: &str) -> bool {
    let mut answer = String::new();

    print!("{} [y/N] ", question);
    let _ = std::io::stdout().flush();
    let result = std::io::stdin().read_line(&mut answer);
    match result {
        Ok(_) => matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes"),
        Err(_) => false,
    }
}

pub fn sync_folder(
    source_map: &FileMap,
    source_folder: &Path,
    dest_map: &FileMap,
    dest_folder: &Path,
    option: SyncOption,
) {
    let mut plan = action_plan(source_map, dest_map);
    let mut create_counter = 0;
    let mut update_counter = 0;
    let mut skip_counter = 0;
    let mut up_to_date_counter = 0;
    plan.sort_by(|a, b| a.0.cmp(&b.0));
    for (path, action) in &plan {
        match action {
            Action::Create => {
                println!("+ {}", path.display());
                create_counter += 1;
            }
            Action::Update => {
                println!("~ {}", path.display());
                update_counter += 1;
            }
            Action::Skip { gap } => {
                eprintln!(
                    "Warning : file {} on destination is newer by {}",
                    path.display(),
                    format_duration(*gap)
                );
                skip_counter += 1;
            }
            Action::UpToDate => {
                up_to_date_counter += 1;
                if option.verbose {
                    println!("file {} already up to date!", path.display());
                }
            }
        }
    }
    println!(
        "{} created, {} updated, {} skipped, {} up-to-date",
        create_counter, update_counter, skip_counter, up_to_date_counter
    );
    if option.dry_run {
        println!("dry run, nothing copied");
        return;
    }

    if create_counter == 0 && update_counter == 0 {
        println!("No update for folder {}", source_folder.display());
        return;
    }

    if !option.yes
        && !confirm(&format!(
            "Copy {} files to {}, Proceed ?",
            create_counter + update_counter,
            dest_folder.display()
        ))
    {
        println!("Skipped");
        return;
    }

    let mut success_copy = 0;
    let mut fails = Vec::new();
    for (path, action) in plan {
        match action {
            Action::Create | Action::Update => {
                let source_path = source_folder.join(&path);
                let dest_path = dest_folder.join(&path);
                match copy_file(&source_path, &dest_path) {
                    Ok(()) => success_copy += 1,
                    Err(err) => {
                        eprintln!(
                            "Failed to copy file {} with error : {}",
                            path.display(),
                            err
                        );
                        fails.push((path, err.to_string()));
                    }
                }
            }
            Action::Skip { .. } | Action::UpToDate => {}
        }
    }
    println!("{} files were copied", success_copy);
    if !fails.is_empty() {
        eprintln!("Failed copies : {}", fails.len());
    }
}

#[cfg(test)]
mod tests;
