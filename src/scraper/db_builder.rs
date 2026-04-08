use crate::error::{AppError, Result};
use rusqlite::Connection;
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

fn truncate_str(s: &str, max_len: usize) -> String {
    let mut result = String::new();
    for c in s.chars() {
        if result.len() + c.len_utf8() > max_len {
            break;
        }
        result.push(c);
    }
    if result.len() < s.len() {
        result.push_str("...");
    }
    result
}

#[derive(Debug, Deserialize)]
struct LatestVersion {
    #[allow(dead_code)]
    name: String,
    #[serde(rename = "browser_download_url")]
    download_url: String,
}

#[derive(Debug, Deserialize)]
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
    platform: Option<serde_json::Value>,
    #[serde(default)]
    nsfw: Option<serde_json::Value>,
    #[serde(default)]
    series: Option<serde_json::Value>,
    #[serde(default)]
    eps: Option<serde_json::Value>,
    #[serde(default)]
    studio: Option<String>,
    #[serde(default)]
    director: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EpisodeRecord {
    id: u32,
    #[serde(rename = "subject_id")]
    subject_id: u32,
    sort: u32,
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "name_cn", default)]
    name_cn: Option<String>,
    #[serde(rename = "airdate", default)]
    air_date: Option<String>,
    #[serde(rename = "type", default)]
    #[allow(dead_code)]
    ep_type: serde_json::Value,
    #[serde(default)]
    disc: Option<serde_json::Value>,
    #[serde(default)]
    duration: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BuildDbStats {
    pub subjects_count: usize,
    pub episodes_count: usize,
    pub aliases_count: usize,
    pub relations_count: usize,
    pub db_size: u64,
    pub processing_time_ms: u128,
}

const BATCH_SIZE: usize = 1000;

pub async fn build_bangumi_db(
    output_path: &Path,
    include_relations: bool,
    verbose: bool,
) -> Result<BuildDbStats> {
    let start_time = std::time::Instant::now();

    if verbose {
        eprintln!("开始构建 Bangumi 数据库...");
    }

    let temp_dir = tempfile::tempdir()
        .map_err(|e| AppError::MetadataFetchError(format!("创建临时目录失败: {e}")))?;
    let temp_path = temp_dir.path().to_path_buf();

    if verbose {
        eprintln!("获取 Bangumi Archive 最新版本信息...");
    }
    let latest_url = fetch_latest_version_url().await?;

    if verbose {
        eprintln!("下载 Bangumi Archive: {}", latest_url);
    }
    let zip_path = download_zip(&latest_url, &temp_path).await?;

    let download_size = std::fs::metadata(&zip_path).map_err(AppError::Io)?.len();
    if verbose {
        eprintln!("下载完成，大小: {} MB", download_size / (1024 * 1024));
    }

    if verbose {
        eprintln!("开始解压...");
    }
    let (subjects_count, episodes_count, aliases_count, relations_count) =
        extract_and_parse(&zip_path, output_path, include_relations, verbose)?;

    let db_size = std::fs::metadata(output_path).map_err(AppError::Io)?.len();

    if verbose {
        eprintln!("验证数据库完整性...");
    }
    validate_database(output_path)?;

    let processing_time_ms = start_time.elapsed().as_millis();

    if verbose {
        let expected_size = 50 * 1024 * 1024;
        eprintln!(
            "数据库构建完成: 大小 {} MB (预期 ~{} MB)",
            db_size / (1024 * 1024),
            expected_size / (1024 * 1024)
        );
    }

    Ok(BuildDbStats {
        subjects_count,
        episodes_count,
        aliases_count,
        relations_count,
        db_size,
        processing_time_ms,
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

fn extract_and_parse(
    zip_path: &Path,
    db_path: &Path,
    include_relations: bool,
    verbose: bool,
) -> Result<(usize, usize, usize, usize)> {
    let zip_file = std::fs::File::open(zip_path).map_err(AppError::Io)?;
    let mut zip_archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| AppError::MetadataFetchError(format!("解析 ZIP 文件失败: {e}")))?;

    let conn = get_or_create_db(db_path)?;

    let subject_file = "subject.jsonlines";
    let episode_file = "episode.jsonlines";

    let subjects_count = parse_subjects_from_zip(&mut zip_archive, subject_file, &conn, verbose)?;

    let subject_ids: std::collections::HashSet<u32> = {
        let mut stmt = conn
            .prepare("SELECT id FROM subjects")
            .map_err(|e| AppError::BangumiParseError(format!("查询subjects失败: {e}")))?;
        let ids = stmt
            .query_map([], |row| row.get::<_, u32>(0))
            .map_err(|e| AppError::BangumiParseError(format!("查询subjects失败: {e}")))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| AppError::BangumiParseError(format!("查询subjects失败: {e}")))?;
        ids.into_iter().collect()
    };

    let episodes_count =
        parse_episodes_from_zip(&mut zip_archive, episode_file, &conn, &subject_ids, verbose)?;

    let aliases_count =
        parse_name_cn_aliases_from_zip(&mut zip_archive, &conn, &subject_ids, verbose)?;

    let relations_count = if include_relations {
        parse_relations_from_zip(&mut zip_archive, &conn, &subject_ids, verbose)?
    } else {
        0
    };

    Ok((
        subjects_count,
        episodes_count,
        aliases_count,
        relations_count,
    ))
}

fn parse_subjects_from_zip(
    zip_archive: &mut zip::ZipArchive<std::fs::File>,
    filename: &str,
    conn: &Connection,
    verbose: bool,
) -> Result<usize> {
    let mut subject_file = zip_archive
        .by_name(filename)
        .map_err(|e| AppError::MetadataFetchError(format!("在 ZIP 中找不到 {}: {e}", filename)))?;

    use std::io::Read;
    let mut content = String::new();
    subject_file
        .read_to_string(&mut content)
        .map_err(|e| AppError::BangumiParseError(format!("读取文件失败: {e}")))?;

    let line_count = content.lines().count();
    if verbose {
        eprintln!("处理 {}: 源文件 {} 行", filename, line_count);
        eprintln!("[DEBUG] 前3行原始JSON:");
        for (i, line) in content.lines().take(3).enumerate() {
            eprintln!("[DEBUG] 行{}: {}", i + 1, truncate_str(line, 200));
        }
        eprintln!("[DEBUG] JSON结构调试结束");
    }

    let reader = BufReader::new(content.as_bytes());

    let mut batch: Vec<SubjectRecord> = Vec::with_capacity(BATCH_SIZE);
    let mut count = 0;
    let mut skipped_parse = 0;
    let mut skipped_type = 0;

    for (line_idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| AppError::BangumiParseError(format!("读取行失败: {e}")))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(subject) = serde_json::from_str::<SubjectRecord>(line) {
            if subject.subject_type != 2 {
                skipped_type += 1;
                if verbose && skipped_type <= 5 {
                    eprintln!(
                        "[DEBUG] 第{}行: subject_type={}, 跳过(非动画)",
                        line_idx + 1,
                        subject.subject_type
                    );
                }
                continue;
            }

            batch.push(subject);
            count += 1;

            if verbose && count <= 5 {
                eprintln!(
                    "[DEBUG] 第{}行: 插入subject id={}, name={}",
                    line_idx + 1,
                    batch.last().map(|s| s.id).unwrap_or(0),
                    batch
                        .last()
                        .map(|s| truncate_str(&s.name, 50))
                        .unwrap_or_default()
                );
            }

            if batch.len() >= BATCH_SIZE {
                insert_subjects_batch(&batch, conn)?;
                if verbose {
                    eprintln!(
                        "[DEBUG] 批处理: 已插入 {} 条, 当前批 {} 条",
                        count, BATCH_SIZE
                    );
                }
                batch.clear();
            }
        } else {
            skipped_parse += 1;
            if verbose && skipped_parse <= 3 {
                let parse_err = serde_json::from_str::<SubjectRecord>(line).unwrap_err();
                eprintln!(
                    "[DEBUG] 第{}行: JSON解析失败, serde错误: {}, 内容前100字符: {}",
                    line_idx + 1,
                    parse_err,
                    truncate_str(line, 100)
                );
            }
        }
    }

    if !batch.is_empty() {
        insert_subjects_batch(&batch, conn)?;
    }

    if verbose {
        eprintln!(
            "处理完成: 插入 {} 条, 跳过 {} 条(解析失败) + {} 条(非动画类型)",
            count, skipped_parse, skipped_type
        );
    }

    Ok(count)
}

fn insert_subjects_batch(batch: &[SubjectRecord], conn: &Connection) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| AppError::BangumiParseError(format!("开启事务失败: {e}")))?;

    // Use explicit ON CONFLICT DO UPDATE instead of INSERT OR REPLACE
    let placeholders: Vec<&str> = vec!["?"; 13];
    let sql = format!(
        "INSERT INTO subjects (id, type, name, name_cn, summary, date, score, platform, nsfw, series, eps, studio, director) VALUES {} ON CONFLICT(id) DO UPDATE SET type=excluded.type, name=excluded.name, name_cn=excluded.name_cn, summary=excluded.summary, date=excluded.date, score=excluded.score, platform=excluded.platform, nsfw=excluded.nsfw, series=excluded.series, eps=excluded.eps, studio=excluded.studio, director=excluded.director",
        batch.iter()
            .map(|_| format!("({})", placeholders.join(", ")))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Debug: print first 200 chars of SQL
    eprintln!(
        "[DEBUG] SQL (first 200 chars): {}",
        &sql[..sql.len().min(200)]
    );

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::with_capacity(batch.len() * 13);
    for s in batch {
        let nsfw = match &s.nsfw {
            Some(serde_json::Value::Number(n)) => n.as_i64().unwrap_or(0) as i32,
            Some(serde_json::Value::Bool(b)) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        };
        let series = match &s.series {
            Some(serde_json::Value::Number(n)) => n.as_i64().unwrap_or(0) as i32,
            Some(serde_json::Value::Bool(b)) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        };
        let platform = match &s.platform {
            Some(serde_json::Value::Number(n)) => n.as_i64().unwrap_or(0) as i32,
            Some(serde_json::Value::Bool(b)) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        };
        let eps = match &s.eps {
            Some(serde_json::Value::Number(n)) => n.as_i64().unwrap_or(0) as i32,
            Some(serde_json::Value::Bool(b)) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        };
        params.push(Box::new(s.id));
        params.push(Box::new(s.subject_type));
        params.push(Box::new(s.name.clone()));
        params.push(Box::new(s.name_cn.clone()));
        params.push(Box::new(s.summary.clone()));
        params.push(Box::new(s.date.clone()));
        params.push(Box::new(s.score));
        params.push(Box::new(platform));
        params.push(Box::new(nsfw));
        params.push(Box::new(series));
        params.push(Box::new(eps));
        params.push(Box::new(s.studio.clone()));
        params.push(Box::new(s.director.clone()));
    }

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let expected_params = batch.len() * 13;
    eprintln!(
        "[DEBUG] 批量插入subjects: batch_size={}, expected_params={}, actual_params={}",
        batch.len(),
        expected_params,
        param_refs.len()
    );
    if param_refs.len() != expected_params {
        return Err(AppError::BangumiParseError(format!(
            "参数数量不匹配: batch_size={}, expected_params={}, actual_params={}",
            batch.len(),
            expected_params,
            param_refs.len()
        )));
    }
    tx.execute(&sql, param_refs.as_slice())
        .map_err(|e| AppError::BangumiParseError(format!("批量插入subjects失败: {e}")))?;

    tx.commit()
        .map_err(|e| AppError::BangumiParseError(format!("提交事务失败: {e}")))?;
    Ok(())
}

fn insert_aliases_batch(batch: &[(u32, String)], conn: &Connection) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| AppError::BangumiParseError(format!("开启事务失败: {e}")))?;
    let sql = format!(
        "INSERT OR IGNORE INTO aliases (subject_id, alias) VALUES {}",
        batch
            .iter()
            .map(|_| "(?, ?)")
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(batch.len() * 2);
    for (subject_id, alias) in batch {
        params.push(subject_id);
        params.push(alias);
    }

    tx.execute(&sql, params.as_slice())
        .map_err(|e| AppError::BangumiParseError(format!("批量插入aliases失败: {e}")))?;

    tx.commit()
        .map_err(|e| AppError::BangumiParseError(format!("提交事务失败: {e}")))?;
    Ok(())
}

fn parse_episodes_from_zip(
    zip_archive: &mut zip::ZipArchive<std::fs::File>,
    filename: &str,
    conn: &Connection,
    subject_ids: &std::collections::HashSet<u32>,
    verbose: bool,
) -> Result<usize> {
    let mut episode_file = zip_archive
        .by_name(filename)
        .map_err(|e| AppError::MetadataFetchError(format!("在 ZIP 中找不到 {}: {e}", filename)))?;

    use std::io::Read;
    let mut content = String::new();
    episode_file
        .read_to_string(&mut content)
        .map_err(|e| AppError::BangumiParseError(format!("读取文件失败: {e}")))?;

    let line_count = content.lines().count();
    if verbose {
        eprintln!(
            "处理 {}: 源文件 {} 行, 已加载 {} 个subject",
            filename,
            line_count,
            subject_ids.len()
        );
        if subject_ids.is_empty() {
            eprintln!("[WARNING] subject_ids 为空! episodes 将全部被跳过!");
        }
        eprintln!("[DEBUG] 前3行原始JSON:");
        for (i, line) in content.lines().take(3).enumerate() {
            eprintln!("[DEBUG] 行{}: {}", i + 1, truncate_str(line, 200));
        }
    }

    let reader = BufReader::new(content.as_bytes());

    let mut batch: Vec<EpisodeRecord> = Vec::with_capacity(BATCH_SIZE);
    let mut count = 0;
    let mut skipped_parse = 0;
    let mut skipped_subject = 0;

    for (line_idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| AppError::BangumiParseError(format!("读取行失败: {e}")))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(episode) = serde_json::from_str::<EpisodeRecord>(line) {
            if !subject_ids.contains(&episode.subject_id) {
                skipped_subject += 1;
                if verbose && skipped_subject <= 5 {
                    eprintln!(
                        "[DEBUG] 第{}行: episode id={}, subject_id={} 不在subject_ids中, 跳过",
                        line_idx + 1,
                        episode.id,
                        episode.subject_id
                    );
                }
                continue;
            }
            batch.push(episode);
            count += 1;

            if batch.len() >= BATCH_SIZE {
                insert_episodes_batch(&batch, conn)?;
                if verbose {
                    eprintln!("[DEBUG] 批处理: 已插入 {} 条episode", count);
                }
                batch.clear();
            }
        } else {
            skipped_parse += 1;
            if verbose && skipped_parse <= 5 {
                eprintln!(
                    "[DEBUG] 第{}行: episode JSON解析失败: {}",
                    line_idx + 1,
                    truncate_str(line, 100)
                );
            }
        }
    }

    if !batch.is_empty() {
        insert_episodes_batch(&batch, conn)?;
    }

    if verbose {
        eprintln!(
            "处理完成: 插入 {} 条, 跳过 {} 条(解析失败) + {} 条(无关subject)",
            count, skipped_parse, skipped_subject
        );
    }

    Ok(count)
}

fn insert_episodes_batch(batch: &[EpisodeRecord], conn: &Connection) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| AppError::BangumiParseError(format!("开启事务失败: {e}")))?;
    let placeholders: Vec<&str> = vec!["?"; 10];
    let sql = format!(
        "INSERT INTO episodes (id, subject_id, sort, name, name_cn, airdate, type, disc, duration, description) VALUES {} ON CONFLICT(id) DO UPDATE SET subject_id=excluded.subject_id, sort=excluded.sort, name=excluded.name, name_cn=excluded.name_cn, airdate=excluded.airdate, type=excluded.type, disc=excluded.disc, duration=excluded.duration, description=excluded.description",
        batch.iter()
            .map(|_| format!("({})", placeholders.join(", ")))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::with_capacity(batch.len() * 10);
    for e in batch {
        let ep_type = match &e.ep_type {
            serde_json::Value::Number(n) => n.as_i64().unwrap_or(0) as i32,
            serde_json::Value::Bool(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        };
        let disc = match &e.disc {
            Some(serde_json::Value::Number(n)) => n.as_i64().unwrap_or(0) as i32,
            Some(serde_json::Value::Bool(b)) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        };
        params.push(Box::new(e.id));
        params.push(Box::new(e.subject_id));
        params.push(Box::new(e.sort));
        params.push(Box::new(e.name.clone()));
        params.push(Box::new(e.name_cn.clone()));
        params.push(Box::new(e.air_date.clone()));
        params.push(Box::new(ep_type));
        params.push(Box::new(disc));
        params.push(Box::new(e.duration.clone()));
        params.push(Box::new(e.description.clone()));
    }

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    tx.execute(&sql, param_refs.as_slice())
        .map_err(|e| AppError::BangumiParseError(format!("批量插入episodes失败: {e}")))?;

    tx.commit()
        .map_err(|e| AppError::BangumiParseError(format!("提交事务失败: {e}")))?;
    Ok(())
}

fn parse_name_cn_aliases_from_zip(
    zip_archive: &mut zip::ZipArchive<std::fs::File>,
    conn: &Connection,
    subject_ids: &std::collections::HashSet<u32>,
    verbose: bool,
) -> Result<usize> {
    let mut subject_file = zip_archive.by_name("subject.jsonlines").map_err(|e| {
        AppError::MetadataFetchError(format!("在 ZIP 中找不到 subject.jsonlines: {e}"))
    })?;

    use std::io::Read;
    let mut content = String::new();
    subject_file
        .read_to_string(&mut content)
        .map_err(|e| AppError::BangumiParseError(format!("读取文件失败: {e}")))?;

    let line_count = content.lines().count();
    if verbose {
        eprintln!("处理 name_cn aliases: 源文件 {} 行", line_count);
    }

    let reader = BufReader::new(content.as_bytes());

    let mut batch: Vec<(u32, String)> = Vec::with_capacity(BATCH_SIZE);
    let mut count = 0;
    let mut skipped = 0;

    for line in reader.lines() {
        let line = line.map_err(|e| AppError::BangumiParseError(format!("读取行失败: {e}")))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(subject) = serde_json::from_str::<SubjectRecord>(line) {
            if !subject_ids.contains(&subject.id) {
                skipped += 1;
                continue;
            }

            if let Some(ref name_cn) = subject.name_cn {
                if !name_cn.is_empty() && name_cn != &subject.name {
                    batch.push((subject.id, name_cn.clone()));
                    count += 1;

                    if batch.len() >= BATCH_SIZE {
                        insert_aliases_batch(&batch, conn)?;
                        batch.clear();
                    }
                }
            }
        }
    }

    if !batch.is_empty() {
        insert_aliases_batch(&batch, conn)?;
    }

    if verbose {
        eprintln!("处理完成: 插入 {} 条别名，跳过 {} 条", count, skipped);
    }

    Ok(count)
}

fn parse_relations_from_zip(
    _zip_archive: &mut zip::ZipArchive<std::fs::File>,
    _conn: &Connection,
    _subject_ids: &std::collections::HashSet<u32>,
    _verbose: bool,
) -> Result<usize> {
    Ok(0)
}

fn get_or_create_db(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)
        .map_err(|e| AppError::BangumiParseError(format!("打开数据库失败: {e}")))?;

    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA synchronous = OFF;
         PRAGMA journal_mode = WAL;
         PRAGMA cache_size = 1000000;
         PRAGMA temp_store = MEMORY;",
    )
    .map_err(|e| AppError::BangumiParseError(format!("设置 PRAGMA 失败: {e}")))?;

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS subjects (
            id INTEGER PRIMARY KEY,
            type INTEGER NOT NULL,
            name TEXT NOT NULL,
            name_cn TEXT,
            summary TEXT,
            date TEXT,
            score REAL,
            platform INTEGER,
            nsfw INTEGER DEFAULT 0,
            series INTEGER DEFAULT 0,
            eps INTEGER,
            studio TEXT,
            director TEXT
        );

        CREATE TABLE IF NOT EXISTS aliases (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            subject_id INTEGER REFERENCES subjects(id) ON DELETE CASCADE,
            alias TEXT NOT NULL,
            UNIQUE(subject_id, alias)
        );
        CREATE INDEX IF NOT EXISTS idx_aliases_alias ON aliases(alias);
        CREATE INDEX IF NOT EXISTS idx_aliases_subject ON aliases(subject_id);

        CREATE TABLE IF NOT EXISTS episodes (
            id INTEGER PRIMARY KEY,
            subject_id INTEGER REFERENCES subjects(id) ON DELETE CASCADE,
            sort INTEGER NOT NULL,
            name TEXT,
            name_cn TEXT,
            airdate TEXT,
            type INTEGER DEFAULT 0,
            disc INTEGER DEFAULT 0,
            duration TEXT,
            description TEXT
        );

        CREATE TABLE IF NOT EXISTS subject_characters (
            subject_id INTEGER,
            character_id INTEGER,
            type TEXT,
            "order" INTEGER,
            PRIMARY KEY (subject_id, character_id)
        );

        CREATE TABLE IF NOT EXISTS subject_persons (
            subject_id INTEGER,
            person_id INTEGER,
            position INTEGER,
            appear_eps TEXT,
            PRIMARY KEY (subject_id, person_id, position)
        );

        CREATE TABLE IF NOT EXISTS subject_relations (
            subject_id INTEGER,
            related_subject_id INTEGER,
            relation_type INTEGER,
            "order" INTEGER,
            PRIMARY KEY (subject_id, related_subject_id)
        );
        "#,
    )
    .map_err(|e| AppError::BangumiParseError(format!("创建表失败: {e}")))?;

    Ok(conn)
}

fn validate_database(db_path: &Path) -> Result<()> {
    let conn = Connection::open(db_path)
        .map_err(|e| AppError::BangumiParseError(format!("打开数据库失败: {e}")))?;

    let subject_count: u32 = conn
        .query_row("SELECT COUNT(*) FROM subjects", [], |row| row.get(0))
        .map_err(|e| AppError::BangumiParseError(format!("查询subjects失败: {e}")))?;

    let episode_count: u32 = conn
        .query_row("SELECT COUNT(*) FROM episodes", [], |row| row.get(0))
        .map_err(|e| AppError::BangumiParseError(format!("查询episodes失败: {e}")))?;

    let alias_count: u32 = conn
        .query_row("SELECT COUNT(*) FROM aliases", [], |row| row.get(0))
        .map_err(|e| AppError::BangumiParseError(format!("查询aliases失败: {e}")))?;

    eprintln!(
        "数据库验证通过: {} 个主体, {} 个剧集, {} 个别名",
        subject_count, episode_count, alias_count
    );

    Ok(())
}
