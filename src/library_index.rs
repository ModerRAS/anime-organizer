//! Media Library Index Protocol (MLIP) SQLite writer.
//!
//! The generated database is a read-only protocol artifact for players. User
//! state such as playback history or favorites belongs in the player database.

use crate::artwork_pack::{build_and_publish, build_and_publish_artwork, ArtworkPacking};
use crate::error::{AppError, Result};
use crate::organizer::FileOrganizer;
use crate::parser::{split_series_and_season, AnimeFileInfo, FilenameParser};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::{HashMap, HashSet};
use std::io::{BufReader, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::UNIX_EPOCH;
use time::format_description::well_known::Rfc3339;
use time::{Date, Month, OffsetDateTime};
use uuid::Uuid;

/// Fixed MLIP database filename in the target library root.
pub const DATABASE_FILENAME: &str = "library.db";

const MLIP_NAMESPACE: Uuid = Uuid::from_u128(0x3f1a60c1_0f29_4f54_96bd_2068841e14c1);
static STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);

/// MLIP v1-v3 compatibility schema, extended by the v4 artifact tables below.
pub const MLIP_SCHEMA_SQL: &str = r#"
PRAGMA foreign_keys = ON;

CREATE TABLE meta
(
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);

CREATE TABLE series
(
    id              INTEGER PRIMARY KEY,
    uuid            TEXT UNIQUE NOT NULL,

    title           TEXT NOT NULL,
    original_title  TEXT,
    sort_title      TEXT,
    summary         TEXT,
    year            INTEGER,

    series_type     INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_series_title
ON series(title);

CREATE TABLE series_release_date
(
    series_id   INTEGER PRIMARY KEY,
    air_date    TEXT NOT NULL,

    FOREIGN KEY(series_id)
        REFERENCES series(id)
        ON DELETE CASCADE
);

CREATE TABLE episode
(
    id          INTEGER PRIMARY KEY,
    uuid        TEXT UNIQUE NOT NULL,

    series_id   INTEGER NOT NULL,
    season      INTEGER NOT NULL DEFAULT 1,
    episode     REAL NOT NULL,
    sort_order  REAL NOT NULL,

    title       TEXT,
    summary     TEXT,
    runtime     INTEGER,

    FOREIGN KEY(series_id)
        REFERENCES series(id)
        ON DELETE CASCADE,

    UNIQUE(series_id, season, episode)
);

CREATE INDEX idx_episode_series
ON episode(series_id);

CREATE TABLE media_file
(
    id              INTEGER PRIMARY KEY,

    episode_id      INTEGER NOT NULL,
    path                TEXT NOT NULL UNIQUE,
    size                INTEGER,
    modified_time       INTEGER,
    sha256_prefix_63m   TEXT CHECK (
        sha256_prefix_63m IS NULL OR length(sha256_prefix_63m) = 64
    ),
    sha256_full         TEXT CHECK (
        sha256_full IS NULL OR length(sha256_full) = 64
    ),

    FOREIGN KEY(episode_id)
        REFERENCES episode(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_media_path
ON media_file(path);

CREATE INDEX idx_media_episode
ON media_file(episode_id);

CREATE TABLE media_subtitle
(
    id              INTEGER PRIMARY KEY,
    media_file_id   INTEGER NOT NULL,
    path            TEXT NOT NULL,
    language        TEXT,
    title           TEXT,
    sort_order      INTEGER NOT NULL DEFAULT 0,

    FOREIGN KEY(media_file_id)
        REFERENCES media_file(id)
        ON DELETE CASCADE,

    UNIQUE(media_file_id, path)
);

CREATE INDEX idx_media_subtitle_file
ON media_subtitle(media_file_id);

CREATE TABLE media_extra
(
    id              INTEGER PRIMARY KEY,
    uuid            TEXT UNIQUE NOT NULL,
    series_id       INTEGER NOT NULL,
    extra_kind      INTEGER NOT NULL,
    ordinal         INTEGER NOT NULL,
    sort_order      INTEGER NOT NULL,
    title           TEXT NOT NULL,
    path            TEXT NOT NULL UNIQUE,
    size            INTEGER,
    modified_time   INTEGER,
    runtime         INTEGER,

    FOREIGN KEY(series_id)
        REFERENCES series(id)
        ON DELETE CASCADE
);

CREATE INDEX idx_media_extra_series
ON media_extra(series_id, extra_kind, sort_order);

CREATE TABLE series_artwork
(
    id              INTEGER PRIMARY KEY,
    series_id       INTEGER NOT NULL,
    artwork_kind    INTEGER NOT NULL,
    path            TEXT NOT NULL,

    FOREIGN KEY(series_id)
        REFERENCES series(id)
        ON DELETE CASCADE,

    UNIQUE(series_id, artwork_kind, path)
);

CREATE INDEX idx_series_artwork_series
ON series_artwork(series_id);

CREATE TABLE episode_artwork
(
    id              INTEGER PRIMARY KEY,
    episode_id      INTEGER NOT NULL,
    artwork_kind    INTEGER NOT NULL,
    path            TEXT NOT NULL,

    FOREIGN KEY(episode_id)
        REFERENCES episode(id)
        ON DELETE CASCADE,

    UNIQUE(episode_id, artwork_kind, path)
);

CREATE INDEX idx_episode_artwork_episode
ON episode_artwork(episode_id);

CREATE TABLE genre
(
    id      INTEGER PRIMARY KEY,
    name    TEXT UNIQUE NOT NULL
);

CREATE TABLE series_genre
(
    series_id   INTEGER NOT NULL,
    genre_id    INTEGER NOT NULL,

    PRIMARY KEY(series_id, genre_id),

    FOREIGN KEY(series_id)
        REFERENCES series(id)
        ON DELETE CASCADE,

    FOREIGN KEY(genre_id)
        REFERENCES genre(id)
        ON DELETE CASCADE
);

CREATE TABLE series_external_id
(
    series_id   INTEGER NOT NULL,
    provider    INTEGER NOT NULL,
    value       TEXT NOT NULL,

    PRIMARY KEY(series_id, provider, value),

    FOREIGN KEY(series_id)
        REFERENCES series(id)
        ON DELETE CASCADE
);

CREATE TABLE episode_external_id
(
    episode_id  INTEGER NOT NULL,
    provider    INTEGER NOT NULL,
    value       TEXT NOT NULL,

    PRIMARY KEY(episode_id, provider, value),

    FOREIGN KEY(episode_id)
        REFERENCES episode(id)
        ON DELETE CASCADE
);

CREATE TABLE capability
(
    name        TEXT PRIMARY KEY,
    enabled     INTEGER NOT NULL
);

PRAGMA user_version = 3;
"#;

/// MLIP v4 immutable artwork-pack schema extension.
pub const MLIP_V4_SCHEMA_SQL: &str = r#"
CREATE TABLE artwork_pack
(
    id              INTEGER PRIMARY KEY,
    sha256          TEXT UNIQUE NOT NULL CHECK(length(sha256) = 64),
    path            TEXT UNIQUE NOT NULL,
    byte_length     INTEGER NOT NULL CHECK(byte_length >= 1024),
    asset_count     INTEGER NOT NULL CHECK(asset_count > 0)
);

CREATE TABLE artwork_asset
(
    id              INTEGER PRIMARY KEY,
    sha256          TEXT UNIQUE NOT NULL CHECK(length(sha256) = 64),
    pack_id         INTEGER NOT NULL,
    member_name     TEXT NOT NULL,
    data_offset     INTEGER NOT NULL CHECK(data_offset >= 512 AND data_offset % 512 = 0),
    byte_length     INTEGER NOT NULL CHECK(byte_length > 0),
    media_type      TEXT NOT NULL,
    width           INTEGER NOT NULL CHECK(width > 0),
    height          INTEGER NOT NULL CHECK(height > 0),

    FOREIGN KEY(pack_id)
        REFERENCES artwork_pack(id)
        ON DELETE RESTRICT,

    UNIQUE(pack_id, member_name)
);

DROP INDEX idx_series_artwork_series;
ALTER TABLE series_artwork RENAME TO series_artwork_v3;
CREATE TABLE series_artwork
(
    id                  INTEGER PRIMARY KEY,
    series_id           INTEGER NOT NULL,
    artwork_kind        INTEGER NOT NULL,
    path                TEXT,
    asset_id            INTEGER,
    source_url          TEXT,
    source_provider     INTEGER,
    source_subject_id   TEXT,
    downloaded_at       TEXT,

    FOREIGN KEY(series_id) REFERENCES series(id) ON DELETE CASCADE,
    FOREIGN KEY(asset_id) REFERENCES artwork_asset(id) ON DELETE RESTRICT,
    CHECK(path IS NOT NULL OR asset_id IS NOT NULL),
    UNIQUE(series_id, artwork_kind, path)
);
INSERT INTO series_artwork (id, series_id, artwork_kind, path)
SELECT id, series_id, artwork_kind, path FROM series_artwork_v3;
DROP TABLE series_artwork_v3;
CREATE INDEX idx_series_artwork_series ON series_artwork(series_id);
CREATE INDEX idx_series_artwork_asset ON series_artwork(asset_id);

DROP INDEX idx_episode_artwork_episode;
ALTER TABLE episode_artwork RENAME TO episode_artwork_v3;
CREATE TABLE episode_artwork
(
    id                  INTEGER PRIMARY KEY,
    episode_id          INTEGER NOT NULL,
    artwork_kind        INTEGER NOT NULL,
    path                TEXT,
    asset_id            INTEGER,
    source_url          TEXT,
    source_provider     INTEGER,
    source_subject_id   TEXT,
    downloaded_at       TEXT,

    FOREIGN KEY(episode_id) REFERENCES episode(id) ON DELETE CASCADE,
    FOREIGN KEY(asset_id) REFERENCES artwork_asset(id) ON DELETE RESTRICT,
    CHECK(path IS NOT NULL OR asset_id IS NOT NULL),
    UNIQUE(episode_id, artwork_kind, path)
);
INSERT INTO episode_artwork (id, episode_id, artwork_kind, path)
SELECT id, episode_id, artwork_kind, path FROM episode_artwork_v3;
DROP TABLE episode_artwork_v3;
CREATE INDEX idx_episode_artwork_episode ON episode_artwork(episode_id);
CREATE INDEX idx_episode_artwork_asset ON episode_artwork(asset_id);

PRAGMA user_version = 4;
"#;

/// MLIP artwork kind integer enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtworkKind {
    Poster = 1,
    Fanart = 2,
    Banner = 3,
    Logo = 4,
    Thumb = 5,
    Clearart = 6,
    SeasonPoster = 7,
}

impl ArtworkKind {
    pub(crate) fn as_i64(self) -> i64 {
        self as i64
    }
}

/// MLIP series-extra kind integer enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtraKind {
    Ova = 1,
    Special = 2,
    Ncop = 3,
    Nced = 4,
    Gallery = 5,
}

impl ExtraKind {
    fn as_i64(self) -> i64 {
        self as i64
    }
}

/// MLIP provider integer enum, shared by external IDs and v4 artwork provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalProvider {
    Bangumi = 1,
    Tmdb = 2,
    Anidb = 3,
}

impl ExternalProvider {
    fn as_i64(self) -> i64 {
        self as i64
    }
}

/// Artwork attached to a series or episode.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Artwork {
    pub kind: ArtworkKind,
    pub path: String,
    pub source_url: Option<String>,
    pub source_provider: Option<ExternalProvider>,
    pub source_subject_id: Option<String>,
    pub downloaded_at: Option<String>,
}

impl Artwork {
    #[must_use]
    pub fn new(kind: ArtworkKind, path: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.into(),
            source_url: None,
            source_provider: None,
            source_subject_id: None,
            downloaded_at: None,
        }
    }

    #[must_use]
    pub fn with_source(
        mut self,
        provider: ExternalProvider,
        subject_id: impl ToString,
        source_url: Option<String>,
        downloaded_at: Option<String>,
    ) -> Self {
        self.source_provider = Some(provider);
        self.source_subject_id = Some(subject_id.to_string());
        self.source_url = source_url;
        self.downloaded_at = downloaded_at;
        self
    }

    pub(crate) fn has_valid_source_identity(&self) -> bool {
        let provider_identity = match (&self.source_provider, &self.source_subject_id) {
            (None, None) => true,
            (Some(_), Some(subject_id)) => !subject_id.trim().is_empty(),
            _ => false,
        };
        provider_identity
            && self
                .source_url
                .as_deref()
                .is_none_or(|url| !url.trim().is_empty())
            && self
                .downloaded_at
                .as_deref()
                .is_none_or(|value| !value.trim().is_empty())
    }
}

/// External provider id attached to a series or episode.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalId {
    pub provider: ExternalProvider,
    pub value: String,
}

impl ExternalId {
    #[must_use]
    pub fn new(provider: ExternalProvider, value: impl ToString) -> Self {
        Self {
            provider,
            value: value.to_string(),
        }
    }
}

/// A validated Gregorian release date stored as MLIP `YYYY-MM-DD`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseDate(Date);

impl ReleaseDate {
    #[must_use]
    pub fn parse_iso(value: &str) -> Option<Self> {
        let mut parts = value.trim().split('-');
        let year = parts.next()?.parse().ok()?;
        let month = Month::try_from(parts.next()?.parse::<u8>().ok()?).ok()?;
        let day = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Date::from_calendar_date(year, month, day).ok().map(Self)
    }
}

impl std::fmt::Display for ReleaseDate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.0.year(),
            u8::from(self.0.month()),
            self.0.day()
        )
    }
}

/// One indexed media file plus its logical series/episode metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryIndexRecord {
    pub series_title: String,
    pub original_title: Option<String>,
    pub sort_title: Option<String>,
    pub summary: Option<String>,
    pub year: Option<i64>,
    pub air_date: Option<ReleaseDate>,
    pub series_type: i64,
    pub season: i64,
    pub episode: f64,
    pub sort_order: f64,
    pub episode_title: Option<String>,
    pub episode_summary: Option<String>,
    pub runtime: Option<i64>,
    pub relative_path: String,
    pub size: Option<i64>,
    pub modified_time: Option<i64>,
    pub sha256_prefix_63m: Option<String>,
    pub sha256_full: Option<String>,
    pub subtitle_paths: Vec<String>,
    pub genres: Vec<String>,
    pub external_ids: Vec<ExternalId>,
    pub series_artwork: Vec<Artwork>,
    pub episode_artwork: Vec<Artwork>,
}

impl LibraryIndexRecord {
    /// Parse an already-organized target path into an MLIP record.
    pub fn from_target_path(target_root: &Path, path: &Path) -> Result<Option<Self>> {
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(DATABASE_FILENAME))
        {
            return Ok(None);
        }

        let relative_path = relative_path(target_root, path)?;
        let relative = path.strip_prefix(target_root).map_err(|_| {
            AppError::LibraryIndexError(format!("媒体文件不在目标目录内: {}", path.display()))
        })?;
        let components = normal_components(relative);
        let Some(file_name) = components.last() else {
            return Ok(None);
        };
        if has_bracket_token(file_name, "menu")
            || components[..components.len() - 1]
                .iter()
                .any(|component| is_supplemental_directory(component))
        {
            return Ok(None);
        }
        let directory_identity = season_directory_identity(&components);

        if let Some(info) = FilenameParser::parse(path) {
            return Self::from_anime_file(target_root, path, &info).map(Some);
        }

        let Some((episode, _tags)) = parse_target_filename(file_name) else {
            return Ok(None);
        };
        let Some((series_title, season)) = directory_identity.or_else(|| {
            let parent = components.get(components.len().checked_sub(2)?)?.clone();
            Some((parent.clone(), title_season_number(&parent).unwrap_or(1)))
        }) else {
            return Ok(None);
        };

        Self::new(series_title, season, episode, relative_path, path)
            .with_external_subtitles(target_root, path)
            .map(Some)
    }

    /// Build an MLIP record from the parser result already selected by the caller.
    pub fn from_anime_file(target_root: &Path, path: &Path, info: &AnimeFileInfo) -> Result<Self> {
        let relative_path = relative_path(target_root, path)?;
        let relative = path.strip_prefix(target_root).map_err(|_| {
            AppError::LibraryIndexError(format!("媒体文件不在目标目录内: {}", path.display()))
        })?;
        let components = normal_components(relative);
        let episode = parse_episode_number(&info.episode)?;
        let (series_title, season) = season_directory_identity(&components).unwrap_or_else(|| {
            (
                info.series_name(),
                i64::from(info.season_number().unwrap_or(1)),
            )
        });
        Self::new(series_title, season, episode, relative_path, path)
            .with_external_subtitles(target_root, path)
    }

    #[must_use]
    pub fn new(
        series_title: String,
        season: i64,
        episode: f64,
        relative_path: String,
        source_path: &Path,
    ) -> Self {
        let (size, modified_time) = file_metadata(source_path);
        Self {
            series_title,
            original_title: None,
            sort_title: None,
            summary: None,
            year: None,
            air_date: None,
            series_type: 1,
            season,
            episode,
            sort_order: episode,
            episode_title: None,
            episode_summary: None,
            runtime: None,
            relative_path,
            size,
            modified_time,
            sha256_prefix_63m: None,
            sha256_full: None,
            subtitle_paths: Vec::new(),
            genres: Vec::new(),
            external_ids: Vec::new(),
            series_artwork: Vec::new(),
            episode_artwork: Vec::new(),
        }
    }

    fn with_external_subtitles(mut self, target_root: &Path, path: &Path) -> Result<Self> {
        self.subtitle_paths = FileOrganizer::find_external_subtitles(path)
            .iter()
            .map(|subtitle_path| relative_path(target_root, subtitle_path))
            .collect::<Result<Vec<_>>>()?;
        Ok(self)
    }

    #[cfg(feature = "metadata")]
    pub fn apply_metadata(&mut self, meta: &crate::metadata::AnimeMetadata) {
        self.series_title = meta
            .title_cn
            .as_deref()
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| meta.title.clone());
        self.original_title = Some(meta.original_title.clone());
        if !meta.summary.is_empty() {
            self.summary = Some(meta.summary.clone());
        }
        self.year = meta.air_date.as_deref().and_then(parse_year).map(i64::from);
        self.air_date = meta.air_date.as_deref().and_then(ReleaseDate::parse_iso);
        self.genres = meta.genre.clone();
        self.external_ids
            .push(ExternalId::new(ExternalProvider::Bangumi, meta.bangumi_id));
        if let Some(tmdb_id) = meta.tmdb_id {
            self.external_ids
                .push(ExternalId::new(ExternalProvider::Tmdb, tmdb_id));
        }
        if let Some(anidb_id) = meta.anidb_id {
            self.external_ids
                .push(ExternalId::new(ExternalProvider::Anidb, anidb_id));
        }
    }
}

/// One locally classified extra attached to a series.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryExtraRecord {
    pub series_title: String,
    pub kind: ExtraKind,
    pub ordinal: i64,
    pub sort_order: i64,
    pub title: String,
    pub relative_path: String,
    pub size: Option<i64>,
    pub modified_time: Option<i64>,
    pub runtime: Option<i64>,
}

impl LibraryExtraRecord {
    /// Classify an extra path already located under the target library.
    pub fn from_target_path(
        target_root: &Path,
        path: &Path,
        series_title: impl Into<String>,
    ) -> Result<Option<Self>> {
        let relative = path.strip_prefix(target_root).map_err(|_| {
            AppError::LibraryIndexError(format!("媒体文件不在目标目录内: {}", path.display()))
        })?;
        let components = normal_components(relative);
        let Some(file_name) = components.last() else {
            return Ok(None);
        };
        if components[..components.len() - 1]
            .iter()
            .any(|component| component.eq_ignore_ascii_case("menu"))
        {
            return Ok(None);
        }
        let Some((kind, ordinal, title)) = classify_extra(file_name) else {
            return Ok(None);
        };
        let (size, modified_time) = file_metadata(path);
        Ok(Some(Self {
            series_title: series_title.into(),
            kind,
            ordinal,
            sort_order: ordinal,
            title,
            relative_path: relative_path(target_root, path)?,
            size,
            modified_time,
            runtime: None,
        }))
    }
}

/// Counts returned after a library index write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LibraryIndexStats {
    pub series: i64,
    pub episodes: i64,
    pub media_files: i64,
    pub extras: i64,
}

/// MLIP database writer.
pub struct LibraryIndex;

impl LibraryIndex {
    #[must_use]
    pub fn database_path(target_root: &Path) -> PathBuf {
        target_root.join(DATABASE_FILENAME)
    }

    /// Load unambiguous Bangumi identities already associated with each library root and season.
    pub fn bangumi_ids_by_series_root(target_root: &Path) -> Result<HashMap<(String, u32), u32>> {
        let db_path = Self::database_path(target_root);
        if !db_path.exists() {
            return Ok(HashMap::new());
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::LibraryIndexError(format!("打开媒体库索引失败: {e}")))?;
        let mut statement = conn
            .prepare(
                "SELECT substr(media_file.path, 1, instr(media_file.path, '/') - 1), \
                        episode.season, MIN(series_external_id.value) \
                 FROM media_file \
                 INNER JOIN episode ON episode.id = media_file.episode_id \
                 INNER JOIN series_external_id \
                    ON series_external_id.series_id = episode.series_id \
                 WHERE series_external_id.provider = ?1 \
                   AND instr(media_file.path, '/') > 0 \
                 GROUP BY 1, 2 \
                 HAVING COUNT(DISTINCT series_external_id.value) = 1",
            )
            .map_err(|e| {
                AppError::LibraryIndexError(format!("准备读取媒体库元数据缓存失败: {e}"))
            })?;
        let rows = statement
            .query_map(params![ExternalProvider::Bangumi.as_i64()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| AppError::LibraryIndexError(format!("读取媒体库元数据缓存失败: {e}")))?;

        let mut ids = HashMap::new();
        for row in rows {
            let (root, season, id) = row.map_err(|e| {
                AppError::LibraryIndexError(format!("解析媒体库元数据缓存失败: {e}"))
            })?;
            if let (Ok(season), Ok(id)) = (u32::try_from(season), id.parse::<u32>()) {
                ids.insert((root, season), id);
            }
        }
        Ok(ids)
    }

    pub fn rebuild(
        target_root: &Path,
        records: &[LibraryIndexRecord],
    ) -> Result<LibraryIndexStats> {
        Self::rebuild_with_extras(target_root, records, &[])
    }

    pub fn rebuild_with_extras(
        target_root: &Path,
        records: &[LibraryIndexRecord],
        extras: &[LibraryExtraRecord],
    ) -> Result<LibraryIndexStats> {
        Self::rebuild_v4(target_root, records, extras)
    }

    /// Explicit v4 entry point retained for fixture generation and staging tools.
    pub fn rebuild_v4_staging(
        target_root: &Path,
        records: &[LibraryIndexRecord],
        extras: &[LibraryExtraRecord],
    ) -> Result<LibraryIndexStats> {
        Self::rebuild_v4(target_root, records, extras)
    }

    fn rebuild_v4(
        target_root: &Path,
        records: &[LibraryIndexRecord],
        extras: &[LibraryExtraRecord],
    ) -> Result<LibraryIndexStats> {
        let db_path = Self::database_path(target_root);
        let mut records = records.to_vec();
        inherit_media_hashes(&db_path, &mut records)?;
        let mut packing = build_and_publish(target_root, &records)?;
        let paths = staging_paths(target_root);
        let result = (|| {
            {
                let mut conn = Connection::open(&paths.local).map_err(|e| {
                    AppError::LibraryIndexError(format!("打开 v4 临时数据库失败: {e}"))
                })?;
                conn.execute_batch(MLIP_SCHEMA_SQL)
                    .and_then(|_| conn.execute_batch(MLIP_V4_SCHEMA_SQL))
                    .map_err(|e| {
                        AppError::LibraryIndexError(format!("创建 MLIP v4 schema 失败: {e}"))
                    })?;
                write_records_v4(&mut conn, target_root, &records, extras, &packing)?;
            }

            let stats = {
                let conn = Connection::open(&paths.local).map_err(|e| {
                    AppError::LibraryIndexError(format!("校验 v4 临时数据库失败: {e}"))
                })?;
                read_stats(&conn)?
            };
            install_staged_database(&paths, &db_path)?;
            Ok(stats)
        })();
        if result.is_ok() {
            packing.commit();
        }
        let _ = std::fs::remove_file(&paths.local);
        result
    }

    /// Converts an already-audited v3 library into v4 without re-resolving titles or metadata.
    ///
    /// Packs are verified before the staged database that references them is published.
    pub fn migrate_v3_to_v4(target_root: &Path) -> Result<LibraryIndexStats> {
        let db_path = Self::database_path(target_root);
        if !db_path.exists() {
            return Err(AppError::LibraryIndexError(format!(
                "无法迁移不存在的媒体库索引: {}",
                db_path.display()
            )));
        }

        let artwork = {
            let conn = Connection::open(&db_path)
                .map_err(|e| AppError::LibraryIndexError(format!("打开媒体库索引失败: {e}")))?;
            match validate_user_version(&conn)? {
                4 => return read_stats(&conn),
                3 => read_v3_artwork(&conn)?,
                version => {
                    return Err(AppError::LibraryIndexError(format!(
                        "只能迁移 MLIP v3 媒体库，当前版本: {version}"
                    )));
                }
            }
        };
        let mut packing = build_and_publish_artwork(target_root, &artwork)?;
        let paths = staging_paths(target_root);
        let result = (|| {
            std::fs::copy(&db_path, &paths.local).map_err(|e| {
                AppError::LibraryIndexError(format!("复制媒体库索引到本地失败: {e}"))
            })?;
            let stats = {
                let mut conn = Connection::open(&paths.local).map_err(|e| {
                    AppError::LibraryIndexError(format!("打开本地媒体库索引失败: {e}"))
                })?;
                conn.execute_batch("PRAGMA foreign_keys = ON;")
                    .and_then(|_| conn.execute_batch(MLIP_V4_SCHEMA_SQL))
                    .map_err(|e| {
                        AppError::LibraryIndexError(format!("升级 MLIP v4 schema 失败: {e}"))
                    })?;
                ensure_media_hash_columns(&conn)?;
                let tx = conn.transaction().map_err(|e| {
                    AppError::LibraryIndexError(format!("开始 v4 迁移事务失败: {e}"))
                })?;
                upsert_meta(&tx, target_root, true, 4)?;
                upsert_capabilities(&tx, true)?;
                let asset_ids = packing.write_catalog(&tx)?;
                attach_packed_v3_artwork(&tx, "series_artwork", &packing, &asset_ids)?;
                attach_packed_v3_artwork(&tx, "episode_artwork", &packing, &asset_ids)?;
                tx.commit().map_err(|e| {
                    AppError::LibraryIndexError(format!("提交 v4 迁移事务失败: {e}"))
                })?;
                read_stats(&conn)?
            };
            install_staged_database(&paths, &db_path)?;
            Ok(stats)
        })();
        if result.is_ok() {
            packing.commit();
        }
        let _ = std::fs::remove_file(&paths.local);
        result
    }

    pub fn update(target_root: &Path, records: &[LibraryIndexRecord]) -> Result<LibraryIndexStats> {
        let db_path = Self::database_path(target_root);
        if !db_path.exists() {
            return Self::rebuild(target_root, records);
        }

        let version = {
            let conn = Connection::open(&db_path)
                .map_err(|e| AppError::LibraryIndexError(format!("打开媒体库索引失败: {e}")))?;
            validate_user_version(&conn)?
        };
        let mut packing = (version == 4)
            .then(|| build_and_publish(target_root, records))
            .transpose()?;
        let paths = staging_paths(target_root);
        let result = (|| {
            std::fs::copy(&db_path, &paths.local).map_err(|e| {
                AppError::LibraryIndexError(format!("复制媒体库索引到本地失败: {e}"))
            })?;

            let stats = {
                let mut conn = Connection::open(&paths.local).map_err(|e| {
                    AppError::LibraryIndexError(format!("打开本地媒体库索引失败: {e}"))
                })?;
                conn.execute_batch("PRAGMA foreign_keys = ON;")
                    .map_err(|e| AppError::LibraryIndexError(format!("设置 PRAGMA 失败: {e}")))?;
                if let Some(packing) = &packing {
                    write_records_v4(&mut conn, target_root, records, &[], packing)?;
                } else {
                    write_records(&mut conn, target_root, records, &[], false)?;
                }
                read_stats(&conn)?
            };

            install_staged_database(&paths, &db_path)?;
            Ok(stats)
        })();
        if result.is_ok() {
            if let Some(packing) = &mut packing {
                packing.commit();
            }
        }
        let _ = std::fs::remove_file(&paths.local);
        result
    }
}

struct StagingPaths {
    local: PathBuf,
    upload: PathBuf,
    backup: PathBuf,
}

fn staging_paths(target_root: &Path) -> StagingPaths {
    let suffix = format!(
        "{}-{}-{}",
        std::process::id(),
        STAGING_COUNTER.fetch_add(1, Ordering::Relaxed),
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let temp_root = std::env::temp_dir();
    StagingPaths {
        local: temp_root.join(format!("aniorg-{DATABASE_FILENAME}-{suffix}.tmp")),
        upload: target_root.join(format!(".{DATABASE_FILENAME}.{suffix}.tmp")),
        backup: temp_root.join(format!("aniorg-{DATABASE_FILENAME}-{suffix}.bak")),
    }
}

fn install_staged_database(paths: &StagingPaths, db_path: &Path) -> Result<()> {
    std::fs::copy(&paths.local, &paths.upload)
        .map_err(|e| AppError::LibraryIndexError(format!("上传媒体库索引失败: {e}")))?;
    if !files_equal(&paths.local, &paths.upload)
        .map_err(|e| AppError::LibraryIndexError(format!("校验已上传媒体库索引失败: {e}")))?
    {
        let _ = std::fs::remove_file(&paths.upload);
        return Err(AppError::LibraryIndexError(
            "已上传媒体库索引内容不完整".to_string(),
        ));
    }

    let had_database = db_path.exists();
    if had_database {
        std::fs::copy(db_path, &paths.backup).map_err(|error| {
            let _ = std::fs::remove_file(&paths.upload);
            let _ = std::fs::remove_file(&paths.backup);
            AppError::LibraryIndexError(format!("备份旧媒体库索引到本地失败: {error}"))
        })?;
    }

    if let Err(error) = std::fs::rename(&paths.upload, db_path) {
        if error.kind() == std::io::ErrorKind::Unsupported {
            if let Err(copy_error) = std::fs::copy(&paths.local, db_path) {
                let restore_error = had_database
                    .then(|| std::fs::copy(&paths.backup, db_path).err())
                    .flatten();
                let _ = std::fs::remove_file(&paths.upload);
                let _ = std::fs::remove_file(&paths.backup);
                return Err(AppError::LibraryIndexError(match restore_error {
                    Some(restore) => format!(
                        "CloudDrive 不支持原子替换且覆盖媒体库索引失败: {copy_error}; 恢复旧索引也失败: {restore}"
                    ),
                    None => format!(
                        "CloudDrive 不支持原子替换且覆盖媒体库索引失败: {copy_error}"
                    ),
                }));
            }
        } else {
            let restore_error = if had_database && !db_path.exists() {
                std::fs::copy(&paths.backup, db_path).err()
            } else {
                None
            };
            let _ = std::fs::remove_file(&paths.upload);
            let _ = std::fs::remove_file(&paths.backup);
            return Err(AppError::LibraryIndexError(match restore_error {
                Some(restore) => {
                    format!("替换媒体库索引失败: {error}; 恢复旧索引也失败: {restore}")
                }
                None => format!("替换媒体库索引失败: {error}"),
            }));
        }
    }
    let _ = std::fs::remove_file(&paths.upload);

    let verification_error = match files_equal(&paths.local, db_path) {
        Ok(true) => None,
        Ok(false) => Some("已安装媒体库索引内容不完整".to_string()),
        Err(error) => Some(format!("校验已安装媒体库索引失败: {error}")),
    };
    if let Some(verification_error) = verification_error {
        let restore_error = if had_database {
            std::fs::copy(&paths.backup, &paths.upload)
                .and_then(|_| std::fs::rename(&paths.upload, db_path))
                .err()
        } else {
            std::fs::remove_file(db_path).err()
        };
        let _ = std::fs::remove_file(&paths.upload);
        let _ = std::fs::remove_file(&paths.backup);
        return Err(AppError::LibraryIndexError(match restore_error {
            Some(restore) => {
                format!("{verification_error}; 恢复旧索引也失败: {restore}")
            }
            None => format!("{verification_error}; 已恢复旧索引"),
        }));
    }

    let _ = std::fs::remove_file(&paths.backup);
    Ok(())
}

pub(crate) fn update_staged_database(
    target_root: &Path,
    update: impl FnOnce(&mut Connection) -> Result<()>,
) -> Result<()> {
    let db_path = LibraryIndex::database_path(target_root);
    if !db_path.exists() {
        return Err(AppError::LibraryIndexError(format!(
            "媒体库索引不存在: {}",
            db_path.display()
        )));
    }
    let paths = staging_paths(target_root);
    let result = (|| {
        std::fs::copy(&db_path, &paths.local)
            .map_err(|e| AppError::LibraryIndexError(format!("复制媒体库索引到本地失败: {e}")))?;
        {
            let mut conn = Connection::open(&paths.local)
                .map_err(|e| AppError::LibraryIndexError(format!("打开本地媒体库索引失败: {e}")))?;
            conn.execute_batch("PRAGMA foreign_keys = ON;")
                .map_err(|e| AppError::LibraryIndexError(format!("设置 PRAGMA 失败: {e}")))?;
            ensure_media_hash_columns(&conn)?;
            update(&mut conn)?;
            let integrity: String = conn
                .query_row("PRAGMA integrity_check", [], |row| row.get(0))
                .map_err(|e| AppError::LibraryIndexError(format!("校验媒体库索引失败: {e}")))?;
            if integrity != "ok" {
                return Err(AppError::LibraryIndexError(format!(
                    "媒体库索引完整性校验失败: {integrity}"
                )));
            }
            let foreign_key_errors: i64 = conn
                .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                    row.get(0)
                })
                .map_err(|e| AppError::LibraryIndexError(format!("校验媒体库外键失败: {e}")))?;
            if foreign_key_errors != 0 {
                return Err(AppError::LibraryIndexError(format!(
                    "媒体库索引存在 {foreign_key_errors} 个外键错误"
                )));
            }
        }
        install_staged_database(&paths, &db_path)
    })();
    let _ = std::fs::remove_file(&paths.local);
    result
}

fn files_equal(left: &Path, right: &Path) -> std::io::Result<bool> {
    let length = std::fs::metadata(left)?.len();
    if std::fs::metadata(right)?.len() != length {
        return Ok(false);
    }

    let mut left = BufReader::new(std::fs::File::open(left)?);
    let mut right = BufReader::new(std::fs::File::open(right)?);
    let mut left_buffer = [0_u8; 64 * 1024];
    let mut right_buffer = [0_u8; 64 * 1024];
    let mut remaining = length;
    while remaining > 0 {
        let chunk = remaining.min(left_buffer.len() as u64) as usize;
        left.read_exact(&mut left_buffer[..chunk])?;
        right.read_exact(&mut right_buffer[..chunk])?;
        if left_buffer[..chunk] != right_buffer[..chunk] {
            return Ok(false);
        }
        remaining -= chunk as u64;
    }
    Ok(true)
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut statement = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| AppError::LibraryIndexError(format!("读取 {table} schema 失败: {e}")))?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| AppError::LibraryIndexError(format!("查询 {table} schema 失败: {e}")))?;
    for value in columns {
        if value
            .map_err(|e| AppError::LibraryIndexError(format!("解析 {table} schema 失败: {e}")))?
            == column
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn ensure_media_hash_columns(conn: &Connection) -> Result<()> {
    if !column_exists(conn, "media_file", "sha256_prefix_63m")? {
        conn.execute_batch(
            "ALTER TABLE media_file ADD COLUMN sha256_prefix_63m TEXT \
             CHECK (sha256_prefix_63m IS NULL OR length(sha256_prefix_63m) = 64)",
        )
        .map_err(|e| AppError::LibraryIndexError(format!("添加媒体 quick hash 字段失败: {e}")))?;
    }
    if !column_exists(conn, "media_file", "sha256_full")? {
        conn.execute_batch(
            "ALTER TABLE media_file ADD COLUMN sha256_full TEXT \
             CHECK (sha256_full IS NULL OR length(sha256_full) = 64)",
        )
        .map_err(|e| AppError::LibraryIndexError(format!("添加媒体 full hash 字段失败: {e}")))?;
    }
    Ok(())
}

fn inherit_media_hashes(db_path: &Path, records: &mut [LibraryIndexRecord]) -> Result<()> {
    if !db_path.exists() {
        return Ok(());
    }
    let conn = Connection::open(db_path)
        .map_err(|e| AppError::LibraryIndexError(format!("打开旧媒体 hash 缓存失败: {e}")))?;
    if !column_exists(&conn, "media_file", "sha256_prefix_63m")?
        || !column_exists(&conn, "media_file", "sha256_full")?
    {
        return Ok(());
    }
    let mut statement = conn
        .prepare(
            "SELECT path, size, modified_time, sha256_prefix_63m, sha256_full \
             FROM media_file WHERE sha256_prefix_63m IS NOT NULL OR sha256_full IS NOT NULL",
        )
        .map_err(|e| AppError::LibraryIndexError(format!("读取旧媒体 hash 缓存失败: {e}")))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                (
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                ),
                (
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ),
            ))
        })
        .map_err(|e| AppError::LibraryIndexError(format!("查询旧媒体 hash 缓存失败: {e}")))?;
    let mut cache = HashMap::new();
    for row in rows {
        let (key, hashes) =
            row.map_err(|e| AppError::LibraryIndexError(format!("解析旧媒体 hash 缓存失败: {e}")))?;
        cache.insert(key, hashes);
    }
    for record in records {
        let key = (
            record.relative_path.clone(),
            record.size,
            record.modified_time,
        );
        if let Some((prefix, full)) = cache.get(&key) {
            if record.sha256_prefix_63m.is_none() {
                record.sha256_prefix_63m.clone_from(prefix);
            }
            if record.sha256_full.is_none() {
                record.sha256_full.clone_from(full);
            }
        }
    }
    Ok(())
}

fn validate_user_version(conn: &Connection) -> Result<i64> {
    let user_version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| AppError::LibraryIndexError(format!("读取 schema 版本失败: {e}")))?;
    if !matches!(user_version, 1..=4) {
        return Err(AppError::LibraryIndexError(format!(
            "不支持的 MLIP schema 版本: {user_version}"
        )));
    }
    Ok(user_version)
}

fn read_v3_artwork(conn: &Connection) -> Result<Vec<Artwork>> {
    let mut artwork = Vec::new();
    for table in ["series_artwork", "episode_artwork"] {
        let mut statement = conn
            .prepare(&format!("SELECT artwork_kind, path FROM {table}"))
            .map_err(|e| AppError::LibraryIndexError(format!("读取 v3 artwork 失败: {e}")))?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| AppError::LibraryIndexError(format!("查询 v3 artwork 失败: {e}")))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| AppError::LibraryIndexError(format!("解析 v3 artwork 失败: {e}")))?;
        for (kind, path) in rows {
            artwork.push(Artwork::new(artwork_kind(kind)?, path));
        }
    }
    Ok(artwork)
}

fn attach_packed_v3_artwork(
    conn: &rusqlite::Transaction<'_>,
    table: &str,
    packing: &ArtworkPacking,
    asset_ids: &HashMap<String, i64>,
) -> Result<()> {
    let rows = {
        let mut statement = conn
            .prepare(&format!("SELECT id, artwork_kind, path FROM {table}"))
            .map_err(|e| AppError::LibraryIndexError(format!("读取 v4 artwork 失败: {e}")))?;
        let result = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|e| AppError::LibraryIndexError(format!("查询 v4 artwork 失败: {e}")))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| AppError::LibraryIndexError(format!("解析 v4 artwork 失败: {e}")))?;
        result
    };
    for (id, kind, path) in rows {
        let Some(path) = path else {
            continue;
        };
        let artwork = Artwork::new(artwork_kind(kind)?, path);
        if let Some(asset_id) = packing.asset_id(&artwork, asset_ids) {
            conn.execute(
                &format!("UPDATE {table} SET asset_id = ?1 WHERE id = ?2"),
                params![asset_id, id],
            )
            .map_err(|e| AppError::LibraryIndexError(format!("绑定 v4 artwork asset 失败: {e}")))?;
        }
    }
    Ok(())
}

fn artwork_kind(value: i64) -> Result<ArtworkKind> {
    match value {
        1 => Ok(ArtworkKind::Poster),
        2 => Ok(ArtworkKind::Fanart),
        3 => Ok(ArtworkKind::Banner),
        4 => Ok(ArtworkKind::Logo),
        5 => Ok(ArtworkKind::Thumb),
        6 => Ok(ArtworkKind::Clearart),
        7 => Ok(ArtworkKind::SeasonPoster),
        _ => Err(AppError::LibraryIndexError(format!(
            "未知 artwork kind: {value}"
        ))),
    }
}

fn write_records(
    conn: &mut Connection,
    target_root: &Path,
    records: &[LibraryIndexRecord],
    extras: &[LibraryExtraRecord],
    include_static_meta: bool,
) -> Result<()> {
    ensure_schema_extensions(conn)?;
    ensure_media_hash_columns(conn)?;
    let tx = conn
        .transaction()
        .map_err(|e| AppError::LibraryIndexError(format!("开始事务失败: {e}")))?;

    upsert_meta(&tx, target_root, include_static_meta, 3)?;
    upsert_capabilities(&tx, false)?;

    for record in records {
        insert_record(&tx, record, None)?;
    }
    for extra in extras {
        insert_extra(&tx, extra)?;
    }

    tx.commit()
        .map_err(|e| AppError::LibraryIndexError(format!("提交事务失败: {e}")))?;
    Ok(())
}

fn write_records_v4(
    conn: &mut Connection,
    target_root: &Path,
    records: &[LibraryIndexRecord],
    extras: &[LibraryExtraRecord],
    packing: &ArtworkPacking,
) -> Result<()> {
    ensure_media_hash_columns(conn)?;
    let tx = conn
        .transaction()
        .map_err(|e| AppError::LibraryIndexError(format!("开始 v4 事务失败: {e}")))?;
    upsert_meta(&tx, target_root, true, 4)?;
    upsert_capabilities(&tx, true)?;
    let asset_ids = packing.write_catalog(&tx)?;
    for record in records {
        insert_record(&tx, record, Some((packing, &asset_ids)))?;
    }
    for extra in extras {
        insert_extra(&tx, extra)?;
    }
    tx.commit()
        .map_err(|e| AppError::LibraryIndexError(format!("提交 v4 事务失败: {e}")))?;
    Ok(())
}

fn ensure_schema_extensions(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS series_release_date (\
         series_id INTEGER PRIMARY KEY, \
         air_date TEXT NOT NULL, \
         FOREIGN KEY(series_id) REFERENCES series(id) ON DELETE CASCADE); \
         CREATE TABLE IF NOT EXISTS media_subtitle (\
         id INTEGER PRIMARY KEY, \
         media_file_id INTEGER NOT NULL, \
         path TEXT NOT NULL, \
         language TEXT, \
         title TEXT, \
         sort_order INTEGER NOT NULL DEFAULT 0, \
         FOREIGN KEY(media_file_id) REFERENCES media_file(id) ON DELETE CASCADE, \
         UNIQUE(media_file_id, path)); \
         CREATE INDEX IF NOT EXISTS idx_media_subtitle_file ON media_subtitle(media_file_id); \
         CREATE TABLE IF NOT EXISTS media_extra (\
         id INTEGER PRIMARY KEY, \
         uuid TEXT UNIQUE NOT NULL, \
         series_id INTEGER NOT NULL, \
         extra_kind INTEGER NOT NULL, \
         ordinal INTEGER NOT NULL, \
         sort_order INTEGER NOT NULL, \
         title TEXT NOT NULL, \
         path TEXT NOT NULL UNIQUE, \
         size INTEGER, \
         modified_time INTEGER, \
         runtime INTEGER, \
         FOREIGN KEY(series_id) REFERENCES series(id) ON DELETE CASCADE); \
         CREATE INDEX IF NOT EXISTS idx_media_extra_series \
         ON media_extra(series_id, extra_kind, sort_order); \
         PRAGMA user_version = 3; \
         UPDATE meta SET value = '3' WHERE key = 'schema'",
    )
    .map_err(|e| AppError::LibraryIndexError(format!("创建 MLIP 扩展表失败: {e}")))?;
    Ok(())
}

fn upsert_meta(
    conn: &Connection,
    target_root: &Path,
    include_static_meta: bool,
    schema: i64,
) -> Result<()> {
    let generated_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|e| AppError::LibraryIndexError(format!("格式化生成时间失败: {e}")))?;
    let canonical_root = target_root
        .canonicalize()
        .unwrap_or_else(|_| target_root.to_path_buf());
    let library_uuid = stable_uuid("library", &canonical_root.to_string_lossy());

    let mut entries = vec![
        ("generated_at", generated_at),
        ("schema", schema.to_string()),
    ];
    if include_static_meta {
        entries.extend([
            ("protocol", "MLIP".to_string()),
            ("generator", "AnimeOrganizer".to_string()),
            ("generator_version", env!("CARGO_PKG_VERSION").to_string()),
            ("library_uuid", library_uuid.to_string()),
            ("library_root", canonical_root.to_string_lossy().to_string()),
        ]);
    }

    for (key, value) in entries {
        conn.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .map_err(|e| AppError::LibraryIndexError(format!("写入 meta 失败: {e}")))?;
    }
    Ok(())
}

fn upsert_capabilities(conn: &Connection, artwork_pack: bool) -> Result<()> {
    const CAPABILITIES: &[(&str, i64)] = &[
        ("artwork", 1),
        ("genre", 1),
        ("external_id", 1),
        ("release_date", 1),
        ("people", 0),
        ("subtitle", 1),
        ("extra", 1),
        ("media_technical", 0),
        ("multi_file", 1),
    ];

    for (name, enabled) in CAPABILITIES
        .iter()
        .copied()
        .chain(artwork_pack.then_some(("artwork_pack", 1)))
    {
        conn.execute(
            "INSERT INTO capability (name, enabled) VALUES (?1, ?2) \
             ON CONFLICT(name) DO UPDATE SET enabled = excluded.enabled",
            params![name, enabled],
        )
        .map_err(|e| AppError::LibraryIndexError(format!("写入 capability 失败: {e}")))?;
    }
    Ok(())
}

fn insert_record(
    conn: &Connection,
    record: &LibraryIndexRecord,
    artwork_catalog: Option<(&ArtworkPacking, &HashMap<String, i64>)>,
) -> Result<()> {
    let series_uuid = resolve_series_uuid(conn, record)?;
    conn.execute(
        "INSERT INTO series \
         (uuid, title, original_title, sort_title, summary, year, series_type) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) \
         ON CONFLICT(uuid) DO UPDATE SET \
         title = excluded.title, \
         original_title = COALESCE(excluded.original_title, series.original_title), \
         sort_title = COALESCE(excluded.sort_title, series.sort_title), \
         summary = COALESCE(excluded.summary, series.summary), \
         year = COALESCE(excluded.year, series.year), \
         series_type = excluded.series_type",
        params![
            series_uuid.to_string(),
            record.series_title,
            record.original_title,
            record.sort_title,
            record.summary,
            record.year,
            record.series_type,
        ],
    )
    .map_err(|e| AppError::LibraryIndexError(format!("写入 series 失败: {e}")))?;
    let series_id: i64 = conn
        .query_row(
            "SELECT id FROM series WHERE uuid = ?1",
            params![series_uuid.to_string()],
            |row| row.get(0),
        )
        .map_err(|e| AppError::LibraryIndexError(format!("读取 series id 失败: {e}")))?;

    upsert_release_date(conn, series_id, record.air_date.as_ref())?;

    let episode_uuid = stable_uuid(
        "episode",
        &format!(
            "{}:{}:{}",
            series_uuid,
            record.season,
            episode_key(record.episode)
        ),
    );
    conn.execute(
        "INSERT INTO episode \
         (uuid, series_id, season, episode, sort_order, title, summary, runtime) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) \
         ON CONFLICT(series_id, season, episode) DO UPDATE SET \
         uuid = excluded.uuid, \
         sort_order = excluded.sort_order, \
         title = COALESCE(excluded.title, episode.title), \
         summary = COALESCE(excluded.summary, episode.summary), \
         runtime = COALESCE(excluded.runtime, episode.runtime)",
        params![
            episode_uuid.to_string(),
            series_id,
            record.season,
            record.episode,
            record.sort_order,
            record.episode_title,
            record.episode_summary,
            record.runtime,
        ],
    )
    .map_err(|e| AppError::LibraryIndexError(format!("写入 episode 失败: {e}")))?;
    let episode_id: i64 = conn
        .query_row(
            "SELECT id FROM episode WHERE series_id = ?1 AND season = ?2 AND episode = ?3",
            params![series_id, record.season, record.episode],
            |row| row.get(0),
        )
        .map_err(|e| AppError::LibraryIndexError(format!("读取 episode id 失败: {e}")))?;

    conn.execute(
        "INSERT INTO media_file \
         (episode_id, path, size, modified_time, sha256_prefix_63m, sha256_full) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
         ON CONFLICT(path) DO UPDATE SET \
         episode_id = excluded.episode_id, \
         sha256_prefix_63m = CASE \
             WHEN media_file.size IS excluded.size \
              AND media_file.modified_time IS excluded.modified_time \
             THEN COALESCE(excluded.sha256_prefix_63m, media_file.sha256_prefix_63m) \
             ELSE excluded.sha256_prefix_63m END, \
         sha256_full = CASE \
             WHEN media_file.size IS excluded.size \
              AND media_file.modified_time IS excluded.modified_time \
             THEN COALESCE(excluded.sha256_full, media_file.sha256_full) \
             ELSE excluded.sha256_full END, \
         size = excluded.size, \
         modified_time = excluded.modified_time",
        params![
            episode_id,
            record.relative_path,
            record.size,
            record.modified_time,
            record.sha256_prefix_63m,
            record.sha256_full,
        ],
    )
    .map_err(|e| AppError::LibraryIndexError(format!("写入 media_file 失败: {e}")))?;
    let media_file_id: i64 = conn
        .query_row(
            "SELECT id FROM media_file WHERE path = ?1",
            params![record.relative_path],
            |row| row.get(0),
        )
        .map_err(|e| AppError::LibraryIndexError(format!("读取 media_file id 失败: {e}")))?;
    conn.execute(
        "DELETE FROM media_subtitle WHERE media_file_id = ?1",
        params![media_file_id],
    )
    .map_err(|e| AppError::LibraryIndexError(format!("清理 media_subtitle 失败: {e}")))?;
    for (sort_order, path) in record.subtitle_paths.iter().enumerate() {
        conn.execute(
            "INSERT INTO media_subtitle (media_file_id, path, sort_order) VALUES (?1, ?2, ?3)",
            params![media_file_id, path, sort_order as i64],
        )
        .map_err(|e| AppError::LibraryIndexError(format!("写入 media_subtitle 失败: {e}")))?;
    }

    insert_genres(conn, series_id, &record.genres)?;
    insert_external_ids(conn, series_id, &record.external_ids)?;
    insert_artwork(
        conn,
        "series_artwork",
        "series_id",
        series_id,
        &record.series_artwork,
        artwork_catalog,
    )?;
    insert_artwork(
        conn,
        "episode_artwork",
        "episode_id",
        episode_id,
        &record.episode_artwork,
        artwork_catalog,
    )?;

    Ok(())
}

fn insert_extra(conn: &Connection, extra: &LibraryExtraRecord) -> Result<()> {
    let series_uuid = root_series_uuid(conn, &extra.relative_path)?
        .unwrap_or_else(|| stable_uuid("series", &normalize_key(&extra.series_title)));
    let series_id: i64 = conn
        .query_row(
            "SELECT id FROM series WHERE uuid = ?1",
            params![series_uuid.to_string()],
            |row| row.get(0),
        )
        .map_err(|e| AppError::LibraryIndexError(format!("读取特典 series id 失败: {e}")))?;
    let extra_uuid = stable_uuid("extra", &extra.relative_path);
    conn.execute(
        "INSERT INTO media_extra \
         (uuid, series_id, extra_kind, ordinal, sort_order, title, path, size, modified_time, runtime) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) \
         ON CONFLICT(path) DO UPDATE SET \
         uuid = excluded.uuid, \
         series_id = excluded.series_id, \
         extra_kind = excluded.extra_kind, \
         ordinal = excluded.ordinal, \
         sort_order = excluded.sort_order, \
         title = excluded.title, \
         size = excluded.size, \
         modified_time = excluded.modified_time, \
         runtime = COALESCE(excluded.runtime, media_extra.runtime)",
        params![
            extra_uuid.to_string(),
            series_id,
            extra.kind.as_i64(),
            extra.ordinal,
            extra.sort_order,
            extra.title,
            extra.relative_path,
            extra.size,
            extra.modified_time,
            extra.runtime,
        ],
    )
    .map_err(|e| AppError::LibraryIndexError(format!("写入 media_extra 失败: {e}")))?;
    Ok(())
}

fn resolve_series_uuid(conn: &Connection, record: &LibraryIndexRecord) -> Result<Uuid> {
    // A confirmed external identity may span several library directories. A
    // directory remains the stable fallback when metadata is unavailable.
    for external_id in &record.external_ids {
        let uuid = conn
            .query_row(
                "SELECT series.uuid FROM series \
                 INNER JOIN series_external_id ON series_external_id.series_id = series.id \
                 WHERE series_external_id.provider = ?1 AND series_external_id.value = ?2 \
                 LIMIT 1",
                params![external_id.provider.as_i64(), external_id.value],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| {
                AppError::LibraryIndexError(format!("按 external_id 读取 series 失败: {e}"))
            })?;
        if let Some(uuid) = uuid.and_then(|value| Uuid::parse_str(&value).ok()) {
            return Ok(uuid);
        }
    }

    if let Some(uuid) = root_series_uuid(conn, &record.relative_path)? {
        return Ok(uuid);
    }
    Ok(stable_uuid("series", &normalize_key(&record.series_title)))
}

fn root_series_uuid(conn: &Connection, relative_path: &str) -> Result<Option<Uuid>> {
    let Some((root, _)) = relative_path.split_once('/') else {
        return Ok(None);
    };
    let prefix = format!("{root}/");
    let uuid = conn
        .query_row(
            "SELECT series.uuid FROM series \
             INNER JOIN episode ON episode.series_id = series.id \
             INNER JOIN media_file ON media_file.episode_id = episode.id \
             WHERE substr(media_file.path, 1, ?1) = ?2 LIMIT 1",
            params![prefix.chars().count() as i64, prefix],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| AppError::LibraryIndexError(format!("按路径读取 series 失败: {e}")))?
        .and_then(|value| Uuid::parse_str(&value).ok());
    Ok(uuid.or_else(|| Some(stable_uuid("series-root", &normalize_key(root)))))
}

fn upsert_release_date(
    conn: &Connection,
    series_id: i64,
    air_date: Option<&ReleaseDate>,
) -> Result<()> {
    let Some(air_date) = air_date.map(ToString::to_string) else {
        return Ok(());
    };
    conn.execute(
        "INSERT INTO series_release_date (series_id, air_date) VALUES (?1, ?2) \
         ON CONFLICT(series_id) DO UPDATE SET air_date = excluded.air_date",
        params![series_id, air_date],
    )
    .map_err(|e| AppError::LibraryIndexError(format!("写入 series_release_date 失败: {e}")))?;
    Ok(())
}

fn insert_genres(conn: &Connection, series_id: i64, genres: &[String]) -> Result<()> {
    let mut seen = HashSet::new();
    for genre in genres
        .iter()
        .map(|value| value.trim())
        .filter(|v| !v.is_empty())
    {
        if !seen.insert(genre.to_string()) {
            continue;
        }
        conn.execute(
            "INSERT INTO genre (name) VALUES (?1) ON CONFLICT(name) DO NOTHING",
            params![genre],
        )
        .map_err(|e| AppError::LibraryIndexError(format!("写入 genre 失败: {e}")))?;
        let genre_id: i64 = conn
            .query_row(
                "SELECT id FROM genre WHERE name = ?1",
                params![genre],
                |row| row.get(0),
            )
            .map_err(|e| AppError::LibraryIndexError(format!("读取 genre id 失败: {e}")))?;
        conn.execute(
            "INSERT OR IGNORE INTO series_genre (series_id, genre_id) VALUES (?1, ?2)",
            params![series_id, genre_id],
        )
        .map_err(|e| AppError::LibraryIndexError(format!("写入 series_genre 失败: {e}")))?;
    }
    Ok(())
}

fn insert_external_ids(
    conn: &Connection,
    series_id: i64,
    external_ids: &[ExternalId],
) -> Result<()> {
    let mut seen = HashSet::new();
    for external_id in external_ids
        .iter()
        .filter(|item| !item.value.trim().is_empty())
    {
        if !seen.insert((external_id.provider, external_id.value.clone())) {
            continue;
        }
        conn.execute(
            "INSERT OR IGNORE INTO series_external_id (series_id, provider, value) \
             VALUES (?1, ?2, ?3)",
            params![
                series_id,
                external_id.provider.as_i64(),
                external_id.value.trim(),
            ],
        )
        .map_err(|e| AppError::LibraryIndexError(format!("写入 external_id 失败: {e}")))?;
    }
    Ok(())
}

fn insert_artwork(
    conn: &Connection,
    table: &str,
    owner_column: &str,
    owner_id: i64,
    artwork: &[Artwork],
    artwork_catalog: Option<(&ArtworkPacking, &HashMap<String, i64>)>,
) -> Result<()> {
    let mut seen = HashSet::new();
    for item in artwork.iter().filter(|item| !item.path.trim().is_empty()) {
        if !seen.insert((item.kind, item.path.clone())) {
            continue;
        }
        if let Some((packing, asset_ids)) = artwork_catalog {
            let sql = format!(
                "INSERT OR IGNORE INTO {table} \
                 ({owner_column}, artwork_kind, path, asset_id, source_url, source_provider, \
                  source_subject_id, downloaded_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
            );
            conn.execute(
                &sql,
                params![
                    owner_id,
                    item.kind.as_i64(),
                    item.path.trim(),
                    packing.asset_id(item, asset_ids),
                    item.source_url.as_deref(),
                    item.source_provider.map(ExternalProvider::as_i64),
                    item.source_subject_id.as_deref(),
                    item.downloaded_at.as_deref(),
                ],
            )
            .map_err(|e| AppError::LibraryIndexError(format!("写入 v4 artwork 失败: {e}")))?;
        } else {
            let sql = format!(
                "INSERT OR IGNORE INTO {table} ({owner_column}, artwork_kind, path) \
                 VALUES (?1, ?2, ?3)"
            );
            conn.execute(
                &sql,
                params![owner_id, item.kind.as_i64(), item.path.trim()],
            )
            .map_err(|e| AppError::LibraryIndexError(format!("写入 artwork 失败: {e}")))?;
        }
    }
    Ok(())
}

fn read_stats(conn: &Connection) -> Result<LibraryIndexStats> {
    Ok(LibraryIndexStats {
        series: count_table(conn, "series")?,
        episodes: count_table(conn, "episode")?,
        media_files: count_table(conn, "media_file")?,
        extras: count_table(conn, "media_extra")?,
    })
}

fn count_table(conn: &Connection, table: &str) -> Result<i64> {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .map_err(|e| AppError::LibraryIndexError(format!("读取统计失败: {e}")))
}

fn relative_path(target_root: &Path, path: &Path) -> Result<String> {
    let relative = path.strip_prefix(target_root).map_err(|_| {
        AppError::LibraryIndexError(format!("媒体文件不在目标目录内: {}", path.display()))
    })?;
    let components = normal_components(relative);
    if components.is_empty() {
        return Err(AppError::LibraryIndexError(format!(
            "无法生成相对路径: {}",
            path.display()
        )));
    }
    Ok(components.join("/"))
}

fn normal_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str().map(ToOwned::to_owned),
            _ => None,
        })
        .collect()
}

fn is_supplemental_directory(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "menu" | "ncop&nced" | "图集" | "圖集" | "特典映像"
    )
}

fn classify_extra(file_name: &str) -> Option<(ExtraKind, i64, String)> {
    let stem = Path::new(file_name).file_stem()?.to_str()?;
    let tokens = bracket_tokens(stem);

    for (index, token) in tokens.iter().enumerate() {
        let upper = token.to_ascii_uppercase();
        let (kind, label, ordinal) = if upper == "OVA" {
            (
                ExtraKind::Ova,
                "OVA",
                next_extra_ordinal(&tokens, index).unwrap_or(1),
            )
        } else if let Some(ordinal) = extra_ordinal(&tokens, index, "NCOP") {
            (ExtraKind::Ncop, "NCOP", ordinal)
        } else if let Some(ordinal) = extra_ordinal(&tokens, index, "NCED") {
            (ExtraKind::Nced, "NCED", ordinal)
        } else if upper == "TOKUTEN" {
            (
                ExtraKind::Special,
                "特典映像",
                next_extra_ordinal(&tokens, index)?,
            )
        } else if upper == "IMAGES" {
            (
                ExtraKind::Gallery,
                "图集",
                next_extra_ordinal(&tokens, index)?,
            )
        } else {
            continue;
        };
        let title = if kind == ExtraKind::Ova && next_extra_ordinal(&tokens, index).is_none() {
            label.to_string()
        } else {
            format!("{label} {ordinal:02}")
        };
        return Some((kind, ordinal, title));
    }
    None
}

fn bracket_tokens(value: &str) -> Vec<&str> {
    value
        .split('[')
        .skip(1)
        .filter_map(|part| part.split_once(']').map(|(token, _)| token.trim()))
        .collect()
}

fn has_bracket_token(value: &str, expected: &str) -> bool {
    bracket_tokens(value)
        .iter()
        .any(|token| token.eq_ignore_ascii_case(expected))
}

fn extra_ordinal(tokens: &[&str], index: usize, prefix: &str) -> Option<i64> {
    let token = tokens.get(index)?.to_ascii_uppercase();
    let suffix = token.strip_prefix(prefix)?;
    if !suffix.is_empty() {
        return suffix.parse().ok();
    }
    next_extra_ordinal(tokens, index)
}

fn next_extra_ordinal(tokens: &[&str], index: usize) -> Option<i64> {
    tokens.get(index + 1)?.trim().parse().ok()
}

fn parse_target_filename(file_name: &str) -> Option<(f64, String)> {
    let stem = Path::new(file_name).file_stem()?.to_str()?.trim();
    let mut parts = stem.splitn(2, char::is_whitespace);
    let episode_raw = parts.next()?.trim();
    let episode = episode_raw.parse::<f64>().ok()?;
    let tags = parts.next().unwrap_or_default().trim().to_string();
    Some((episode, tags))
}

fn season_directory_identity(components: &[String]) -> Option<(String, i64)> {
    let season_dir = components.get(components.len().checked_sub(2)?)?;
    let season = parse_season_dir(season_dir)?;
    let series = components.get(components.len().checked_sub(3)?)?.clone();
    Some((series, season))
}

fn parse_season_dir(value: &str) -> Option<i64> {
    let lower = value.trim().to_ascii_lowercase();
    let raw = lower.strip_prefix("season")?.trim();
    raw.parse::<i64>().ok().filter(|season| *season > 0)
}

fn title_season_number(value: &str) -> Option<i64> {
    split_series_and_season(value)
        .1
        .map(i64::from)
        .filter(|season| *season > 0)
}

fn parse_episode_number(value: &str) -> Result<f64> {
    let value = value.trim();
    let numeric = value
        .rsplit_once(['v', 'V'])
        .filter(|(episode, version)| {
            !episode.is_empty()
                && !version.is_empty()
                && version.chars().all(|character| character.is_ascii_digit())
        })
        .map_or(value, |(episode, _version)| episode);
    numeric
        .parse::<f64>()
        .map_err(|e| AppError::LibraryIndexError(format!("无法解析集数 {value}: {e}")))
}

fn file_metadata(path: &Path) -> (Option<i64>, Option<i64>) {
    let Ok(metadata) = std::fs::metadata(path) else {
        return (None, None);
    };
    let size = i64::try_from(metadata.len()).ok();
    let modified_time = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .and_then(|duration| i64::try_from(duration.as_secs()).ok());
    (size, modified_time)
}

fn stable_uuid(kind: &str, value: &str) -> Uuid {
    Uuid::new_v5(&MLIP_NAMESPACE, format!("{kind}:{value}").as_bytes())
}

fn normalize_key(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn episode_key(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

#[cfg(feature = "metadata")]
fn parse_year(value: &str) -> Option<i32> {
    value.get(0..4)?.parse().ok()
}

#[cfg(test)]
mod staged_install_tests {
    use super::*;

    #[test]
    fn replaces_database_and_cleans_staging_files() {
        let directory = tempfile::tempdir().unwrap();
        let db_path = directory.path().join(DATABASE_FILENAME);
        let paths = StagingPaths {
            local: directory.path().join("local.tmp"),
            upload: directory.path().join("upload.tmp"),
            backup: directory.path().join("backup.bak"),
        };
        std::fs::write(&db_path, b"old database").unwrap();
        std::fs::write(&paths.local, b"new database").unwrap();

        install_staged_database(&paths, &db_path).unwrap();

        assert_eq!(std::fs::read(&db_path).unwrap(), b"new database");
        assert!(!paths.upload.exists());
        assert!(!paths.backup.exists());
    }

    #[test]
    fn detects_same_length_content_mismatch() {
        let directory = tempfile::tempdir().unwrap();
        let left = directory.path().join("left");
        let right = directory.path().join("right");
        std::fs::write(&left, b"abcdef").unwrap();
        std::fs::write(&right, b"abcdeg").unwrap();

        assert!(!files_equal(&left, &right).unwrap());
    }
}
