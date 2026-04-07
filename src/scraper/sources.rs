//! 数据源刮削
//!
//! 从 Bangumi Archive、TMDB、DMHY 等来源采集最新动画信息。

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// 刮削到的动画条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedAnime {
    /// 标题（日文/原文）
    pub title: String,
    /// 中文标题
    pub title_cn: Option<String>,
    /// 播出日期（YYYY-MM-DD）
    pub date: Option<String>,
    /// 数据来源
    pub source: ScrapedSource,
    /// 来源 URL
    pub source_url: Option<String>,
    /// Bangumi ID（如果来自 Bangumi）
    pub bangumi_id: Option<u32>,
    /// TMDB ID（如果来自 TMDB）
    pub tmdb_id: Option<u32>,
    /// 别名（来自 infobox）
    pub aliases: Vec<String>,
}

/// 数据来源类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScrapedSource {
    /// Bangumi Archive
    Bangumi,
    /// TMDB Discover
    Tmdb,
    /// 动漫花园 RSS
    Dmhy,
}

impl std::fmt::Display for ScrapedSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bangumi => write!(f, "Bangumi"),
            Self::Tmdb => write!(f, "TMDB"),
            Self::Dmhy => write!(f, "DMHY"),
        }
    }
}

/// 刮削器
///
/// 管理多数据源的动画信息采集。
pub struct Scraper {
    http: reqwest::Client,
}

impl Scraper {
    /// 创建刮削器实例
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    /// 从 Bangumi Archive 刮削最近的动画
    ///
    /// 下载 subject dump，筛选指定天数内的 type=2（动画）条目。
    ///
    /// # 参数
    ///
    /// - `days` - 筛选最近多少天内的条目
    pub async fn scrape_bangumi(&self, days: u32) -> Result<Vec<ScrapedAnime>> {
        let url = "https://raw.githubusercontent.com/bangumi/archive/master/data/subject.jsonlines";

        let resp =
            self.http.get(url).send().await.map_err(|e| {
                AppError::MetadataFetchError(format!("下载 Bangumi dump 失败: {e}"))
            })?;

        let text = resp
            .text()
            .await
            .map_err(|e| AppError::MetadataFetchError(format!("读取 Bangumi dump 失败: {e}")))?;

        let cutoff = chrono_cutoff_date(days);
        let mut results = Vec::new();

        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let subject: serde_json::Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            // 只保留 type=2（动画）
            if subject.get("type").and_then(|v| v.as_u64()) != Some(2) {
                continue;
            }

            // 筛选日期
            if let Some(date) = subject.get("date").and_then(|v| v.as_str()) {
                if date < cutoff.as_str() {
                    continue;
                }

                let id = subject.get("id").and_then(|v| v.as_u64()).map(|v| v as u32);

                results.push(ScrapedAnime {
                    title: subject
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    title_cn: subject
                        .get("name_cn")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string()),
                    date: Some(date.to_string()),
                    source: ScrapedSource::Bangumi,
                    source_url: id.map(|id| format!("https://bgm.tv/subject/{id}")),
                    bangumi_id: id,
                    tmdb_id: None,
                    aliases: Vec::new(),
                });
            }
        }

        Ok(results)
    }

    /// 从 TMDB Discover 刮削最近的动画
    ///
    /// 使用 TMDB API v3 的 discover/tv 端点，筛选动画类型。
    ///
    /// # 参数
    ///
    /// - `days` - 筛选最近多少天内的条目
    /// - `api_key` - TMDB API Key
    pub async fn scrape_tmdb(&self, days: u32, api_key: &str) -> Result<Vec<ScrapedAnime>> {
        let cutoff = chrono_cutoff_date(days);

        let url = format!(
            "https://api.themoviedb.org/3/discover/tv?api_key={}&with_genres=16&first_air_date.gte={}&sort_by=first_air_date.desc&page=1",
            api_key, cutoff
        );

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::TmdbApiError(format!("TMDB discover 请求失败: {e}")))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::TmdbApiError(format!("TMDB 响应解析失败: {e}")))?;

        let mut results = Vec::new();

        if let Some(items) = data.get("results").and_then(|v| v.as_array()) {
            for item in items {
                let id = item.get("id").and_then(|v| v.as_u64()).map(|v| v as u32);

                results.push(ScrapedAnime {
                    title: item
                        .get("original_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    title_cn: item
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    date: item
                        .get("first_air_date")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    source: ScrapedSource::Tmdb,
                    source_url: id.map(|id| format!("https://www.themoviedb.org/tv/{id}")),
                    bangumi_id: None,
                    tmdb_id: id,
                    aliases: Vec::new(),
                });
            }
        }

        Ok(results)
    }

    /// 从动漫花园 RSS 刮削最新的字幕组发布
    ///
    /// 解析 DMHY 的 RSS XML，提取番剧标题。
    ///
    /// # 参数
    ///
    /// - `days` - 筛选最近多少天内的条目（通过 RSS 自然截止）
    pub async fn scrape_dmhy(&self, _days: u32) -> Result<Vec<ScrapedAnime>> {
        let url = "https://share.dmhy.org/topics/rss/rss.xml";

        let resp = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::MetadataFetchError(format!("DMHY RSS 请求失败: {e}")))?;

        let text = resp
            .text()
            .await
            .map_err(|e| AppError::MetadataFetchError(format!("DMHY RSS 读取失败: {e}")))?;

        let results = parse_dmhy_rss(&text);
        Ok(results)
    }

    /// 从所有来源刮削动画信息
    ///
    /// 合并所有数据源的结果。
    ///
    /// # 参数
    ///
    /// - `days` - 筛选最近多少天
    /// - `tmdb_api_key` - TMDB API Key（可选，为 None 则跳过 TMDB）
    pub async fn scrape_all(
        &self,
        days: u32,
        tmdb_api_key: Option<&str>,
    ) -> Result<Vec<ScrapedAnime>> {
        let mut all = Vec::new();

        // Bangumi
        match self.scrape_bangumi(days).await {
            Ok(items) => all.extend(items),
            Err(e) => eprintln!("Bangumi 刮削失败: {e}"),
        }

        // TMDB
        if let Some(key) = tmdb_api_key {
            match self.scrape_tmdb(days, key).await {
                Ok(items) => all.extend(items),
                Err(e) => eprintln!("TMDB 刮削失败: {e}"),
            }
        }

        // DMHY
        match self.scrape_dmhy(days).await {
            Ok(items) => all.extend(items),
            Err(e) => eprintln!("DMHY 刮削失败: {e}"),
        }

        Ok(all)
    }
}

impl Default for Scraper {
    fn default() -> Self {
        Self::new()
    }
}

/// 解析 DMHY RSS XML，提取番剧标题
fn parse_dmhy_rss(xml: &str) -> Vec<ScrapedAnime> {
    let mut results = Vec::new();
    let mut seen_titles: std::collections::HashSet<String> = std::collections::HashSet::new();

    // 简单 XML 解析（提取 <title> 标签内容）
    for segment in xml.split("<item>").skip(1) {
        let title = extract_xml_value(segment, "title").unwrap_or_default();
        if title.is_empty() {
            continue;
        }

        // 从 DMHY 标题中提取动画名
        // 典型格式：[字幕组] 动画名 - 01 [1080P][...].mkv
        let anime_name = extract_anime_name_from_dmhy_title(&title);
        if anime_name.is_empty() {
            continue;
        }

        // 去重
        if !seen_titles.insert(anime_name.clone()) {
            continue;
        }

        let link = extract_xml_value(segment, "link");

        results.push(ScrapedAnime {
            title: anime_name,
            title_cn: None,
            date: None,
            source: ScrapedSource::Dmhy,
            source_url: link,
            bangumi_id: None,
            tmdb_id: None,
            aliases: Vec::new(),
        });
    }

    results
}

/// 从 XML 片段中提取标签值
fn extract_xml_value(segment: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");

    let start = segment.find(&start_tag)?;
    let after_start = start + start_tag.len();
    let end = segment[after_start..].find(&end_tag)?;

    Some(segment[after_start..after_start + end].trim().to_string())
}

/// 从 DMHY 标题中提取动画名
///
/// 典型 DMHY 标题格式：
/// - `[SubsPlease] Anime Name - 01 (1080p) [HASH].mkv`
/// - `[字幕组] 动画名 / Anime Name - 01 [1080P].mp4`
fn extract_anime_name_from_dmhy_title(title: &str) -> String {
    let title = title.trim();

    // 去掉开头的 [字幕组]
    let rest = if title.starts_with('[') {
        if let Some(end) = title.find(']') {
            title[end + 1..].trim()
        } else {
            title
        }
    } else {
        title
    };

    // 去掉 " - 01" 及之后的内容
    let name = if let Some(idx) = rest.find(" - ") {
        &rest[..idx]
    } else {
        rest
    };

    // 去掉末尾的 [tag]
    let name = name.trim();
    let name = if let Some(idx) = name.rfind('[') {
        name[..idx].trim()
    } else {
        name
    };

    // 如果有 "/"，取后半部分（通常是日文/英文名）
    if let Some(idx) = name.find('/') {
        let after = name[idx + 1..].trim();
        if !after.is_empty() {
            return after.to_string();
        }
    }

    name.to_string()
}

/// 计算截止日期字符串（YYYY-MM-DD）
fn chrono_cutoff_date(days: u32) -> String {
    // 不引入 chrono 依赖，手动计算
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let cutoff = now - (days as u64) * 86400;

    // 简单日期计算
    let days_since_epoch = (cutoff / 86400) as i64;
    let (year, month, day) = days_to_ymd(days_since_epoch);

    format!("{year:04}-{month:02}-{day:02}")
}

/// 将从 Unix 纪元开始的天数转换为 (年, 月, 日)
fn days_to_ymd(mut days: i64) -> (i32, u32, u32) {
    // 基于 1970-01-01
    days += 719_468; // 转换到从 0000-03-01 开始计数
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = (days - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    (year as i32, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_anime_name_basic() {
        let title = "[SubsPlease] Bocchi the Rock! - 01 (1080p) [ABC].mkv";
        assert_eq!(
            extract_anime_name_from_dmhy_title(title),
            "Bocchi the Rock!"
        );
    }

    #[test]
    fn test_extract_anime_name_with_chinese() {
        let title = "[字幕组] 孤独摇滚 / Bocchi the Rock! - 01 [1080P].mp4";
        assert_eq!(
            extract_anime_name_from_dmhy_title(title),
            "Bocchi the Rock!"
        );
    }

    #[test]
    fn test_extract_anime_name_no_bracket() {
        let title = "Some Anime - 05 [720P].mkv";
        assert_eq!(extract_anime_name_from_dmhy_title(title), "Some Anime");
    }

    #[test]
    fn test_chrono_cutoff_date_format() {
        let date = chrono_cutoff_date(7);
        // 应为 YYYY-MM-DD 格式
        assert_eq!(date.len(), 10);
        assert_eq!(&date[4..5], "-");
        assert_eq!(&date[7..8], "-");
    }

    #[test]
    fn test_parse_dmhy_rss_xml() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0">
<channel>
<title>DMHY</title>
<item>
<title>[SubsPlease] Test Anime - 01 (1080p) [ABC].mkv</title>
<link>https://share.dmhy.org/topics/view/123</link>
</item>
<item>
<title>[ANi] Another Anime - 02 [1080P].mp4</title>
<link>https://share.dmhy.org/topics/view/456</link>
</item>
</channel>
</rss>"#;

        let results = parse_dmhy_rss(xml);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Test Anime");
        assert_eq!(results[0].source, ScrapedSource::Dmhy);
        assert_eq!(results[1].title, "Another Anime");
    }

    #[test]
    fn test_scraped_source_display() {
        assert_eq!(ScrapedSource::Bangumi.to_string(), "Bangumi");
        assert_eq!(ScrapedSource::Tmdb.to_string(), "TMDB");
        assert_eq!(ScrapedSource::Dmhy.to_string(), "DMHY");
    }

    #[test]
    fn test_days_to_ymd_epoch() {
        let (y, m, d) = days_to_ymd(0);
        assert_eq!((y, m, d), (1970, 1, 1));
    }

    #[test]
    fn test_days_to_ymd_known_date() {
        // 2024-01-01 = 19723 days since epoch
        let (y, m, d) = days_to_ymd(19723);
        assert_eq!((y, m, d), (2024, 1, 1));
    }
}
