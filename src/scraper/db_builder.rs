use crate::error::{AppError, Result};
use flate2::read::GzDecoder;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct LatestVersion {
    #[allow(dead_code)]
    name: String,
    #[serde(rename = "browser_download_url")]
    download_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubjectRecord {
    id: u32,
    #[serde(rename = "type")]
    subject_type: u32,
    name: String,
    #[serde(default)]
    name_cn: Option<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    score: Option<f32>,
    #[serde(default)]
    platform: Option<u32>,
    #[serde(default)]
    infobox: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EpisodeRecord {
    id: u32,
    #[serde(rename = "subject_id")]
    subject_id: u32,
    sort: u32,
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "airdate", default)]
    air_date: Option<String>,
    #[serde(rename = "type", default)]
    #[allow(dead_code)]
    ep_type: u32,
}

#[derive(Debug, Clone)]
pub struct BuildDbStats {
    pub subjects_count: usize,
    pub episodes_count: usize,
    pub db_size: u64,
}

pub async fn build_bangumi_db(output_path: &Path) -> Result<BuildDbStats> {
    let temp_dir = tempfile::tempdir()
        .map_err(|e| AppError::MetadataFetchError(format!("创建临时目录失败: {e}")))?;
    let temp_path = temp_dir.path().to_path_buf();

    eprintln!("获取 Bangumi Archive 最新版本信息...");
    let latest_url = fetch_latest_version_url().await?;

    eprintln!("下载 Bangumi Archive: {}", latest_url);
    let zip_path = download_zip(&latest_url, &temp_path).await?;

    eprintln!("解压并解析 dump 文件...");
    let (subjects_count, episodes_count) = extract_and_parse(&zip_path, output_path)?;

    let db_size = std::fs::metadata(output_path).map_err(AppError::Io)?.len();

    Ok(BuildDbStats {
        subjects_count,
        episodes_count,
        db_size,
    })
}

async fn fetch_latest_version_url() -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("anime-organizer/0.1")
        .build()
        .map_err(|e| AppError::MetadataFetchError(format!("创建 HTTP 客户端失败: {e}")))?;

    let latest_url = "https://raw.githubusercontent.com/bangumi/Archive/master/aux/latest.json";
    let resp = client
        .get(latest_url)
        .send()
        .await
        .map_err(|e| AppError::MetadataFetchError(format!("获取版本信息失败: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::MetadataFetchError(format!(
            "获取版本信息失败 (HTTP {})",
            resp.status()
        )));
    }

    let version: LatestVersion = resp
        .json()
        .await
        .map_err(|e| AppError::MetadataFetchError(format!("解析版本信息失败: {e}")))?;

    Ok(version.download_url)
}

async fn download_zip(url: &str, temp_dir: &Path) -> Result<PathBuf> {
    let client = reqwest::Client::builder()
        .user_agent("anime-organizer/0.1")
        .build()
        .map_err(|e| AppError::MetadataFetchError(format!("创建 HTTP 客户端失败: {e}")))?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::MetadataFetchError(format!("下载 ZIP 失败: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::MetadataFetchError(format!(
            "下载 ZIP 失败 (HTTP {})",
            resp.status()
        )));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| AppError::MetadataFetchError(format!("读取 ZIP 数据失败: {e}")))?;

    let zip_path = temp_dir.join("bangumi_dump.zip");
    std::fs::write(&zip_path, &bytes).map_err(AppError::Io)?;

    Ok(zip_path)
}

fn extract_and_parse(zip_path: &Path, db_path: &Path) -> Result<(usize, usize)> {
    let zip_file = std::fs::File::open(zip_path).map_err(AppError::Io)?;
    let mut zip_archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| AppError::MetadataFetchError(format!("解析 ZIP 文件失败: {e}")))?;

    let subject_file = "subject.jsonlines";
    let episode_file = "episode.jsonlines";

    let subjects_count = parse_subjects_from_zip(&mut zip_archive, subject_file, db_path)?;
    let episodes_count = parse_episodes_from_zip(&mut zip_archive, episode_file, db_path)?;

    Ok((subjects_count, episodes_count))
}

fn parse_subjects_from_zip(
    zip_archive: &mut zip::ZipArchive<std::fs::File>,
    filename: &str,
    db_path: &Path,
) -> Result<usize> {
    let conn = get_or_create_db(db_path)?;

    let subject_file = zip_archive
        .by_name(filename)
        .map_err(|e| AppError::MetadataFetchError(format!("在 ZIP 中找不到 {}: {e}", filename)))?;

    let gz_reader = GzDecoder::new(BufReader::new(subject_file));
    let reader = BufReader::new(gz_reader);

    let mut stmt = conn
        .prepare_cached(
            "INSERT OR REPLACE INTO subjects (id, name, name_cn, summary, date, score, platform, infobox) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .map_err(|e| AppError::BangumiParseError(format!("预处理 SQL 失败: {e}")))?;

    let mut count = 0;
    for line in reader.lines() {
        let line = line.map_err(|e| AppError::BangumiParseError(format!("读取行失败: {e}")))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(subject) = serde_json::from_str::<SubjectRecord>(line) {
            if subject.subject_type == 2 {
                stmt.execute(params![
                    subject.id,
                    subject.name,
                    subject.name_cn,
                    subject.summary,
                    subject.date,
                    subject.score,
                    subject.platform,
                    subject.infobox,
                ])
                .map_err(|e| AppError::BangumiParseError(format!("插入数据失败: {e}")))?;
                count += 1;
            }
        }
    }

    Ok(count)
}

fn parse_episodes_from_zip(
    zip_archive: &mut zip::ZipArchive<std::fs::File>,
    filename: &str,
    db_path: &Path,
) -> Result<usize> {
    let episode_file = zip_archive
        .by_name(filename)
        .map_err(|e| AppError::MetadataFetchError(format!("在 ZIP 中找不到 {}: {e}", filename)))?;

    let gz_reader = GzDecoder::new(BufReader::new(episode_file));
    let reader = BufReader::new(gz_reader);

    let conn = get_or_create_db(db_path)?;
    let mut stmt = conn
        .prepare_cached(
            "INSERT OR REPLACE INTO episodes (id, subject_id, episode_number, title, air_date) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .map_err(|e| AppError::BangumiParseError(format!("预处理 SQL 失败: {e}")))?;

    let mut count = 0;
    for line in reader.lines() {
        let line = line.map_err(|e| AppError::BangumiParseError(format!("读取行失败: {e}")))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(episode) = serde_json::from_str::<EpisodeRecord>(line) {
            stmt.execute(params![
                episode.id,
                episode.subject_id,
                episode.sort,
                episode.name,
                episode.air_date,
            ])
            .map_err(|e| AppError::BangumiParseError(format!("插入数据失败: {e}")))?;
            count += 1;
        }
    }

    Ok(count)
}

fn get_or_create_db(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)
        .map_err(|e| AppError::BangumiParseError(format!("打开数据库失败: {e}")))?;

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS subjects (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            name_cn TEXT,
            summary TEXT,
            date TEXT,
            score REAL,
            platform INTEGER,
            infobox TEXT
        );

        CREATE TABLE IF NOT EXISTS episodes (
            id INTEGER PRIMARY KEY,
            subject_id INTEGER REFERENCES subjects(id),
            episode_number INTEGER,
            title TEXT,
            air_date TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_subjects_name ON subjects(name);
        CREATE INDEX IF NOT EXISTS idx_subjects_name_cn ON subjects(name_cn);
        CREATE INDEX IF NOT EXISTS idx_episodes_subject ON episodes(subject_id);
        "#,
    )
    .map_err(|e| AppError::BangumiParseError(format!("创建表失败: {e}")))?;

    Ok(conn)
}
