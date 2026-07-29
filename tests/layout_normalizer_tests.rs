use anime_organizer::layout_normalizer::{
    apply_layout_plan, build_layout_plan, LayoutActionKind, LayoutKind, SidecarActionKind,
};
use anime_organizer::{LibraryIndex, LibraryIndexRecord};
use rusqlite::Connection;
use std::fs;
use std::path::Path;
use std::process::Command;

fn record(target: &Path, path: &Path) -> LibraryIndexRecord {
    LibraryIndexRecord::from_target_path(target, path)
        .unwrap()
        .unwrap()
}

fn run_aniorg(args: &[String]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_aniorg"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn normalize_layout_cli_creates_plan_and_rejects_direct_apply() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("library");
    let video = target
        .join("CLI Show")
        .join("Season 1")
        .join("[Group] CLI Show - 01.mkv");
    fs::create_dir_all(video.parent().unwrap()).unwrap();
    fs::write(&video, b"video").unwrap();
    LibraryIndex::rebuild(&target, &[record(&target, &video)]).unwrap();
    let plan = directory.path().join("plan.json");

    let output = run_aniorg(&[
        "normalize-layout".to_string(),
        "--target".to_string(),
        target.display().to_string(),
        "--dry-run".to_string(),
        "--plan".to_string(),
        plan.display().to_string(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(plan.exists());

    let apply = run_aniorg(&[
        "normalize-layout".to_string(),
        "--target".to_string(),
        target.display().to_string(),
        "--apply-plan".to_string(),
        plan.display().to_string(),
    ]);
    assert!(!apply.status.success());
    assert!(String::from_utf8_lossy(&apply.stderr).contains("confirmed=true"));
}

#[test]
fn planner_classifies_four_layouts_and_apply_keeps_original_canonical_copy() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("library");
    fs::create_dir_all(&target).unwrap();

    let flat_generated = target.join("Show S2").join("01 [1080p].mkv");
    let season_generated = target.join("Show").join("Season 2").join("01 [1080p].mkv");
    let season_original = target
        .join("Show")
        .join("Season 2")
        .join("[Group] Show S2 - 01 [1080p].mkv");
    let flat_original = target
        .join("Show S2")
        .join("[Group] Show S2 - 01 [1080p].mkv");
    let second_episode = target.join("Show S2").join("02 [1080p].mkv");
    let nested = target.join("Show").join("BDMV").join("03.mkv");
    for path in [
        &flat_generated,
        &season_generated,
        &season_original,
        &flat_original,
    ] {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"identical episode one").unwrap();
    }
    fs::write(&second_episode, b"episode two").unwrap();
    fs::create_dir_all(nested.parent().unwrap()).unwrap();
    fs::write(&nested, b"nested extra").unwrap();
    let source_subtitle = flat_generated.with_file_name("01 [1080p].zh-CN.ass");
    let duplicate_subtitle = season_generated.with_file_name("01 [1080p].zh-CN.ass");
    fs::write(&source_subtitle, b"subtitle").unwrap();
    fs::write(&duplicate_subtitle, b"subtitle").unwrap();

    let records = [
        record(&target, &flat_generated),
        record(&target, &season_generated),
        record(&target, &season_original),
        record(&target, &flat_original),
        record(&target, &second_episode),
    ];
    LibraryIndex::rebuild(&target, &records).unwrap();
    let plan_path = directory.path().join("layout-plan.json");
    let plan = build_layout_plan(&target, &plan_path, false, &|_| {}).unwrap();

    assert_eq!(plan.summary.keep, 1);
    assert_eq!(plan.summary.move_files, 1);
    assert_eq!(plan.summary.deduplicate, 3);
    assert_eq!(plan.summary.conflict, 0);
    assert_eq!(plan.summary.unresolved, 1);
    assert!(flat_generated.exists(), "dry-run must not move files");
    assert!(nested.exists(), "other nested layouts stay unresolved");

    let canonical = plan
        .actions
        .iter()
        .find(|action| {
            action.source.contains("[Group] Show S2 - 01")
                && action.layout == LayoutKind::SeasonOriginal
        })
        .unwrap();
    assert_eq!(canonical.kind, LayoutActionKind::Keep);
    let old_flat = plan
        .actions
        .iter()
        .find(|action| action.source == "Show S2/01 [1080p].mkv")
        .unwrap();
    assert_eq!(old_flat.kind, LayoutActionKind::Deduplicate);
    assert_eq!(
        old_flat.keeper.as_deref(),
        Some("Show/Season 2/[Group] Show S2 - 01 [1080p].mkv")
    );
    assert_eq!(old_flat.sidecars.len(), 1);
    assert_eq!(old_flat.sidecars[0].kind, SidecarActionKind::Move);

    assert!(apply_layout_plan(&target, &plan_path, false, &|_| {}).is_err());
    let applied = apply_layout_plan(&target, &plan_path, true, &|_| {}).unwrap();
    assert_eq!(applied.moved, 1);
    assert_eq!(applied.deduplicated, 3);
    assert_eq!(applied.sidecars_moved, 1);
    assert_eq!(applied.sidecars_deduplicated, 1);
    assert!(season_original.exists());
    assert!(target
        .join("Show")
        .join("Season 2")
        .join("02 [1080p].mkv")
        .exists());
    assert!(target
        .join("Show")
        .join("Season 2")
        .join("[Group] Show S2 - 01 [1080p].zh-CN.ass")
        .exists());
    assert!(!flat_generated.exists());
    assert!(!season_generated.exists());
    assert!(!flat_original.exists());
    assert!(nested.exists());

    let conn = Connection::open(target.join("library.db")).unwrap();
    assert_eq!(
        conn.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM media_file", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM media_file WHERE sha256_prefix_63m IS NOT NULL",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        2
    );
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM media_subtitle", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap(),
        1
    );
    assert_eq!(
        conn.query_row("SELECT path FROM media_subtitle", [], |row| {
            row.get::<_, String>(0)
        })
        .unwrap(),
        "Show/Season 2/[Group] Show S2 - 01 [1080p].zh-CN.ass"
    );
}
