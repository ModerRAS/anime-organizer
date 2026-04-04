//! 别名库查找
//!
//! 提供动画名称别名到 Bangumi/TMDB/AniDB ID 的映射查找功能。
//! 别名数据通过 `rust-embed` 嵌入到二进制文件中，也支持用户自定义别名文件覆盖。

use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::{AppError, Result};

/// 嵌入的数据资源
#[derive(Embed)]
#[folder = "data/"]
struct DataAssets;

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

        Ok(Self { aliases })
    }

    /// 根据名称精确查找别名
    pub fn find(&self, name: &str) -> Option<&AliasEntry> {
        self.aliases.get(name)
    }

    /// 根据名称模糊查找（大小写不敏感，去除空格）
    pub fn find_fuzzy(&self, name: &str) -> Option<&AliasEntry> {
        let normalized = normalize_name(name);
        self.aliases
            .iter()
            .find(|(k, _)| normalize_name(k) == normalized)
            .map(|(_, v)| v)
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
}

/// 规范化名称：转小写，去除空格和常见分隔符
fn normalize_name(name: &str) -> String {
    name.to_lowercase().replace([' ', '　', '・', '·'], "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_bundled_aliases() {
        let lookup = AliasLookup::load(None).unwrap();
        assert!(!lookup.is_empty());
        assert!(lookup.len() > 50);
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
}
