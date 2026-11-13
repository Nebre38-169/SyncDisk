use super::*;

#[test]
#[cfg(windows)]
fn parse_line_test() {
    assert_eq!(
        parse_line("PC -> Projects -> C:\\Projects").unwrap(),
        SyncConfig {
            machine_name: "PC".to_string(),
            machine_path: Path::new("C:\\Projects").to_path_buf(),
            disk_path: Path::new("Projects").to_path_buf()
        }
    );
    assert!(
        parse_line("PC Projects -> C:\\Projects")
            .unwrap_err()
            .contains("syntax")
    );
    assert!(
        parse_line("PC -> D:\\Projects -> C:\\Projects")
            .unwrap_err()
            .contains("relative")
    );
    assert!(
        parse_line("PC -> Projects -> Projects")
            .unwrap_err()
            .contains("absolute")
    );
}

#[test]
#[cfg(unix)]
fn parse_line_test() {
    assert_eq!(
        parse_line("PC -> Projects -> /home/julie/Projects").unwrap(),
        SyncConfig {
            machine_name: "PC".to_string(),
            machine_path: Path::new("/home/julie/Projects").to_path_buf(),
            disk_path: Path::new("Projects").to_path_buf()
        }
    );
    assert!(
        parse_line("PC Projects -> /home/julie/Projects")
            .unwrap_err()
            .contains("syntax")
    );
    assert!(
        parse_line("PC -> /media/disk/Projects -> /home/julie/Projects")
            .unwrap_err()
            .contains("relative")
    );
    assert!(
        parse_line("PC -> Projects -> Projects")
            .unwrap_err()
            .contains("absolute")
    );
    // Un chemin Windows n'est pas absolu sous Linux : il doit être refusé.
    assert!(
        parse_line("PC -> Projects -> C:\\Projects")
            .unwrap_err()
            .contains("absolute")
    );
}
