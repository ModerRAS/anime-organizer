use anime_organizer::library_index::{
    ArtworkKind, ExternalId, ExternalProvider, LibraryIndex, LibraryIndexRecord,
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
fn rebuild_creates_mlip_v1_schema_with_real_foreign_keys() {
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
    assert_eq!(user_version, 1);

    assert_eq!(
        table_names(&conn),
        vec![
            "capability",
            "episode",
            "episode_artwork",
            "episode_external_id",
            "genre",
            "media_file",
            "meta",
            "series",
            "series_artwork",
            "series_external_id",
            "series_genre",
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
fn target_path_parser_reads_flat_and_season_layouts() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path();
    let flat = target.join("Flat Show").join("01 [1080P].mkv");
    let seasonal = target
        .join("Seasonal Show")
        .join("Season 2")
        .join("03.5 [WEB-DL].mkv");
    fs::create_dir_all(flat.parent().unwrap()).unwrap();
    fs::create_dir_all(seasonal.parent().unwrap()).unwrap();
    fs::write(&flat, b"flat").unwrap();
    fs::write(&seasonal, b"seasonal").unwrap();

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

    let artwork_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM series_artwork", [], |row| row.get(0))
        .unwrap();
    assert_eq!(artwork_count, 1);
}
