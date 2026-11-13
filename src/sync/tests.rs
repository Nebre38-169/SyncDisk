use super::*;

fn at(secondes: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(secondes)
}

fn write_file(path: &Path, date: SystemTime, content: &str) -> std::io::Result<()> {
    std::fs::write(path, content)?;
    let file = OpenOptions::new().write(true).open(path)?;
    file.set_modified(date)?;
    Ok(())
}

#[test]
fn format_duration_conversion() {
    let tests = [
        (0, "0m"),
        (59, "0m"),
        (60, "1m"),
        (3599, "59m"),
        (3600, "1h"),
        (86399, "23h"),
        (86400, "1 days"),
        (1000000, "11 days"),
    ];
    for (sec, expected_text) in tests {
        assert_eq!(
            format_duration(Duration::from_secs(sec)),
            expected_text,
            "{} did not give the correct conversion ({})",
            sec,
            expected_text
        );
    }
}

#[test]
fn file_decision_test() {
    let default_date = SystemTime::UNIX_EPOCH;
    assert_eq!(
        file_decision(default_date, None),
        Action::Create,
        "Failed to detect create with no destination date"
    );

    assert_eq!(
        file_decision(default_date, Some(default_date)),
        Action::UpToDate,
        "Failed to detect up-to-date with same date"
    );
    assert_eq!(
        file_decision(at(30), Some(default_date)),
        Action::UpToDate,
        "Failed to detect up-to-date with 30sec gap"
    );
    assert_eq!(
        file_decision(at(60 * 2), Some(default_date)),
        Action::Update,
        "Failed to detect up-to-date with 2mins gap"
    );
    assert_eq!(
        file_decision(default_date, Some(at(2 * 60))),
        Action::Skip {
            gap: Duration::from_secs(2 * 60)
        },
        "Failed to detect Skip with 2mins gap"
    );
    assert_eq!(
        file_decision(at(60), Some(default_date)),
        Action::Update,
        "Failed to detect Update with 60secs gap"
    );
}

#[test]
fn copy_file_creates_dirs_and_keeps_date() -> std::io::Result<()> {
    let default_date = at(1_700_000_000);
    let temp_dir = tempfile::tempdir().expect("Test");
    let src = temp_dir.path().join("source");
    let dst = temp_dir.path().join("dst").join("a");
    std::fs::create_dir(&src).expect("Test");
    let src_file = src.join("test");
    let dst_file = dst.join("test");
    write_file(src_file.as_path(), default_date, "lorem ipsum").expect("Test");

    copy_file(&src_file, &dst_file)?;

    assert!(dst_file.is_file(), "File was not created");
    assert_eq!(
        std::fs::read_to_string(&dst_file).unwrap(),
        "lorem ipsum",
        "Content was not written"
    );
    assert_eq!(
        std::fs::metadata(&dst_file).unwrap().modified().unwrap(),
        default_date,
        "Date was not modified"
    );
    Ok(())
}

#[test]
fn copy_file_with_existing_file_test() -> std::io::Result<()> {
    let default_date = at(1_700_000_000);
    let temp_dir = tempfile::tempdir().expect("Test");
    let src = temp_dir.path().join("source");
    std::fs::create_dir(&src).expect("Test");
    let dst = temp_dir.path().join("dst").join("a");
    std::fs::create_dir_all(&dst).expect("Test");
    let src_file = src.join("test");
    let dst_file = dst.join("test");

    write_file(
        src_file.as_path(),
        default_date + Duration::from_hours(1),
        "nouveau",
    )?;
    write_file(dst_file.as_path(), default_date, "ancien")?;

    copy_file(&src_file, &dst_file)?;

    assert_eq!(
        std::fs::read_to_string(&dst_file).unwrap(),
        "nouveau",
        "Content was not written"
    );
    assert_eq!(
        std::fs::metadata(&dst_file).unwrap().modified().unwrap(),
        default_date + Duration::from_hours(1),
        "Date was not modified"
    );
    Ok(())
}

#[test]
fn copy_file_missing_source_fails() {
    let temp_dir = tempfile::tempdir().expect("Test");
    let src = temp_dir.path().join("sub");
    let dst = temp_dir.path().join("dst");
    let src_file = src.join("copy.txt");
    let dst_file = dst.join("copy.txt");

    let result = copy_file(src_file.as_path(), dst_file.as_path());
    assert!(result.is_err());
    assert!(!dst.exists());
}
