use crate::error::{AppError, Result};
#[cfg(feature = "clouddrive")]
use crate::rss::proxy::{build_http_client, ProxyConfig};
use crate::scraper::download::HttpDownloader;
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

// infobox: Bangumi Archive stores raw wiki text, NOT JSON.
// e.g. "{{|type=动画|话数=12|制作公司=Studio X|}}"
// Store as-is; parse with wiki-parser later if needed.
fn deserialize_infobox<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(s) => Ok(Some(s)),
        _ => Ok(None),
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Tag {
    name: String,
    #[serde(default)]
    count: u32,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct ScoreDetails {
    #[serde(default)]
    total: u32,
    #[serde(default)]
    d10: u32,
    #[serde(default)]
    d9: u32,
    #[serde(default)]
    d8: u32,
    #[serde(default)]
    d7: u32,
    #[serde(default)]
    d6: u32,
    #[serde(default)]
    d5: u32,
    #[serde(default)]
    d4: u32,
    #[serde(default)]
    d3: u32,
    #[serde(default)]
    d2: u32,
    #[serde(default)]
    d1: u32,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct Favorite {
    #[serde(default)]
    wish: u32,
    #[serde(default)]
    collect: u32,
    #[serde(default)]
    doing: u32,
    #[serde(default)]
    dropped: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SubjectRecord {
    id: u32,
    #[serde(rename = "type")]
    subject_type: u32,
    name: String,
    #[serde(default)]
    name_cn: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_infobox")]
    infobox: Option<String>,
    #[serde(default)]
    platform: u32,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    nsfw: bool,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    score: Option<f64>,
    #[serde(default)]
    tags: Vec<Tag>,
    #[serde(default)]
    score_details: Option<ScoreDetails>,
    #[serde(default)]
    rank: u32,
    #[serde(default)]
    favorite: Option<Favorite>,
    #[serde(default)]
    meta_tags: Vec<String>,
    #[serde(default)]
    series: bool,
}

#[derive(Debug, Deserialize)]
struct LatestVersion {
    #[allow(dead_code)]
    name: String,
    #[serde(rename = "browser_download_url")]
    download_url: String,
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

/// Record for subject_characters.jsonlines
#[derive(Debug, Deserialize)]
struct SubjectCharacterRecord {
    #[serde(rename = "subject_id")]
    subject_id: u32,
    #[serde(rename = "character_id")]
    character_id: u32,
    #[serde(rename = "type", default)]
    character_type: Option<String>,
    #[serde(default)]
    order: Option<i32>,
}

/// Record for subject_persons.jsonlines
#[derive(Debug, Deserialize)]
struct SubjectPersonRecord {
    #[serde(rename = "subject_id")]
    subject_id: u32,
    #[serde(rename = "person_id")]
    person_id: u32,
    #[serde(default)]
    position: Option<String>,
    #[serde(rename = "appear_eps", default)]
    appear_eps: Option<String>,
}

/// Record for subject_relations.jsonlines
#[derive(Debug, Deserialize)]
struct SubjectRelationRecord {
    #[serde(rename = "subject_id")]
    subject_id: u32,
    #[serde(rename = "related_subject_id")]
    related_subject_id: u32,
    #[serde(rename = "relation_type", default)]
    relation_type: Option<i32>,
    #[serde(default)]
    order: Option<i32>,
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
    let zip_path = download_zip(&latest_url, &temp_path)
        .await
        .map_err(|e| AppError::MetadataFetchError(format!("下载任务失败: {e}")))?;

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
    #[cfg(feature = "clouddrive")]
    let client = {
        let proxy_config = ProxyConfig::from_env();
        build_http_client(&proxy_config)
            .map_err(|e| AppError::MetadataFetchError(format!("创建 HTTP 客户端失败: {e}")))?
    };

    #[cfg(not(feature = "clouddrive"))]
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(60))
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
    let zip_path = temp_dir.join("bangumi_dump.zip");

    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| AppError::MetadataFetchError(format!("创建 HTTP 客户端失败: {e}")))?;

    let downloader =
        HttpDownloader::with_client(url.to_string(), temp_dir.to_path_buf(), client, true)
            .with_output_path(zip_path.clone())
            .with_chunk_count(16);

    downloader
        .download_with_progress()
        .await
        .map_err(|e| AppError::MetadataFetchError(format!("下载失败: {e}")))?;

    if !zip_path.exists() {
        return Err(AppError::MetadataFetchError(
            "下载完成但文件不存在".to_string(),
        ));
    }

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

    if verbose {
        let zip_contents: Vec<String> = (0..zip_archive.len())
            .filter_map(|i| {
                zip_archive
                    .by_index(i)
                    .ok()
                    .map(|f| (f.name().to_string(), f.size()))
            })
            .map(|(name, size)| format!("{} ({} bytes)", name, size))
            .collect();
        eprintln!("[INFO] ZIP 包含 {} 个文件:", zip_contents.len());
        for name in &zip_contents {
            eprintln!("[INFO]   - {}", name);
        }
        eprintln!("[INFO] 开始解析数据...\n");
    }

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
        let chars_count =
            parse_characters_from_zip(&mut zip_archive, &conn, &subject_ids, verbose)?;
        if verbose && chars_count > 0 {
            eprintln!("[DEBUG] 解析了 {} 条角色关系", chars_count);
        }
        let persons_count = parse_persons_from_zip(&mut zip_archive, &conn, &subject_ids, verbose)?;
        if verbose && persons_count > 0 {
            eprintln!("[DEBUG] 解析了 {} 条人物关系", persons_count);
        }
        let rels_count = parse_relations_from_zip(&mut zip_archive, &conn, &subject_ids, verbose)?;
        if verbose && rels_count > 0 {
            eprintln!("[DEBUG] 解析了 {} 条subject关系", rels_count);
        }
        chars_count + persons_count + rels_count
    } else {
        0
    };

    if verbose {
        eprintln!(
            "[INFO] ===== 数据库构建完成 =====\n\
             [INFO] subjects: {} 条\n\
             [INFO] episodes: {} 条\n\
             [INFO] aliases: {} 条\n\
             [INFO] relations: {} 条\n\
             [INFO] ==============================",
            subjects_count, episodes_count, aliases_count, relations_count
        );
    }

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

fn insert_subjects_batch(batch: &[SubjectRecord], conn: &Connection) -> Result<usize> {
    if batch.is_empty() {
        return Ok(0);
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| AppError::BangumiParseError(format!("开启事务失败: {e}")))?;

    let placeholders: Vec<&str> = vec!["?"; 13];
    let sql = format!(
        "INSERT INTO subjects (id, type, name, name_cn, summary, date, score, platform, nsfw, series, studio, director, infobox) VALUES {} ON CONFLICT(id) DO UPDATE SET type=excluded.type, name=excluded.name, name_cn=excluded.name_cn, summary=excluded.summary, date=excluded.date, score=excluded.score, platform=excluded.platform, nsfw=excluded.nsfw, series=excluded.series, studio=excluded.studio, director=excluded.director, infobox=excluded.infobox",
        batch.iter()
            .map(|_| format!("({})", placeholders.join(", ")))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::with_capacity(batch.len() * 13);
    let mut infobox_aliases: Vec<(u32, String)> = Vec::new();

    for s in batch {
        let nsfw_val: i32 = if s.nsfw { 1 } else { 0 };
        let series_val: i32 = if s.series { 1 } else { 0 };
        let platform_val: i32 = s.platform as i32;
        let infobox_fields =
            crate::scraper::wiki_parser::InfoboxFields::parse(s.infobox.as_deref().unwrap_or(""));
        infobox_aliases.extend(
            infobox_fields
                .aliases
                .into_iter()
                .filter(|a| !a.is_empty() && a != &s.name)
                .map(|a| (s.id, a)),
        );
        params.push(Box::new(s.id));
        params.push(Box::new(s.subject_type));
        params.push(Box::new(s.name.clone()));
        let cleaned =
            crate::scraper::wiki_parser::cleanup_value(s.name_cn.as_deref().unwrap_or(""));
        params.push(if cleaned.is_empty() {
            Box::new(None::<String>) as Box<dyn rusqlite::ToSql>
        } else {
            Box::new(cleaned)
        });
        params.push(Box::new(s.summary.clone()));
        params.push(Box::new(s.date.clone()));
        params.push(Box::new(s.score));
        params.push(Box::new(platform_val));
        params.push(Box::new(nsfw_val));
        params.push(Box::new(series_val));
        params.push(Box::new(infobox_fields.studio));
        params.push(Box::new(infobox_fields.director));
        params.push(Box::new(s.infobox.clone()));
    }

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    tx.execute(&sql, param_refs.as_slice())
        .map_err(|e| AppError::BangumiParseError(format!("批量插入subjects失败: {e}")))?;

    let alias_count = infobox_aliases.len();
    if !infobox_aliases.is_empty() {
        let alias_sql = format!(
            "INSERT OR IGNORE INTO aliases (subject_id, alias) VALUES {}",
            infobox_aliases
                .iter()
                .map(|_| "(?, ?)")
                .collect::<Vec<_>>()
                .join(", ")
        );
        let mut alias_params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(alias_count * 2);
        for (subject_id, alias) in &infobox_aliases {
            alias_params.push(subject_id);
            alias_params.push(alias);
        }
        tx.execute(&alias_sql, alias_params.as_slice())
            .map_err(|e| AppError::BangumiParseError(format!("批量插入infobox别名失败: {e}")))?;
    }

    tx.commit()
        .map_err(|e| AppError::BangumiParseError(format!("提交事务失败: {e}")))?;
    Ok(alias_count)
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
    // Use episode id as primary key for proper deduplication
    // Use DO NOTHING to avoid overwriting existing records on conflict
    let sql = format!(
        "INSERT INTO episodes (id, subject_id, sort, name, name_cn, airdate, type, disc, duration, description) VALUES {} ON CONFLICT(id) DO NOTHING",
        batch.iter()
            .map(|_| format!("({})", placeholders.join(", ")))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::with_capacity(batch.len() * 10);
    for e in batch {
        let ep_type = match &e.ep_type {
            serde_json::Value::Number(n) => n.as_i64().unwrap_or(0) as i32,
            serde_json::Value::Bool(b) if *b => 1,
            serde_json::Value::Bool(_) => 0,
            _ => 0,
        };
        let disc = match &e.disc {
            Some(serde_json::Value::Number(n)) => n.as_i64().unwrap_or(0) as i32,
            Some(serde_json::Value::String(s)) => s.parse().unwrap_or(0),
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
    zip_archive: &mut zip::ZipArchive<std::fs::File>,
    conn: &Connection,
    subject_ids: &std::collections::HashSet<u32>,
    verbose: bool,
) -> Result<usize> {
    let filename = "subject_relations.jsonlines";
    let mut relation_file = match zip_archive.by_name(filename) {
        Ok(f) => f,
        Err(_) => {
            if verbose {
                eprintln!("[DEBUG] 文件 {} 不在ZIP中，跳过", filename);
            }
            return Ok(0);
        }
    };

    use std::io::Read;
    let mut content = String::new();
    relation_file
        .read_to_string(&mut content)
        .map_err(|e| AppError::BangumiParseError(format!("读取文件失败: {e}")))?;

    let line_count = content.lines().count();
    if verbose {
        eprintln!("处理 {}: 源文件 {} 行", filename, line_count);
    }

    let reader = BufReader::new(content.as_bytes());

    let mut batch: Vec<SubjectRelationRecord> = Vec::with_capacity(BATCH_SIZE);
    let mut count = 0;
    let mut skipped_parse = 0;
    let mut skipped_subject = 0;
    let mut processed_lines = 0;

    for line in reader.lines() {
        let line = line.map_err(|e| AppError::BangumiParseError(format!("读取行失败: {e}")))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        processed_lines += 1;

        if verbose && processed_lines <= 3 {
            eprintln!(
                "[DEBUG] 关系行{}: {}",
                processed_lines,
                truncate_str(line, 150)
            );
        }

        if let Ok(relation) = serde_json::from_str::<SubjectRelationRecord>(line) {
            if !subject_ids.contains(&relation.subject_id) {
                skipped_subject += 1;
                continue;
            }
            batch.push(relation);
            count += 1;

            if batch.len() >= BATCH_SIZE {
                insert_relations_batch(&batch, conn)?;
                if verbose {
                    eprintln!(
                        "[DEBUG] 关系批处理: 已插入 {} 条, 当前批 {} 条",
                        count, BATCH_SIZE
                    );
                }
                batch.clear();
            }
        } else {
            skipped_parse += 1;
            if verbose && skipped_parse <= 3 {
                eprintln!("[DEBUG] 关系第{}行: JSON解析失败", processed_lines);
            }
        }
    }

    if !batch.is_empty() {
        insert_relations_batch(&batch, conn)?;
    }

    if verbose {
        eprintln!(
            "[INFO] 关系处理完成: 插入 {} 条, 跳过 {} 条(解析失败) + {} 条(无关subject), 总行数 {}",
            count, skipped_parse, skipped_subject, processed_lines
        );
    }

    Ok(count)
}

fn insert_relations_batch(batch: &[SubjectRelationRecord], conn: &Connection) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| AppError::BangumiParseError(format!("开启事务失败: {e}")))?;
    let sql = format!(
        "INSERT INTO subject_relations (subject_id, related_subject_id, relation_type, \"order\") VALUES {} ON CONFLICT(subject_id, related_subject_id) DO NOTHING",
        batch
            .iter()
            .map(|_| "(?, ?, ?, ?)")
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(batch.len() * 4);
    for r in batch {
        params.push(&r.subject_id);
        params.push(&r.related_subject_id);
        params.push(&r.relation_type);
        params.push(&r.order);
    }

    tx.execute(&sql, params.as_slice())
        .map_err(|e| AppError::BangumiParseError(format!("批量插入relations失败: {e}")))?;

    tx.commit()
        .map_err(|e| AppError::BangumiParseError(format!("提交事务失败: {e}")))?;
    Ok(())
}

fn parse_characters_from_zip(
    zip_archive: &mut zip::ZipArchive<std::fs::File>,
    conn: &Connection,
    subject_ids: &std::collections::HashSet<u32>,
    verbose: bool,
) -> Result<usize> {
    let filename = "subject_characters.jsonlines";
    let mut char_file = match zip_archive.by_name(filename) {
        Ok(f) => f,
        Err(_) => {
            if verbose {
                eprintln!("[DEBUG] 文件 {} 不在ZIP中，跳过", filename);
            }
            return Ok(0);
        }
    };

    use std::io::Read;
    let mut content = String::new();
    char_file
        .read_to_string(&mut content)
        .map_err(|e| AppError::BangumiParseError(format!("读取文件失败: {e}")))?;

    let line_count = content.lines().count();
    if verbose {
        eprintln!("处理 {}: 源文件 {} 行", filename, line_count);
    }

    let reader = BufReader::new(content.as_bytes());

    let mut batch: Vec<SubjectCharacterRecord> = Vec::with_capacity(BATCH_SIZE);
    let mut count = 0;
    let mut skipped_parse = 0;
    let mut skipped_subject = 0;
    let mut processed_lines = 0;

    for line in reader.lines() {
        let line = line.map_err(|e| AppError::BangumiParseError(format!("读取行失败: {e}")))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        processed_lines += 1;

        if verbose && processed_lines <= 3 {
            eprintln!(
                "[DEBUG] 角色行{}: {}",
                processed_lines,
                truncate_str(line, 150)
            );
        }

        if let Ok(char_rec) = serde_json::from_str::<SubjectCharacterRecord>(line) {
            if !subject_ids.contains(&char_rec.subject_id) {
                skipped_subject += 1;
                continue;
            }
            batch.push(char_rec);
            count += 1;

            if batch.len() >= BATCH_SIZE {
                insert_characters_batch(&batch, conn)?;
                if verbose {
                    eprintln!(
                        "[DEBUG] 角色批处理: 已插入 {} 条, 当前批 {} 条",
                        count, BATCH_SIZE
                    );
                }
                batch.clear();
            }
        } else {
            skipped_parse += 1;
            if verbose && skipped_parse <= 3 {
                eprintln!("[DEBUG] 角色第{}行: JSON解析失败", processed_lines);
            }
        }
    }

    if !batch.is_empty() {
        insert_characters_batch(&batch, conn)?;
    }

    if verbose {
        eprintln!(
            "[INFO] 角色处理完成: 插入 {} 条, 跳过 {} 条(解析失败) + {} 条(无关subject), 总行数 {}",
            count, skipped_parse, skipped_subject, processed_lines
        );
    }

    Ok(count)
}

fn insert_characters_batch(batch: &[SubjectCharacterRecord], conn: &Connection) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| AppError::BangumiParseError(format!("开启事务失败: {e}")))?;
    let sql = format!(
        "INSERT INTO subject_characters (subject_id, character_id, type, \"order\") VALUES {} ON CONFLICT(subject_id, character_id) DO NOTHING",
        batch
            .iter()
            .map(|_| "(?, ?, ?, ?)")
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(batch.len() * 4);
    for c in batch {
        params.push(&c.subject_id);
        params.push(&c.character_id);
        params.push(&c.character_type);
        params.push(&c.order);
    }

    tx.execute(&sql, params.as_slice())
        .map_err(|e| AppError::BangumiParseError(format!("批量插入characters失败: {e}")))?;

    tx.commit()
        .map_err(|e| AppError::BangumiParseError(format!("提交事务失败: {e}")))?;
    Ok(())
}

fn parse_persons_from_zip(
    zip_archive: &mut zip::ZipArchive<std::fs::File>,
    conn: &Connection,
    subject_ids: &std::collections::HashSet<u32>,
    verbose: bool,
) -> Result<usize> {
    let filename = "subject_persons.jsonlines";
    let mut person_file = match zip_archive.by_name(filename) {
        Ok(f) => f,
        Err(_) => {
            if verbose {
                eprintln!("[DEBUG] 文件 {} 不在ZIP中，跳过", filename);
            }
            return Ok(0);
        }
    };

    use std::io::Read;
    let mut content = String::new();
    person_file
        .read_to_string(&mut content)
        .map_err(|e| AppError::BangumiParseError(format!("读取文件失败: {e}")))?;

    let line_count = content.lines().count();
    if verbose {
        eprintln!("处理 {}: 源文件 {} 行", filename, line_count);
    }

    let reader = BufReader::new(content.as_bytes());

    let mut batch: Vec<SubjectPersonRecord> = Vec::with_capacity(BATCH_SIZE);
    let mut count = 0;
    let mut skipped_parse = 0;
    let mut skipped_subject = 0;
    let mut processed_lines = 0;

    for line in reader.lines() {
        let line = line.map_err(|e| AppError::BangumiParseError(format!("读取行失败: {e}")))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        processed_lines += 1;

        if verbose && processed_lines <= 3 {
            eprintln!(
                "[DEBUG] 人物行{}: {}",
                processed_lines,
                truncate_str(line, 150)
            );
        }

        if let Ok(person_rec) = serde_json::from_str::<SubjectPersonRecord>(line) {
            if !subject_ids.contains(&person_rec.subject_id) {
                skipped_subject += 1;
                continue;
            }
            batch.push(person_rec);
            count += 1;

            if batch.len() >= BATCH_SIZE {
                insert_persons_batch(&batch, conn)?;
                if verbose {
                    eprintln!(
                        "[DEBUG] 人物批处理: 已插入 {} 条, 当前批 {} 条",
                        count, BATCH_SIZE
                    );
                }
                batch.clear();
            }
        } else {
            skipped_parse += 1;
            if verbose && skipped_parse <= 3 {
                eprintln!("[DEBUG] 人物第{}行: JSON解析失败", processed_lines);
            }
        }
    }

    if !batch.is_empty() {
        insert_persons_batch(&batch, conn)?;
    }

    if verbose {
        eprintln!(
            "[INFO] 人物处理完成: 插入 {} 条, 跳过 {} 条(解析失败) + {} 条(无关subject), 总行数 {}",
            count, skipped_parse, skipped_subject, processed_lines
        );
    }

    Ok(count)
}

fn insert_persons_batch(batch: &[SubjectPersonRecord], conn: &Connection) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| AppError::BangumiParseError(format!("开启事务失败: {e}")))?;
    let sql = format!(
        "INSERT INTO subject_persons (subject_id, person_id, position, appear_eps) VALUES {} ON CONFLICT(subject_id, person_id, position) DO NOTHING",
        batch
            .iter()
            .map(|_| "(?, ?, ?, ?)")
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(batch.len() * 4);
    for p in batch {
        params.push(&p.subject_id);
        params.push(&p.person_id);
        params.push(&p.position);
        params.push(&p.appear_eps);
    }

    tx.execute(&sql, params.as_slice())
        .map_err(|e| AppError::BangumiParseError(format!("批量插入persons失败: {e}")))?;

    tx.commit()
        .map_err(|e| AppError::BangumiParseError(format!("提交事务失败: {e}")))?;
    Ok(())
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
            platform INTEGER NOT NULL DEFAULT 0,
            nsfw INTEGER NOT NULL DEFAULT 0,
            series INTEGER NOT NULL DEFAULT 0,
            studio TEXT,
            director TEXT,
            infobox TEXT
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
            position TEXT,
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

    let char_count: u32 = conn
        .query_row("SELECT COUNT(*) FROM subject_characters", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    let person_count: u32 = conn
        .query_row("SELECT COUNT(*) FROM subject_persons", [], |row| row.get(0))
        .unwrap_or(0);

    let relation_count: u32 = conn
        .query_row("SELECT COUNT(*) FROM subject_relations", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    eprintln!(
        "[INFO] 数据库验证通过:\n\
         [INFO]   - subjects: {} 个主体\n\
         [INFO]   - episodes: {} 个剧集\n\
         [INFO]   - aliases: {} 个别名\n\
         [INFO]   - subject_characters: {} 个角色\n\
         [INFO]   - subject_persons: {} 个人物\n\
         [INFO]   - subject_relations: {} 个关系",
        subject_count, episode_count, alias_count, char_count, person_count, relation_count
    );

    Ok(())
}
