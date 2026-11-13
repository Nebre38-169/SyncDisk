use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use config::{filter_for_machine, load_config, machine_name};
use scan::scan;
use setup::{folder_add, folder_remove, ignore_add, ignore_remove};

use crate::sync::sync_folder;

mod config;
mod scan;
mod setup;
mod sync;

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.command {
        Command::Pull(args) => run_sync(Direction::Pull, &args)?,
        Command::Push(args) => run_sync(Direction::Push, &args)?,
        Command::Folder { action } => match action {
            FolderAction::Add {
                disk,
                disk_folder,
                machine_folder,
                machine,
            } => {
                let machine = match machine {
                    Some(name) => name,
                    None => machine_name()?,
                };
                folder_add(&disk, &machine, &disk_folder, &machine_folder)?
            }
            FolderAction::Remove {
                disk,
                disk_folder,
                machine,
            } => {
                let machine = match machine {
                    Some(name) => name,
                    None => machine_name()?,
                };
                folder_remove(&disk, &machine, &disk_folder)?
            }
        },
        Command::Ignore { action } => match action {
            IgnoreAction::Add { disk, pattern } => ignore_add(&disk, &pattern)?,
            IgnoreAction::Remove { disk, pattern } => ignore_remove(&disk, &pattern)?,
        },
    }
    Ok(())
}

fn run_sync(direction: Direction, args: &SyncArgs) -> Result<(), String> {
    let disk_path = &args.disk;
    if !disk_path.is_dir() {
        return Err(format!("Unable to find disk {}", disk_path.display()));
    }

    let ignore_path = disk_path.join(".syncignore");
    let computer_name = machine_name()?;
    let configs = load_config(&disk_path.join(".filesync"))?;
    let filtered_configs = filter_for_machine(configs, &computer_name);
    if filtered_configs.is_empty() {
        return Err(format!(
            "File sync does not contain action for {}",
            computer_name
        ));
    }
    for config in &filtered_configs {
        let disk_side = disk_path.join(&config.disk_path);
        let machine_side = &config.machine_path;
        let disk_map = scan(&disk_side, &ignore_path);
        let machine_map = scan(machine_side, &ignore_path);
        let (source_map, source_folder, destination_map, destination_folder) = match direction {
            Direction::Pull => (&disk_map, &disk_side, &machine_map, machine_side),
            Direction::Push => (&machine_map, machine_side, &disk_map, &disk_side),
        };
        println!(
            "From {} to {} :",
            source_folder.display(),
            destination_folder.display()
        );
        let option = sync::SyncOption {
            verbose: args.verbose,
            dry_run: args.dry_run,
            yes: args.yes,
        };
        sync_folder(
            source_map,
            source_folder,
            destination_map,
            destination_folder,
            option,
        );
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    ///Sync using machine as reference
    Push,
    ///Sync using disk as reference
    Pull,
}

///Synchronize files between a disk and the machine
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    ///Push modification to disk
    Push(SyncArgs),
    ///Pull modification from disk
    Pull(SyncArgs),
    ///Help to write .filesync file
    Folder {
        #[command(subcommand)]
        action: FolderAction,
    },
    ///Help to write .syncignore
    Ignore {
        #[command(subcommand)]
        action: IgnoreAction,
    },
}

#[derive(Args)]
struct SyncArgs {
    ///Disk from or to sync files
    disk: PathBuf,
    /// Print sync action but does not act
    #[arg(long, conflicts_with = "yes")]
    dry_run: bool,
    ///Print more information
    #[arg(short, long)]
    verbose: bool,
    ///Skip confirmation
    #[arg(short, long)]
    yes: bool,
}

#[derive(Subcommand)]
enum FolderAction {
    ///Add a folder with its destination on the machine
    Add {
        ///Path to disk to sync
        disk: PathBuf,
        ///Folder on disk to sync
        disk_folder: PathBuf,
        ///Folder on machine to sync
        machine_folder: PathBuf,
        ///Machine name (if not given, use current one)
        #[arg(long)]
        machine: Option<String>,
    },
    ///Remove a folder with its destination on the machine
    Remove {
        ///Path to disk to sync
        disk: PathBuf,
        ///Folder on disk to sync,
        disk_folder: PathBuf,
        /// Machine name (if not given, use the current one)
        #[arg(long)]
        machine: Option<String>,
    },
}

#[derive(Subcommand)]
enum IgnoreAction {
    ///Add file/folder to the .syncignore
    Add {
        ///Path to disk to sync
        disk: PathBuf,
        ///Pattern which will be ignore by sync
        pattern: String,
    },
    ///Remove file/folder to the .syncignore
    Remove {
        ///Path to disk to sync
        disk: PathBuf,
        ///Pattern which will no longer by ignored by sync
        pattern: String,
    },
}
