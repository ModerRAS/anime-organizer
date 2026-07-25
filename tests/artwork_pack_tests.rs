use anime_organizer::library_index::{
    Artwork, ArtworkKind, ExternalProvider, LibraryIndex, LibraryIndexRecord, MLIP_SCHEMA_SQL,
};
use image::{codecs::png::PngEncoder, ExtendedColorType, ImageEncoder};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

fn png(width: u32, height: u32, marker: u8) -> Vec<u8> {
    let pixels = vec![marker; (width * height * 4) as usize];
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(&pixels, width, height, ExtendedColorType::Rgba8)
        .unwrap();
    bytes
}

fn fixture(target: &Path) -> Vec<LibraryIndexRecord> {
    let series = target.join("Pack Show");
    fs::create_dir_all(&series).unwrap();
    let video = series.join("01.mkv");
    fs::write(&video, b"video").unwrap();
    let shared = png(2, 3, 1);
    fs::write(series.join("poster.png"), &shared).unwrap();
    fs::write(series.join("poster-copy.png"), &shared).unwrap();
    let season = png(3, 4, 2);
    fs::write(series.join("season.png"), &season).unwrap();
    fs::write(series.join("invalid-source.png"), &season).unwrap();
    fs::write(series.join("landscape.png"), png(4, 2, 3)).unwrap();
    fs::write(series.join("legacy.jpg"), b"legacy path only").unwrap();

    let mut invalid_source = Artwork::new(ArtworkKind::Logo, "Pack Show/invalid-source.png");
    invalid_source.source_provider = Some(ExternalProvider::Tmdb);
    let mut record = LibraryIndexRecord::from_target_path(target, &video)
        .unwrap()
        .unwrap();
    record.series_artwork = vec![
        Artwork::new(ArtworkKind::Poster, "Pack Show/poster.png").with_source(
            ExternalProvider::Bangumi,
            123,
            Some("https://example.test/poster.png".to_string()),
            Some("2026-07-16T00:00:00Z".to_string()),
        ),
        Artwork::new(ArtworkKind::Thumb, "Pack Show/poster-copy.png"),
        Artwork::new(ArtworkKind::SeasonPoster, "Pack Show/season.png"),
        invalid_source,
        Artwork::new(ArtworkKind::Poster, "Pack Show/landscape.png"),
        Artwork::new(ArtworkKind::Poster, "Pack Show/legacy.jpg"),
    ];
    vec![record]
}

fn pack_path(target: &Path) -> PathBuf {
    let entries = fs::read_dir(target.join("MLIP-Artwork"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 1);
    entries[0].clone()
}

#[test]
fn v3_migration_preserves_bindings_and_publishes_verified_packs() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path();
    let series = target.join("Legacy Show");
    fs::create_dir_all(&series).unwrap();
    fs::write(series.join("01.mkv"), b"video").unwrap();
    fs::write(series.join("poster.jpg"), png(2, 3, 9)).unwrap();

    let db_path = target.join("library.db");
    let conn = Connection::open(&db_path).unwrap();
    conn.execute_batch(MLIP_SCHEMA_SQL).unwrap();
    conn.execute(
        "INSERT INTO series (uuid, title) VALUES ('legacy-series', 'Legacy Show')",
        [],
    )
    .unwrap();
    let series_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO episode (uuid, series_id, season, episode, sort_order) \
         VALUES ('legacy-episode', ?1, 1, 1, 1)",
        [series_id],
    )
    .unwrap();
    let episode_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO media_file (episode_id, path) VALUES (?1, 'Legacy Show/01.mkv')",
        [episode_id],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO series_artwork (series_id, artwork_kind, path) \
         VALUES (?1, ?2, 'Legacy Show/poster.jpg')",
        [series_id, 1],
    )
    .unwrap();
    drop(conn);

    let stats = LibraryIndex::migrate_v3_to_v4(target).unwrap();
    assert_eq!((stats.series, stats.episodes, stats.media_files), (1, 1, 1));

    let conn = Connection::open(db_path).unwrap();
    assert_eq!(
        conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        4
    );
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM artwork_pack", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert!(conn
        .query_row(
            "SELECT asset_id IS NOT NULL FROM series_artwork \
             WHERE path = 'Legacy Show/poster.jpg'",
            [],
            |row| row.get::<_, bool>(0),
        )
        .unwrap());
    assert!(target.join("MLIP-Artwork").is_dir());
}

#[test]
fn v4_schema_bindings_offsets_and_content_dedup_are_consistent() {
    let directory = tempfile::tempdir().unwrap();
    let records = fixture(directory.path());
    LibraryIndex::rebuild_v4_staging(directory.path(), &records, &[]).unwrap();

    let conn = Connection::open(directory.path().join("library.db")).unwrap();
    assert_eq!(
        conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        4
    );
    assert_eq!(
        conn.query_row("SELECT value FROM meta WHERE key = 'schema'", [], |row| {
            row.get::<_, String>(0)
        })
        .unwrap(),
        "4"
    );
    assert_eq!(
        conn.query_row(
            "SELECT enabled FROM capability WHERE name = 'artwork_pack'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        1
    );
    for table in ["series_artwork", "episode_artwork"] {
        let columns = conn
            .prepare(&format!("PRAGMA table_info({table})"))
            .unwrap()
            .query_map([], |row| {
                Ok((row.get::<_, String>(1)?, row.get::<_, i64>(3)?))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        for expected in [
            "asset_id",
            "source_url",
            "source_provider",
            "source_subject_id",
            "downloaded_at",
        ] {
            assert!(columns.iter().any(|(name, _)| name == expected));
        }
        assert_eq!(
            columns.iter().find(|(name, _)| name == "path").unwrap().1,
            0
        );
    }
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM artwork_pack", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap(),
        1
    );
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM artwork_asset", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap(),
        2
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(DISTINCT asset_id) FROM series_artwork WHERE asset_id IS NOT NULL",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        2
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM series_artwork WHERE asset_id IS NULL \
             AND path IN ('Pack Show/legacy.jpg', 'Pack Show/landscape.png', 'Pack Show/invalid-source.png')",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        3
    );
    let provenance: (String, i64, String, String) = conn
        .query_row(
            "SELECT source_url, source_provider, source_subject_id, downloaded_at \
             FROM series_artwork WHERE path = 'Pack Show/poster.png'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert_eq!(
        provenance,
        (
            "https://example.test/poster.png".to_string(),
            1,
            "123".to_string(),
            "2026-07-16T00:00:00Z".to_string(),
        )
    );

    let pack = fs::read(pack_path(directory.path())).unwrap();
    let mut statement = conn
        .prepare("SELECT member_name, data_offset, byte_length FROM artwork_asset ORDER BY sha256")
        .unwrap();
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)? as usize,
                row.get::<_, i64>(2)? as usize,
            ))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    for (member_name, offset, length) in rows {
        assert_eq!(offset % 512, 0);
        assert!(!member_name.contains('/'));
        let member = &pack[offset..offset + length];
        assert!(member == png(2, 3, 1) || member == png(3, 4, 2));
        let header_name = &pack[offset - 512..offset - 412];
        assert_eq!(
            header_name.split(|byte| *byte == 0).next().unwrap(),
            member_name.as_bytes()
        );
    }
}

#[test]
fn pack_bytes_are_deterministic_and_unchanged_rebuild_reuses_the_pack() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let first_records = fixture(first.path());
    let second_records = fixture(second.path());
    LibraryIndex::rebuild_v4_staging(first.path(), &first_records, &[]).unwrap();
    LibraryIndex::rebuild_v4_staging(second.path(), &second_records, &[]).unwrap();

    let first_pack = pack_path(first.path());
    let second_pack = pack_path(second.path());
    assert_eq!(first_pack.file_name(), second_pack.file_name());
    let expected_bytes = fs::read(&first_pack).unwrap();
    assert_eq!(expected_bytes, fs::read(&second_pack).unwrap());
    let modified = fs::metadata(&first_pack).unwrap().modified().unwrap();

    LibraryIndex::rebuild_v4_staging(first.path(), &first_records, &[]).unwrap();

    assert_eq!(pack_path(first.path()), first_pack);
    assert_eq!(fs::read(&first_pack).unwrap(), expected_bytes);
    assert_eq!(
        fs::metadata(&first_pack).unwrap().modified().unwrap(),
        modified
    );
}

#[test]
fn v4_incremental_update_preserves_existing_catalog_and_adds_one_pack() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path();
    let records = fixture(target);
    LibraryIndex::rebuild(target, &records).unwrap();

    let existing_pack = pack_path(target);
    let existing_pack_name = existing_pack.file_name().unwrap().to_owned();
    let existing_pack_bytes = fs::read(&existing_pack).unwrap();

    let video = target.join("Pack Show/02.mkv");
    fs::write(&video, b"video 2").unwrap();
    fs::write(target.join("Pack Show/episode-02.png"), png(5, 4, 4)).unwrap();
    let mut record = LibraryIndexRecord::from_target_path(target, &video)
        .unwrap()
        .unwrap();
    record.episode_artwork = vec![Artwork::new(ArtworkKind::Thumb, "Pack Show/episode-02.png")];

    LibraryIndex::update(target, &[record]).unwrap();

    let conn = Connection::open(target.join("library.db")).unwrap();
    assert_eq!(
        conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        4
    );
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM artwork_pack", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM artwork_asset", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM media_file", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM episode_artwork WHERE asset_id IS NOT NULL",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        1
    );
    assert_eq!(
        fs::read(target.join("MLIP-Artwork").join(existing_pack_name)).unwrap(),
        existing_pack_bytes
    );
}

#[test]
fn checked_shared_fixture_covers_base_and_incremental_contracts() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mlip-v4");
    let base = Connection::open(fixture.join("base/library.db")).unwrap();
    let incremental = Connection::open(fixture.join("incremental/library.db")).unwrap();
    assert_eq!(
        base.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_eq!(
        incremental
            .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_eq!(
        base.query_row("SELECT COUNT(*) FROM artwork_pack", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        base.query_row("SELECT COUNT(*) FROM artwork_asset", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        base.query_row("SELECT COUNT(*) FROM series_artwork", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        base.query_row("SELECT COUNT(*) FROM episode_artwork", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        incremental
            .query_row("SELECT COUNT(*) FROM artwork_pack", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert_eq!(
        incremental
            .query_row("SELECT COUNT(*) FROM artwork_asset", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        3
    );
    let base_hash = base
        .query_row("SELECT sha256 FROM artwork_pack", [], |row| {
            row.get::<_, String>(0)
        })
        .unwrap();
    assert_eq!(
        incremental
            .query_row(
                "SELECT COUNT(*) FROM artwork_pack WHERE sha256 = ?1",
                [&base_hash],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
}

#[test]
fn missing_or_corrupt_reused_pack_never_replaces_the_database() {
    let directory = tempfile::tempdir().unwrap();
    let records = fixture(directory.path());
    LibraryIndex::rebuild_v4_staging(directory.path(), &records, &[]).unwrap();
    let database_path = directory.path().join("library.db");
    let database = fs::read(&database_path).unwrap();
    let pack = pack_path(directory.path());
    let pack_bytes = fs::read(&pack).unwrap();

    fs::write(&pack, b"corrupt").unwrap();
    assert!(LibraryIndex::rebuild_v4_staging(directory.path(), &records, &[]).is_err());
    assert_eq!(fs::read(&database_path).unwrap(), database);

    fs::write(&pack, &pack_bytes).unwrap();
    fs::remove_file(&pack).unwrap();
    assert!(LibraryIndex::rebuild_v4_staging(directory.path(), &records, &[]).is_err());
    assert_eq!(fs::read(&database_path).unwrap(), database);
}
