//! 别名库查找
//!
//! 提供动画名称别名到 Bangumi/TMDB/AniDB ID 的映射查找功能。
//! 别名数据通过 `rust-embed` 嵌入到二进制文件中，也支持用户自定义别名文件覆盖。

use regex::Regex;
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::LazyLock;

use crate::error::{AppError, Result};

/// 嵌入的数据资源
#[derive(Embed)]
#[folder = "data/"]
struct DataAssets;

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

/// 别名库查找
///
/// 管理动画名称别名到元数据 ID 的映射关系，支持从嵌入的 JSON 文件加载。
///
/// # 示例
///
/// ```no_run
/// use anime_organizer::metadata::AliasLookup;
///
/// let lookup = AliasLookup::load(None).unwrap();
/// if let Some(entry) = lookup.find("孤独摇滚") {
///     println!("Bangumi ID: {}", entry.bangumi_id);
/// }
/// ```
pub struct AliasLookup {
    aliases: HashMap<String, AliasEntry>,
    normalized_keys: HashMap<String, String>,
}

impl AliasLookup {
    /// 从嵌入的 `aliases.json` 加载别名库，可选用户自定义文件覆盖
    ///
    /// # 参数
    ///
    /// - `user_file` - 用户自定义别名文件路径（可选）。
    ///   用户文件中的条目会覆盖内置别名库中相同键的条目。
    pub fn load(user_file: Option<&Path>) -> Result<Self> {
        let data = DataAssets::get("aliases.json")
            .ok_or_else(|| AppError::AliasLoadError("嵌入的 aliases.json 未找到".to_string()))?;

        let json_str = std::str::from_utf8(data.data.as_ref())
            .map_err(|e| AppError::AliasLoadError(format!("UTF-8 解码失败: {e}")))?;

        let mut aliases: HashMap<String, AliasEntry> = serde_json::from_str(json_str)
            .map_err(|e| AppError::AliasLoadError(format!("JSON 解析失败: {e}")))?;

        // 加载用户自定义文件（覆盖内置条目）
        if let Some(path) = user_file {
            let user_data = std::fs::read_to_string(path)
                .map_err(|e| AppError::AliasLoadError(format!("读取用户别名文件失败: {e}")))?;
            let user_aliases: HashMap<String, AliasEntry> = serde_json::from_str(&user_data)
                .map_err(|e| AppError::AliasLoadError(format!("解析用户别名文件失败: {e}")))?;
            aliases.extend(user_aliases);
        }

        expand_generated_aliases(&mut aliases);

        let normalized_keys = build_normalized_index(&aliases);

        Ok(Self {
            aliases,
            normalized_keys,
        })
    }

    /// 根据名称精确查找别名
    pub fn find(&self, name: &str) -> Option<&AliasEntry> {
        self.aliases.get(name)
    }

    /// 根据名称模糊查找（大小写不敏感，去除空格）
    pub fn find_fuzzy(&self, name: &str) -> Option<&AliasEntry> {
        extract_lookup_candidates(name)
            .into_iter()
            .find_map(|candidate| {
                let normalized = normalize_name(&candidate);
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
        self.aliases.len()
    }

    /// 别名库是否为空
    pub fn is_empty(&self) -> bool {
        self.aliases.is_empty()
    }

    /// 返回底层别名表，供匹配/导出流程使用。
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_bundled_aliases() {
        let lookup = AliasLookup::load(None).unwrap();
        assert!(!lookup.is_empty());
        assert!(lookup.len() > 500);
    }

    #[test]
    fn test_find_exact_match() {
        let lookup = AliasLookup::load(None).unwrap();
        let entry = lookup.find("进击的巨人");
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert!(entry.bangumi_id > 0);
    }

    #[test]
    fn test_find_returns_none_for_unknown() {
        let lookup = AliasLookup::load(None).unwrap();
        assert!(lookup.find("这个动画不存在_xyz_123").is_none());
    }

    #[test]
    fn test_find_fuzzy() {
        let lookup = AliasLookup::load(None).unwrap();
        // 测试大小写不敏感
        let entry = lookup.find_fuzzy("CLANNAD");
        // 如果别名库中有 clannad 相关条目则应找到
        if let Some(e) = entry {
            assert!(e.bangumi_id > 0);
        }
    }

    #[test]
    fn test_find_by_bangumi_id() {
        let lookup = AliasLookup::load(None).unwrap();
        // 找到任意一个条目的 bangumi_id 进行反查
        if let Some(entry) = lookup.find("进击的巨人") {
            let result = lookup.find_by_bangumi_id(entry.bangumi_id);
            assert!(result.is_some());
        }
    }

    #[test]
    fn test_user_file_override() {
        let dir = tempfile::tempdir().unwrap();
        let user_file = dir.path().join("user_aliases.json");
        std::fs::write(
            &user_file,
            r#"{"测试动画": {"bangumi_id": 99999, "name": "テスト", "tmdb_id": null, "anidb_id": null}}"#,
        )
        .unwrap();

        let lookup = AliasLookup::load(Some(&user_file)).unwrap();
        let entry = lookup.find("测试动画").unwrap();
        assert_eq!(entry.bangumi_id, 99999);
        assert_eq!(entry.name, "テスト");
    }

    #[test]
    fn test_normalize_name() {
        assert_eq!(normalize_name("Hello World"), "helloworld");
        assert_eq!(normalize_name("ぼっち・ざ・ろっく"), "ぼっちざろっく");
    }

    #[test]
    fn test_find_fuzzy_from_release_filename() {
        let lookup = AliasLookup::load(None).unwrap();
        let entry = lookup.find_fuzzy("[ANi] 进击的巨人 / Attack on Titan - 01 [1080P].mp4");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().name, "進撃の巨人");
    }
}
