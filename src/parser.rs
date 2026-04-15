//! 文件名解析模块
//!
//! 该模块负责解析符合特定格式的动漫文件名，并提取关键信息。
#![allow(non_snake_case)]
//!
//! # 支持的文件名格式
//!
//! ```text
//! [发布组] 动漫名称（可含季度） - 集数 [标签信息].扩展名
//! ```
//!
//! # 示例
//!
//! ```
//! use anime_organizer::parser::FilenameParser;
//! use std::path::Path;
//!
//! let path = Path::new("[ANi] 妖怪旅館營業中 貳 - 07 [1080P].mp4");
//! if let Some(info) = FilenameParser::parse(path) {
//!     assert_eq!(info.publisher, "ANi");
//!     assert_eq!(info.anime_name, "妖怪旅館營業中 貳");
//!     assert_eq!(info.episode, "07");
//! }
//! ```
//!
//! # 测试模块
//!
//! 测试位于 `tests/parser/` 目录，按发布组分类组织。

use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

/// 预编译的正则表达式
static ANIME_FILE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^\[(?P<publisher>[^\]]+)\]\s+(?P<anime>.+?)\s+-\s+(?P<episode>\d+)\s+(?P<tags>\[.+\])(?P<ext>\.\w+)$",
    )
    .expect("正则表达式编译失败")
});

static SEASON_SUFFIX_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)^(?P<title>.+?)\s+season\s*(?P<num>\d{1,2})$")
            .expect("季信息正则表达式编译失败"),
        Regex::new(r"(?i)^(?P<title>.+?)\s+s(?P<num>\d{1,2})$").expect("季信息正则表达式编译失败"),
        Regex::new(r"(?i)^(?P<title>.+?)\s+(?P<num>\d{1,2})(?:st|nd|rd|th)\s+season$")
            .expect("季信息正则表达式编译失败"),
        Regex::new(r"^(?P<title>.+?)\s*第(?P<num>\d{1,2}|[一二三四五六七八九十]+)季$")
            .expect("季信息正则表达式编译失败"),
        Regex::new(r"^(?P<title>.+?)\s*(?P<num>\d{1,2}|[一二三四五六七八九十]+)期$")
            .expect("季信息正则表达式编译失败"),
        Regex::new(
            r"^(?P<title>.+?)\s+(?P<num>II|III|IV|V|VI|VII|VIII|IX|X|貳|贰|弐|二期|三期|四期)$",
        )
        .expect("季信息正则表达式编译失败"),
    ]
});

/// 动漫文件信息结构体
///
/// 包含从文件名中解析出的所有关键信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimeFileInfo {
    /// 发布组名称
    pub publisher: String,
    /// 动漫名称
    pub anime_name: String,
    /// 集数（补齐为两位数）
    pub episode: String,
    /// 标签信息（如分辨率、编码格式等）
    pub tags: String,
    /// 文件扩展名（小写）
    pub extension: String,
    /// 原始文件路径
    pub original_path: String,
}

impl AnimeFileInfo {
    /// 生成目标文件名
    ///
    /// 返回格式为 `{episode} {tags}{extension}` 的文件名。
    #[must_use]
    pub fn target_filename(&self) -> String {
        format!("{} {}{}", self.episode, self.tags, self.extension)
    }

    /// 返回系列主标题，不包含可识别的季后缀。
    #[must_use]
    pub fn series_name(&self) -> String {
        split_series_and_season(&self.anime_name).0
    }

    /// 返回从文件名中识别出的季号。
    #[must_use]
    pub fn season_number(&self) -> Option<u32> {
        split_series_and_season(&self.anime_name).1
    }

    /// 返回用于季目录的名称，无法识别时回落到 `Season 1`。
    #[must_use]
    pub fn season_dir_name(&self) -> String {
        format!("Season {}", self.season_number().unwrap_or(1))
    }
}

/// 将带季信息的动画标题拆分为系列名与季号。
#[must_use]
pub fn split_series_and_season(name: &str) -> (String, Option<u32>) {
    let trimmed = name.trim();

    for pattern in SEASON_SUFFIX_PATTERNS.iter() {
        if let Some(caps) = pattern.captures(trimmed) {
            let Some(title) = caps.name("title") else {
                continue;
            };
            let Some(num_match) = caps.name("num") else {
                continue;
            };

            let season = parse_season_number(num_match.as_str());
            if season.is_some() {
                return (title.as_str().trim().to_string(), season);
            }
        }
    }

    (trimmed.to_string(), None)
}

fn parse_season_number(raw: &str) -> Option<u32> {
    let normalized = raw.trim();

    if let Ok(num) = normalized.parse::<u32>() {
        return Some(num);
    }

    match normalized.to_ascii_uppercase().as_str() {
        "II" => return Some(2),
        "III" => return Some(3),
        "IV" => return Some(4),
        "V" => return Some(5),
        "VI" => return Some(6),
        "VII" => return Some(7),
        "VIII" => return Some(8),
        "IX" => return Some(9),
        "X" => return Some(10),
        _ => {}
    }

    match normalized {
        "貳" | "贰" | "弐" | "二期" => Some(2),
        "三期" => Some(3),
        "四期" => Some(4),
        _ => parse_cjk_number(normalized),
    }
}

fn parse_cjk_number(raw: &str) -> Option<u32> {
    let mut total = 0;
    let mut current = 0;

    for ch in raw.chars() {
        match ch {
            '一' => current += 1,
            '二' => current += 2,
            '三' => current += 3,
            '四' => current += 4,
            '五' => current += 5,
            '六' => current += 6,
            '七' => current += 7,
            '八' => current += 8,
            '九' => current += 9,
            '十' => {
                total += if current == 0 { 10 } else { current * 10 };
                current = 0;
            }
            _ => return None,
        }
    }

    Some(total + current)
}

/// 文件名解析器
///
/// 使用正则表达式解析符合特定格式的动漫文件名。
pub struct FilenameParser;

impl FilenameParser {
    /// 解析文件路径，提取动漫文件信息
    #[must_use]
    pub fn parse<P: AsRef<Path>>(file_path: P) -> Option<AnimeFileInfo> {
        let path = file_path.as_ref();
        let filename = path.file_name()?.to_str()?;

        let caps = ANIME_FILE_REGEX.captures(filename)?;
        let publisher = caps.name("publisher")?.as_str().trim().to_string();
        let anime_name = caps.name("anime")?.as_str().trim().to_string();
        let episode_raw = caps.name("episode")?.as_str();
        let episode = format!("{:0>2}", episode_raw);
        let tags = caps.name("tags")?.as_str().trim().to_string();
        let extension = caps.name("ext")?.as_str().to_lowercase();

        Some(AnimeFileInfo {
            publisher,
            anime_name,
            episode,
            tags,
            extension,
            original_path: path.to_string_lossy().to_string(),
        })
    }
}
