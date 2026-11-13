use std::path::{Path, PathBuf};

pub const SEPARATOR: &str = "->";
pub const UTF8_BOM: char = '\u{feff}';

#[derive(Debug, PartialEq)]
pub struct SyncConfig {
    machine_name: String,
    pub(crate) disk_path: PathBuf,
    pub(crate) machine_path: PathBuf,
}

impl SyncConfig {
    pub(crate) fn matches(&self, machine: &str, disk: &Path) -> bool {
        self.machine_name.eq_ignore_ascii_case(machine) && self.disk_path == disk
    }
}

pub fn parse_line(line: &str) -> Result<SyncConfig, String> {
    let parts: Vec<&str> = line.split(SEPARATOR).map(|x| x.trim()).collect();
    if parts.len() != 3 {
        return Err(format!("'{}' doesn't match syntax", line));
    }
    if parts.iter().any(|x| x.is_empty()) {
        return Err(format!(
            "'{}' doesn't match syntax, contains an empty field",
            line
        ));
    }
    let machine_path = PathBuf::from(parts[2]);
    let disk_path = PathBuf::from(parts[1]);
    if !machine_path.is_absolute() {
        return Err("Machine path must be absolute".to_string());
    }
    if !disk_path.is_relative() {
        return Err("Disk path must be relative".to_string());
    }
    Ok(SyncConfig {
        machine_name: parts[0].to_string(),
        disk_path,
        machine_path,
    })
}

pub fn load_config(path: &Path) -> Result<Vec<SyncConfig>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|err| format!("{} with file {}", err, path.display()))?;
    let content = content.trim_start_matches(UTF8_BOM);
    let mut content_sync = Vec::new();
    for (index, line) in content.lines().enumerate() {
        let line_trim = line.trim();
        if line_trim.is_empty() || line_trim.starts_with('#') {
            continue;
        }
        let config =
            parse_line(line_trim).map_err(|err| format!("line {} : {}", index + 1, err))?;
        content_sync.push(config);
    }
    Ok(content_sync)
}

pub fn filter_for_machine(config: Vec<SyncConfig>, machine_name: &str) -> Vec<SyncConfig> {
    config
        .into_iter()
        .filter(|conf| conf.machine_name.eq_ignore_ascii_case(machine_name))
        .collect()
}

#[cfg(windows)]
pub fn machine_name() -> Result<String, String> {
    std::env::var("COMPUTERNAME").map_err(|err| format!("Unable to find COMPUTERNAME var :{}", err))
}

#[cfg(target_os = "linux")]
pub fn machine_name() -> Result<String, String> {
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .map_err(|err| format!("Unable to find machine name : {}", err))
        .map(|content| content.trim().to_string())
}

#[cfg(test)]
mod tests;
