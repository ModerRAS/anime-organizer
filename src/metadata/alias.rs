//! 别名库查找
//!
//! 提供动画名称别名到 Bangumi/TMDB/AniDB ID 的映射查找功能。
//! 别名数据从 Bangumi SQLite 或 AnimeAtlas SQLite 数据库加载。

use regex::Regex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::LazyLock;

use crate::error::{AppError, Result};

enum AliasDatabaseSchema {
    Bangumi,
    AnimeAtlas,
}

struct AnimeAtlasAliasRow {
    alias: String,
    bangumi_id: String,
    title: Option<String>,
    metadata_json: String,
    tmdb_id: Option<String>,
    anidb_id: Option<String>,
    season_number: Option<u32>,
}

static RELEASE_TITLE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(?P<title>.+?)\s+-\s+(?:ep?\s*)?\d{1,4}(?:[vV]\d+)?(?:\s.*)?$")
        .expect("标题清洗正则表达式编译失败")
});

static FILE_EXTENSION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\.[a-z0-9]{2,4}$").expect("扩展名清洗正则表达式编译失败"));

static LEADING_GROUP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:\[[^\]]+\]|【[^】]+】|\([^\)]+\))\s*").expect("发布组清洗正则表达式编译失败")
});

static TRAILING_TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\s*(?:\[[^\]]*\]|【[^】]*】|\([^\)]*\)|（[^）]*）)+\s*$")
        .expect("标签清洗正则表达式编译失败")
});

/// 别名条目
///
/// 存储单个别名对应的 Bangumi、TMDB 和 AniDB ID。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasEntry {
    /// Bangumi Subject ID
    pub bangumi_id: u32,
    /// 标准名称（日文）
    pub name: String,
    /// TMDB ID
    pub tmdb_id: Option<u32>,
    /// AniDB ID
    pub anidb_id: Option<u32>,
}

#[derive(Debug, Clone)]
struct SeriesAliasCandidate {
    entry: AliasEntry,
    season_number: Option<u32>,
}

/// 别名库查找
///
/// 管理动画名称别名到元数据 ID 的映射关系，支持从 Bangumi 或 AnimeAtlas SQLite 数据库加载。
///
/// # 示例
///
/// ```no_run
/// use anime_organizer::metadata::AliasLookup;
///
/// let lookup = AliasLookup::load("bangumi.db".as_ref()).unwrap();
/// if let Some(entry) = lookup.find("孤独摇滚") {
///     println!("Bangumi ID: {}", entry.bangumi_id);
/// }
/// ```
pub struct AliasLookup {
    aliases: HashMap<String, AliasEntry>,
    normalized_keys: HashMap<String, String>,
    series_aliases: HashMap<String, Vec<SeriesAliasCandidate>>,
    normalized_series_aliases: HashMap<String, Vec<SeriesAliasCandidate>>,
}

impl AliasLookup {
    /// 从可选的数据源加载别名库。
    ///
    /// - `db_path` 存在时，从 Bangumi 或 AnimeAtlas SQLite 别名表加载。
    /// - `alias_file` 存在时，从 JSON 文件加载，并覆盖同名数据库别名。
    /// - 两者都缺失时，返回空的别名库，供首次运行时回退到名称搜索。
    pub fn load_from_sources(db_path: Option<&Path>, alias_file: Option<&Path>) -> Result<Self> {
        let mut aliases = HashMap::new();
        let mut series_aliases = HashMap::new();

        if let Some(path) = db_path.filter(|path| path.is_file()) {
            Self::load_database_aliases(path, &mut aliases, &mut series_aliases)?;
        }

        if let Some(path) = alias_file {
            Self::load_alias_file(path, &mut aliases, &mut series_aliases)?;
        }

        Ok(Self::from_aliases(aliases, series_aliases))
    }

    /// 从 Bangumi 或 AnimeAtlas SQLite 数据库加载别名库
    ///
    /// # 参数
    ///
    /// - `db_path` - Bangumi 数据库路径（通常为 `bangumi.db`）。
    pub fn load(db_path: &Path) -> Result<Self> {
        if !db_path.exists() {
            return Err(AppError::AliasLoadError(format!(
                "数据库文件不存在: {}",
                db_path.display()
            )));
        }

        let mut aliases = HashMap::new();
        let mut series_aliases = HashMap::new();
        Self::load_database_aliases(db_path, &mut aliases, &mut series_aliases)?;

        Ok(Self::from_aliases(aliases, series_aliases))
    }

    fn from_aliases(
        mut aliases: HashMap<String, AliasEntry>,
        series_aliases: HashMap<String, Vec<SeriesAliasCandidate>>,
    ) -> Self {
        expand_generated_aliases(&mut aliases);

        let normalized_series_aliases = build_normalized_series_aliases(&series_aliases);
        let ambiguous_series_keys: HashSet<_> = normalized_series_aliases
            .iter()
            .filter(|(_, candidates)| unique_candidate_count(candidates) > 1)
            .map(|(key, _)| key.clone())
            .collect();
        aliases.retain(|alias, _| !ambiguous_series_keys.contains(&normalize_name(alias)));

        let normalized_keys = build_normalized_index(&aliases);

        Self {
            aliases,
            normalized_keys,
            series_aliases,
            normalized_series_aliases,
        }
    }

    fn load_database_aliases(
        db_path: &Path,
        aliases: &mut HashMap<String, AliasEntry>,
        series_aliases: &mut HashMap<String, Vec<SeriesAliasCandidate>>,
    ) -> Result<()> {
        let conn = Connection::open(db_path)
            .map_err(|e| AppError::AliasLoadError(format!("打开数据库失败: {e}")))?;

        match detect_database_schema(&conn)? {
            AliasDatabaseSchema::Bangumi => Self::load_bangumi_database_aliases(&conn, aliases),
            AliasDatabaseSchema::AnimeAtlas => {
                Self::load_animeatlas_database_aliases(&conn, aliases)?;
                if table_exists(&conn, "series_aliases")? && table_exists(&conn, "series")? {
                    Self::load_animeatlas_series_aliases(&conn, series_aliases)?;
                }
                Ok(())
            }
        }
    }

    fn load_bangumi_database_aliases(
        conn: &Connection,
        aliases: &mut HashMap<String, AliasEntry>,
    ) -> Result<()> {
        let mut stmt = conn
            .prepare(
                "SELECT a.alias, a.subject_id, s.name FROM aliases a JOIN subjects s ON a.subject_id = s.id",
            )
            .map_err(|e| AppError::AliasLoadError(format!("预处理 SQL 失败: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, u32>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| AppError::AliasLoadError(format!("查询别名失败: {e}")))?;

        for row in rows {
            let (alias, subject_id, name) =
                row.map_err(|e| AppError::AliasLoadError(format!("读取别名失败: {e}")))?;
            let entry = AliasEntry {
                bangumi_id: subject_id,
                name,
                tmdb_id: None,
                anidb_id: None,
            };
            aliases.insert(alias, entry);
        }

        Ok(())
    }

    fn load_animeatlas_database_aliases(
        conn: &Connection,
        aliases: &mut HashMap<String, AliasEntry>,
    ) -> Result<()> {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT
                    a.value,
                    bgm.provider_id,
                    m.title,
                    m.metadata_json,
                    (SELECT provider_id FROM provider_refs WHERE media_id = a.media_id AND provider = 'tmdb' LIMIT 1),
                    (SELECT provider_id FROM provider_refs WHERE media_id = a.media_id AND provider = 'anidb' LIMIT 1),
                    NULL
                FROM aliases a
                JOIN provider_refs bgm
                    ON bgm.media_id = a.media_id
                    AND bgm.provider = 'bangumi'
                    AND bgm.entity = 'subject'
                JOIN media m ON m.id = a.media_id
                "#,
            )
            .map_err(|e| AppError::AliasLoadError(format!("预处理 AnimeAtlas SQL 失败: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(AnimeAtlasAliasRow {
                    alias: row.get::<_, String>(0)?,
                    bangumi_id: row.get::<_, String>(1)?,
                    title: row.get::<_, Option<String>>(2)?,
                    metadata_json: row.get::<_, String>(3)?,
                    tmdb_id: row.get::<_, Option<String>>(4)?,
                    anidb_id: row.get::<_, Option<String>>(5)?,
                    season_number: row.get::<_, Option<u32>>(6)?,
                })
            })
            .map_err(|e| AppError::AliasLoadError(format!("查询 AnimeAtlas 别名失败: {e}")))?;

        for row in rows {
            let row = row
                .map_err(|e| AppError::AliasLoadError(format!("读取 AnimeAtlas 别名失败: {e}")))?;
            let alias = row.alias.trim();
            if alias.is_empty() {
                continue;
            }

            aliases.insert(alias.to_string(), animeatlas_alias_entry(&row)?);
        }

        Ok(())
    }

    fn load_animeatlas_series_aliases(
        conn: &Connection,
        aliases: &mut HashMap<String, Vec<SeriesAliasCandidate>>,
    ) -> Result<()> {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT
                    a.value,
                    bgm.provider_id,
                    m.title,
                    m.metadata_json,
                    (SELECT provider_id FROM provider_refs WHERE media_id = m.id AND provider = 'tmdb' LIMIT 1),
                    (SELECT provider_id FROM provider_refs WHERE media_id = m.id AND provider = 'anidb' LIMIT 1),
                    m.season_number
                FROM series_aliases a
                JOIN media m ON m.series_id = a.series_id
                JOIN provider_refs bgm
                    ON bgm.media_id = m.id
                    AND bgm.provider = 'bangumi'
                    AND bgm.entity = 'subject'
                "#,
            )
            .map_err(|e| {
                AppError::AliasLoadError(format!("预处理 AnimeAtlas 系列别名 SQL 失败: {e}"))
            })?;

        let rows = stmt
            .query_map([], |row| {
                Ok(AnimeAtlasAliasRow {
                    alias: row.get::<_, String>(0)?,
                    bangumi_id: row.get::<_, String>(1)?,
                    title: row.get::<_, Option<String>>(2)?,
                    metadata_json: row.get::<_, String>(3)?,
                    tmdb_id: row.get::<_, Option<String>>(4)?,
                    anidb_id: row.get::<_, Option<String>>(5)?,
                    season_number: row.get::<_, Option<u32>>(6)?,
                })
            })
            .map_err(|e| AppError::AliasLoadError(format!("查询 AnimeAtlas 系列别名失败: {e}")))?;

        for row in rows {
            let row = row.map_err(|e| {
                AppError::AliasLoadError(format!("读取 AnimeAtlas 系列别名失败: {e}"))
            })?;
            let alias = row.alias.trim();
            if alias.is_empty() {
                continue;
            }
            let season_number = row.season_number;
            let candidate = SeriesAliasCandidate {
                entry: animeatlas_alias_entry(&row)?,
                season_number,
            };
            let candidates = aliases.entry(alias.to_string()).or_default();
            if !candidates
                .iter()
                .any(|existing| existing.entry.bangumi_id == candidate.entry.bangumi_id)
            {
                candidates.push(candidate);
            }
        }

        Ok(())
    }

    fn load_alias_file(
        alias_file: &Path,
        aliases: &mut HashMap<String, AliasEntry>,
        series_aliases: &mut HashMap<String, Vec<SeriesAliasCandidate>>,
    ) -> Result<()> {
        let content = std::fs::read_to_string(alias_file).map_err(|e| {
            AppError::AliasLoadError(format!(
                "读取自定义别名文件失败 {}: {e}",
                alias_file.display()
            ))
        })?;
        let file_aliases: HashMap<String, AliasEntry> =
            serde_json::from_str(&content).map_err(|e| {
                AppError::AliasLoadError(format!(
                    "解析自定义别名文件失败 {}: {e}",
                    alias_file.display()
                ))
            })?;

        for (alias, entry) in file_aliases {
            let trimmed = alias.trim();
            if trimmed.is_empty() {
                continue;
            }

            series_aliases.retain(|key, _| normalize_name(key) != normalize_name(trimmed));
            aliases.insert(trimmed.to_string(), entry);
        }

        Ok(())
    }

    /// 根据名称精确查找无歧义别名。
    pub fn find(&self, name: &str) -> Option<&AliasEntry> {
        if let Some(candidates) = self.series_aliases.get(name) {
            return select_series_candidate(candidates, None);
        }
        self.aliases.get(name)
    }

    /// 根据名称和 Season 查找 v2 系列别名；无 Season 的多媒体候选不会猜测。
    pub fn find_with_season(&self, name: &str, season_hint: Option<u32>) -> Option<&AliasEntry> {
        if let Some(candidates) = self.series_aliases.get(name) {
            return select_series_candidate(candidates, season_hint);
        }
        if let Some(entry) = self.aliases.get(name) {
            return Some(entry);
        }

        for candidate in extract_lookup_candidates(name) {
            let normalized = normalize_name(&candidate);
            if let Some(candidates) = self.normalized_series_aliases.get(&normalized) {
                return select_series_candidate(candidates, season_hint);
            }
            if let Some(entry) = self
                .normalized_keys
                .get(&normalized)
                .and_then(|key| self.aliases.get(key))
            {
                return Some(entry);
            }
        }
        None
    }

    /// 根据名称模糊查找（大小写不敏感，去除空格）
    pub fn find_fuzzy(&self, name: &str) -> Option<&AliasEntry> {
        extract_lookup_candidates(name)
            .into_iter()
            .find_map(|candidate| {
                let normalized = normalize_name(&candidate);
                if let Some(candidates) = self.normalized_series_aliases.get(&normalized) {
                    return select_series_candidate(candidates, None);
                }
                self.normalized_keys
                    .get(&normalized)
                    .and_then(|key| self.aliases.get(key))
            })
    }

    /// 根据 Bangumi ID 反向查找
    pub fn find_by_bangumi_id(&self, bangumi_id: u32) -> Option<(&str, &AliasEntry)> {
        self.aliases
            .iter()
            .find(|(_, v)| v.bangumi_id == bangumi_id)
            .map(|(k, v)| (k.as_str(), v))
    }

    /// 返回别名库中的条目数量
    pub fn len(&self) -> usize {
        self.aliases.len() + self.series_aliases.len()
    }

    /// 别名库是否为空
    pub fn is_empty(&self) -> bool {
        self.aliases.is_empty() && self.series_aliases.is_empty()
    }

    /// 返回底层无歧义媒体别名表，供匹配/导出流程使用。
    pub fn entries(&self) -> &HashMap<String, AliasEntry> {
        &self.aliases
    }
}

fn build_normalized_index(aliases: &HashMap<String, AliasEntry>) -> HashMap<String, String> {
    let mut index = HashMap::new();

    for (key, entry) in aliases {
        index.insert(normalize_name(key), key.clone());
        index
            .entry(normalize_name(&entry.name))
            .or_insert_with(|| key.clone());
    }

    index
}

fn build_normalized_series_aliases(
    aliases: &HashMap<String, Vec<SeriesAliasCandidate>>,
) -> HashMap<String, Vec<SeriesAliasCandidate>> {
    let mut index: HashMap<String, Vec<SeriesAliasCandidate>> = HashMap::new();
    for (alias, candidates) in aliases {
        let normalized = normalize_name(alias);
        let target = index.entry(normalized).or_default();
        for candidate in candidates {
            if !target
                .iter()
                .any(|existing| existing.entry.bangumi_id == candidate.entry.bangumi_id)
            {
                target.push(candidate.clone());
            }
        }
    }
    index
}

fn unique_candidate_count(candidates: &[SeriesAliasCandidate]) -> usize {
    candidates
        .iter()
        .map(|candidate| candidate.entry.bangumi_id)
        .collect::<HashSet<_>>()
        .len()
}

fn select_series_candidate(
    candidates: &[SeriesAliasCandidate],
    season_hint: Option<u32>,
) -> Option<&AliasEntry> {
    let mut matches = candidates.iter().filter(|candidate| {
        season_hint.is_none_or(|season| candidate.season_number == Some(season))
    });
    let first = matches.next()?;
    matches
        .all(|candidate| candidate.entry.bangumi_id == first.entry.bangumi_id)
        .then_some(&first.entry)
}

fn expand_generated_aliases(aliases: &mut HashMap<String, AliasEntry>) {
    let snapshot: Vec<(String, AliasEntry)> = aliases
        .iter()
        .map(|(key, entry)| (key.clone(), entry.clone()))
        .collect();
    let mut seen = aliases.keys().cloned().collect::<HashSet<_>>();

    for (key, entry) in snapshot {
        for variant in generate_alias_variants(&key, &entry.name) {
            if variant.is_empty() || !seen.insert(variant.clone()) {
                continue;
            }

            aliases.insert(variant, entry.clone());
        }
    }
}

fn generate_alias_variants(key: &str, canonical_name: &str) -> Vec<String> {
    let mut variants = Vec::new();

    for source in [key, canonical_name] {
        push_variant(&mut variants, source.to_lowercase());
        push_variant(&mut variants, normalize_spacing(source));
        push_variant(&mut variants, normalize_punctuation(source));
        push_variant(
            &mut variants,
            normalize_spacing(&normalize_punctuation(source)),
        );
        push_variant(&mut variants, remove_ascii_punctuation(source));
        push_variant(&mut variants, source.replace('×', "x"));
        push_variant(&mut variants, source.replace('×', "X"));
        push_variant(&mut variants, source.replace('·', " "));
        push_variant(&mut variants, source.replace('・', " "));
        push_variant(&mut variants, source.replace('/', " "));
        push_variant(&mut variants, source.replace('／', " "));
        push_variant(&mut variants, source.replace(':', " "));
        push_variant(&mut variants, source.replace(';', " "));
        push_variant(&mut variants, source.replace('!', ""));
        push_variant(&mut variants, source.replace('！', ""));
        push_variant(&mut variants, source.replace('?', ""));
        push_variant(&mut variants, source.replace('？', ""));
        push_variant(&mut variants, source.replace('～', " "));
        push_variant(&mut variants, source.replace('〜', " "));
        push_variant(&mut variants, source.split_whitespace().collect::<String>());

        if source.contains(':') {
            let prefix = source.split(':').next().unwrap_or_default().trim();
            push_variant(&mut variants, prefix.to_string());
        }
    }

    variants
}

fn push_variant(variants: &mut Vec<String>, value: String) {
    let trimmed = value.trim();
    if trimmed.len() > 1 && !variants.iter().any(|item| item == trimmed) {
        variants.push(trimmed.to_string());
    }
}

fn normalize_spacing(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_punctuation(value: &str) -> String {
    value
        .replace(['·', '・'], " ")
        .replace(['／', '/'], " ")
        .replace(['：', ':'], " ")
        .replace(['；', ';'], " ")
        .replace(['～', '〜'], " ")
        .replace('×', "x")
}

fn remove_ascii_punctuation(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_alphanumeric() || ch.is_alphabetic() || ch.is_whitespace())
        .collect::<String>()
}

/// 规范化名称：去除发布组/标签/集数并压缩为可比对键。
fn normalize_name(name: &str) -> String {
    extract_lookup_candidates(name)
        .into_iter()
        .next()
        .unwrap_or_else(|| compact_name(name))
}

fn extract_lookup_candidates(name: &str) -> Vec<String> {
    let mut value = name.trim().to_string();

    value = FILE_EXTENSION_REGEX.replace(&value, "").into_owned();

    if RELEASE_TITLE_REGEX.is_match(&value) {
        value = RELEASE_TITLE_REGEX
            .replace(&value, "$title")
            .trim()
            .to_string();
    }

    loop {
        let replaced = LEADING_GROUP_REGEX.replace(&value, "").to_string();
        if replaced == value {
            break;
        }
        value = replaced.trim().to_string();
    }

    value = TRAILING_TAG_REGEX.replace(&value, "").trim().to_string();

    let mut candidates = Vec::new();
    for part in value.split(['/', '／', '|']) {
        let cleaned = part.trim();
        if cleaned.is_empty() {
            continue;
        }
        let compact = compact_name(cleaned);
        if !compact.is_empty() && !candidates.contains(&compact) {
            candidates.push(compact);
        }
    }

    if candidates.is_empty() {
        let compact = compact_name(&value);
        if !compact.is_empty() {
            candidates.push(compact);
        }
    }

    candidates
}

fn compact_name(name: &str) -> String {
    name.chars()
        .filter(|ch| ch.is_alphanumeric() || ch.is_alphabetic())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn detect_database_schema(conn: &Connection) -> Result<AliasDatabaseSchema> {
    if table_exists(conn, "aliases")?
        && table_exists(conn, "media")?
        && table_exists(conn, "provider_refs")?
    {
        return Ok(AliasDatabaseSchema::AnimeAtlas);
    }

    if table_exists(conn, "aliases")?
        && table_exists(conn, "subjects")?
        && column_exists(conn, "aliases", "alias")?
        && column_exists(conn, "aliases", "subject_id")?
    {
        return Ok(AliasDatabaseSchema::Bangumi);
    }

    Err(AppError::AliasLoadError(
        "不支持的别名数据库格式，期望 Bangumi 或 AnimeAtlas SQLite".to_string(),
    ))
}

fn table_exists(conn: &Connection, table: &str) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        [table],
        |row| row.get::<_, i64>(0),
    )
    .map(|exists| exists != 0)
    .map_err(|e| AppError::AliasLoadError(format!("检查数据库表失败: {e}")))
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| AppError::AliasLoadError(format!("检查数据库列失败: {e}")))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| AppError::AliasLoadError(format!("检查数据库列失败: {e}")))?;

    for row in rows {
        if row.map_err(|e| AppError::AliasLoadError(format!("读取数据库列失败: {e}")))? == column
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn animeatlas_alias_entry(row: &AnimeAtlasAliasRow) -> Result<AliasEntry> {
    Ok(AliasEntry {
        bangumi_id: parse_provider_id(&row.bangumi_id, "bangumi")?,
        name: row
            .title
            .as_deref()
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| metadata_json_title(&row.metadata_json))
            .unwrap_or_else(|| row.alias.trim().to_string()),
        tmdb_id: parse_optional_provider_id(row.tmdb_id.as_deref(), "tmdb")?,
        anidb_id: parse_optional_provider_id(row.anidb_id.as_deref(), "anidb")?,
    })
}

fn metadata_json_title(metadata_json: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(metadata_json)
        .ok()?
        .get("title")?
        .as_str()
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_provider_id(value: &str, provider: &str) -> Result<u32> {
    value.trim().parse::<u32>().map_err(|e| {
        AppError::AliasLoadError(format!(
            "解析 AnimeAtlas {provider} provider_id 失败: {value}: {e}"
        ))
    })
}

fn parse_optional_provider_id(value: Option<&str>, provider: &str) -> Result<Option<u32>> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| parse_provider_id(value, provider))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();

        // Create schema
        conn.execute_batch(
            r#"
            CREATE TABLE subjects (
                id INTEGER PRIMARY KEY,
                type INTEGER NOT NULL,
                name TEXT NOT NULL,
                name_cn TEXT
            );
            CREATE TABLE aliases (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                subject_id INTEGER REFERENCES subjects(id) ON DELETE CASCADE,
                alias TEXT NOT NULL,
                UNIQUE(subject_id, alias)
            );
            "#,
        )
        .unwrap();

        // Insert test data
        conn.execute(
            "INSERT INTO subjects (id, type, name, name_cn) VALUES (?1, 2, ?2, ?3)",
            rusqlite::params![1, "進撃の巨人", "进击的巨人"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO aliases (subject_id, alias) VALUES (1, '进击的巨人')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO subjects (id, type, name, name_cn) VALUES (?1, 2, ?2, ?3)",
            rusqlite::params![2, "CLANNAD", "CLANNAD"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO aliases (subject_id, alias) VALUES (2, 'clannad')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO subjects (id, type, name, name_cn) VALUES (?1, 2, ?2, ?3)",
            rusqlite::params![3, "ぼっち・ざ・ろっく！", "孤独摇滚"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO aliases (subject_id, alias) VALUES (3, '孤独摇滚')",
            [],
        )
        .unwrap();

        dir
    }

    fn create_animeatlas_test_db() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("animeatlas.sqlite");
        let conn = Connection::open(&db_path).unwrap();

        conn.execute_batch(
            r#"
            CREATE TABLE media (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                title TEXT,
                summary TEXT,
                metadata_json TEXT NOT NULL,
                provenance_json TEXT NOT NULL
            );
            CREATE TABLE aliases (
                media_id TEXT NOT NULL REFERENCES media(id),
                value TEXT NOT NULL,
                normalized TEXT NOT NULL,
                language TEXT,
                type TEXT,
                source TEXT,
                confidence REAL,
                PRIMARY KEY (media_id, value)
            );
            CREATE TABLE provider_refs (
                media_id TEXT NOT NULL REFERENCES media(id),
                provider TEXT NOT NULL,
                entity TEXT NOT NULL,
                provider_id TEXT NOT NULL,
                provider_key TEXT NOT NULL UNIQUE
            );
            CREATE TABLE release_info (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            "#,
        )
        .unwrap();

        conn.execute(
            "INSERT INTO media (id, kind, title, summary, metadata_json, provenance_json) VALUES (?1, 'anime', ?2, '', ?3, '{}')",
            rusqlite::params!["media-000001", "葬送のフリーレン", r#"{"title":"葬送のフリーレン"}"#],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO aliases (media_id, value, normalized, language, type, source, confidence) VALUES ('media-000001', '芙莉莲', '芙莉莲', 'zh-Hans', 'localized', 'community', 0.9)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO provider_refs (media_id, provider, entity, provider_id, provider_key) VALUES ('media-000001', 'bangumi', 'subject', '414680', 'anime:bangumi:subject:414680')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO provider_refs (media_id, provider, entity, provider_id, provider_key) VALUES ('media-000001', 'tmdb', 'tv', '209867', 'anime:tmdb:tv:209867')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO provider_refs (media_id, provider, entity, provider_id, provider_key) VALUES ('media-000001', 'anidb', 'anime', '18597', 'anime:anidb:anime:18597')",
            [],
        )
        .unwrap();

        dir
    }

    fn create_animeatlas_v2_test_db() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("animeatlas-v2.sqlite");
        let conn = Connection::open(&db_path).unwrap();

        conn.execute_batch(
            r#"
            CREATE TABLE series (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                relationships_json TEXT NOT NULL
            );
            CREATE TABLE media (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                series_id TEXT REFERENCES series(id),
                title TEXT,
                season_number INTEGER,
                part_number INTEGER,
                cour_number INTEGER,
                summary TEXT,
                metadata_json TEXT NOT NULL,
                provenance_json TEXT NOT NULL
            );
            CREATE TABLE series_aliases (
                series_id TEXT NOT NULL REFERENCES series(id),
                value TEXT NOT NULL,
                normalized TEXT NOT NULL,
                language TEXT,
                type TEXT,
                source TEXT,
                confidence REAL,
                PRIMARY KEY (series_id, value)
            );
            CREATE TABLE aliases (
                media_id TEXT NOT NULL REFERENCES media(id),
                value TEXT NOT NULL,
                normalized TEXT NOT NULL,
                language TEXT,
                type TEXT,
                source TEXT,
                confidence REAL,
                PRIMARY KEY (media_id, value)
            );
            CREATE TABLE provider_refs (
                media_id TEXT NOT NULL REFERENCES media(id),
                provider TEXT NOT NULL,
                entity TEXT NOT NULL,
                provider_id TEXT NOT NULL,
                provider_key TEXT NOT NULL UNIQUE
            );

            INSERT INTO series VALUES ('series-frieren', '葬送的芙莉莲', '{}');
            INSERT INTO media VALUES ('media-frieren-s1', 'anime', 'series-frieren', '葬送的芙莉莲', 1, NULL, NULL, '', '{}', '{}');
            INSERT INTO media VALUES ('media-frieren-s2', 'anime', 'series-frieren', '葬送的芙莉莲 第二季', 2, NULL, NULL, '', '{}', '{}');
            INSERT INTO series_aliases VALUES ('series-frieren', '葬送的芙莉莲', '葬送的芙莉莲', 'zh-Hans', 'localized', 'community', 1.0);
            INSERT INTO series_aliases VALUES ('series-frieren', 'Sousou no Frieren', 'sousou no frieren', 'ja-Latn', 'romanized', 'community', 1.0);
            INSERT INTO aliases VALUES ('media-frieren-s1', '芙莉莲', '芙莉莲', 'zh-Hans', 'localized', 'community', 1.0);
            INSERT INTO aliases VALUES ('media-frieren-s2', '芙莉莲 第二季', '芙莉莲 第二季', 'zh-Hans', 'localized', 'community', 1.0);
            INSERT INTO provider_refs VALUES ('media-frieren-s1', 'bangumi', 'subject', '400602', 'anime:bangumi:subject:400602');
            INSERT INTO provider_refs VALUES ('media-frieren-s2', 'bangumi', 'subject', '515759', 'anime:bangumi:subject:515759');

            INSERT INTO series VALUES ('series-bocchi', 'ぼっち・ざ・ろっく！', '{}');
            INSERT INTO media VALUES ('media-bocchi', 'anime', 'series-bocchi', 'ぼっち・ざ・ろっく！', 1, NULL, NULL, '', '{}', '{}');
            INSERT INTO series_aliases VALUES ('series-bocchi', '孤独摇滚', '孤独摇滚', 'zh-Hans', 'localized', 'community', 1.0);
            INSERT INTO aliases VALUES ('media-bocchi', 'ぼっち・ざ・ろっく！', 'ぼっち・ざ・ろっく！', 'ja', 'official', 'community', 1.0);
            INSERT INTO provider_refs VALUES ('media-bocchi', 'bangumi', 'subject', '378862', 'anime:bangumi:subject:378862');
            "#,
        )
        .unwrap();

        dir
    }

    #[test]
    fn test_load_from_database() {
        let dir = create_test_db();
        let db_path = dir.path().join("test.db");
        let lookup = AliasLookup::load(&db_path).unwrap();
        assert!(!lookup.is_empty());
        assert!(
            lookup.len() >= 3,
            "Expected >= 3 aliases after expansion, got {}",
            lookup.len()
        );
    }

    #[test]
    fn test_load_from_animeatlas_database() {
        let dir = create_animeatlas_test_db();
        let db_path = dir.path().join("animeatlas.sqlite");
        let lookup = AliasLookup::load(&db_path).unwrap();
        let entry = lookup.find("芙莉莲").unwrap();
        assert_eq!(entry.bangumi_id, 414680);
        assert_eq!(entry.name, "葬送のフリーレン");
        assert_eq!(entry.tmdb_id, Some(209867));
        assert_eq!(entry.anidb_id, Some(18597));
    }

    #[test]
    fn test_load_from_animeatlas_v2_database() {
        let dir = create_animeatlas_v2_test_db();
        let lookup = AliasLookup::load(&dir.path().join("animeatlas-v2.sqlite")).unwrap();

        assert!(lookup.find("葬送的芙莉莲").is_none());
        assert!(lookup.find_fuzzy("Sousou no Frieren").is_none());
        assert_eq!(
            lookup
                .find_with_season("葬送的芙莉莲", Some(1))
                .unwrap()
                .bangumi_id,
            400602
        );
        assert_eq!(
            lookup
                .find_with_season("[ANi] Sousou no Frieren - 01.mkv", Some(2))
                .unwrap()
                .bangumi_id,
            515759
        );
        assert!(lookup.find_with_season("葬送的芙莉莲", Some(3)).is_none());
        assert_eq!(lookup.find("孤独摇滚").unwrap().bangumi_id, 378862);
        assert_eq!(lookup.find("芙莉莲 第二季").unwrap().bangumi_id, 515759);
    }

    #[test]
    fn test_animeatlas_v2_custom_alias_overrides_series_candidates() {
        let dir = create_animeatlas_v2_test_db();
        let alias_path = dir.path().join("aliases.json");
        std::fs::write(
            &alias_path,
            r#"{
  "葬送的芙莉莲": {
    "bangumi_id": 999,
    "name": "Custom Frieren",
    "tmdb_id": null,
    "anidb_id": null
  }
}"#,
        )
        .unwrap();

        let lookup = AliasLookup::load_from_sources(
            Some(&dir.path().join("animeatlas-v2.sqlite")),
            Some(&alias_path),
        )
        .unwrap();
        assert_eq!(lookup.find("葬送的芙莉莲").unwrap().bangumi_id, 999);
        assert_eq!(
            lookup
                .find_with_season("葬送的芙莉莲", Some(2))
                .unwrap()
                .bangumi_id,
            999
        );
    }

    #[test]
    fn test_find_exact_match() {
        let dir = create_test_db();
        let db_path = dir.path().join("test.db");
        let lookup = AliasLookup::load(&db_path).unwrap();
        let entry = lookup.find("进击的巨人");
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.bangumi_id, 1);
    }

    #[test]
    fn test_find_returns_none_for_unknown() {
        let dir = create_test_db();
        let db_path = dir.path().join("test.db");
        let lookup = AliasLookup::load(&db_path).unwrap();
        assert!(lookup.find("这个动画不存在_xyz_123").is_none());
    }

    #[test]
    fn test_find_fuzzy() {
        let dir = create_test_db();
        let db_path = dir.path().join("test.db");
        let lookup = AliasLookup::load(&db_path).unwrap();
        let entry = lookup.find_fuzzy("CLANNAD");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().bangumi_id, 2);
    }

    #[test]
    fn test_find_by_bangumi_id() {
        let dir = create_test_db();
        let db_path = dir.path().join("test.db");
        let lookup = AliasLookup::load(&db_path).unwrap();
        let result = lookup.find_by_bangumi_id(1);
        assert!(result.is_some());
        let (key, entry) = result.unwrap();
        // entry.name is the canonical Japanese name which is always correct
        assert_eq!(entry.name, "進撃の巨人");
        // key could be either the Chinese alias or canonical name due to expand_generated_aliases
        assert!(
            key == "进击的巨人" || key == "進撃の巨人",
            "key was '{}'",
            key
        );
    }

    #[test]
    fn test_normalize_name() {
        assert_eq!(normalize_name("Hello World"), "helloworld");
        assert_eq!(normalize_name("ぼっち・ざ・ろっく"), "ぼっちざろっく");
    }

    #[test]
    fn test_find_fuzzy_from_release_filename() {
        let dir = create_test_db();
        let db_path = dir.path().join("test.db");
        let lookup = AliasLookup::load(&db_path).unwrap();
        let entry = lookup.find_fuzzy("[ANi] 进击的巨人 / Attack on Titan - 01 [1080P].mp4");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().name, "進撃の巨人");
    }

    #[test]
    fn test_load_nonexistent_db() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("nonexistent.db");
        let result = AliasLookup::load(&db_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_sources_returns_empty_when_nothing_exists() {
        let lookup = AliasLookup::load_from_sources(None, None).unwrap();
        assert!(lookup.is_empty());
    }

    #[test]
    fn test_load_from_sources_with_custom_alias_file() {
        let dir = tempfile::tempdir().unwrap();
        let alias_path = dir.path().join("aliases.json");
        std::fs::write(
            &alias_path,
            r#"{
  "芙莉莲": {
    "bangumi_id": 100,
    "name": "葬送のフリーレン",
    "tmdb_id": 209867,
    "anidb_id": 18597
  }
}"#,
        )
        .unwrap();

        let lookup = AliasLookup::load_from_sources(None, Some(&alias_path)).unwrap();
        let entry = lookup.find("芙莉莲").unwrap();
        assert_eq!(entry.bangumi_id, 100);
        assert_eq!(entry.name, "葬送のフリーレン");
        assert_eq!(entry.tmdb_id, Some(209867));
        assert_eq!(entry.anidb_id, Some(18597));
    }

    #[test]
    fn test_load_from_sources_alias_file_overrides_database() {
        let dir = create_test_db();
        let db_path = dir.path().join("test.db");
        let alias_path = dir.path().join("aliases.json");
        std::fs::write(
            &alias_path,
            r#"{
  "进击的巨人": {
    "bangumi_id": 999,
    "name": "Attack on Titan Override",
    "tmdb_id": 12345,
    "anidb_id": null
  }
}"#,
        )
        .unwrap();

        let lookup = AliasLookup::load_from_sources(Some(&db_path), Some(&alias_path)).unwrap();
        let entry = lookup.find("进击的巨人").unwrap();
        assert_eq!(entry.bangumi_id, 999);
        assert_eq!(entry.name, "Attack on Titan Override");
        assert_eq!(entry.tmdb_id, Some(12345));
    }
}
