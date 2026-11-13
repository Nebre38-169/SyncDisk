use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::SystemTime,
};

use ignore::WalkBuilder;

pub type FileMap = HashMap<PathBuf, SystemTime>;

pub fn scan(root: &Path, ignore_file: &Path) -> FileMap {
    if !root.is_dir() {
        return HashMap::new();
    }
    let mut map = HashMap::new();
    let mut walker = WalkBuilder::new(root);
    walker.standard_filters(false);
    if ignore_file.is_file() {
        walker.current_dir(root);
        if let Some(err) = walker.add_ignore(ignore_file) {
            eprintln!("Error while loading .syncignore : {}", err);
        }
    }
    for entry_result in walker.build() {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("Warning : {}", err);
                continue;
            }
        };
        let Some(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        }
        let metadata = match entry.metadata() {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Warning for file {} : {}", entry.path().display(), err);
                continue;
            }
        };
        let modified_date = match metadata.modified() {
            Ok(date) => date,
            Err(err) => {
                eprintln!("Warning for file {} : {}", entry.path().display(), err);
                continue;
            }
        };
        let Ok(relative) = entry.path().strip_prefix(root) else {
            continue;
        };
        map.insert(relative.to_path_buf(), modified_date);
    }
    map
}
