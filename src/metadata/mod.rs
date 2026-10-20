//! 元数据模块
//!
//! 提供动画元数据的获取、解析和管理功能。
//!
//! ## 子模块
//!
//! - [`alias`] - 别名库查找
//! - [`bangumi`] - Bangumi Archive 数据加载与查询
//! - [`wiki`] - Wiki Infobox 解析
//! - [`tmdb`] - TMDB API 客户端

pub mod alias;
pub mod bangumi;
pub mod tmdb;
pub mod wiki;

pub use alias::AliasLookup;
pub use bangumi::BangumiClient;
pub use tmdb::TmdbClient;
pub use wiki::WikiParser;

use serde::{Deserialize, Serialize};

/// 动画元数据
///
/// 包含从 Bangumi、Wiki Infobox 等来源获取的元数据信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeMetadata {
    /// Bangumi Subject ID
    pub bangumi_id: u32,
    /// 标准日文标题
    pub title: String,
    /// 中文标题（保证含汉字且不含假名；无中文时为 `None`，英文 `name_cn` 不会存入此处）
    pub title_cn: Option<String>,
    /// 原始标题
    pub original_title: String,
    /// 简介/概要
    pub summary: String,
    /// 类型标签
    pub genre: Vec<String>,
    /// 制作公司
    pub studio: Option<String>,
    /// 导演
    pub director: Option<String>,
    /// 集数
    pub episode_count: u32,
    /// 放送日期（格式：YYYY-MM-DD）
    pub air_date: Option<String>,
    /// 评分（0.0 - 10.0）
    pub rating: f32,
    /// TMDB ID（用于图片下载）
    pub tmdb_id: Option<u32>,
    /// AniDB ID（用于图片回退）
    pub anidb_id: Option<u32>,
    /// Bangumi 封面图 URL
    pub poster_url: Option<String>,
}

/// 是否包含汉字（CJK 统一表意文字）。
#[must_use]
pub fn contains_han(value: &str) -> bool {
    value
        .chars()
        .any(|ch| matches!(ch, '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}'))
}

/// 是否包含日文假名（平假名/片假名）。
#[must_use]
pub fn contains_kana(value: &str) -> bool {
    value.chars().any(|ch| {
        matches!(
            ch,
            '\u{3040}'..='\u{309f}' | '\u{30a0}'..='\u{30ff}' | '\u{ff61}'..='\u{ff9f}'
        )
    })
}

/// 是否可视为中文标题：含汉字且不含假名。
///
/// 日文标题常混有假名会被排除；全汉字日文（如 `幻想水滸伝`）无法与中文区分，
/// 会视为中文——可读性优先于语种纯粹性。
#[must_use]
pub fn is_chinese_title(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty() && contains_han(value) && !contains_kana(value)
}

impl AnimeMetadata {
    /// 中文标题：按 `title_cn` → `title` → `original_title` 顺序取第一个中文。
    ///
    /// Bangumi 的 `name_cn` 有时是英文（如 `Girls Band Cry`），调用方用此方法
    /// 可保证“要中文时一定拿到中文”，而不是拿到英文 `name_cn` 或罗马音。
    #[must_use]
    pub fn chinese_title(&self) -> Option<String> {
        [
            self.title_cn.as_deref(),
            Some(self.title.as_str()),
            Some(self.original_title.as_str()),
        ]
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .find(|title| is_chinese_title(title))
        .map(str::to_string)
    }

    /// 展示用标题：有中文必用中文，否则回退日文原名。
    #[must_use]
    pub fn preferred_title(&self) -> String {
        self.chinese_title().unwrap_or_else(|| self.title.clone())
    }

    /// 创建一个空的元数据实例
    pub fn new(bangumi_id: u32, title: String) -> Self {
        Self {
            bangumi_id,
            original_title: title.clone(),
            title,
            title_cn: None,
            summary: String::new(),
            genre: Vec::new(),
            studio: None,
            director: None,
            episode_count: 0,
            air_date: None,
            rating: 0.0,
            tmdb_id: None,
            anidb_id: None,
            poster_url: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinguishes_chinese_from_japanese_and_romaji() {
        assert!(is_chinese_title("进击的巨人"));
        assert!(is_chinese_title("钻石王牌 act2 第二季"));
        // 日文假名混排不是中文。
        assert!(!is_chinese_title("ガールズバンドクライ"));
        assert!(!is_chinese_title("幻想水滸伝のアニメ"));
        assert!(!is_chinese_title("Girls Band Cry"));
        assert!(!is_chinese_title("Gensou Suikoden"));
        assert!(!is_chinese_title("  "));
    }

    #[test]
    fn preferred_title_demands_chinese() {
        let mut meta = AnimeMetadata::new(431767, "ガールズバンドクライ".to_string());
        meta.title_cn = Some("Girls Band Cry".to_string());
        // 英文 name_cn 不算中文，回退日文原名也不算中文时用 title。
        assert_eq!(meta.chinese_title(), None);
        assert_eq!(meta.preferred_title(), "ガールズバンドクライ");

        meta.title_cn = Some("少女乐队的呐喊".to_string());
        assert_eq!(meta.chinese_title().as_deref(), Some("少女乐队的呐喊"));
        assert_eq!(meta.preferred_title(), "少女乐队的呐喊");
    }

    #[test]
    fn preferred_title_falls_back_to_chinese_name() {
        // 国产动画：name 本身就是中文，name_cn 为空。
        let meta = AnimeMetadata::new(1, "请吃红小豆吧！第五季".to_string());
        assert_eq!(
            meta.chinese_title().as_deref(),
            Some("请吃红小豆吧！第五季")
        );
    }
}
