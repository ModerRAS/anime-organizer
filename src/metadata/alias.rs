//! 别名库查找模块
//!
//! 提供动漫名称别名查找功能，支持内置别名库和用户自定义别名文件。

use crate::error::{AppError, Result};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// 内嵌数据资源
#[derive(Embed)]
#[folder = "data/"]
struct Asset;

/// 别名条目
///
/// 包含动漫在各数据源中的 ID 映射。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasEntry {
    /// Bangumi subject ID
    pub bangumi_id: u32,
    /// 标准日文名称
    pub name: String,
    /// TMDB TV ID（可选）
    pub tmdb_id: Option<u32>,
    /// AniDB anime ID（可选）
    pub anidb_id: Option<u32>,
}

/// 别名库
///
/// 管理内置和用户自定义的动漫名称别名。
pub struct AliasLookup {
    /// 别名映射：fan translation → AliasEntry
    aliases: HashMap<String, AliasEntry>,
}

impl AliasLookup {
    /// 创建新的别名库，加载内置别名
    ///
    /// # 错误
    ///
    /// 如果内置别名 JSON 加载失败，返回 `AliasLoadError`。
    pub fn new() -> Result<Self> {
        let mut aliases = HashMap::new();

        // 加载内置别名库
        if let Some(data) = Asset::get("aliases.json") {
            let json_str = std::str::from_utf8(data.data.as_ref())
                .map_err(|e| AppError::AliasLoadError(format!("UTF-8 解码失败: {e}")))?;
            let loaded: HashMap<String, AliasEntry> = serde_json::from_str(json_str)
                .map_err(|e| AppError::AliasLoadError(format!("JSON 解析失败: {e}")))?;
            aliases.extend(loaded);
        }

        Ok(Self { aliases })
    }

    /// 加载用户自定义别名文件（覆盖内置条目）
    ///
    /// # 参数
    ///
    /// * `path` - 用户别名 JSON 文件路径
    pub fn load_user_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            AppError::AliasLoadError(format!(
                "读取别名文件失败 {}: {e}",
                path.as_ref().display()
            ))
        })?;
        let user_aliases: HashMap<String, AliasEntry> = serde_json::from_str(&content)
            .map_err(|e| AppError::AliasLoadError(format!("JSON 解析失败: {e}")))?;
        self.aliases.extend(user_aliases);
        Ok(())
    }

    /// 根据名称查找别名
    ///
    /// 先进行精确匹配，再进行规范化后匹配。
    pub fn find(&self, name: &str) -> Option<&AliasEntry> {
        // 精确匹配
        if let Some(entry) = self.aliases.get(name) {
            return Some(entry);
        }

        // 规范化后匹配
        let normalized = Self::normalize(name);
        self.aliases
            .iter()
            .find(|(key, _)| Self::normalize(key) == normalized)
            .map(|(_, entry)| entry)
    }

    /// 规范化名称
    ///
    /// 去除方括号标签、集数编号、多余空格等。
    fn normalize(name: &str) -> String {
        let mut result = name.to_string();

        // 去除方括号标签 [ANi], [1080P] 等
        let bracket_re = regex::Regex::new(r"\[[^\]]*\]").unwrap();
        result = bracket_re.replace_all(&result, "").to_string();

        // 去除集数部分 " - 01" 等
        let episode_re = regex::Regex::new(r"\s*-\s*\d+\s*$").unwrap();
        result = episode_re.replace(&result, "").to_string();

        // 统一全角/半角空格，去除前后空白
        result = result.replace('\u{3000}', " ");
        result = result.trim().to_string();

        // 转小写用于比较
        result.to_lowercase()
    }

    /// 获取所有别名条目数
    pub fn len(&self) -> usize {
        self.aliases.len()
    }

    /// 判断别名库是否为空
    pub fn is_empty(&self) -> bool {
        self.aliases.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_lookup_loads_bundled() {
        let lookup = AliasLookup::new().unwrap();
        assert!(!lookup.is_empty(), "内置别名库不应为空");
        assert!(lookup.len() >= 500, "内置别名库应至少有 500 条");
    }

    #[test]
    fn test_alias_exact_match() {
        let lookup = AliasLookup::new().unwrap();
        let entry = lookup.find("进击的巨人").unwrap();
        assert_eq!(entry.name, "進撃の巨人");
    }

    #[test]
    fn test_alias_normalized_match() {
        let lookup = AliasLookup::new().unwrap();
        let entry = lookup.find("[ANi] 进击的巨人 - 01").unwrap();
        assert_eq!(entry.name, "進撃の巨人");
    }

    #[test]
    fn test_alias_not_found() {
        let lookup = AliasLookup::new().unwrap();
        assert!(lookup.find("不存在的动画名称").is_none());
    }

    #[test]
    fn test_normalize() {
        assert_eq!(AliasLookup::normalize("[ANi] 测试 - 01 [1080P]"), "测试");
        assert_eq!(AliasLookup::normalize("  测试  "), "测试");
        assert_eq!(AliasLookup::normalize("[Sub] 测试 - 12"), "测试");
    }

    #[test]
    fn test_user_file_override() {
        let mut lookup = AliasLookup::new().unwrap();
        let original = lookup.find("进击的巨人").map(|e| e.bangumi_id);

        // 创建临时用户文件
        let dir = tempfile::TempDir::new().unwrap();
        let user_file = dir.path().join("user_aliases.json");
        let content = r#"{"进击的巨人": {"bangumi_id": 99999, "name": "テスト", "tmdb_id": null, "anidb_id": null}}"#;
        std::fs::write(&user_file, content).unwrap();

        lookup.load_user_file(&user_file).unwrap();
        let entry = lookup.find("进击的巨人").unwrap();
        assert_eq!(entry.bangumi_id, 99999);
        assert_ne!(Some(entry.bangumi_id), original);
    }
}
