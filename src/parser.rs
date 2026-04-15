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

static ANIME_FILE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\[(?P<publisher>[^\]]+)\]").expect("正则表达式编译失败"));

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
        let filename = path.to_str()?;

        let filename = if filename.starts_with('[') {
            filename.to_string()
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            name.to_string()
        } else {
            filename.to_string()
        };

        let caps = ANIME_FILE_REGEX.captures(&filename)?;
        let publisher = caps.name("publisher")?.as_str().trim().to_string();
        let publisher_end = caps.get(0)?.end();

        let after_publisher = &filename[publisher_end..];

        let (anime_name, episode, after_episode) = Self::parse_anime_episode(after_publisher)?;

        let (tags, extension) = Self::parse_tags_and_ext(after_episode)?;

        Some(AnimeFileInfo {
            publisher,
            anime_name,
            episode,
            tags,
            extension,
            original_path: path.to_string_lossy().to_string(),
        })
    }

    fn parse_anime_episode(input: &str) -> Option<(String, String, &str)> {
        let input = input.trim_start();
        let bytes = input.as_bytes();

        let mut episode_info: Option<(usize, usize, usize)> = None;

        for i in 0..bytes.len() {
            if bytes[i] == b'-' && i > 0 {
                let before = bytes[i - 1];
                if before == b' ' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    continue;
                }
                if before == b' ' && i + 2 < bytes.len() && bytes[i + 1] == b' ' {
                    let mut num_start = i + 2;
                    while num_start < bytes.len() && bytes[num_start] == b' ' {
                        num_start += 1;
                    }
                    if num_start < bytes.len()
                        && (bytes[num_start].is_ascii_digit() || bytes[num_start] == b'.')
                    {
                        let mut num_end = num_start;
                        while num_end < bytes.len()
                            && (bytes[num_end].is_ascii_digit() || bytes[num_end] == b'.')
                        {
                            num_end += 1;
                        }
                        let after_digits = if num_end < bytes.len() {
                            bytes[num_end]
                        } else {
                            b' '
                        };

                        if after_digits == b' '
                            || after_digits == b'['
                            || after_digits == b'-'
                            || num_end >= bytes.len()
                        {
                            episode_info = Some((i, num_start, num_end));
                        }
                    }
                }
            }
        }

        if let Some((dash_pos, digit_start, digit_end)) = episode_info {
            let episode_raw = std::str::from_utf8(&bytes[digit_start..digit_end]).ok()?;
            let episode = if episode_raw.contains('.') {
                episode_raw.to_string()
            } else {
                format!("{:0>2}", episode_raw)
            };

            let anime_name = input[..dash_pos].trim().to_string();
            let after_episode = std::str::from_utf8(&bytes[digit_end..]).ok()?.trim_start();

            return Some((anime_name, episode, after_episode));
        }

        // Try to find "[XX]" pattern (dmhy.org format with episode in brackets)
        // Look for last occurrence of "[" followed by digits and "]"
        for i in (0..bytes.len()).rev() {
            if bytes[i] == b']' {
                let mut j = i;
                while j > 0 && bytes[j - 1] != b'[' {
                    j -= 1;
                }
                if j > 0 && bytes[j - 1] == b'[' {
                    let content = &bytes[j..i];
                    if !content.is_empty() && content.iter().all(|&b| b.is_ascii_digit()) {
                        let episode_str = std::str::from_utf8(content).ok()?;
                        if let Ok(ep_num) = episode_str.parse::<u32>() {
                            if (1..=9999).contains(&ep_num) {
                                let episode = format!("{:0>2}", episode_str);
                                let anime_name = input[..j - 1].trim().to_string();
                                let after_episode =
                                    std::str::from_utf8(&bytes[i + 1..]).ok()?.trim_start();
                                return Some((anime_name, episode, after_episode));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn parse_tags_and_ext(input: &str) -> Option<(String, String)> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }

        if input.ends_with(']') {
            let mut open_stack: Vec<usize> = Vec::new();
            for (i, c) in input.char_indices() {
                if c == '[' {
                    open_stack.push(i);
                } else if c == ']' {
                    if let Some(open_pos) = open_stack.pop() {
                        let bracket_content = &input[open_pos + 1..i];
                        if Self::looks_like_extension(bracket_content) {
                            let tags = input[..open_pos].trim().to_string();
                            return Some((tags, format!(".{}", bracket_content.to_lowercase())));
                        }
                    }
                }
            }
        }

        if let Some(dot_pos) = input.rfind('.') {
            let ext = input[dot_pos + 1..].to_lowercase();
            if Self::looks_like_extension(&ext) {
                let tags = input[..dot_pos].trim().to_string();
                return Some((tags, format!(".{}", ext)));
            }
        }

        if input.ends_with(']') && input.contains('.') {
            if let Some(dot_pos) = input.rfind('.') {
                let ext = &input[dot_pos + 1..];
                if Self::looks_like_extension(ext) {
                    let before_dot = &input[..dot_pos];
                    if before_dot.ends_with(']') {
                        let mut bracket_positions: Vec<usize> = Vec::new();
                        for (i, c) in input.char_indices() {
                            if c == '[' {
                                bracket_positions.push(i);
                            }
                        }
                        for &bracket_start in bracket_positions.iter().rev() {
                            let bracket_end = input[bracket_start..]
                                .find(']')
                                .map(|p| bracket_start + p)
                                .unwrap_or(bracket_start);
                            let content = &input[bracket_start + 1..bracket_end];
                            if Self::looks_like_extension(content) {
                                let tags = input[..bracket_start].trim().to_string();
                                return Some((tags, format!(".{}", content.to_lowercase())));
                            }
                        }
                    }
                    let tags = before_dot.trim().to_string();
                    return Some((tags, format!(".{}", ext.to_lowercase())));
                }
            }
        }

        if input.ends_with(']') && !input.contains('.') {
            let mut bracket_positions: Vec<usize> = Vec::new();
            for (i, c) in input.char_indices() {
                if c == '[' {
                    bracket_positions.push(i);
                }
            }
            for &bracket_start in bracket_positions.iter().rev() {
                let bracket_end = input[bracket_start..]
                    .find(']')
                    .map(|p| bracket_start + p)
                    .unwrap_or(bracket_start);
                let content = &input[bracket_start + 1..bracket_end];
                if Self::looks_like_extension(content) {
                    let tags = input[..bracket_start].trim().to_string();
                    return Some((tags, format!(".{}", content.to_lowercase())));
                }
            }
        }

        None
    }

    fn looks_like_extension(s: &str) -> bool {
        let s = s.trim();
        if s.is_empty() {
            return false;
        }
        let upper = s.to_uppercase();
        matches!(
            upper.as_str(),
            "MP4"
                | "MKV"
                | "AVI"
                | "MOV"
                | "WMV"
                | "FLV"
                | "WEBM"
                | "MPG"
                | "MPEG"
                | "3GP"
                | "OGG"
                | "TS"
                | "M2TS"
        )
    }
}
