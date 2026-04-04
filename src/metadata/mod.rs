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
    /// 中文标题
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
}

impl AnimeMetadata {
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
        }
    }
}
