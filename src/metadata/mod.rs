//! 元数据模块
//!
//! 提供动漫元数据获取、解析和管理功能。
//!
//! ## 子模块
//!
//! - [`alias`] - 别名库查找
//! - [`bangumi`] - Bangumi dump 客户端
//! - [`wiki`] - Wiki Infobox 解析
//! - [`tmdb`] - TMDB API 客户端

pub mod alias;
pub mod bangumi;
pub mod tmdb;
pub mod wiki;

pub use alias::{AliasEntry, AliasLookup};
pub use bangumi::{BangumiClient, BangumiSubject};
pub use tmdb::TmdbClient;
pub use wiki::WikiParser;

use serde::{Deserialize, Serialize};

/// 动漫元数据
///
/// 包含从各数据源合并后的动漫完整元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeMetadata {
    /// Bangumi subject ID
    pub bangumi_id: u32,
    /// 标准标题（日文）
    pub title: String,
    /// 中文标题
    pub title_cn: Option<String>,
    /// 原始标题
    pub original_title: String,
    /// 简介/剧情
    pub summary: String,
    /// 类型标签
    pub genre: Vec<String>,
    /// 制作公司
    pub studio: Option<String>,
    /// 导演
    pub director: Option<String>,
    /// 集数
    pub episode_count: u32,
    /// 首播日期
    pub air_date: Option<String>,
    /// 评分
    pub rating: f32,
    /// TMDB ID
    pub tmdb_id: Option<u32>,
}

/// 动漫剧集元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeMetadata {
    /// 所属 Bangumi subject ID
    pub subject_id: u32,
    /// 剧集 ID
    pub id: u32,
    /// 集数编号
    pub episode_number: u32,
    /// 剧集标题
    pub title: String,
    /// 中文标题
    pub title_cn: Option<String>,
    /// 播出日期
    pub air_date: Option<String>,
    /// 时长（分钟）
    pub duration: Option<u32>,
}
