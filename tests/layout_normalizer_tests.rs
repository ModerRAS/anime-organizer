use anime_organizer::layout_normalizer::{
    apply_layout_plan, build_layout_plan, build_layout_plan_with_cancel, LayoutActionKind,
    LayoutKind, SidecarActionKind,
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

    let force_rehash = run_aniorg(&[
        "normalize-layout".to_string(),
        "--target".to_string(),
        target.display().to_string(),
        "--dry-run".to_string(),
        "--plan".to_string(),
        directory.path().join("force.json").display().to_string(),
        "--force-rehash".to_string(),
    ]);
    assert!(!force_rehash.status.success());
    assert!(String::from_utf8_lossy(&force_rehash.stderr).contains("unexpected argument"));

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
fn planner_cancels_before_publishing_a_partial_plan() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("library");
    let video = target.join("Show").join("[Group] Show - 01.mkv");
    fs::create_dir_all(video.parent().unwrap()).unwrap();
    fs::write(&video, b"video").unwrap();
    LibraryIndex::rebuild(&target, &[record(&target, &video)]).unwrap();
    let plan_path = directory.path().join("canceled-plan.json");

    let result = build_layout_plan_with_cancel(&target, &plan_path, &|_| {}, &|| true);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("取消"));
    assert!(!plan_path.exists());
}

#[test]
fn planner_does_not_hash_duplicate_candidates_without_cached_hashes() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("library");
    let canonical = target
        .join("Show")
        .join("Season 1")
        .join("[Group] Show - 01.mkv");
    let flat = target.join("Show").join("[Group] Show - 01.mkv");
    fs::create_dir_all(canonical.parent().unwrap()).unwrap();
    fs::write(&canonical, b"identical media bytes").unwrap();
    fs::write(&flat, b"identical media bytes").unwrap();
    LibraryIndex::rebuild(
        &target,
        &[record(&target, &canonical), record(&target, &flat)],
    )
    .unwrap();

    let plan_path = directory.path().join("metadata-only-plan.json");
    let plan = build_layout_plan(&target, &plan_path, &|_| {}).unwrap();

    assert_eq!(plan.summary.deduplicate, 0);
    assert_eq!(plan.summary.conflict, 2);
    assert!(plan
        .actions
        .iter()
        .all(|action| action.sha256_full.is_none()));
    assert_eq!(fs::read(&canonical).unwrap(), b"identical media bytes");
    assert_eq!(fs::read(&flat).unwrap(), b"identical media bytes");
}

#[test]
fn planner_recognizes_observed_episode_revision_and_title_formats() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("library");
    let files = [
        (
            "Shibou Yuugi de Meshi wo Kuu",
            "[LoliHouse] Shibou Yuugi de Meshi wo Kuu. 44 Cloudy Beach [WebRip 1080p HEVC-10bit AAC SRTx2].mkv",
            44.0,
        ),
        (
            "The World Is Dancing",
            "[Studio GreenTea&LoliHouse] The World Is Dancing - 01v2 [WebRip 1080p HEVC-10bit AAC ASSx2].mkv",
            1.0,
        ),
    ];
    let mut records = Vec::new();
    for (root, file, episode) in files {
        let path = target.join(root).join("Season 1").join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"video").unwrap();
        records.push(LibraryIndexRecord::new(
            root.to_string(),
            1,
            episode,
            format!("{root}/Season 1/{file}"),
            &path,
        ));
    }
    LibraryIndex::rebuild(&target, &records).unwrap();

    let plan = build_layout_plan(&target, &directory.path().join("plan.json"), &|_| {}).unwrap();
    assert_eq!(plan.summary.unresolved, 0);
    assert_eq!(plan.summary.keep, 2);
    for (root, file, episode) in files {
        let source = format!("{root}/Season 1/{file}");
        let action = plan
            .actions
            .iter()
            .find(|action| action.source == source)
            .unwrap_or_else(|| panic!("missing plan action for {source}"));
        assert_eq!(action.kind, LayoutActionKind::Keep);
        assert_eq!(action.identity.as_ref().unwrap().episode, episode);
    }
}

#[test]
fn planner_normalizes_observed_season_markers_in_roots() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("library");
    let cases = [
        (
            "[VCB-Studio] Maou Gakuin no Futekigousha S2 [Ma10p_1080p]",
            None,
            "[VCB-Studio] Maou Gakuin no Futekigousha [Ma10p_1080p]",
            2,
        ),
        ("一拳超人(第三季)", None, "一拳超人", 3),
        (
            "卡片戰鬥!! 先導者 Divinez 第五季「幻真星戰篇」",
            None,
            "卡片戰鬥!! 先導者 Divinez「幻真星戰篇」",
            5,
        ),
        (
            "歡迎來到實力至上主義的教室 第四季 2年級篇 第一學期",
            Some("Season 1"),
            "歡迎來到實力至上主義的教室 2年級篇 第一學期",
            4,
        ),
        (
            "青之壬生浪 第二季 芹澤暗殺篇",
            None,
            "青之壬生浪 芹澤暗殺篇",
            2,
        ),
    ];
    let mut records = Vec::new();
    for (root, directory, _, _) in &cases {
        let path = directory
            .as_ref()
            .map_or_else(
                || target.join(root),
                |directory| target.join(root).join(directory),
            )
            .join("01 [1080p].mkv");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"video").unwrap();
        records.push(record(&target, &path));
    }
    LibraryIndex::rebuild(&target, &records).unwrap();

    let plan = build_layout_plan(&target, &directory.path().join("plan.json"), &|_| {}).unwrap();
    for (root, directory, expected_root, expected_season) in cases {
        let source = directory.map_or_else(
            || format!("{root}/01 [1080p].mkv"),
            |directory| format!("{root}/{directory}/01 [1080p].mkv"),
        );
        let action = plan
            .actions
            .iter()
            .find(|action| action.source == source)
            .unwrap_or_else(|| panic!("missing plan action for {source}"));
        assert_eq!(action.kind, LayoutActionKind::Move);
        let expected_target = format!("{expected_root}/Season {expected_season}/01 [1080p].mkv");
        assert_eq!(action.target.as_deref(), Some(expected_target.as_str()));
    }
}

#[test]
fn planner_preserves_ambiguous_roman_title_in_season_directory() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("library");
    let path = target
        .join("Ace of Diamond Act II")
        .join("Season 2")
        .join("01 [1080p].mkv");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"video").unwrap();
    LibraryIndex::rebuild(&target, &[record(&target, &path)]).unwrap();

    let plan = build_layout_plan(&target, &directory.path().join("plan.json"), &|_| {}).unwrap();
    let action = plan
        .actions
        .iter()
        .find(|action| action.source == "Ace of Diamond Act II/Season 2/01 [1080p].mkv")
        .unwrap();
    assert_eq!(action.kind, LayoutActionKind::Keep);
    assert_eq!(
        action.target.as_deref(),
        Some("Ace of Diamond Act II/Season 2/01 [1080p].mkv")
    );
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
    let source_subtitle = second_episode.with_file_name("02 [1080p].zh-CN.ass");
    fs::write(&source_subtitle, b"subtitle").unwrap();

    let full_hash = anime_organizer::sha256_file(&season_original).unwrap();
    let mut records = [
        record(&target, &flat_generated),
        record(&target, &season_generated),
        record(&target, &season_original),
        record(&target, &flat_original),
        record(&target, &second_episode),
    ];
    for record in &mut records[..4] {
        record.sha256_full = Some(full_hash.clone());
    }
    LibraryIndex::rebuild(&target, &records).unwrap();
    let plan_path = directory.path().join("layout-plan.json");
    let plan = build_layout_plan(&target, &plan_path, &|_| {}).unwrap();

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
    assert!(old_flat.sidecars.is_empty());
    let unique = plan
        .actions
        .iter()
        .find(|action| action.source == "Show S2/02 [1080p].mkv")
        .unwrap();
    assert!(unique.sha256_full.is_none());
    assert_eq!(unique.sidecars.len(), 1);
    assert_eq!(unique.sidecars[0].kind, SidecarActionKind::Move);
    assert!(unique.sidecars[0].sha256_full.is_none());

    assert!(apply_layout_plan(&target, &plan_path, false, &|_| {}).is_err());
    let applied = apply_layout_plan(&target, &plan_path, true, &|_| {}).unwrap();
    assert_eq!(applied.moved, 1);
    assert_eq!(applied.deduplicated, 3);
    assert_eq!(applied.sidecars_moved, 1);
    assert_eq!(applied.sidecars_deduplicated, 0);
    assert!(season_original.exists());
    assert!(target
        .join("Show")
        .join("Season 2")
        .join("02 [1080p].mkv")
        .exists());
    assert!(target
        .join("Show")
        .join("Season 2")
        .join("02 [1080p].zh-CN.ass")
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
            "SELECT COUNT(*) FROM media_file WHERE sha256_full IS NOT NULL",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        1
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
        "Show/Season 2/02 [1080p].zh-CN.ass"
    );
}
