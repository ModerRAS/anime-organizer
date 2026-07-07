use rusqlite::Connection;
use std::fs;
use std::process::Command;

fn run_aniorg(args: &[String]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_aniorg"))
        .args(args)
        .output()
        .unwrap()
}

fn media_count(db_path: &std::path::Path) -> i64 {
    let conn = Connection::open(db_path).unwrap();
    conn.query_row("SELECT COUNT(*) FROM media_file", [], |row| row.get(0))
        .unwrap()
}

#[test]
fn library_index_flag_creates_target_root_database() {
    let source = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    fs::write(
        source.path().join("[ANi] Test Show - 01 [1080P].mkv"),
        b"video",
    )
    .unwrap();

    let output = run_aniorg(&[
        "--source".to_string(),
        source.path().display().to_string(),
        "--target".to_string(),
        target.path().display().to_string(),
        "--mode".to_string(),
        "copy".to_string(),
        "--library-index".to_string(),
    ]);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let db_path = target.path().join("library.db");
    assert!(db_path.exists());
    assert_eq!(media_count(&db_path), 1);

    let conn = Connection::open(db_path).unwrap();
    let media_path: String = conn
        .query_row("SELECT path FROM media_file", [], |row| row.get(0))
        .unwrap();
    assert_eq!(media_path, "Test Show/01 [1080P].mkv");
}

#[test]
fn existing_database_is_incremental_until_rebuild_is_requested() {
    let initial_source = tempfile::tempdir().unwrap();
    let empty_source = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    fs::write(
        initial_source
            .path()
            .join("[ANi] Indexed Show - 01 [1080P].mkv"),
        b"video",
    )
    .unwrap();

    let first = run_aniorg(&[
        "--source".to_string(),
        initial_source.path().display().to_string(),
        "--target".to_string(),
        target.path().display().to_string(),
        "--mode".to_string(),
        "copy".to_string(),
        "--library-index".to_string(),
    ]);
    assert!(first.status.success());

    fs::create_dir_all(target.path().join("Manual Show")).unwrap();
    fs::write(
        target.path().join("Manual Show").join("01 [1080P].mkv"),
        b"manual",
    )
    .unwrap();

    let incremental = run_aniorg(&[
        "--source".to_string(),
        empty_source.path().display().to_string(),
        "--target".to_string(),
        target.path().display().to_string(),
        "--mode".to_string(),
        "copy".to_string(),
        "--library-index".to_string(),
    ]);
    assert!(incremental.status.success());
    assert_eq!(media_count(&target.path().join("library.db")), 1);

    let rebuild = run_aniorg(&[
        "--source".to_string(),
        empty_source.path().display().to_string(),
        "--target".to_string(),
        target.path().display().to_string(),
        "--mode".to_string(),
        "copy".to_string(),
        "--library-index".to_string(),
        "--rebuild-library-index".to_string(),
    ]);
    assert!(rebuild.status.success());
    assert_eq!(media_count(&target.path().join("library.db")), 2);
}

#[test]
fn dry_run_library_index_does_not_create_database() {
    let source = tempfile::tempdir().unwrap();
    fs::write(
        source.path().join("[ANi] Dry Run Show - 01 [1080P].mkv"),
        b"video",
    )
    .unwrap();

    let output = run_aniorg(&[
        "--source".to_string(),
        source.path().display().to_string(),
        "--mode".to_string(),
        "copy".to_string(),
        "--dry-run".to_string(),
        "--library-index".to_string(),
    ]);

    assert!(output.status.success());
    assert!(!source.path().join("library.db").exists());
}
