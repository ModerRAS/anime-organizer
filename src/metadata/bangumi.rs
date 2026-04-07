//! Bangumi Archive 客户端
//!
//! 通过 GitHub 托管的 Bangumi Archive 仓库获取动画元数据，
//! 无需 API Key，直接访问静态 JSON 文件。
//!
//! ## 数据源
//!
//! - 单条查询：`https://raw.githubusercontent.com/bangumi/archive/master/data/subject/{id}.json`
//! - 批量数据：Bangumi Archive Release 中的 `subject.jsonlines`（`type=2` 为动画）

use crate::error::{AppError, Result};
use crate::metadata::wiki::WikiParser;
use crate::metadata::AnimeMetadata;
#[cfg(feature = "metadata")]
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::BufRead;
use std::path::{Path, PathBuf};

#[cfg(feature = "scraper")]
use rusqlite::Connection;

/// Bangumi Archive 基础 URL
const BANGUMI_ARCHIVE_BASE: &str = "https://raw.githubusercontent.com/bangumi/archive/master/data";

/// Bangumi Archive Release URL
const BANGUMI_ARCHIVE_RELEASE: &str = "https://github.com/bangumi/archive/releases/latest/download";

/// 默认本地 dump 文件名
const SUBJECT_DUMP_FILENAME: &str = "subject.jsonlines";

/// 默认本地数据库文件名
#[cfg(feature = "scraper")]
const BANGUMI_DATABASE_FILENAME: &str = "bangumi.db";

/// Bangumi Subject 基本信息（从 Archive 获取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiSubject {
    /// Subject ID
    pub id: u32,
    /// 类型（2 = 动画）
    #[serde(rename = "type")]
    pub subject_type: u32,
    /// 标题
    pub name: String,
    /// 中文标题
    #[serde(default)]
    pub name_cn: Option<String>,
    /// 简介
    #[serde(default)]
    pub summary: Option<String>,
    /// 放送日期
    #[serde(default)]
    pub date: Option<String>,
    /// 评分
    #[serde(default)]
    pub score: Option<f32>,
    /// 话数
    #[serde(default)]
    pub eps: Option<u32>,
    /// Wiki Infobox 原始文本
    #[serde(default)]
    pub infobox: Option<String>,
}

/// Bangumi Archive 客户端
///
/// 从 Bangumi Archive GitHub 仓库获取动画元数据。
/// 支持单条在线查询和本地缓存的批量数据查询。
///
/// # 示例
///
/// ```no_run
/// # async fn example() -> anime_organizer::error::Result<()> {
/// use anime_organizer::metadata::BangumiClient;
///
/// let client = BangumiClient::new(None);
/// let metadata = client.fetch_metadata(328609).await?;
/// println!("{}: {}", metadata.title, metadata.summary);
/// # Ok(())
/// # }
/// ```
pub struct BangumiClient {
    #[cfg(feature = "metadata")]
    http: reqwest::Client,
    /// 本地缓存目录
    cache_dir: PathBuf,
    /// 显式指定的数据源（subject.jsonlines）
    source_path: Option<PathBuf>,
    /// 内存中的 Subject 索引（从 dump 加载后缓存）
    index: std::sync::Mutex<Option<HashMap<u32, BangumiSubject>>>,
    #[cfg(feature = "scraper")]
    /// 内存中的别名索引（从数据库加载后缓存）：alias -> (subject_id, canonical_name)
    aliases_index: std::sync::Mutex<Option<HashMap<String, (u32, String)>>>,
}

impl BangumiClient {
    /// 创建新的 Bangumi 客户端
    ///
    /// # 参数
    ///
    /// - `cache_dir` - 本地缓存目录，默认使用系统临时目录下的 `bangumi-cache`
    pub fn new(cache_dir: Option<PathBuf>) -> Self {
        Self::with_source(cache_dir, None)
    }

    /// 创建新的 Bangumi 客户端，并允许显式指定 dump 路径。
    pub fn with_source(cache_dir: Option<PathBuf>, source_path: Option<PathBuf>) -> Self {
        let cache_dir = cache_dir.unwrap_or_else(|| std::env::temp_dir().join("bangumi-cache"));
        Self {
            #[cfg(feature = "metadata")]
            http: reqwest::Client::builder()
                .user_agent("anime-organizer/0.1")
                .build()
                .expect("创建 HTTP 客户端失败"),
            cache_dir,
            source_path,
            index: std::sync::Mutex::new(None),
            #[cfg(feature = "scraper")]
            aliases_index: std::sync::Mutex::new(None),
        }
    }

    /// 从 Bangumi Archive 在线获取单个 Subject 信息
    #[cfg(feature = "metadata")]
    pub async fn fetch_subject(&self, bangumi_id: u32) -> Result<BangumiSubject> {
        let url = format!("{BANGUMI_ARCHIVE_BASE}/subject/{bangumi_id}.json");
        let resp =
            self.http.get(&url).send().await.map_err(|e| {
                AppError::MetadataFetchError(format!("请求 Bangumi Archive 失败: {e}"))
            })?;

        if !resp.status().is_success() {
            return Err(AppError::AnimeNotFoundError(format!(
                "Bangumi Subject {bangumi_id} 未找到 (HTTP {})",
                resp.status()
            )));
        }

        let subject: BangumiSubject = resp
            .json()
            .await
            .map_err(|e| AppError::BangumiParseError(format!("解析 Subject JSON 失败: {e}")))?;

        Ok(subject)
    }

    /// 获取完整的动画元数据
    ///
    /// 从 Archive 获取 Subject 信息并解析 Wiki Infobox。
    #[cfg(feature = "metadata")]
    pub async fn fetch_metadata(&self, bangumi_id: u32) -> Result<AnimeMetadata> {
        self.prepare_index().await?;

        if let Some(subject) = self.find_by_id(bangumi_id)? {
            return Ok(self.subject_to_metadata(&subject));
        }

        let subject = self.fetch_subject(bangumi_id).await?;
        Ok(self.subject_to_metadata(&subject))
    }

    /// 从本地 dump 文件中按 ID 查找
    pub fn find_by_id(&self, bangumi_id: u32) -> Result<Option<BangumiSubject>> {
        let index = self
            .index
            .lock()
            .map_err(|e| AppError::BangumiParseError(format!("锁定索引失败: {e}")))?;
        Ok(index.as_ref().and_then(|idx| idx.get(&bangumi_id).cloned()))
    }

    /// 从本地 dump 文件中按名称精确查找
    pub fn find_by_name(&self, name: &str) -> Result<Option<BangumiSubject>> {
        let index = self
            .index
            .lock()
            .map_err(|e| AppError::BangumiParseError(format!("锁定索引失败: {e}")))?;
        if let Some(subject) = index.as_ref().and_then(|idx| {
            idx.values()
                .find(|s| s.name == name || s.name_cn.as_deref() == Some(name))
                .cloned()
        }) {
            return Ok(Some(subject));
        }
        #[cfg(feature = "scraper")]
        {
            drop(index);
            if let Ok(Some((bangumi_id, _))) = self.find_by_alias(name) {
                return self.find_by_id(bangumi_id);
            }
        }
        Ok(None)
    }

    #[cfg(feature = "scraper")]
    pub fn find_by_alias(&self, alias: &str) -> Result<Option<(u32, String)>> {
        self.prepare_aliases_index()?;
        let aliases_index = self
            .aliases_index
            .lock()
            .map_err(|e| AppError::BangumiParseError(format!("锁定别名索引失败: {e}")))?;
        Ok(aliases_index
            .as_ref()
            .and_then(|idx| idx.get(alias).cloned()))
    }

    #[cfg(feature = "scraper")]
    pub fn get_aliases(&self, bangumi_id: u32) -> Result<Vec<String>> {
        self.prepare_aliases_index()?;
        let db_path = self.cache_dir.join(BANGUMI_DATABASE_FILENAME);
        if !db_path.is_file() {
            return Ok(Vec::new());
        }
        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::BangumiParseError(format!("打开数据库失败: {e}")))?;
        let mut stmt = conn
            .prepare("SELECT alias FROM aliases WHERE subject_id = ?1")
            .map_err(|e| AppError::BangumiParseError(format!("预处理 SQL 失败: {e}")))?;
        let aliases = stmt
            .query_map([bangumi_id], |row| row.get::<_, String>(0))
            .map_err(|e| AppError::BangumiParseError(format!("查询别名失败: {e}")))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| AppError::BangumiParseError(format!("读取别名失败: {e}")))?;
        Ok(aliases)
    }

    #[cfg(feature = "scraper")]
    fn prepare_aliases_index(&self) -> Result<()> {
        {
            let aliases_index = self
                .aliases_index
                .lock()
                .map_err(|e| AppError::BangumiParseError(format!("锁定别名索引失败: {e}")))?;
            if aliases_index.is_some() {
                return Ok(());
            }
        }
        let db_path = self.cache_dir.join(BANGUMI_DATABASE_FILENAME);
        if !db_path.is_file() {
            let mut aliases_index = self
                .aliases_index
                .lock()
                .map_err(|e| AppError::BangumiParseError(format!("锁定别名索引失败: {e}")))?;
            *aliases_index = Some(HashMap::new());
            return Ok(());
        }
        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::BangumiParseError(format!("打开数据库失败: {e}")))?;
        let mut stmt = conn
            .prepare(
                "SELECT a.alias, a.subject_id, s.name FROM aliases a JOIN subjects s ON a.subject_id = s.id",
            )
            .map_err(|e| AppError::BangumiParseError(format!("预处理 SQL 失败: {e}")))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, u32>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| AppError::BangumiParseError(format!("查询别名失败: {e}")))?;
        let mut aliases: HashMap<String, (u32, String)> = HashMap::new();
        for row in rows {
            let (alias, subject_id, name) =
                row.map_err(|e| AppError::BangumiParseError(format!("读取别名失败: {e}")))?;
            aliases.insert(alias, (subject_id, name));
        }
        let mut aliases_index = self
            .aliases_index
            .lock()
            .map_err(|e| AppError::BangumiParseError(format!("锁定别名索引失败: {e}")))?;
        *aliases_index = Some(aliases);
        Ok(())
    }

    #[cfg(feature = "scraper")]
    fn resolve_existing_db_path(&self) -> Option<PathBuf> {
        let db_path = self.cache_dir.join(BANGUMI_DATABASE_FILENAME);
        if db_path.is_file() {
            Some(db_path)
        } else {
            None
        }
    }

    /// 从本地 dump 文件中模糊搜索
    pub fn search(&self, query: &str) -> Result<Vec<BangumiSubject>> {
        let query_lower = query.to_lowercase();
        let index = self
            .index
            .lock()
            .map_err(|e| AppError::BangumiParseError(format!("锁定索引失败: {e}")))?;
        Ok(index
            .as_ref()
            .map(|idx| {
                idx.values()
                    .filter(|s| {
                        s.name.to_lowercase().contains(&query_lower)
                            || s.name_cn
                                .as_ref()
                                .is_some_and(|cn| cn.to_lowercase().contains(&query_lower))
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default())
    }

    /// 加载本地 JSONL dump 文件
    ///
    /// 从缓存目录中加载 `subject.jsonlines`，仅保留 `type=2`（动画）的条目。
    pub fn load_dump(&self, dump_path: &Path) -> Result<usize> {
        let file = std::fs::File::open(dump_path)
            .map_err(|e| AppError::BangumiParseError(format!("打开 dump 文件失败: {e}")))?;

        let reader = std::io::BufReader::new(file);
        let mut subjects = HashMap::new();

        for line in reader.lines() {
            let line = line.map_err(|e| AppError::BangumiParseError(format!("读取行失败: {e}")))?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Ok(subject) = serde_json::from_str::<BangumiSubject>(line) {
                // 仅保留动画类型
                if subject.subject_type == 2 {
                    subjects.insert(subject.id, subject);
                }
            }
        }

        let count = subjects.len();
        let mut index = self
            .index
            .lock()
            .map_err(|e| AppError::BangumiParseError(format!("锁定索引失败: {e}")))?;
        *index = Some(subjects);

        Ok(count)
    }

    /// 准备本地索引：优先加载现有 dump；若不存在则尝试下载。
    #[cfg(feature = "metadata")]
    pub async fn prepare_index(&self) -> Result<usize> {
        if let Some(len) = self.index_len()? {
            return Ok(len);
        }

        if let Some(path) = self.resolve_existing_dump_path() {
            return self.load_dump(&path);
        }

        let dump_path = self.download_dump().await?;
        self.load_dump(&dump_path)
    }

    fn index_len(&self) -> Result<Option<usize>> {
        let index = self
            .index
            .lock()
            .map_err(|e| AppError::BangumiParseError(format!("锁定索引失败: {e}")))?;
        Ok(index.as_ref().map(HashMap::len))
    }

    pub fn resolve_existing_dump_path(&self) -> Option<PathBuf> {
        let candidates = [
            self.source_path.clone(),
            Some(self.cache_dir.join(SUBJECT_DUMP_FILENAME)),
        ];

        for candidate in candidates.into_iter().flatten() {
            if candidate.is_file() {
                return Some(candidate);
            }

            let nested = candidate.join(SUBJECT_DUMP_FILENAME);
            if nested.is_file() {
                return Some(nested);
            }
        }

        None
    }

    /// 下载最新的 Bangumi Archive dump
    #[cfg(feature = "metadata")]
    pub async fn download_dump(&self) -> Result<PathBuf> {
        std::fs::create_dir_all(&self.cache_dir)
            .map_err(|e| AppError::MetadataFetchError(format!("创建缓存目录失败: {e}")))?;

        let dump_path = self.cache_dir.join(SUBJECT_DUMP_FILENAME);

        let plain_url = format!("{BANGUMI_ARCHIVE_RELEASE}/{SUBJECT_DUMP_FILENAME}");
        if let Ok(resp) = self.http.get(&plain_url).send().await {
            if resp.status().is_success() {
                let bytes = resp.bytes().await.map_err(|e| {
                    AppError::MetadataFetchError(format!("读取 dump 数据失败: {e}"))
                })?;
                std::fs::write(&dump_path, &bytes).map_err(|e| {
                    AppError::MetadataFetchError(format!("写入 dump 文件失败: {e}"))
                })?;
                return Ok(dump_path);
            }
        }

        let gz_url = format!("{BANGUMI_ARCHIVE_RELEASE}/{SUBJECT_DUMP_FILENAME}.gz");
        let resp =
            self.http.get(&gz_url).send().await.map_err(|e| {
                AppError::MetadataFetchError(format!("下载 Bangumi dump 失败: {e}"))
            })?;

        if !resp.status().is_success() {
            return Err(AppError::MetadataFetchError(format!(
                "下载 dump 失败 (HTTP {})",
                resp.status()
            )));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| AppError::MetadataFetchError(format!("读取 dump 数据失败: {e}")))?;

        let mut decoder = GzDecoder::new(bytes.as_ref());
        let mut content = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut content)
            .map_err(|e| AppError::MetadataFetchError(format!("解压 dump 失败: {e}")))?;

        std::fs::write(&dump_path, &content)
            .map_err(|e| AppError::MetadataFetchError(format!("写入 dump 文件失败: {e}")))?;

        Ok(dump_path)
    }

    /// 获取缓存目录路径
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// 检查本地 dump 是否存在
    pub fn has_local_dump(&self) -> bool {
        self.resolve_existing_dump_path().is_some()
    }

    /// 将 BangumiSubject 转为 AnimeMetadata
    fn subject_to_metadata(&self, subject: &BangumiSubject) -> AnimeMetadata {
        let mut metadata = AnimeMetadata::new(subject.id, subject.name.clone());

        metadata.title_cn = subject.name_cn.clone();
        metadata.original_title = subject.name.clone();
        metadata.summary = subject.summary.clone().unwrap_or_default();
        metadata.air_date = subject.date.clone();
        metadata.rating = subject.score.unwrap_or(0.0);
        metadata.episode_count = subject.eps.unwrap_or(0);

        // 解析 Wiki Infobox 提取额外信息
        if let Some(ref infobox) = subject.infobox {
            let parser = WikiParser::new();
            if let Ok(info) = parser.parse_anime_infobox(infobox) {
                if info.studio.is_some() {
                    metadata.studio = info.studio;
                }
                if info.director.is_some() {
                    metadata.director = info.director;
                }
            }
        }

        metadata
    }
}

impl Default for BangumiClient {
    fn default() -> Self {
        Self::new(None)
    }
}
