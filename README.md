# file_sync

A command-line tool that syncs folders between a computer and an external drive, based on file modification dates.

The external drive holds the configuration: which folders to sync, where to put them, and on which machine. You can plug the same drive into several computers, and each one syncs its own folders.

```
file_sync pull E:\        # drive → machine
file_sync push E:\        # machine → drive
```

## Contents

- [How it works](#how-it-works)
- [Installation](#installation)
- [The `.filesync` file](#the-filesync-file)
- [The `.syncignore` file](#the-syncignore-file)
- [Commands](#commands)
  - [`push` / `pull`: sync](#push--pull-sync)
  - [`folder add` / `folder remove`: edit `.filesync`](#folder-add--folder-remove-edit-filesync)
  - [`ignore add` / `ignore remove`: edit `.syncignore`](#ignore-add--ignore-remove-edit-syncignore)
- [Full example](#full-example)
- [Good to know](#good-to-know)
- [Development](#development)

---

## How it works

A sync always goes **one way**, chosen on the command line:

| Command | Source | Destination |
|---|---|---|
| `pull` | the drive | the machine |
| `push` | the machine | the drive |

For each file in the source, the tool compares its modification date with the date of the same file in the destination, then picks an action:

| Symbol | Action | When? |
|---|---|---|
| `+` | **Create** | the file doesn't exist in the destination |
| `~` | **Update** | the source is newer by **at least 1 minute** |
| *(none)* | **Up to date** | less than a minute apart, either way |
| `Warning` | **Skip** | the destination is newer by more than a minute: **it is not overwritten**, and a warning is shown |

General rules:

- **Nothing is ever deleted.** A file deleted in the source is never deleted in the destination.
- **Dates are kept.** A copied file keeps the modification date of the original, so running `pull` twice in a row copies nothing the second time.
- **Empty folders are not synced.** Intermediate folders are created when needed.
- **One error doesn't stop the rest.** An unreadable file, or a copy that fails (access denied…), shows a warning, and the sync carries on with the other files.
- **You are asked to confirm** before any copy, unless you pass `-y`.

---

## Installation

You need [Rust](https://rustup.rs/) (2024 edition).

```sh
cd file_sync
cargo install --path .
```

The `file_sync` command is then available everywhere. You can also build it with `cargo build --release`: the executable is then in `target/release/`.

The tool works on **Windows** and **Linux**.

### Machine name

Each line of `.filesync` belongs to a machine. The tool detects the name of the current machine from:

- **Windows**: the `COMPUTERNAME` environment variable;
- **Linux**: the content of `/proc/sys/kernel/hostname`.

Names are compared **case-insensitively**: `desktop-pc` and `DESKTOP-PC` are the same machine.

---

## The `.filesync` file

This file lives **at the root of the external drive**. It lists the folders to sync, one line per folder:

```
machine -> folder_on_drive -> folder_on_machine
```

| Field | Rule |
|---|---|
| `machine` | the name of the machine this line is for |
| `folder_on_drive` | a path **relative** to the root of the drive |
| `folder_on_machine` | an **absolute** path on the machine |

Example:

```
# Desktop computer
DESKTOP-PC -> Blender -> C:\Users\me\Documents\Blender
DESKTOP-PC -> Photos -> D:\Photos

# Linux laptop
LAPTOP -> Blender -> /home/me/Blender
```

- Empty lines and lines starting with `#` are ignored.
- The spaces around `->` are optional.
- Only the lines for the current machine are used. Here, on `DESKTOP-PC`, `pull` syncs `Blender` and `Photos`, and ignores the `LAPTOP` line.
- The file can be saved as UTF-8, with or without a BOM, so a file saved with Windows Notepad works.
- An invalid line stops the program, with a message giving its line number.

You can write this file by hand, or with the [`folder add`](#folder-add--folder-remove-edit-filesync) command, which checks each line.

---

## The `.syncignore` file

This **optional** file also lives at the root of the drive. It lists the files and folders that must **never be synced**, using the same syntax as a `.gitignore`:

```
# Blender automatic backups
*.blend1

# System files
Thumbs.db
desktop.ini

# The "cache" folder at the root of each synced folder
/cache/

# Any "renders" folder, at any depth
renders/
```

- The file is **shared** by all the folders in `.filesync`.
- A pattern starting with `/` is anchored **to the root of each synced folder**. For example, `/cache/` ignores `Blender/cache/`, but not `Blender/project/cache/`.
- Exclusions apply on both sides, on the drive and on the machine.

You can write this file by hand, or with the [`ignore add`](#ignore-add--ignore-remove-edit-syncignore) command, which checks the syntax of each pattern.

---

## Commands

```
file_sync <COMMAND> --help     # detailed help for each command
file_sync --version
```

### `push` / `pull`: sync

```
file_sync pull <DRIVE> [--dry-run] [-y] [-v]
file_sync push <DRIVE> [--dry-run] [-y] [-v]
```

| Option | Effect |
|---|---|
| `--dry-run` | show what would be done, **without copying anything** |
| `-y`, `--yes` | don't ask for confirmation before copying |
| `-v`, `--verbose` | also list files that are already up to date |

`--dry-run` and `--yes` can't be used together.

**Example: simulating a `pull`**

```
> file_sync pull E:\ --dry-run
From E:\Projects to C:\Users\me\Projects :
~ notes with spaces and accents é.txt
~ newer_on_drive.txt
+ keep\ignored_root\file.txt
Warning : file newer_on_machine.txt on destination is newer by 2h
+ only_on_drive.txt
+ sub_folder\deep\new.txt
3 created, 2 updated, 1 skipped, 2 up-to-date
dry run, nothing copied
```

In this example, `newer_on_machine.txt` was edited on the machine since the last sync. It is **not overwritten**: a `push` will copy it to the drive.

**Example: `pull` with confirmation**

```
> file_sync pull E:\
From E:\Projects to C:\Users\me\Projects :
...
3 created, 2 updated, 1 skipped, 2 up-to-date
Copy 5 files to C:\Users\me\Projects, Proceed ? [y/N] y
5 files were copied
```

The default answer is **no**: only `y` and `yes` (any case) start the copy. Each folder in `.filesync` gets its own confirmation.

**Example: running it again right away**

```
> file_sync pull E:\ -y
From E:\Projects to C:\Users\me\Projects :
Warning : file newer_on_machine.txt on destination is newer by 2h
0 created, 0 updated, 1 skipped, 7 up-to-date
No update for folder E:\Projects
```

The dates were kept during the copy, so there is nothing left to do.

**Example: verbose `push`**

```
> file_sync push E:\ --dry-run -v
From C:\Users\me\Projects to E:\Projects :
Warning : file notes with spaces and accents é.txt on destination is newer by 1h
Warning : file newer_on_drive.txt on destination is newer by 2h
file identical.txt already up to date!
~ newer_on_machine.txt
+ only_on_machine.txt
file within_tolerance_30s.txt already up to date!
1 created, 1 updated, 2 skipped, 2 up-to-date
dry run, nothing copied
```

**Copy errors**

A failed copy is reported, and the other copies carry on:

```
Failed to copy file report.pdf with error : Access is denied. (os error 5)
4 files were copied
Failed copies : 1
```

### `folder add` / `folder remove`: edit `.filesync`

```
file_sync folder add    <DRIVE> <DRIVE_FOLDER> <MACHINE_FOLDER> [--machine NAME]
file_sync folder remove <DRIVE> <DRIVE_FOLDER> [--machine NAME]
```

Without `--machine`, the command applies to the **current machine**.

**Adding a folder**

```
> file_sync folder add E:\ Blender C:\Users\me\Documents\Blender
Added Blender -> C:\Users\me\Documents\Blender for machine DESKTOP-PC to E:\.filesync
```

When the line is for the current machine, the machine folder can be a **relative** path. It is turned into an absolute path:

```
> cd C:\Users\me\Documents
> file_sync folder add E:\ Blender .\Blender
Added Blender -> C:\Users\me\Documents\Blender for machine DESKTOP-PC to E:\.filesync
```

**Preparing a line for another machine**

```
> file_sync folder add E:\ Blender /home/me/Blender --machine LAPTOP
Added Blender -> /home/me/Blender for machine LAPTOP to E:\.filesync
```

What `folder add` checks:

| Check | Rejected example |
|---|---|
| the drive exists | `file_sync folder add Z:\ ...` |
| the drive folder is relative | `folder add E:\ E:\Blender ...` |
| the machine folder is absolute (for another machine) | `folder add E:\ Blender Blender --machine LAPTOP` |
| the machine folder exists (for the current machine) | `folder add E:\ Blender C:\Missing` |
| no path contains the `->` separator | `folder add E:\ "a->b" ...` |
| the machine doesn't already have a line for this drive folder | adding `Blender` twice for `DESKTOP-PC`, even to two different machine folders |

The `.filesync` file is created if it doesn't exist. Comments and existing lines are kept.

**Removing a folder**

The line is identified by the machine and the drive folder:

```
> file_sync folder remove E:\ Blender
Removed Blender for machine DESKTOP-PC from E:\.filesync

> file_sync folder remove E:\ Blender --machine LAPTOP
Removed Blender for machine LAPTOP from E:\.filesync
```

Only that machine's line is removed: other machines' lines for the same folder, comments and empty lines stay where they are. If no line matches, the command returns an error.

Removing a folder from `.filesync` **doesn't delete any file**, on the drive or on the machine.

### `ignore add` / `ignore remove`: edit `.syncignore`

```
file_sync ignore add    <DRIVE> <PATTERN>
file_sync ignore remove <DRIVE> <PATTERN>
```

```
> file_sync ignore add E:\ "*.blend1"
Added *.blend1 to E:\.syncignore

> file_sync ignore remove E:\ "*.blend1"
Removed *.blend1 from E:\.syncignore
```

With `ignore add`:

- the pattern is checked: `{a,b` (unclosed brace) or `[z-a]` (reversed range) are rejected;
- a pattern that is already there is rejected, instead of creating a duplicate;
- an empty pattern, or one starting with `#` (it would be a comment), is rejected;
- the `.syncignore` file is created if it doesn't exist.

`ignore remove` only removes lines that are **exactly** equal to the pattern (ignoring surrounding spaces). For example, `ignore remove E:\ blend1` doesn't remove the `*.blend1` line.

> **On Linux**, always put the pattern in quotes. Otherwise the shell replaces `*.blend1` with the matching files in the current folder before the tool even receives it.

---

## Full example

First use of an `E:\` drive with two computers.

**On the desktop:**

```
file_sync folder add E:\ Blender C:\Users\me\Documents\Blender
file_sync ignore add E:\ "*.blend1"
file_sync ignore add E:\ Thumbs.db

file_sync push E:\ --dry-run      # check what will be copied
file_sync push E:\                # copy to the drive
```

**On the laptop**, with the same drive:

```
file_sync folder add E:\ Blender C:\Users\me\Blender
file_sync pull E:\                # get the files
```

**Then, day to day**, on the machine you work on:

```
file_sync pull E:\    # before working: get the latest versions
file_sync push E:\    # afterwards: send your changes to the drive
```

---

## Good to know

- **Drive path on Windows**: write `E:\` (without quotes) or `"E:\\"`. With `"E:\"`, Windows reads `\"` as an escaped quote, and the tool receives the wrong path. As for `E:` on its own, it means the *current folder* of drive E:, not its root.
- **Renamed files**: a file renamed in the source is copied under its new name, and the old one stays in the destination, since nothing is ever deleted.
- **Conflicts**: if the same file was edited on both sides, the newest version wins. When syncing in the wrong direction, the file is skipped with a warning, and nothing is overwritten.
- **The one-minute tolerance** absorbs differences in date precision between file systems (for example FAT32 or exFAT on a USB stick).
- **Copies are not atomic**: if the drive is unplugged during a copy, the file being copied may be left incomplete in the destination. Its date is then recent, so run a `--dry-run` sync in the other direction to check, or copy that file again.
- **Exit code**: on a blocking error (drive not found, invalid `.filesync`…), the program stops with a message and a non-zero exit code.

---

## Development

```sh
cargo test       # unit tests
cargo clippy     # static analysis
```

### Code layout

| File | Role |
|---|---|
| `src/main.rs` | the command line (clap) and dispatching to each command |
| `src/config.rs` | reading and validating `.filesync`, and the machine name |
| `src/scan.rs` | walking a folder while applying `.syncignore` |
| `src/sync.rs` | the decisions (create / update / skip), printing the plan, and copying |
| `src/setup.rs` | the `folder` and `ignore` commands, which edit `.filesync` and `.syncignore` |

Each module's tests are in `src/<module>/tests.rs`.

### Test folders

The `make_test_dirs.ps1` script (Windows, PowerShell) generates a fake drive and a fake machine folder, with fixed modification dates, covering every case: identical file, 30-second gap, file newer on one side or the other, files present on one side only, sub-folders, accents and spaces, excluded files.

```powershell
.\make_test_dirs.ps1           # .filesync saved with a BOM
.\make_test_dirs.ps1 -NoBom    # .filesync without a BOM

cargo run -- pull test_disk --dry-run
cargo run -- push test_disk --dry-run
```

Expected results:

| Command | Created | Updated | Skipped | Up to date |
|---|---|---|---|---|
| `pull` | 3 | 2 | 1 | 2 |
| `push` | 1 | 1 | 2 | 2 |

You can run the script again at any time to start from scratch. The `test_disk/` and `test_machine/` folders are ignored by git.

The file names generated by the script are in French (`identique.txt`, `disque_plus_recent.txt`, `seulement_machine.txt`…). The examples above use translated names.

---

## License

This project is released under the [MIT License](LICENSE).
