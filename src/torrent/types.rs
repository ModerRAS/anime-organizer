//! 爬取到的种子标题类型定义

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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

pub fn sorted_unique_title_lines(titles: &[ScrapedTitle]) -> Vec<&str> {
    titles
        .iter()
        .map(|title| title.title.as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn sorted_unique_title_text(titles: &[ScrapedTitle]) -> String {
    sorted_unique_title_lines(titles).join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_unique_helpers_sort_and_dedup_titles() {
        let titles = vec![
            ScrapedTitle {
                title: "b.mkv".to_string(),
                source: TorrentSource::Dmhy,
                url: None,
            },
            ScrapedTitle {
                title: "a.mkv".to_string(),
                source: TorrentSource::Nyaa,
                url: Some("https://nyaa.si/view/1".to_string()),
            },
            ScrapedTitle {
                title: "b.mkv".to_string(),
                source: TorrentSource::Nyaa,
                url: Some("https://nyaa.si/view/2".to_string()),
            },
        ];

        assert_eq!(sorted_unique_title_lines(&titles), vec!["a.mkv", "b.mkv"]);
        assert_eq!(sorted_unique_title_text(&titles), "a.mkv\nb.mkv");
    }
}
