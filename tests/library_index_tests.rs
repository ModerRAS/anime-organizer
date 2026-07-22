use anime_organizer::library_index::{
    ArtworkKind, ExternalId, ExternalProvider, ExtraKind, LibraryExtraRecord, LibraryIndex,
    LibraryIndexRecord, ReleaseDate,
};
use rusqlite::Connection;
use std::fs;

fn table_names(conn: &Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master \
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%' \
             ORDER BY name",
        )
        .unwrap();
    stmt.query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

#[test]
fn rebuild_creates_mlip_v3_schema_with_real_foreign_keys() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path();
    fs::create_dir_all(target.join("Test Show")).unwrap();
    fs::write(target.join("Test Show").join("01 [1080P].mkv"), b"video").unwrap();

    let record = LibraryIndexRecord::from_target_path(
        target,
        &target.join("Test Show").join("01 [1080P].mkv"),
    )
    .unwrap()
    .unwrap();

    LibraryIndex::rebuild(target, &[record]).unwrap();

    let db_path = target.join("library.db");
    assert!(db_path.exists());

    let conn = Connection::open(db_path).unwrap();
    let user_version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(user_version, 3);
    let schema: String = conn
        .query_row("SELECT value FROM meta WHERE key = 'schema'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(schema, "3");

    assert_eq!(
        table_names(&conn),
        vec![
            "capability",
            "episode",
            "episode_artwork",
            "episode_external_id",
            "genre",
            "media_extra",
            "media_file",
            "media_subtitle",
            "meta",
            "series",
            "series_artwork",
            "series_external_id",
            "series_genre",
            "series_release_date",
        ]
    );

    let schema_sql: String = conn
        .prepare("SELECT sql FROM sqlite_master WHERE sql IS NOT NULL")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    assert!(!schema_sql.contains("owner_type"));

    conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
    let invalid_episode = conn.execute(
        "INSERT INTO episode (uuid, series_id, season, episode, sort_order) \
         VALUES ('bad-episode', 999, 1, 1.0, 1.0)",
        [],
    );
    assert!(invalid_episode.is_err());

    let invalid_media = conn.execute(
        "INSERT INTO media_file (episode_id, path) VALUES (999, 'missing.mkv')",
        [],
    );
    assert!(invalid_media.is_err());
}

#[test]
fn rebuild_exports_external_subtitle_relations() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path();
    let series = target.join("Subtitle Show");
    fs::create_dir_all(&series).unwrap();
    let video = series.join("01 [1080P].mkv");
    fs::write(&video, b"video").unwrap();
    fs::write(series.join("01 [1080P].zh-CN.ass"), b"subtitle").unwrap();
    fs::write(series.join("010 [1080P].srt"), b"other").unwrap();

    let record = LibraryIndexRecord::from_target_path(target, &video)
        .unwrap()
        .unwrap();
    assert_eq!(
        record.subtitle_paths,
        vec!["Subtitle Show/01 [1080P].zh-CN.ass"]
    );
    LibraryIndex::rebuild(target, &[record]).unwrap();

    let conn = Connection::open(target.join("library.db")).unwrap();
    let subtitle_path: String = conn
        .query_row("SELECT path FROM media_subtitle", [], |row| row.get(0))
        .unwrap();
    let subtitle_capability: i64 = conn
        .query_row(
            "SELECT enabled FROM capability WHERE name = 'subtitle'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(subtitle_path, "Subtitle Show/01 [1080P].zh-CN.ass");
    assert_eq!(subtitle_capability, 1);
}

#[test]
fn incremental_update_stages_locally_and_replaces_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path();
    let series = target.join("Incremental Show");
    fs::create_dir_all(&series).unwrap();

    let first_path = series.join("01 [1080P].mkv");
    fs::write(&first_path, b"one").unwrap();
    let first = LibraryIndexRecord::from_target_path(target, &first_path)
        .unwrap()
        .unwrap();
    LibraryIndex::rebuild(target, &[first]).unwrap();

    let second_path = series.join("02 [1080P].mkv");
    fs::write(&second_path, b"two").unwrap();
    let second = LibraryIndexRecord::from_target_path(target, &second_path)
        .unwrap()
        .unwrap();
    let stats = LibraryIndex::update(target, &[second]).unwrap();
    assert_eq!(stats.media_files, 2);

    let conn = Connection::open(target.join("library.db")).unwrap();
    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .unwrap();
    assert_eq!(integrity, "ok");
    drop(conn);

    let leftovers: Vec<_> = fs::read_dir(target)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".library.db")
        })
        .collect();
    assert!(leftovers.is_empty());
}

#[test]
fn target_path_parser_reads_flat_and_season_layouts() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path();
    let flat = target.join("Flat Show").join("01 [1080P].mkv");
    let seasonal = target
        .join("Seasonal Show")
        .join("Season 2")
        .join("03.5 [WEB-DL].mkv");
    let raw_in_season = target
        .join("Seasonal Show")
        .join("Season 2")
        .join("[ANi] Wrong Filename S3 - 04 [1080P].mp4");
    let title_season = target
        .join("終究，與你相戀。第二季")
        .join("13 [1080P][Baha].mp4");
    fs::create_dir_all(flat.parent().unwrap()).unwrap();
    fs::create_dir_all(seasonal.parent().unwrap()).unwrap();
    fs::create_dir_all(title_season.parent().unwrap()).unwrap();
    fs::write(&flat, b"flat").unwrap();
    fs::write(&seasonal, b"seasonal").unwrap();
    fs::write(&raw_in_season, b"raw-seasonal").unwrap();
    fs::write(&title_season, b"title-season").unwrap();

    let flat_record = LibraryIndexRecord::from_target_path(target, &flat)
        .unwrap()
        .unwrap();
    assert_eq!(flat_record.series_title, "Flat Show");
    assert_eq!(flat_record.season, 1);
    assert_eq!(flat_record.episode, 1.0);
    assert_eq!(flat_record.relative_path, "Flat Show/01 [1080P].mkv");

    let seasonal_record = LibraryIndexRecord::from_target_path(target, &seasonal)
        .unwrap()
        .unwrap();
    assert_eq!(seasonal_record.series_title, "Seasonal Show");
    assert_eq!(seasonal_record.season, 2);
    assert_eq!(seasonal_record.episode, 3.5);
    assert_eq!(
        seasonal_record.relative_path,
        "Seasonal Show/Season 2/03.5 [WEB-DL].mkv"
    );

    let raw_in_season_record = LibraryIndexRecord::from_target_path(target, &raw_in_season)
        .unwrap()
        .unwrap();
    assert_eq!(raw_in_season_record.series_title, "Seasonal Show");
    assert_eq!(raw_in_season_record.season, 2);
    assert_eq!(raw_in_season_record.episode, 4.0);

    let title_season_record = LibraryIndexRecord::from_target_path(target, &title_season)
        .unwrap()
        .unwrap();
    assert_eq!(title_season_record.series_title, "終究，與你相戀。第二季");
    assert_eq!(title_season_record.season, 2);
    assert_eq!(title_season_record.episode, 13.0);
}

#[test]
fn target_scan_skips_known_supplemental_video_directories() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path();
    let collection = target.join("Boruto Collection");
    let main = collection.join(
        "[DBD-Raws][Boruto Naruto Next Generations][001][1080P][BDRip][HEVC-10bit][FLAC].mkv",
    );
    fs::create_dir_all(&collection).unwrap();
    fs::write(&main, b"main").unwrap();
    assert!(LibraryIndexRecord::from_target_path(target, &main)
        .unwrap()
        .is_some());

    for directory in ["menu", "NCOP&NCED", "图集", "特典映像"] {
        let extra = collection
            .join(directory)
            .join("[DBD-Raws][Boruto Naruto Next Generations][Images][01][1080P].mkv");
        fs::create_dir_all(extra.parent().unwrap()).unwrap();
        fs::write(&extra, b"extra").unwrap();
        assert!(
            LibraryIndexRecord::from_target_path(target, &extra)
                .unwrap()
                .is_none(),
            "{directory} should be skipped"
        );
    }
    let root_menu =
        collection.join("[DBD-Raws][Boruto Naruto Next Generations][menu][S5][D2][01].mkv");
    fs::write(&root_menu, b"menu").unwrap();
    assert!(LibraryIndexRecord::from_target_path(target, &root_menu)
        .unwrap()
        .is_none());
}

#[test]
fn rebuild_classifies_local_extras_and_ignores_disc_menus() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path();
    let collection = target.join("Boruto Collection");
    let main_path =
        collection.join("[DBD-Raws][Boruto Naruto Next Generations][001][1080P][BDRip].mkv");
    fs::create_dir_all(&collection).unwrap();
    fs::write(&main_path, b"main").unwrap();
    let main = LibraryIndexRecord::from_target_path(target, &main_path)
        .unwrap()
        .unwrap();

    let paths = [
        (
            "[DBD-Raws][Boruto Naruto Next Generations][OVA][1080P].mkv",
            ExtraKind::Ova,
            "OVA",
        ),
        (
            "[DBD-Raws][Boruto Naruto Next Generations][OVA][02][1080P].mkv",
            ExtraKind::Ova,
            "OVA 02",
        ),
        (
            "NCOP&NCED/[DBD-Raws][Boruto Naruto Next Generations][NCOP01][1080P].mkv",
            ExtraKind::Ncop,
            "NCOP 01",
        ),
        (
            "NCOP&NCED/[DBD-Raws][Boruto Naruto Next Generations][NCED10][1080P].mkv",
            ExtraKind::Nced,
            "NCED 10",
        ),
        (
            "特典映像/[DBD-Raws][Boruto Naruto Next Generations][Tokuten][08][1080P].mkv",
            ExtraKind::Special,
            "特典映像 08",
        ),
        (
            "图集/[DBD-Raws][Boruto Naruto Next Generations][Images][39][1080P].mkv",
            ExtraKind::Gallery,
            "图集 39",
        ),
    ];
    let mut extras = Vec::new();
    for (relative, kind, title) in paths {
        let path = collection.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"extra").unwrap();
        let extra = LibraryExtraRecord::from_target_path(target, &path, main.series_title.clone())
            .unwrap()
            .unwrap();
        assert_eq!(extra.kind, kind);
        assert_eq!(extra.title, title);
        extras.push(extra);
    }
    let menu = collection
        .join("menu")
        .join("[DBD-Raws][Boruto Naruto Next Generations][menu][S5][D2][01].mkv");
    fs::create_dir_all(menu.parent().unwrap()).unwrap();
    fs::write(&menu, b"menu").unwrap();
    assert!(
        LibraryExtraRecord::from_target_path(target, &menu, main.series_title.clone())
            .unwrap()
            .is_none()
    );

    let stats = LibraryIndex::rebuild_with_extras(target, &[main], &extras).unwrap();
    assert_eq!(stats.extras, 6);
    let conn = Connection::open(target.join("library.db")).unwrap();
    let rows = conn
        .prepare("SELECT extra_kind, ordinal, title FROM media_extra ORDER BY extra_kind, ordinal")
        .unwrap()
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(rows.len(), 6);
    assert_eq!(rows[0], (1, 1, "OVA".to_string()));
    assert_eq!(rows[1], (1, 2, "OVA 02".to_string()));
}

#[test]
fn incremental_update_keeps_series_identity_after_metadata_title_change() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path();
    let collection = target.join("原始合集");
    fs::create_dir_all(&collection).unwrap();
    let first_path = collection.join("01.mkv");
    let extra_path = collection.join("[Group][Show][OVA].mkv");
    fs::write(&first_path, b"one").unwrap();
    fs::write(&extra_path, b"extra").unwrap();
    let first = LibraryIndexRecord::from_target_path(target, &first_path)
        .unwrap()
        .unwrap();
    let extra =
        LibraryExtraRecord::from_target_path(target, &extra_path, first.series_title.clone())
            .unwrap()
            .unwrap();
    LibraryIndex::rebuild_with_extras(target, &[first], &[extra]).unwrap();

    let second_path = collection.join("02.mkv");
    fs::write(&second_path, b"two").unwrap();
    let mut second = LibraryIndexRecord::from_target_path(target, &second_path)
        .unwrap()
        .unwrap();
    second.series_title = "规范标题".to_string();
    second.external_ids = vec![ExternalId::new(ExternalProvider::Bangumi, 123)];
    LibraryIndex::update(target, &[second]).unwrap();

    let conn = Connection::open(target.join("library.db")).unwrap();
    let series_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM series", [], |row| row.get(0))
        .unwrap();
    let title: String = conn
        .query_row("SELECT title FROM series", [], |row| row.get(0))
        .unwrap();
    let extra_owner: String = conn
        .query_row(
            "SELECT series.title FROM media_extra INNER JOIN series ON series.id = media_extra.series_id",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(series_count, 1);
    assert_eq!(title, "规范标题");
    assert_eq!(extra_owner, "规范标题");
}

#[test]
fn confirmed_external_id_merges_series_across_roots() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path();
    let first_path = target.join("First Show").join("Season 1").join("01.mkv");
    let second_path = target.join("Second Show").join("Season 1").join("01.mkv");
    fs::create_dir_all(first_path.parent().unwrap()).unwrap();
    fs::create_dir_all(second_path.parent().unwrap()).unwrap();
    fs::write(&first_path, b"first").unwrap();
    fs::write(&second_path, b"second").unwrap();

    let mut first = LibraryIndexRecord::from_target_path(target, &first_path)
        .unwrap()
        .unwrap();
    let mut second = LibraryIndexRecord::from_target_path(target, &second_path)
        .unwrap()
        .unwrap();
    for record in [&mut first, &mut second] {
        record.series_title = "Incorrect shared metadata".to_string();
        record.external_ids = vec![ExternalId::new(ExternalProvider::Bangumi, 123)];
    }

    LibraryIndex::rebuild(target, &[first, second]).unwrap();

    let conn = Connection::open(target.join("library.db")).unwrap();
    let series_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM series", [], |row| row.get(0))
        .unwrap();
    let episode_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM episode", [], |row| row.get(0))
        .unwrap();
    let media_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM media_file", [], |row| row.get(0))
        .unwrap();
    assert_eq!(series_count, 1);
    assert_eq!(episode_count, 1);
    assert_eq!(media_count, 2);
}

#[test]
fn release_date_requires_a_valid_iso_calendar_date() {
    assert_eq!(
        ReleaseDate::parse_iso("2024-02-29").unwrap().to_string(),
        "2024-02-29"
    );
    assert!(ReleaseDate::parse_iso("2024-02-30").is_none());
    assert!(ReleaseDate::parse_iso("2024").is_none());
    assert!(ReleaseDate::parse_iso("2024-2-03-extra").is_none());
}

#[test]
fn incremental_update_adds_optional_tables_to_legacy_v1_database() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path();
    let media_path = target.join("Legacy Show").join("01.mkv");
    fs::create_dir_all(media_path.parent().unwrap()).unwrap();
    fs::write(&media_path, b"video").unwrap();

    LibraryIndex::rebuild(target, &[]).unwrap();
    let legacy = Connection::open(target.join("library.db")).unwrap();
    legacy
        .execute_batch(
            "DROP TABLE series_release_date; \
         DROP TABLE media_subtitle; \
         DROP TABLE media_extra; \
         PRAGMA user_version = 1; \
         UPDATE meta SET value = '1' WHERE key = 'schema';",
        )
        .unwrap();
    drop(legacy);
    fs::write(target.join("Legacy Show").join("01.zh-CN.ass"), b"subtitle").unwrap();

    let mut record = LibraryIndexRecord::from_target_path(target, &media_path)
        .unwrap()
        .unwrap();
    record.air_date = ReleaseDate::parse_iso("2024-07-05");
    LibraryIndex::update(target, &[record]).unwrap();

    let conn = Connection::open(target.join("library.db")).unwrap();
    let air_date: String = conn
        .query_row("SELECT air_date FROM series_release_date", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(air_date, "2024-07-05");
    let subtitle_path: String = conn
        .query_row("SELECT path FROM media_subtitle", [], |row| row.get(0))
        .unwrap();
    assert_eq!(subtitle_path, "Legacy Show/01.zh-CN.ass");
    let user_version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    let schema: String = conn
        .query_row("SELECT value FROM meta WHERE key = 'schema'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(user_version, 3);
    assert_eq!(schema, "3");
}

#[test]
fn records_store_metadata_genres_external_ids_and_artwork() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path();
    fs::create_dir_all(target.join("Meta Show")).unwrap();
    fs::write(target.join("Meta Show").join("01 [1080P].mkv"), b"video").unwrap();
    fs::write(target.join("Meta Show").join("poster.jpg"), b"poster").unwrap();

    let mut record = LibraryIndexRecord::from_target_path(
        target,
        &target.join("Meta Show").join("01 [1080P].mkv"),
    )
    .unwrap()
    .unwrap();
    record.original_title = Some("Original Meta Show".to_string());
    record.summary = Some("Summary".to_string());
    record.year = Some(2026);
    record.air_date = ReleaseDate::parse_iso("2026-04-03");
    record.genres = vec![
        "Action".to_string(),
        "Action".to_string(),
        "Comedy".to_string(),
    ];
    record.external_ids = vec![
        ExternalId::new(ExternalProvider::Bangumi, "123"),
        ExternalId::new(ExternalProvider::Tmdb, "456"),
        ExternalId::new(ExternalProvider::Anidb, "789"),
    ];
    record.series_artwork = vec![anime_organizer::library_index::Artwork::new(
        ArtworkKind::Poster,
        "Meta Show/poster.jpg",
    )];

    LibraryIndex::rebuild(target, &[record]).unwrap();

    let conn = Connection::open(target.join("library.db")).unwrap();
    let genre_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM genre", [], |row| row.get(0))
        .unwrap();
    assert_eq!(genre_count, 2);

    let external_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM series_external_id", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(external_count, 3);

    let air_date: String = conn
        .query_row("SELECT air_date FROM series_release_date", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(air_date, "2026-04-03");

    let artwork_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM series_artwork", [], |row| row.get(0))
        .unwrap();
    assert_eq!(artwork_count, 1);
}
