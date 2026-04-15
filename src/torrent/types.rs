//! 爬取到的种子标题类型定义

use serde::{Deserialize, Serialize};

/// 种子标题数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedTitle {
    /// 原始发布标题（包含字幕组、分辨率等信息）
    pub title: String,
    /// 数据来源
    pub source: TorrentSource,
    /// 来源页面 URL（如果有）
    pub url: Option<String>,
}

/// 支持的种子来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TorrentSource {
    /// 动漫花园
    Dmhy,
    /// Nyaa.si
    Nyaa,
}

impl std::fmt::Display for TorrentSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dmhy => write!(f, "DMHY"),
            Self::Nyaa => write!(f, "Nyaa"),
        }
    }
}
