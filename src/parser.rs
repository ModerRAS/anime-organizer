//! 文件名解析模块
//!
//! 该模块负责解析符合特定格式的动漫文件名，并提取关键信息。
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_valid_filename_ani() {
        let path = PathBuf::from(
            "test/[ANi] 妖怪旅館營業中 貳 - 07 [1080P][Baha][WEB-DL][AAC AVC][CHT].mp4",
        );
        let result = FilenameParser::parse(&path).unwrap();

        assert_eq!(result.publisher, "ANi");
        assert_eq!(result.anime_name, "妖怪旅館營業中 貳");
        assert_eq!(result.episode, "07");
        assert_eq!(result.tags, "[1080P][Baha][WEB-DL][AAC AVC][CHT]");
        assert_eq!(result.extension, ".mp4");
    }

    #[test]
    fn test_parse_valid_filename_subsplease() {
        let path = PathBuf::from("test/[SubsPlease] 间谍过家家 - 12 [1080p].mkv");
        let result = FilenameParser::parse(&path).unwrap();

        assert_eq!(result.publisher, "SubsPlease");
        assert_eq!(result.anime_name, "间谍过家家");
        assert_eq!(result.episode, "12");
        assert_eq!(result.tags, "[1080p]");
        assert_eq!(result.extension, ".mkv");
    }

    #[test]
    fn test_parse_valid_filename_ember() {
        let path = PathBuf::from(
            "test/[EMBER] 进击的巨人 The Final Season - 01 [1080p][Multiple Subtitle].avi",
        );
        let result = FilenameParser::parse(&path).unwrap();

        assert_eq!(result.publisher, "EMBER");
        assert_eq!(result.anime_name, "进击的巨人 The Final Season");
        assert_eq!(result.episode, "01");
        assert_eq!(result.tags, "[1080p][Multiple Subtitle]");
        assert_eq!(result.extension, ".avi");
    }

    #[test]
    fn test_parse_single_digit_episode_pads_with_zero() {
        let test_cases = [
            ("[ANi] 测试 - 1 [Tag].mp4", "01"),
            ("[ANi] 测试 - 5 [Tag].mp4", "05"),
            ("[ANi] 测试 - 9 [Tag].mp4", "09"),
            ("[ANi] 测试 - 10 [Tag].mp4", "10"),
        ];

        for (filename, expected_episode) in test_cases {
            let path = PathBuf::from(format!("test/{filename}"));
            let result = FilenameParser::parse(&path).unwrap();
            assert_eq!(result.episode, expected_episode, "文件名: {filename}");
        }
    }

    #[test]
    fn test_parse_invalid_filename_returns_none() {
        let invalid_filenames = [
            "测试 - 01.mp4",
            "[ANi] 测试.mp4",
            "测试 - 01 [Tag].mp4",
            "[ANi] 测试 - 01 Tag.mp4",
            "",
            "random_file.txt",
        ];

        for filename in invalid_filenames {
            let path = PathBuf::from(format!("test/{filename}"));
            let result = FilenameParser::parse(&path);
            assert!(result.is_none(), "应返回 None: {filename}");
        }
    }

    #[test]
    fn test_parse_extension_normalized_to_lowercase() {
        let test_cases = [
            ("[ANi] 测试 - 01 [Tag].MP4", ".mp4"),
            ("[ANi] 测试 - 01 [Tag].Mp4", ".mp4"),
            ("[ANi] 测试 - 01 [Tag].MKV", ".mkv"),
        ];

        for (filename, expected_ext) in test_cases {
            let path = PathBuf::from(format!("test/{filename}"));
            let result = FilenameParser::parse(&path).unwrap();
            assert_eq!(result.extension, expected_ext, "文件名: {filename}");
        }
    }

    #[test]
    fn test_target_filename() {
        let info = AnimeFileInfo {
            publisher: "ANi".to_string(),
            anime_name: "测试".to_string(),
            episode: "01".to_string(),
            tags: "[1080P]".to_string(),
            extension: ".mp4".to_string(),
            original_path: "/path/to/file".to_string(),
        };

        assert_eq!(info.target_filename(), "01 [1080P].mp4");
    }

    #[test]
    fn test_split_series_and_season() {
        assert_eq!(
            split_series_and_season("Test Anime Season 2"),
            ("Test Anime".to_string(), Some(2))
        );
        assert_eq!(
            split_series_and_season("测试动画 第3季"),
            ("测试动画".to_string(), Some(3))
        );
        assert_eq!(
            split_series_and_season("妖怪旅館營業中 貳"),
            ("妖怪旅館營業中".to_string(), Some(2))
        );
        assert_eq!(
            split_series_and_season("進撃の巨人 The Final Season"),
            ("進撃の巨人 The Final Season".to_string(), None)
        );
    }

    #[test]
    fn test_series_helpers() {
        let info = AnimeFileInfo {
            publisher: "ANi".to_string(),
            anime_name: "测试动画 Season 2".to_string(),
            episode: "01".to_string(),
            tags: "[1080P]".to_string(),
            extension: ".mp4".to_string(),
            original_path: "test.mp4".to_string(),
        };

        assert_eq!(info.series_name(), "测试动画");
        assert_eq!(info.season_number(), Some(2));
        assert_eq!(info.season_dir_name(), "Season 2");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_01() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 01 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_02() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 02 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_03() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 03 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_04() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 04 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_05() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 05 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_06() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 06 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_07() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 07 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_08() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 08 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_09() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 09 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_10() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 10 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_11() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 11 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_12() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 12 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_13() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 13 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "13");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_14() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 14 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "14");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_15() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 15 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "15");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_16() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 16 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "16");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_17() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 17 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "17");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_18() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 18 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "18");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_19() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 19 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "19");
    }

    #[test]
    fn test_parse_ani_葬送的芙莉莲_20() {
        let path = PathBuf::from("[ANi] 葬送的芙莉莲 - 20 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "20");
    }

    #[test]
    fn test_parse_ani_间谍过家家_01() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 01 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_ani_间谍过家家_02() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 02 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_ani_间谍过家家_03() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 03 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_ani_间谍过家家_04() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 04 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_ani_间谍过家家_05() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 05 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_ani_间谍过家家_06() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 06 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_ani_间谍过家家_07() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 07 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_ani_间谍过家家_08() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 08 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_ani_间谍过家家_09() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 09 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_ani_间谍过家家_10() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 10 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_ani_间谍过家家_11() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 11 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_ani_间谍过家家_12() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 12 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_ani_间谍过家家_13() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 13 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "13");
    }

    #[test]
    fn test_parse_ani_间谍过家家_14() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 14 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "14");
    }

    #[test]
    fn test_parse_ani_间谍过家家_15() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 15 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "15");
    }

    #[test]
    fn test_parse_ani_间谍过家家_16() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 16 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "16");
    }

    #[test]
    fn test_parse_ani_间谍过家家_17() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 17 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "17");
    }

    #[test]
    fn test_parse_ani_间谍过家家_18() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 18 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "18");
    }

    #[test]
    fn test_parse_ani_间谍过家家_19() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 19 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "19");
    }

    #[test]
    fn test_parse_ani_间谍过家家_20() {
        let path = PathBuf::from("[ANi] 间谍过家家 - 20 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 间谍过家家 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "20");
    }

    #[test]
    fn test_parse_ani_怪物猎人_01() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 01 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_ani_怪物猎人_02() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 02 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_ani_怪物猎人_03() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 03 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_ani_怪物猎人_04() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 04 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_ani_怪物猎人_05() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 05 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_ani_怪物猎人_06() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 06 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_ani_怪物猎人_07() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 07 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_ani_怪物猎人_08() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 08 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_ani_怪物猎人_09() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 09 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_ani_怪物猎人_10() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 10 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_ani_怪物猎人_11() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 11 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_ani_怪物猎人_12() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 12 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_ani_怪物猎人_13() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 13 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "13");
    }

    #[test]
    fn test_parse_ani_怪物猎人_14() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 14 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "14");
    }

    #[test]
    fn test_parse_ani_怪物猎人_15() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 15 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "15");
    }

    #[test]
    fn test_parse_ani_怪物猎人_16() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 16 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "16");
    }

    #[test]
    fn test_parse_ani_怪物猎人_17() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 17 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "17");
    }

    #[test]
    fn test_parse_ani_怪物猎人_18() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 18 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "18");
    }

    #[test]
    fn test_parse_ani_怪物猎人_19() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 19 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "19");
    }

    #[test]
    fn test_parse_ani_怪物猎人_20() {
        let path = PathBuf::from("[ANi] 怪物猎人 - 20 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [ANi] 怪物猎人 format");
        assert_eq!(info.publisher, "ANi");
        assert_eq!(info.anime_name, "怪物猎人");
        assert_eq!(info.episode, "20");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_01() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 01 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_02() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 02 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_03() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 03 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_04() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 04 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_05() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 05 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_06() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 06 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_07() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 07 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_08() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 08 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_09() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 09 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_10() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 10 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_11() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 11 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_12() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 12 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_13() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 13 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "13");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_14() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 14 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "14");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_15() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 15 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "15");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_16() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 16 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "16");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_17() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 17 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "17");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_18() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 18 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "18");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_19() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 19 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "19");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_20() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 20 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "20");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_21() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 21 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "21");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_22() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 22 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "22");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_23() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 23 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "23");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_24() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 24 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "24");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_25() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 25 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "25");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_26() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 26 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "26");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_27() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 27 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "27");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_28() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 28 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "28");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_29() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 29 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "29");
    }

    #[test]
    fn test_parse_lolihouse_Shangri_La_Frontier_30() {
        let path =
            PathBuf::from("[LoliHouse] Shangri-La Frontier - 30 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Shangri-La Frontier format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Shangri-La Frontier");
        assert_eq!(info.episode, "30");
    }

    #[test]
    fn test_parse_lolihouse_Blue_Archive_01() {
        let path = PathBuf::from("[LoliHouse] Blue Archive - 01 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Blue Archive format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_lolihouse_Blue_Archive_02() {
        let path = PathBuf::from("[LoliHouse] Blue Archive - 02 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Blue Archive format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_lolihouse_Blue_Archive_03() {
        let path = PathBuf::from("[LoliHouse] Blue Archive - 03 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Blue Archive format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_lolihouse_Blue_Archive_04() {
        let path = PathBuf::from("[LoliHouse] Blue Archive - 04 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Blue Archive format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_lolihouse_Blue_Archive_05() {
        let path = PathBuf::from("[LoliHouse] Blue Archive - 05 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Blue Archive format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_lolihouse_Blue_Archive_06() {
        let path = PathBuf::from("[LoliHouse] Blue Archive - 06 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Blue Archive format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_lolihouse_Blue_Archive_07() {
        let path = PathBuf::from("[LoliHouse] Blue Archive - 07 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Blue Archive format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_lolihouse_Blue_Archive_08() {
        let path = PathBuf::from("[LoliHouse] Blue Archive - 08 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Blue Archive format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_lolihouse_Blue_Archive_09() {
        let path = PathBuf::from("[LoliHouse] Blue Archive - 09 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Blue Archive format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_lolihouse_Blue_Archive_10() {
        let path = PathBuf::from("[LoliHouse] Blue Archive - 10 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Blue Archive format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_lolihouse_Blue_Archive_11() {
        let path = PathBuf::from("[LoliHouse] Blue Archive - 11 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Blue Archive format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_lolihouse_Blue_Archive_12() {
        let path = PathBuf::from("[LoliHouse] Blue Archive - 12 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Blue Archive format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_lolihouse_Make_Heroine_ga_Oruchaya_01() {
        let path = PathBuf::from(
            "[LoliHouse] Make Heroine ga Oruchaya! - 01 [WebRip 1080p HEVC-10bit AAC].mkv",
        );
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [LoliHouse] Make Heroine ga Oruchaya! format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Make Heroine ga Oruchaya!");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_lolihouse_Make_Heroine_ga_Oruchaya_02() {
        let path = PathBuf::from(
            "[LoliHouse] Make Heroine ga Oruchaya! - 02 [WebRip 1080p HEVC-10bit AAC].mkv",
        );
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [LoliHouse] Make Heroine ga Oruchaya! format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Make Heroine ga Oruchaya!");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_lolihouse_Make_Heroine_ga_Oruchaya_03() {
        let path = PathBuf::from(
            "[LoliHouse] Make Heroine ga Oruchaya! - 03 [WebRip 1080p HEVC-10bit AAC].mkv",
        );
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [LoliHouse] Make Heroine ga Oruchaya! format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Make Heroine ga Oruchaya!");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_lolihouse_Make_Heroine_ga_Oruchaya_04() {
        let path = PathBuf::from(
            "[LoliHouse] Make Heroine ga Oruchaya! - 04 [WebRip 1080p HEVC-10bit AAC].mkv",
        );
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [LoliHouse] Make Heroine ga Oruchaya! format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Make Heroine ga Oruchaya!");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_lolihouse_Make_Heroine_ga_Oruchaya_05() {
        let path = PathBuf::from(
            "[LoliHouse] Make Heroine ga Oruchaya! - 05 [WebRip 1080p HEVC-10bit AAC].mkv",
        );
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [LoliHouse] Make Heroine ga Oruchaya! format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Make Heroine ga Oruchaya!");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_lolihouse_Make_Heroine_ga_Oruchaya_06() {
        let path = PathBuf::from(
            "[LoliHouse] Make Heroine ga Oruchaya! - 06 [WebRip 1080p HEVC-10bit AAC].mkv",
        );
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [LoliHouse] Make Heroine ga Oruchaya! format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Make Heroine ga Oruchaya!");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_lolihouse_Make_Heroine_ga_Oruchaya_07() {
        let path = PathBuf::from(
            "[LoliHouse] Make Heroine ga Oruchaya! - 07 [WebRip 1080p HEVC-10bit AAC].mkv",
        );
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [LoliHouse] Make Heroine ga Oruchaya! format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Make Heroine ga Oruchaya!");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_lolihouse_Make_Heroine_ga_Oruchaya_08() {
        let path = PathBuf::from(
            "[LoliHouse] Make Heroine ga Oruchaya! - 08 [WebRip 1080p HEVC-10bit AAC].mkv",
        );
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [LoliHouse] Make Heroine ga Oruchaya! format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Make Heroine ga Oruchaya!");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_lolihouse_Make_Heroine_ga_Oruchaya_09() {
        let path = PathBuf::from(
            "[LoliHouse] Make Heroine ga Oruchaya! - 09 [WebRip 1080p HEVC-10bit AAC].mkv",
        );
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [LoliHouse] Make Heroine ga Oruchaya! format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Make Heroine ga Oruchaya!");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_lolihouse_Make_Heroine_ga_Oruchaya_10() {
        let path = PathBuf::from(
            "[LoliHouse] Make Heroine ga Oruchaya! - 10 [WebRip 1080p HEVC-10bit AAC].mkv",
        );
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [LoliHouse] Make Heroine ga Oruchaya! format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Make Heroine ga Oruchaya!");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_lolihouse_Make_Heroine_ga_Oruchaya_11() {
        let path = PathBuf::from(
            "[LoliHouse] Make Heroine ga Oruchaya! - 11 [WebRip 1080p HEVC-10bit AAC].mkv",
        );
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [LoliHouse] Make Heroine ga Oruchaya! format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Make Heroine ga Oruchaya!");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_lolihouse_Make_Heroine_ga_Oruchaya_12() {
        let path = PathBuf::from(
            "[LoliHouse] Make Heroine ga Oruchaya! - 12 [WebRip 1080p HEVC-10bit AAC].mkv",
        );
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [LoliHouse] Make Heroine ga Oruchaya! format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Make Heroine ga Oruchaya!");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_lolihouse_29_sai_Dokushin_01() {
        let path =
            PathBuf::from("[LoliHouse] 29-sai Dokushin - 01 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 29-sai Dokushin format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "29-sai Dokushin");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_lolihouse_29_sai_Dokushin_02() {
        let path =
            PathBuf::from("[LoliHouse] 29-sai Dokushin - 02 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 29-sai Dokushin format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "29-sai Dokushin");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_lolihouse_29_sai_Dokushin_03() {
        let path =
            PathBuf::from("[LoliHouse] 29-sai Dokushin - 03 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 29-sai Dokushin format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "29-sai Dokushin");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_lolihouse_29_sai_Dokushin_04() {
        let path =
            PathBuf::from("[LoliHouse] 29-sai Dokushin - 04 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 29-sai Dokushin format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "29-sai Dokushin");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_lolihouse_29_sai_Dokushin_05() {
        let path =
            PathBuf::from("[LoliHouse] 29-sai Dokushin - 05 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 29-sai Dokushin format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "29-sai Dokushin");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_lolihouse_29_sai_Dokushin_06() {
        let path =
            PathBuf::from("[LoliHouse] 29-sai Dokushin - 06 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 29-sai Dokushin format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "29-sai Dokushin");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_lolihouse_29_sai_Dokushin_07() {
        let path =
            PathBuf::from("[LoliHouse] 29-sai Dokushin - 07 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 29-sai Dokushin format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "29-sai Dokushin");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_lolihouse_29_sai_Dokushin_08() {
        let path =
            PathBuf::from("[LoliHouse] 29-sai Dokushin - 08 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 29-sai Dokushin format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "29-sai Dokushin");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_lolihouse_29_sai_Dokushin_09() {
        let path =
            PathBuf::from("[LoliHouse] 29-sai Dokushin - 09 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 29-sai Dokushin format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "29-sai Dokushin");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_lolihouse_29_sai_Dokushin_10() {
        let path =
            PathBuf::from("[LoliHouse] 29-sai Dokushin - 10 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 29-sai Dokushin format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "29-sai Dokushin");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_lolihouse_29_sai_Dokushin_11() {
        let path =
            PathBuf::from("[LoliHouse] 29-sai Dokushin - 11 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 29-sai Dokushin format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "29-sai Dokushin");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_lolihouse_29_sai_Dokushin_12() {
        let path =
            PathBuf::from("[LoliHouse] 29-sai Dokushin - 12 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 29-sai Dokushin format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "29-sai Dokushin");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_01() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 01 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_02() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 02 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_03() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 03 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_04() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 04 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_05() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 05 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_06() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 06 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_07() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 07 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_08() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 08 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_09() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 09 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_10() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 10 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_11() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 11 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_12() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 12 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_13() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 13 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "13");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_14() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 14 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "14");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_15() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 15 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "15");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_16() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 16 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "16");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_17() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 17 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "17");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_18() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 18 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "18");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_19() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 19 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "19");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_20() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 20 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "20");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_21() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 21 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "21");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_22() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 22 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "22");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_23() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 23 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "23");
    }

    #[test]
    fn test_parse_lolihouse_Dungeon_Meshi_24() {
        let path =
            PathBuf::from("[LoliHouse] Dungeon Meshi - 24 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] Dungeon Meshi format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "Dungeon Meshi");
        assert_eq!(info.episode, "24");
    }

    #[test]
    fn test_parse_lolihouse_勇者之渣_01() {
        let path = PathBuf::from("[LoliHouse] 勇者之渣 - 01 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 勇者之渣 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "勇者之渣");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_lolihouse_勇者之渣_02() {
        let path = PathBuf::from("[LoliHouse] 勇者之渣 - 02 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 勇者之渣 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "勇者之渣");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_lolihouse_勇者之渣_03() {
        let path = PathBuf::from("[LoliHouse] 勇者之渣 - 03 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 勇者之渣 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "勇者之渣");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_lolihouse_勇者之渣_04() {
        let path = PathBuf::from("[LoliHouse] 勇者之渣 - 04 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 勇者之渣 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "勇者之渣");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_lolihouse_勇者之渣_05() {
        let path = PathBuf::from("[LoliHouse] 勇者之渣 - 05 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 勇者之渣 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "勇者之渣");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_lolihouse_勇者之渣_06() {
        let path = PathBuf::from("[LoliHouse] 勇者之渣 - 06 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 勇者之渣 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "勇者之渣");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_lolihouse_勇者之渣_07() {
        let path = PathBuf::from("[LoliHouse] 勇者之渣 - 07 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 勇者之渣 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "勇者之渣");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_lolihouse_勇者之渣_08() {
        let path = PathBuf::from("[LoliHouse] 勇者之渣 - 08 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 勇者之渣 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "勇者之渣");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_lolihouse_勇者之渣_09() {
        let path = PathBuf::from("[LoliHouse] 勇者之渣 - 09 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 勇者之渣 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "勇者之渣");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_lolihouse_勇者之渣_10() {
        let path = PathBuf::from("[LoliHouse] 勇者之渣 - 10 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 勇者之渣 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "勇者之渣");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_lolihouse_勇者之渣_11() {
        let path = PathBuf::from("[LoliHouse] 勇者之渣 - 11 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 勇者之渣 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "勇者之渣");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_lolihouse_勇者之渣_12() {
        let path = PathBuf::from("[LoliHouse] 勇者之渣 - 12 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 勇者之渣 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "勇者之渣");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_lolihouse_非人学生_01() {
        let path = PathBuf::from("[LoliHouse] 非人学生 - 01 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 非人学生 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "非人学生");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_lolihouse_非人学生_02() {
        let path = PathBuf::from("[LoliHouse] 非人学生 - 02 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 非人学生 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "非人学生");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_lolihouse_非人学生_03() {
        let path = PathBuf::from("[LoliHouse] 非人学生 - 03 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 非人学生 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "非人学生");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_lolihouse_非人学生_04() {
        let path = PathBuf::from("[LoliHouse] 非人学生 - 04 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 非人学生 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "非人学生");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_lolihouse_非人学生_05() {
        let path = PathBuf::from("[LoliHouse] 非人学生 - 05 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 非人学生 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "非人学生");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_lolihouse_非人学生_06() {
        let path = PathBuf::from("[LoliHouse] 非人学生 - 06 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 非人学生 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "非人学生");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_lolihouse_非人学生_07() {
        let path = PathBuf::from("[LoliHouse] 非人学生 - 07 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 非人学生 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "非人学生");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_lolihouse_非人学生_08() {
        let path = PathBuf::from("[LoliHouse] 非人学生 - 08 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 非人学生 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "非人学生");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_lolihouse_非人学生_09() {
        let path = PathBuf::from("[LoliHouse] 非人学生 - 09 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 非人学生 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "非人学生");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_lolihouse_非人学生_10() {
        let path = PathBuf::from("[LoliHouse] 非人学生 - 10 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 非人学生 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "非人学生");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_lolihouse_非人学生_11() {
        let path = PathBuf::from("[LoliHouse] 非人学生 - 11 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 非人学生 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "非人学生");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_lolihouse_非人学生_12() {
        let path = PathBuf::from("[LoliHouse] 非人学生 - 12 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [LoliHouse] 非人学生 format");
        assert_eq!(info.publisher, "LoliHouse");
        assert_eq!(info.anime_name, "非人学生");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_01() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 01 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_02() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 02 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_03() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 03 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_04() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 04 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_05() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 05 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_06() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 06 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_07() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 07 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_08() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 08 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_09() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 09 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_10() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 10 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_11() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 11 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_12() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 12 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_13() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 13 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "13");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_14() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 14 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "14");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_15() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 15 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "15");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_16() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 16 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "16");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_17() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 17 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "17");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_18() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 18 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "18");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_19() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 19 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "19");
    }

    #[test]
    fn test_parse_nc_raws_Dragon_Ball_Z_Kai_20() {
        let path = PathBuf::from("[NC-Raws] Dragon Ball Z Kai - 20 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Dragon Ball Z Kai format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Dragon Ball Z Kai");
        assert_eq!(info.episode, "20");
    }

    #[test]
    fn test_parse_nc_raws_Pocket_Monsters_2023_01() {
        let path = PathBuf::from("[NC-Raws] Pocket Monsters 2023 - 01 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Pocket Monsters 2023 format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Pocket Monsters 2023");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_nc_raws_Pocket_Monsters_2023_02() {
        let path = PathBuf::from("[NC-Raws] Pocket Monsters 2023 - 02 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Pocket Monsters 2023 format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Pocket Monsters 2023");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_nc_raws_Pocket_Monsters_2023_03() {
        let path = PathBuf::from("[NC-Raws] Pocket Monsters 2023 - 03 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Pocket Monsters 2023 format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Pocket Monsters 2023");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_nc_raws_Pocket_Monsters_2023_04() {
        let path = PathBuf::from("[NC-Raws] Pocket Monsters 2023 - 04 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Pocket Monsters 2023 format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Pocket Monsters 2023");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_nc_raws_Pocket_Monsters_2023_05() {
        let path = PathBuf::from("[NC-Raws] Pocket Monsters 2023 - 05 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Pocket Monsters 2023 format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Pocket Monsters 2023");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_nc_raws_Pocket_Monsters_2023_06() {
        let path = PathBuf::from("[NC-Raws] Pocket Monsters 2023 - 06 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Pocket Monsters 2023 format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Pocket Monsters 2023");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_nc_raws_Pocket_Monsters_2023_07() {
        let path = PathBuf::from("[NC-Raws] Pocket Monsters 2023 - 07 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Pocket Monsters 2023 format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Pocket Monsters 2023");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_nc_raws_Pocket_Monsters_2023_08() {
        let path = PathBuf::from("[NC-Raws] Pocket Monsters 2023 - 08 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Pocket Monsters 2023 format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Pocket Monsters 2023");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_nc_raws_Pocket_Monsters_2023_09() {
        let path = PathBuf::from("[NC-Raws] Pocket Monsters 2023 - 09 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Pocket Monsters 2023 format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Pocket Monsters 2023");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_nc_raws_Pocket_Monsters_2023_10() {
        let path = PathBuf::from("[NC-Raws] Pocket Monsters 2023 - 10 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Pocket Monsters 2023 format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Pocket Monsters 2023");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_nc_raws_Pocket_Monsters_2023_11() {
        let path = PathBuf::from("[NC-Raws] Pocket Monsters 2023 - 11 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Pocket Monsters 2023 format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Pocket Monsters 2023");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_nc_raws_Pocket_Monsters_2023_12() {
        let path = PathBuf::from("[NC-Raws] Pocket Monsters 2023 - 12 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Pocket Monsters 2023 format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Pocket Monsters 2023");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_nc_raws_Pocket_Monsters_2023_13() {
        let path = PathBuf::from("[NC-Raws] Pocket Monsters 2023 - 13 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Pocket Monsters 2023 format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Pocket Monsters 2023");
        assert_eq!(info.episode, "13");
    }

    #[test]
    fn test_parse_nc_raws_Pocket_Monsters_2023_14() {
        let path = PathBuf::from("[NC-Raws] Pocket Monsters 2023 - 14 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Pocket Monsters 2023 format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Pocket Monsters 2023");
        assert_eq!(info.episode, "14");
    }

    #[test]
    fn test_parse_nc_raws_Pocket_Monsters_2023_15() {
        let path = PathBuf::from("[NC-Raws] Pocket Monsters 2023 - 15 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Pocket Monsters 2023 format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Pocket Monsters 2023");
        assert_eq!(info.episode, "15");
    }

    #[test]
    fn test_parse_nc_raws_Gachiakuta_01() {
        let path = PathBuf::from("[NC-Raws] Gachiakuta - 01 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Gachiakuta format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Gachiakuta");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_nc_raws_Gachiakuta_02() {
        let path = PathBuf::from("[NC-Raws] Gachiakuta - 02 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Gachiakuta format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Gachiakuta");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_nc_raws_Gachiakuta_03() {
        let path = PathBuf::from("[NC-Raws] Gachiakuta - 03 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Gachiakuta format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Gachiakuta");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_nc_raws_Gachiakuta_04() {
        let path = PathBuf::from("[NC-Raws] Gachiakuta - 04 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Gachiakuta format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Gachiakuta");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_nc_raws_Gachiakuta_05() {
        let path = PathBuf::from("[NC-Raws] Gachiakuta - 05 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Gachiakuta format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Gachiakuta");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_nc_raws_Gachiakuta_06() {
        let path = PathBuf::from("[NC-Raws] Gachiakuta - 06 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Gachiakuta format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Gachiakuta");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_nc_raws_Gachiakuta_07() {
        let path = PathBuf::from("[NC-Raws] Gachiakuta - 07 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Gachiakuta format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Gachiakuta");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_nc_raws_Gachiakuta_08() {
        let path = PathBuf::from("[NC-Raws] Gachiakuta - 08 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Gachiakuta format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Gachiakuta");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_nc_raws_Gachiakuta_09() {
        let path = PathBuf::from("[NC-Raws] Gachiakuta - 09 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Gachiakuta format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Gachiakuta");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_nc_raws_Gachiakuta_10() {
        let path = PathBuf::from("[NC-Raws] Gachiakuta - 10 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Gachiakuta format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Gachiakuta");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_nc_raws_Gachiakuta_11() {
        let path = PathBuf::from("[NC-Raws] Gachiakuta - 11 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Gachiakuta format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Gachiakuta");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_nc_raws_Gachiakuta_12() {
        let path = PathBuf::from("[NC-Raws] Gachiakuta - 12 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Gachiakuta format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Gachiakuta");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_nc_raws_Gachiakuta_13() {
        let path = PathBuf::from("[NC-Raws] Gachiakuta - 13 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Gachiakuta format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Gachiakuta");
        assert_eq!(info.episode, "13");
    }

    #[test]
    fn test_parse_nc_raws_Gachiakuta_14() {
        let path = PathBuf::from("[NC-Raws] Gachiakuta - 14 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Gachiakuta format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Gachiakuta");
        assert_eq!(info.episode, "14");
    }

    #[test]
    fn test_parse_nc_raws_Gachiakuta_15() {
        let path = PathBuf::from("[NC-Raws] Gachiakuta - 15 [1080P][Baha][WEB-DL].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [NC-Raws] Gachiakuta format");
        assert_eq!(info.publisher, "NC-Raws");
        assert_eq!(info.anime_name, "Gachiakuta");
        assert_eq!(info.episode, "15");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_01() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_02() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_03() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_04() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_05() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_06() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_07() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_08() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_09() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_10() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_11() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 11 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_12() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 12 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_13() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 13 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "13");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_14() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 14 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "14");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_15() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 15 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "15");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_16() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 16 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "16");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_17() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 17 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "17");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_18() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 18 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "18");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_19() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 19 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "19");
    }

    #[test]
    fn test_parse_lilith_raws_Golden_Kamuy_20() {
        let path = PathBuf::from("[Lilith-Raws] Golden Kamuy - 20 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Golden Kamuy format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Golden Kamuy");
        assert_eq!(info.episode, "20");
    }

    #[test]
    fn test_parse_lilith_raws_Yuusha_no_Kuzu_01() {
        let path = PathBuf::from("[Lilith-Raws] Yuusha no Kuzu - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Yuusha no Kuzu format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Yuusha no Kuzu");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_lilith_raws_Yuusha_no_Kuzu_02() {
        let path = PathBuf::from("[Lilith-Raws] Yuusha no Kuzu - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Yuusha no Kuzu format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Yuusha no Kuzu");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_lilith_raws_Yuusha_no_Kuzu_03() {
        let path = PathBuf::from("[Lilith-Raws] Yuusha no Kuzu - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Yuusha no Kuzu format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Yuusha no Kuzu");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_lilith_raws_Yuusha_no_Kuzu_04() {
        let path = PathBuf::from("[Lilith-Raws] Yuusha no Kuzu - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Yuusha no Kuzu format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Yuusha no Kuzu");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_lilith_raws_Yuusha_no_Kuzu_05() {
        let path = PathBuf::from("[Lilith-Raws] Yuusha no Kuzu - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Yuusha no Kuzu format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Yuusha no Kuzu");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_lilith_raws_Yuusha_no_Kuzu_06() {
        let path = PathBuf::from("[Lilith-Raws] Yuusha no Kuzu - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Yuusha no Kuzu format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Yuusha no Kuzu");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_lilith_raws_Yuusha_no_Kuzu_07() {
        let path = PathBuf::from("[Lilith-Raws] Yuusha no Kuzu - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Yuusha no Kuzu format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Yuusha no Kuzu");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_lilith_raws_Yuusha_no_Kuzu_08() {
        let path = PathBuf::from("[Lilith-Raws] Yuusha no Kuzu - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Yuusha no Kuzu format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Yuusha no Kuzu");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_lilith_raws_Yuusha_no_Kuzu_09() {
        let path = PathBuf::from("[Lilith-Raws] Yuusha no Kuzu - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Yuusha no Kuzu format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Yuusha no Kuzu");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_lilith_raws_Yuusha_no_Kuzu_10() {
        let path = PathBuf::from("[Lilith-Raws] Yuusha no Kuzu - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Yuusha no Kuzu format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Yuusha no Kuzu");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_lilith_raws_Yuusha_no_Kuzu_11() {
        let path = PathBuf::from("[Lilith-Raws] Yuusha no Kuzu - 11 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Yuusha no Kuzu format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Yuusha no Kuzu");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_lilith_raws_Yuusha_no_Kuzu_12() {
        let path = PathBuf::from("[Lilith-Raws] Yuusha no Kuzu - 12 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Yuusha no Kuzu format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Yuusha no Kuzu");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_lilith_raws_Rooster_Fighter_01() {
        let path = PathBuf::from("[Lilith-Raws] Rooster Fighter - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Rooster Fighter format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Rooster Fighter");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_lilith_raws_Rooster_Fighter_02() {
        let path = PathBuf::from("[Lilith-Raws] Rooster Fighter - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Rooster Fighter format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Rooster Fighter");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_lilith_raws_Rooster_Fighter_03() {
        let path = PathBuf::from("[Lilith-Raws] Rooster Fighter - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Rooster Fighter format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Rooster Fighter");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_lilith_raws_Rooster_Fighter_04() {
        let path = PathBuf::from("[Lilith-Raws] Rooster Fighter - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Rooster Fighter format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Rooster Fighter");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_lilith_raws_Rooster_Fighter_05() {
        let path = PathBuf::from("[Lilith-Raws] Rooster Fighter - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Rooster Fighter format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Rooster Fighter");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_lilith_raws_Rooster_Fighter_06() {
        let path = PathBuf::from("[Lilith-Raws] Rooster Fighter - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Rooster Fighter format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Rooster Fighter");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_lilith_raws_Rooster_Fighter_07() {
        let path = PathBuf::from("[Lilith-Raws] Rooster Fighter - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Rooster Fighter format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Rooster Fighter");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_lilith_raws_Rooster_Fighter_08() {
        let path = PathBuf::from("[Lilith-Raws] Rooster Fighter - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Rooster Fighter format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Rooster Fighter");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_lilith_raws_Rooster_Fighter_09() {
        let path = PathBuf::from("[Lilith-Raws] Rooster Fighter - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Rooster Fighter format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Rooster Fighter");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_lilith_raws_Rooster_Fighter_10() {
        let path = PathBuf::from("[Lilith-Raws] Rooster Fighter - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Rooster Fighter format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Rooster Fighter");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_lilith_raws_Rooster_Fighter_11() {
        let path = PathBuf::from("[Lilith-Raws] Rooster Fighter - 11 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Rooster Fighter format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Rooster Fighter");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_lilith_raws_Rooster_Fighter_12() {
        let path = PathBuf::from("[Lilith-Raws] Rooster Fighter - 12 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [Lilith-Raws] Rooster Fighter format");
        assert_eq!(info.publisher, "Lilith-Raws");
        assert_eq!(info.anime_name, "Rooster Fighter");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_chixia_某科学的超电磁炮_01() {
        let path = PathBuf::from("[千夏字幕组] 某科学的超电磁炮 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_chixia_某科学的超电磁炮_02() {
        let path = PathBuf::from("[千夏字幕组] 某科学的超电磁炮 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_chixia_某科学的超电磁炮_03() {
        let path = PathBuf::from("[千夏字幕组] 某科学的超电磁炮 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_chixia_某科学的超电磁炮_04() {
        let path = PathBuf::from("[千夏字幕组] 某科学的超电磁炮 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_chixia_某科学的超电磁炮_05() {
        let path = PathBuf::from("[千夏字幕组] 某科学的超电磁炮 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_chixia_某科学的超电磁炮_06() {
        let path = PathBuf::from("[千夏字幕组] 某科学的超电磁炮 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_chixia_某科学的超电磁炮_07() {
        let path = PathBuf::from("[千夏字幕组] 某科学的超电磁炮 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_chixia_某科学的超电磁炮_08() {
        let path = PathBuf::from("[千夏字幕组] 某科学的超电磁炮 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_chixia_某科学的超电磁炮_09() {
        let path = PathBuf::from("[千夏字幕组] 某科学的超电磁炮 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_chixia_某科学的超电磁炮_10() {
        let path = PathBuf::from("[千夏字幕组] 某科学的超电磁炮 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_chixia_青春猪头少年_01() {
        let path = PathBuf::from("[千夏字幕组] 青春猪头少年 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 青春猪头少年 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "青春猪头少年");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_chixia_青春猪头少年_02() {
        let path = PathBuf::from("[千夏字幕组] 青春猪头少年 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 青春猪头少年 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "青春猪头少年");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_chixia_青春猪头少年_03() {
        let path = PathBuf::from("[千夏字幕组] 青春猪头少年 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 青春猪头少年 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "青春猪头少年");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_chixia_青春猪头少年_04() {
        let path = PathBuf::from("[千夏字幕组] 青春猪头少年 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 青春猪头少年 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "青春猪头少年");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_chixia_青春猪头少年_05() {
        let path = PathBuf::from("[千夏字幕组] 青春猪头少年 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 青春猪头少年 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "青春猪头少年");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_chixia_青春猪头少年_06() {
        let path = PathBuf::from("[千夏字幕组] 青春猪头少年 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 青春猪头少年 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "青春猪头少年");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_chixia_青春猪头少年_07() {
        let path = PathBuf::from("[千夏字幕组] 青春猪头少年 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 青春猪头少年 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "青春猪头少年");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_chixia_青春猪头少年_08() {
        let path = PathBuf::from("[千夏字幕组] 青春猪头少年 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 青春猪头少年 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "青春猪头少年");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_chixia_青春猪头少年_09() {
        let path = PathBuf::from("[千夏字幕组] 青春猪头少年 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 青春猪头少年 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "青春猪头少年");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_chixia_青春猪头少年_10() {
        let path = PathBuf::from("[千夏字幕组] 青春猪头少年 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 青春猪头少年 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "青春猪头少年");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_chixia_四月是你的谎言_01() {
        let path = PathBuf::from("[千夏字幕组] 四月是你的谎言 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 四月是你的谎言 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_chixia_四月是你的谎言_02() {
        let path = PathBuf::from("[千夏字幕组] 四月是你的谎言 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 四月是你的谎言 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_chixia_四月是你的谎言_03() {
        let path = PathBuf::from("[千夏字幕组] 四月是你的谎言 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 四月是你的谎言 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_chixia_四月是你的谎言_04() {
        let path = PathBuf::from("[千夏字幕组] 四月是你的谎言 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 四月是你的谎言 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_chixia_四月是你的谎言_05() {
        let path = PathBuf::from("[千夏字幕组] 四月是你的谎言 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 四月是你的谎言 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_chixia_四月是你的谎言_06() {
        let path = PathBuf::from("[千夏字幕组] 四月是你的谎言 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 四月是你的谎言 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_chixia_四月是你的谎言_07() {
        let path = PathBuf::from("[千夏字幕组] 四月是你的谎言 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 四月是你的谎言 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_chixia_四月是你的谎言_08() {
        let path = PathBuf::from("[千夏字幕组] 四月是你的谎言 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 四月是你的谎言 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_chixia_四月是你的谎言_09() {
        let path = PathBuf::from("[千夏字幕组] 四月是你的谎言 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 四月是你的谎言 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_chixia_四月是你的谎言_10() {
        let path = PathBuf::from("[千夏字幕组] 四月是你的谎言 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 四月是你的谎言 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_chixia_未闻花名_01() {
        let path = PathBuf::from("[千夏字幕组] 未闻花名 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 未闻花名 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_chixia_未闻花名_02() {
        let path = PathBuf::from("[千夏字幕组] 未闻花名 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 未闻花名 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_chixia_未闻花名_03() {
        let path = PathBuf::from("[千夏字幕组] 未闻花名 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 未闻花名 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_chixia_未闻花名_04() {
        let path = PathBuf::from("[千夏字幕组] 未闻花名 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 未闻花名 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_chixia_未闻花名_05() {
        let path = PathBuf::from("[千夏字幕组] 未闻花名 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 未闻花名 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_chixia_未闻花名_06() {
        let path = PathBuf::from("[千夏字幕组] 未闻花名 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 未闻花名 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_chixia_未闻花名_07() {
        let path = PathBuf::from("[千夏字幕组] 未闻花名 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 未闻花名 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_chixia_未闻花名_08() {
        let path = PathBuf::from("[千夏字幕组] 未闻花名 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 未闻花名 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_chixia_未闻花名_09() {
        let path = PathBuf::from("[千夏字幕组] 未闻花名 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 未闻花名 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_chixia_未闻花名_10() {
        let path = PathBuf::from("[千夏字幕组] 未闻花名 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [千夏字幕组] 未闻花名 format");
        assert_eq!(info.publisher, "千夏字幕组");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_xingkong_葬送的芙莉莲_01() {
        let path = PathBuf::from("[星空字幕组] 葬送的芙莉莲 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_xingkong_葬送的芙莉莲_02() {
        let path = PathBuf::from("[星空字幕组] 葬送的芙莉莲 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_xingkong_葬送的芙莉莲_03() {
        let path = PathBuf::from("[星空字幕组] 葬送的芙莉莲 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_xingkong_葬送的芙莉莲_04() {
        let path = PathBuf::from("[星空字幕组] 葬送的芙莉莲 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_xingkong_葬送的芙莉莲_05() {
        let path = PathBuf::from("[星空字幕组] 葬送的芙莉莲 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_xingkong_葬送的芙莉莲_06() {
        let path = PathBuf::from("[星空字幕组] 葬送的芙莉莲 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_xingkong_葬送的芙莉莲_07() {
        let path = PathBuf::from("[星空字幕组] 葬送的芙莉莲 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_xingkong_葬送的芙莉莲_08() {
        let path = PathBuf::from("[星空字幕组] 葬送的芙莉莲 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_xingkong_葬送的芙莉莲_09() {
        let path = PathBuf::from("[星空字幕组] 葬送的芙莉莲 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_xingkong_葬送的芙莉莲_10() {
        let path = PathBuf::from("[星空字幕组] 葬送的芙莉莲 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_xingkong_进击的巨人_01() {
        let path = PathBuf::from("[星空字幕组] 进击的巨人 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 进击的巨人 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "进击的巨人");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_xingkong_进击的巨人_02() {
        let path = PathBuf::from("[星空字幕组] 进击的巨人 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 进击的巨人 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "进击的巨人");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_xingkong_进击的巨人_03() {
        let path = PathBuf::from("[星空字幕组] 进击的巨人 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 进击的巨人 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "进击的巨人");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_xingkong_进击的巨人_04() {
        let path = PathBuf::from("[星空字幕组] 进击的巨人 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 进击的巨人 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "进击的巨人");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_xingkong_进击的巨人_05() {
        let path = PathBuf::from("[星空字幕组] 进击的巨人 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 进击的巨人 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "进击的巨人");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_xingkong_进击的巨人_06() {
        let path = PathBuf::from("[星空字幕组] 进击的巨人 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 进击的巨人 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "进击的巨人");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_xingkong_进击的巨人_07() {
        let path = PathBuf::from("[星空字幕组] 进击的巨人 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 进击的巨人 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "进击的巨人");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_xingkong_进击的巨人_08() {
        let path = PathBuf::from("[星空字幕组] 进击的巨人 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 进击的巨人 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "进击的巨人");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_xingkong_进击的巨人_09() {
        let path = PathBuf::from("[星空字幕组] 进击的巨人 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 进击的巨人 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "进击的巨人");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_xingkong_进击的巨人_10() {
        let path = PathBuf::from("[星空字幕组] 进击的巨人 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 进击的巨人 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "进击的巨人");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_xingkong_鬼灭之刃_01() {
        let path = PathBuf::from("[星空字幕组] 鬼灭之刃 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 鬼灭之刃 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "鬼灭之刃");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_xingkong_鬼灭之刃_02() {
        let path = PathBuf::from("[星空字幕组] 鬼灭之刃 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 鬼灭之刃 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "鬼灭之刃");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_xingkong_鬼灭之刃_03() {
        let path = PathBuf::from("[星空字幕组] 鬼灭之刃 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 鬼灭之刃 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "鬼灭之刃");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_xingkong_鬼灭之刃_04() {
        let path = PathBuf::from("[星空字幕组] 鬼灭之刃 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 鬼灭之刃 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "鬼灭之刃");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_xingkong_鬼灭之刃_05() {
        let path = PathBuf::from("[星空字幕组] 鬼灭之刃 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 鬼灭之刃 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "鬼灭之刃");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_xingkong_鬼灭之刃_06() {
        let path = PathBuf::from("[星空字幕组] 鬼灭之刃 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 鬼灭之刃 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "鬼灭之刃");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_xingkong_鬼灭之刃_07() {
        let path = PathBuf::from("[星空字幕组] 鬼灭之刃 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 鬼灭之刃 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "鬼灭之刃");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_xingkong_鬼灭之刃_08() {
        let path = PathBuf::from("[星空字幕组] 鬼灭之刃 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 鬼灭之刃 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "鬼灭之刃");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_xingkong_鬼灭之刃_09() {
        let path = PathBuf::from("[星空字幕组] 鬼灭之刃 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 鬼灭之刃 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "鬼灭之刃");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_xingkong_鬼灭之刃_10() {
        let path = PathBuf::from("[星空字幕组] 鬼灭之刃 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 鬼灭之刃 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "鬼灭之刃");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_xingkong_咒术回战_01() {
        let path = PathBuf::from("[星空字幕组] 咒术回战 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 咒术回战 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "咒术回战");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_xingkong_咒术回战_02() {
        let path = PathBuf::from("[星空字幕组] 咒术回战 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 咒术回战 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "咒术回战");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_xingkong_咒术回战_03() {
        let path = PathBuf::from("[星空字幕组] 咒术回战 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 咒术回战 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "咒术回战");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_xingkong_咒术回战_04() {
        let path = PathBuf::from("[星空字幕组] 咒术回战 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 咒术回战 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "咒术回战");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_xingkong_咒术回战_05() {
        let path = PathBuf::from("[星空字幕组] 咒术回战 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 咒术回战 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "咒术回战");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_xingkong_咒术回战_06() {
        let path = PathBuf::from("[星空字幕组] 咒术回战 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 咒术回战 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "咒术回战");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_xingkong_咒术回战_07() {
        let path = PathBuf::from("[星空字幕组] 咒术回战 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 咒术回战 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "咒术回战");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_xingkong_咒术回战_08() {
        let path = PathBuf::from("[星空字幕组] 咒术回战 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 咒术回战 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "咒术回战");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_xingkong_咒术回战_09() {
        let path = PathBuf::from("[星空字幕组] 咒术回战 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 咒术回战 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "咒术回战");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_xingkong_咒术回战_10() {
        let path = PathBuf::from("[星空字幕组] 咒术回战 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [星空字幕组] 咒术回战 format");
        assert_eq!(info.publisher, "星空字幕组");
        assert_eq!(info.anime_name, "咒术回战");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_menglan_Blue_Archive_01() {
        let path = PathBuf::from("[梦蓝字幕组] Blue Archive - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] Blue Archive format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_menglan_Blue_Archive_02() {
        let path = PathBuf::from("[梦蓝字幕组] Blue Archive - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] Blue Archive format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_menglan_Blue_Archive_03() {
        let path = PathBuf::from("[梦蓝字幕组] Blue Archive - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] Blue Archive format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_menglan_Blue_Archive_04() {
        let path = PathBuf::from("[梦蓝字幕组] Blue Archive - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] Blue Archive format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_menglan_Blue_Archive_05() {
        let path = PathBuf::from("[梦蓝字幕组] Blue Archive - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] Blue Archive format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_menglan_Blue_Archive_06() {
        let path = PathBuf::from("[梦蓝字幕组] Blue Archive - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] Blue Archive format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_menglan_Blue_Archive_07() {
        let path = PathBuf::from("[梦蓝字幕组] Blue Archive - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] Blue Archive format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_menglan_Blue_Archive_08() {
        let path = PathBuf::from("[梦蓝字幕组] Blue Archive - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] Blue Archive format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_menglan_Blue_Archive_09() {
        let path = PathBuf::from("[梦蓝字幕组] Blue Archive - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] Blue Archive format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_menglan_Blue_Archive_10() {
        let path = PathBuf::from("[梦蓝字幕组] Blue Archive - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] Blue Archive format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "Blue Archive");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_menglan_赛博朋克边缘行者_01() {
        let path = PathBuf::from("[梦蓝字幕组] 赛博朋克边缘行者 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 赛博朋克边缘行者 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "赛博朋克边缘行者");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_menglan_赛博朋克边缘行者_02() {
        let path = PathBuf::from("[梦蓝字幕组] 赛博朋克边缘行者 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 赛博朋克边缘行者 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "赛博朋克边缘行者");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_menglan_赛博朋克边缘行者_03() {
        let path = PathBuf::from("[梦蓝字幕组] 赛博朋克边缘行者 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 赛博朋克边缘行者 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "赛博朋克边缘行者");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_menglan_赛博朋克边缘行者_04() {
        let path = PathBuf::from("[梦蓝字幕组] 赛博朋克边缘行者 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 赛博朋克边缘行者 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "赛博朋克边缘行者");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_menglan_赛博朋克边缘行者_05() {
        let path = PathBuf::from("[梦蓝字幕组] 赛博朋克边缘行者 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 赛博朋克边缘行者 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "赛博朋克边缘行者");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_menglan_赛博朋克边缘行者_06() {
        let path = PathBuf::from("[梦蓝字幕组] 赛博朋克边缘行者 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 赛博朋克边缘行者 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "赛博朋克边缘行者");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_menglan_赛博朋克边缘行者_07() {
        let path = PathBuf::from("[梦蓝字幕组] 赛博朋克边缘行者 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 赛博朋克边缘行者 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "赛博朋克边缘行者");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_menglan_赛博朋克边缘行者_08() {
        let path = PathBuf::from("[梦蓝字幕组] 赛博朋克边缘行者 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 赛博朋克边缘行者 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "赛博朋克边缘行者");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_menglan_赛博朋克边缘行者_09() {
        let path = PathBuf::from("[梦蓝字幕组] 赛博朋克边缘行者 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 赛博朋克边缘行者 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "赛博朋克边缘行者");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_menglan_赛博朋克边缘行者_10() {
        let path = PathBuf::from("[梦蓝字幕组] 赛博朋克边缘行者 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 赛博朋克边缘行者 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "赛博朋克边缘行者");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_menglan_孤独摇滚_01() {
        let path = PathBuf::from("[梦蓝字幕组] 孤独摇滚 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 孤独摇滚 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "孤独摇滚");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_menglan_孤独摇滚_02() {
        let path = PathBuf::from("[梦蓝字幕组] 孤独摇滚 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 孤独摇滚 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "孤独摇滚");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_menglan_孤独摇滚_03() {
        let path = PathBuf::from("[梦蓝字幕组] 孤独摇滚 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 孤独摇滚 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "孤独摇滚");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_menglan_孤独摇滚_04() {
        let path = PathBuf::from("[梦蓝字幕组] 孤独摇滚 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 孤独摇滚 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "孤独摇滚");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_menglan_孤独摇滚_05() {
        let path = PathBuf::from("[梦蓝字幕组] 孤独摇滚 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 孤独摇滚 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "孤独摇滚");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_menglan_孤独摇滚_06() {
        let path = PathBuf::from("[梦蓝字幕组] 孤独摇滚 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 孤独摇滚 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "孤独摇滚");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_menglan_孤独摇滚_07() {
        let path = PathBuf::from("[梦蓝字幕组] 孤独摇滚 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 孤独摇滚 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "孤独摇滚");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_menglan_孤独摇滚_08() {
        let path = PathBuf::from("[梦蓝字幕组] 孤独摇滚 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 孤独摇滚 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "孤独摇滚");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_menglan_孤独摇滚_09() {
        let path = PathBuf::from("[梦蓝字幕组] 孤独摇滚 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 孤独摇滚 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "孤独摇滚");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_menglan_孤独摇滚_10() {
        let path = PathBuf::from("[梦蓝字幕组] 孤独摇滚 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] 孤独摇滚 format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "孤独摇滚");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_menglan_MyGO_01() {
        let path = PathBuf::from("[梦蓝字幕组] MyGO - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] MyGO format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "MyGO");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_menglan_MyGO_02() {
        let path = PathBuf::from("[梦蓝字幕组] MyGO - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] MyGO format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "MyGO");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_menglan_MyGO_03() {
        let path = PathBuf::from("[梦蓝字幕组] MyGO - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] MyGO format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "MyGO");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_menglan_MyGO_04() {
        let path = PathBuf::from("[梦蓝字幕组] MyGO - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] MyGO format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "MyGO");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_menglan_MyGO_05() {
        let path = PathBuf::from("[梦蓝字幕组] MyGO - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] MyGO format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "MyGO");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_menglan_MyGO_06() {
        let path = PathBuf::from("[梦蓝字幕组] MyGO - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] MyGO format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "MyGO");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_menglan_MyGO_07() {
        let path = PathBuf::from("[梦蓝字幕组] MyGO - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] MyGO format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "MyGO");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_menglan_MyGO_08() {
        let path = PathBuf::from("[梦蓝字幕组] MyGO - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] MyGO format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "MyGO");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_menglan_MyGO_09() {
        let path = PathBuf::from("[梦蓝字幕组] MyGO - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] MyGO format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "MyGO");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_menglan_MyGO_10() {
        let path = PathBuf::from("[梦蓝字幕组] MyGO - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [梦蓝字幕组] MyGO format");
        assert_eq!(info.publisher, "梦蓝字幕组");
        assert_eq!(info.anime_name, "MyGO");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_weiyang_迷宫饭_01() {
        let path = PathBuf::from("[未央阁联盟] 迷宫饭 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 迷宫饭 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_weiyang_迷宫饭_02() {
        let path = PathBuf::from("[未央阁联盟] 迷宫饭 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 迷宫饭 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_weiyang_迷宫饭_03() {
        let path = PathBuf::from("[未央阁联盟] 迷宫饭 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 迷宫饭 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_weiyang_迷宫饭_04() {
        let path = PathBuf::from("[未央阁联盟] 迷宫饭 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 迷宫饭 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_weiyang_迷宫饭_05() {
        let path = PathBuf::from("[未央阁联盟] 迷宫饭 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 迷宫饭 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_weiyang_迷宫饭_06() {
        let path = PathBuf::from("[未央阁联盟] 迷宫饭 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 迷宫饭 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_weiyang_迷宫饭_07() {
        let path = PathBuf::from("[未央阁联盟] 迷宫饭 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 迷宫饭 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_weiyang_迷宫饭_08() {
        let path = PathBuf::from("[未央阁联盟] 迷宫饭 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 迷宫饭 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_weiyang_迷宫饭_09() {
        let path = PathBuf::from("[未央阁联盟] 迷宫饭 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 迷宫饭 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_weiyang_迷宫饭_10() {
        let path = PathBuf::from("[未央阁联盟] 迷宫饭 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 迷宫饭 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_weiyang_不死之王_01() {
        let path = PathBuf::from("[未央阁联盟] 不死之王 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 不死之王 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "不死之王");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_weiyang_不死之王_02() {
        let path = PathBuf::from("[未央阁联盟] 不死之王 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 不死之王 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "不死之王");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_weiyang_不死之王_03() {
        let path = PathBuf::from("[未央阁联盟] 不死之王 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 不死之王 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "不死之王");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_weiyang_不死之王_04() {
        let path = PathBuf::from("[未央阁联盟] 不死之王 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 不死之王 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "不死之王");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_weiyang_不死之王_05() {
        let path = PathBuf::from("[未央阁联盟] 不死之王 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 不死之王 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "不死之王");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_weiyang_不死之王_06() {
        let path = PathBuf::from("[未央阁联盟] 不死之王 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 不死之王 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "不死之王");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_weiyang_不死之王_07() {
        let path = PathBuf::from("[未央阁联盟] 不死之王 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 不死之王 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "不死之王");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_weiyang_不死之王_08() {
        let path = PathBuf::from("[未央阁联盟] 不死之王 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 不死之王 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "不死之王");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_weiyang_不死之王_09() {
        let path = PathBuf::from("[未央阁联盟] 不死之王 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 不死之王 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "不死之王");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_weiyang_不死之王_10() {
        let path = PathBuf::from("[未央阁联盟] 不死之王 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 不死之王 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "不死之王");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_weiyang_盾之勇者_01() {
        let path = PathBuf::from("[未央阁联盟] 盾之勇者 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 盾之勇者 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "盾之勇者");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_weiyang_盾之勇者_02() {
        let path = PathBuf::from("[未央阁联盟] 盾之勇者 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 盾之勇者 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "盾之勇者");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_weiyang_盾之勇者_03() {
        let path = PathBuf::from("[未央阁联盟] 盾之勇者 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 盾之勇者 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "盾之勇者");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_weiyang_盾之勇者_04() {
        let path = PathBuf::from("[未央阁联盟] 盾之勇者 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 盾之勇者 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "盾之勇者");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_weiyang_盾之勇者_05() {
        let path = PathBuf::from("[未央阁联盟] 盾之勇者 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 盾之勇者 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "盾之勇者");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_weiyang_盾之勇者_06() {
        let path = PathBuf::from("[未央阁联盟] 盾之勇者 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 盾之勇者 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "盾之勇者");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_weiyang_盾之勇者_07() {
        let path = PathBuf::from("[未央阁联盟] 盾之勇者 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 盾之勇者 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "盾之勇者");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_weiyang_盾之勇者_08() {
        let path = PathBuf::from("[未央阁联盟] 盾之勇者 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 盾之勇者 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "盾之勇者");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_weiyang_盾之勇者_09() {
        let path = PathBuf::from("[未央阁联盟] 盾之勇者 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 盾之勇者 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "盾之勇者");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_weiyang_盾之勇者_10() {
        let path = PathBuf::from("[未央阁联盟] 盾之勇者 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 盾之勇者 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "盾之勇者");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_weiyang_无职转生_01() {
        let path = PathBuf::from("[未央阁联盟] 无职转生 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 无职转生 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "无职转生");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_weiyang_无职转生_02() {
        let path = PathBuf::from("[未央阁联盟] 无职转生 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 无职转生 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "无职转生");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_weiyang_无职转生_03() {
        let path = PathBuf::from("[未央阁联盟] 无职转生 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 无职转生 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "无职转生");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_weiyang_无职转生_04() {
        let path = PathBuf::from("[未央阁联盟] 无职转生 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 无职转生 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "无职转生");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_weiyang_无职转生_05() {
        let path = PathBuf::from("[未央阁联盟] 无职转生 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 无职转生 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "无职转生");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_weiyang_无职转生_06() {
        let path = PathBuf::from("[未央阁联盟] 无职转生 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 无职转生 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "无职转生");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_weiyang_无职转生_07() {
        let path = PathBuf::from("[未央阁联盟] 无职转生 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 无职转生 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "无职转生");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_weiyang_无职转生_08() {
        let path = PathBuf::from("[未央阁联盟] 无职转生 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 无职转生 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "无职转生");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_weiyang_无职转生_09() {
        let path = PathBuf::from("[未央阁联盟] 无职转生 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 无职转生 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "无职转生");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_weiyang_无职转生_10() {
        let path = PathBuf::from("[未央阁联盟] 无职转生 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [未央阁联盟] 无职转生 format");
        assert_eq!(info.publisher, "未央阁联盟");
        assert_eq!(info.anime_name, "无职转生");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_zhushen_Overlord_01() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_zhushen_Overlord_02() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_zhushen_Overlord_03() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_zhushen_Overlord_04() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_zhushen_Overlord_05() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_zhushen_Overlord_06() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_zhushen_Overlord_07() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_zhushen_Overlord_08() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_zhushen_Overlord_09() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_zhushen_Overlord_10() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_zhushen_Overlord_11() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 11 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_zhushen_Overlord_12() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 12 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_zhushen_Overlord_13() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 13 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "13");
    }

    #[test]
    fn test_parse_zhushen_Overlord_14() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 14 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "14");
    }

    #[test]
    fn test_parse_zhushen_Overlord_15() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 15 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "15");
    }

    #[test]
    fn test_parse_zhushen_Overlord_16() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 16 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "16");
    }

    #[test]
    fn test_parse_zhushen_Overlord_17() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 17 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "17");
    }

    #[test]
    fn test_parse_zhushen_Overlord_18() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 18 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "18");
    }

    #[test]
    fn test_parse_zhushen_Overlord_19() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 19 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "19");
    }

    #[test]
    fn test_parse_zhushen_Overlord_20() {
        let path = PathBuf::from("[诸神kamigami字幕组] Overlord - 20 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [诸神kamigami字幕组] Overlord format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Overlord");
        assert_eq!(info.episode, "20");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_01() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_02() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_03() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_04() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_05() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_06() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_07() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_08() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_09() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_10() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_11() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 11 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "11");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_12() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 12 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "12");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_13() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 13 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "13");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_14() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 14 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "14");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_15() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 15 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "15");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_16() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 16 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "16");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_17() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 17 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "17");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_18() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 18 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "18");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_19() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 19 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "19");
    }

    #[test]
    fn test_parse_zhushen_Re从零开始的异世界生活_20() {
        let path = PathBuf::from("[诸神kamigami字幕组] Re从零开始的异世界生活 - 20 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result
            .expect("Parser should handle [诸神kamigami字幕组] Re从零开始的异世界生活 format");
        assert_eq!(info.publisher, "诸神kamigami字幕组");
        assert_eq!(info.anime_name, "Re从零开始的异世界生活");
        assert_eq!(info.episode, "20");
    }

    #[test]
    fn test_parse_miaomeng_Love_Live_01() {
        let path = PathBuf::from("[喵萌奶茶屋] Love Live - 01 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] Love Live format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "Love Live");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_miaomeng_Love_Live_02() {
        let path = PathBuf::from("[喵萌奶茶屋] Love Live - 02 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] Love Live format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "Love Live");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_miaomeng_Love_Live_03() {
        let path = PathBuf::from("[喵萌奶茶屋] Love Live - 03 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] Love Live format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "Love Live");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_miaomeng_Love_Live_04() {
        let path = PathBuf::from("[喵萌奶茶屋] Love Live - 04 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] Love Live format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "Love Live");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_miaomeng_Love_Live_05() {
        let path = PathBuf::from("[喵萌奶茶屋] Love Live - 05 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] Love Live format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "Love Live");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_miaomeng_Love_Live_06() {
        let path = PathBuf::from("[喵萌奶茶屋] Love Live - 06 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] Love Live format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "Love Live");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_miaomeng_Love_Live_07() {
        let path = PathBuf::from("[喵萌奶茶屋] Love Live - 07 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] Love Live format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "Love Live");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_miaomeng_Love_Live_08() {
        let path = PathBuf::from("[喵萌奶茶屋] Love Live - 08 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] Love Live format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "Love Live");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_miaomeng_Love_Live_09() {
        let path = PathBuf::from("[喵萌奶茶屋] Love Live - 09 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] Love Live format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "Love Live");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_miaomeng_Love_Live_10() {
        let path = PathBuf::from("[喵萌奶茶屋] Love Live - 10 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] Love Live format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "Love Live");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_miaomeng_舰队Collection_01() {
        let path = PathBuf::from("[喵萌奶茶屋] 舰队Collection - 01 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 舰队Collection format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "舰队Collection");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_miaomeng_舰队Collection_02() {
        let path = PathBuf::from("[喵萌奶茶屋] 舰队Collection - 02 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 舰队Collection format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "舰队Collection");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_miaomeng_舰队Collection_03() {
        let path = PathBuf::from("[喵萌奶茶屋] 舰队Collection - 03 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 舰队Collection format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "舰队Collection");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_miaomeng_舰队Collection_04() {
        let path = PathBuf::from("[喵萌奶茶屋] 舰队Collection - 04 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 舰队Collection format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "舰队Collection");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_miaomeng_舰队Collection_05() {
        let path = PathBuf::from("[喵萌奶茶屋] 舰队Collection - 05 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 舰队Collection format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "舰队Collection");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_miaomeng_舰队Collection_06() {
        let path = PathBuf::from("[喵萌奶茶屋] 舰队Collection - 06 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 舰队Collection format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "舰队Collection");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_miaomeng_舰队Collection_07() {
        let path = PathBuf::from("[喵萌奶茶屋] 舰队Collection - 07 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 舰队Collection format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "舰队Collection");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_miaomeng_舰队Collection_08() {
        let path = PathBuf::from("[喵萌奶茶屋] 舰队Collection - 08 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 舰队Collection format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "舰队Collection");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_miaomeng_舰队Collection_09() {
        let path = PathBuf::from("[喵萌奶茶屋] 舰队Collection - 09 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 舰队Collection format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "舰队Collection");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_miaomeng_舰队Collection_10() {
        let path = PathBuf::from("[喵萌奶茶屋] 舰队Collection - 10 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 舰队Collection format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "舰队Collection");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_miaomeng_偶像大师_01() {
        let path = PathBuf::from("[喵萌奶茶屋] 偶像大师 - 01 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 偶像大师 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "偶像大师");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_miaomeng_偶像大师_02() {
        let path = PathBuf::from("[喵萌奶茶屋] 偶像大师 - 02 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 偶像大师 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "偶像大师");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_miaomeng_偶像大师_03() {
        let path = PathBuf::from("[喵萌奶茶屋] 偶像大师 - 03 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 偶像大师 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "偶像大师");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_miaomeng_偶像大师_04() {
        let path = PathBuf::from("[喵萌奶茶屋] 偶像大师 - 04 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 偶像大师 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "偶像大师");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_miaomeng_偶像大师_05() {
        let path = PathBuf::from("[喵萌奶茶屋] 偶像大师 - 05 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 偶像大师 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "偶像大师");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_miaomeng_偶像大师_06() {
        let path = PathBuf::from("[喵萌奶茶屋] 偶像大师 - 06 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 偶像大师 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "偶像大师");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_miaomeng_偶像大师_07() {
        let path = PathBuf::from("[喵萌奶茶屋] 偶像大师 - 07 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 偶像大师 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "偶像大师");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_miaomeng_偶像大师_08() {
        let path = PathBuf::from("[喵萌奶茶屋] 偶像大师 - 08 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 偶像大师 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "偶像大师");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_miaomeng_偶像大师_09() {
        let path = PathBuf::from("[喵萌奶茶屋] 偶像大师 - 09 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 偶像大师 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "偶像大师");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_miaomeng_偶像大师_10() {
        let path = PathBuf::from("[喵萌奶茶屋] 偶像大师 - 10 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 偶像大师 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "偶像大师");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_miaomeng_公主连结_01() {
        let path = PathBuf::from("[喵萌奶茶屋] 公主连结 - 01 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 公主连结 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "公主连结");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_miaomeng_公主连结_02() {
        let path = PathBuf::from("[喵萌奶茶屋] 公主连结 - 02 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 公主连结 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "公主连结");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_miaomeng_公主连结_03() {
        let path = PathBuf::from("[喵萌奶茶屋] 公主连结 - 03 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 公主连结 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "公主连结");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_miaomeng_公主连结_04() {
        let path = PathBuf::from("[喵萌奶茶屋] 公主连结 - 04 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 公主连结 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "公主连结");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_miaomeng_公主连结_05() {
        let path = PathBuf::from("[喵萌奶茶屋] 公主连结 - 05 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 公主连结 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "公主连结");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_miaomeng_公主连结_06() {
        let path = PathBuf::from("[喵萌奶茶屋] 公主连结 - 06 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 公主连结 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "公主连结");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_miaomeng_公主连结_07() {
        let path = PathBuf::from("[喵萌奶茶屋] 公主连结 - 07 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 公主连结 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "公主连结");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_miaomeng_公主连结_08() {
        let path = PathBuf::from("[喵萌奶茶屋] 公主连结 - 08 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 公主连结 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "公主连结");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_miaomeng_公主连结_09() {
        let path = PathBuf::from("[喵萌奶茶屋] 公主连结 - 09 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 公主连结 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "公主连结");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_miaomeng_公主连结_10() {
        let path = PathBuf::from("[喵萌奶茶屋] 公主连结 - 10 [WebRip 1080p].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [喵萌奶茶屋] 公主连结 format");
        assert_eq!(info.publisher, "喵萌奶茶屋");
        assert_eq!(info.anime_name, "公主连结");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_yysub_raw_电锯人_01() {
        let path = PathBuf::from("[YYSUB-RAW] 电锯人 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 电锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "电锯人");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_yysub_raw_电锯人_02() {
        let path = PathBuf::from("[YYSUB-RAW] 电锯人 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 电锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "电锯人");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_yysub_raw_电锯人_03() {
        let path = PathBuf::from("[YYSUB-RAW] 电锯人 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 电锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "电锯人");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_yysub_raw_电锯人_04() {
        let path = PathBuf::from("[YYSUB-RAW] 电锯人 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 电锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "电锯人");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_yysub_raw_电锯人_05() {
        let path = PathBuf::from("[YYSUB-RAW] 电锯人 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 电锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "电锯人");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_yysub_raw_电锯人_06() {
        let path = PathBuf::from("[YYSUB-RAW] 电锯人 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 电锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "电锯人");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_yysub_raw_电锯人_07() {
        let path = PathBuf::from("[YYSUB-RAW] 电锯人 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 电锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "电锯人");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_yysub_raw_电锯人_08() {
        let path = PathBuf::from("[YYSUB-RAW] 电锯人 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 电锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "电锯人");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_yysub_raw_电锯人_09() {
        let path = PathBuf::from("[YYSUB-RAW] 电锯人 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 电锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "电锯人");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_yysub_raw_电锯人_10() {
        let path = PathBuf::from("[YYSUB-RAW] 电锯人 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 电锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "电锯人");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_yysub_raw_链锯人_01() {
        let path = PathBuf::from("[YYSUB-RAW] 链锯人 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 链锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "链锯人");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_yysub_raw_链锯人_02() {
        let path = PathBuf::from("[YYSUB-RAW] 链锯人 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 链锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "链锯人");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_yysub_raw_链锯人_03() {
        let path = PathBuf::from("[YYSUB-RAW] 链锯人 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 链锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "链锯人");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_yysub_raw_链锯人_04() {
        let path = PathBuf::from("[YYSUB-RAW] 链锯人 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 链锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "链锯人");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_yysub_raw_链锯人_05() {
        let path = PathBuf::from("[YYSUB-RAW] 链锯人 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 链锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "链锯人");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_yysub_raw_链锯人_06() {
        let path = PathBuf::from("[YYSUB-RAW] 链锯人 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 链锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "链锯人");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_yysub_raw_链锯人_07() {
        let path = PathBuf::from("[YYSUB-RAW] 链锯人 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 链锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "链锯人");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_yysub_raw_链锯人_08() {
        let path = PathBuf::from("[YYSUB-RAW] 链锯人 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 链锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "链锯人");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_yysub_raw_链锯人_09() {
        let path = PathBuf::from("[YYSUB-RAW] 链锯人 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 链锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "链锯人");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_yysub_raw_链锯人_10() {
        let path = PathBuf::from("[YYSUB-RAW] 链锯人 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 链锯人 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "链锯人");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_yysub_raw_夏日重现_01() {
        let path = PathBuf::from("[YYSUB-RAW] 夏日重现 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 夏日重现 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "夏日重现");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_yysub_raw_夏日重现_02() {
        let path = PathBuf::from("[YYSUB-RAW] 夏日重现 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 夏日重现 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "夏日重现");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_yysub_raw_夏日重现_03() {
        let path = PathBuf::from("[YYSUB-RAW] 夏日重现 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 夏日重现 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "夏日重现");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_yysub_raw_夏日重现_04() {
        let path = PathBuf::from("[YYSUB-RAW] 夏日重现 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 夏日重现 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "夏日重现");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_yysub_raw_夏日重现_05() {
        let path = PathBuf::from("[YYSUB-RAW] 夏日重现 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 夏日重现 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "夏日重现");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_yysub_raw_夏日重现_06() {
        let path = PathBuf::from("[YYSUB-RAW] 夏日重现 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 夏日重现 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "夏日重现");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_yysub_raw_夏日重现_07() {
        let path = PathBuf::from("[YYSUB-RAW] 夏日重现 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 夏日重现 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "夏日重现");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_yysub_raw_夏日重现_08() {
        let path = PathBuf::from("[YYSUB-RAW] 夏日重现 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 夏日重现 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "夏日重现");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_yysub_raw_夏日重现_09() {
        let path = PathBuf::from("[YYSUB-RAW] 夏日重现 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 夏日重现 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "夏日重现");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_yysub_raw_夏日重现_10() {
        let path = PathBuf::from("[YYSUB-RAW] 夏日重现 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 夏日重现 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "夏日重现");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_yysub_raw_莉可丽丝_01() {
        let path = PathBuf::from("[YYSUB-RAW] 莉可丽丝 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 莉可丽丝 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "莉可丽丝");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_yysub_raw_莉可丽丝_02() {
        let path = PathBuf::from("[YYSUB-RAW] 莉可丽丝 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 莉可丽丝 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "莉可丽丝");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_yysub_raw_莉可丽丝_03() {
        let path = PathBuf::from("[YYSUB-RAW] 莉可丽丝 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 莉可丽丝 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "莉可丽丝");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_yysub_raw_莉可丽丝_04() {
        let path = PathBuf::from("[YYSUB-RAW] 莉可丽丝 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 莉可丽丝 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "莉可丽丝");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_yysub_raw_莉可丽丝_05() {
        let path = PathBuf::from("[YYSUB-RAW] 莉可丽丝 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 莉可丽丝 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "莉可丽丝");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_yysub_raw_莉可丽丝_06() {
        let path = PathBuf::from("[YYSUB-RAW] 莉可丽丝 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 莉可丽丝 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "莉可丽丝");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_yysub_raw_莉可丽丝_07() {
        let path = PathBuf::from("[YYSUB-RAW] 莉可丽丝 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 莉可丽丝 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "莉可丽丝");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_yysub_raw_莉可丽丝_08() {
        let path = PathBuf::from("[YYSUB-RAW] 莉可丽丝 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 莉可丽丝 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "莉可丽丝");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_yysub_raw_莉可丽丝_09() {
        let path = PathBuf::from("[YYSUB-RAW] 莉可丽丝 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 莉可丽丝 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "莉可丽丝");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_yysub_raw_莉可丽丝_10() {
        let path = PathBuf::from("[YYSUB-RAW] 莉可丽丝 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [YYSUB-RAW] 莉可丽丝 format");
        assert_eq!(info.publisher, "YYSUB-RAW");
        assert_eq!(info.anime_name, "莉可丽丝");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_chenxi_物语系列_01() {
        let path = PathBuf::from("[晨曦制作] 物语系列 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 物语系列 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "物语系列");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_chenxi_物语系列_02() {
        let path = PathBuf::from("[晨曦制作] 物语系列 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 物语系列 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "物语系列");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_chenxi_物语系列_03() {
        let path = PathBuf::from("[晨曦制作] 物语系列 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 物语系列 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "物语系列");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_chenxi_物语系列_04() {
        let path = PathBuf::from("[晨曦制作] 物语系列 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 物语系列 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "物语系列");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_chenxi_物语系列_05() {
        let path = PathBuf::from("[晨曦制作] 物语系列 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 物语系列 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "物语系列");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_chenxi_物语系列_06() {
        let path = PathBuf::from("[晨曦制作] 物语系列 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 物语系列 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "物语系列");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_chenxi_物语系列_07() {
        let path = PathBuf::from("[晨曦制作] 物语系列 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 物语系列 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "物语系列");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_chenxi_物语系列_08() {
        let path = PathBuf::from("[晨曦制作] 物语系列 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 物语系列 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "物语系列");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_chenxi_物语系列_09() {
        let path = PathBuf::from("[晨曦制作] 物语系列 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 物语系列 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "物语系列");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_chenxi_物语系列_10() {
        let path = PathBuf::from("[晨曦制作] 物语系列 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 物语系列 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "物语系列");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_chenxi_不死者的丧尸_01() {
        let path = PathBuf::from("[晨曦制作] 不死者的丧尸 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 不死者的丧尸 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "不死者的丧尸");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_chenxi_不死者的丧尸_02() {
        let path = PathBuf::from("[晨曦制作] 不死者的丧尸 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 不死者的丧尸 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "不死者的丧尸");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_chenxi_不死者的丧尸_03() {
        let path = PathBuf::from("[晨曦制作] 不死者的丧尸 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 不死者的丧尸 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "不死者的丧尸");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_chenxi_不死者的丧尸_04() {
        let path = PathBuf::from("[晨曦制作] 不死者的丧尸 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 不死者的丧尸 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "不死者的丧尸");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_chenxi_不死者的丧尸_05() {
        let path = PathBuf::from("[晨曦制作] 不死者的丧尸 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 不死者的丧尸 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "不死者的丧尸");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_chenxi_不死者的丧尸_06() {
        let path = PathBuf::from("[晨曦制作] 不死者的丧尸 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 不死者的丧尸 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "不死者的丧尸");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_chenxi_不死者的丧尸_07() {
        let path = PathBuf::from("[晨曦制作] 不死者的丧尸 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 不死者的丧尸 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "不死者的丧尸");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_chenxi_不死者的丧尸_08() {
        let path = PathBuf::from("[晨曦制作] 不死者的丧尸 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 不死者的丧尸 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "不死者的丧尸");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_chenxi_不死者的丧尸_09() {
        let path = PathBuf::from("[晨曦制作] 不死者的丧尸 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 不死者的丧尸 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "不死者的丧尸");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_chenxi_不死者的丧尸_10() {
        let path = PathBuf::from("[晨曦制作] 不死者的丧尸 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 不死者的丧尸 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "不死者的丧尸");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_chenxi_异世界舅舅_01() {
        let path = PathBuf::from("[晨曦制作] 异世界舅舅 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 异世界舅舅 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "异世界舅舅");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_chenxi_异世界舅舅_02() {
        let path = PathBuf::from("[晨曦制作] 异世界舅舅 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 异世界舅舅 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "异世界舅舅");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_chenxi_异世界舅舅_03() {
        let path = PathBuf::from("[晨曦制作] 异世界舅舅 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 异世界舅舅 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "异世界舅舅");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_chenxi_异世界舅舅_04() {
        let path = PathBuf::from("[晨曦制作] 异世界舅舅 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 异世界舅舅 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "异世界舅舅");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_chenxi_异世界舅舅_05() {
        let path = PathBuf::from("[晨曦制作] 异世界舅舅 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 异世界舅舅 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "异世界舅舅");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_chenxi_异世界舅舅_06() {
        let path = PathBuf::from("[晨曦制作] 异世界舅舅 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 异世界舅舅 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "异世界舅舅");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_chenxi_异世界舅舅_07() {
        let path = PathBuf::from("[晨曦制作] 异世界舅舅 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 异世界舅舅 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "异世界舅舅");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_chenxi_异世界舅舅_08() {
        let path = PathBuf::from("[晨曦制作] 异世界舅舅 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 异世界舅舅 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "异世界舅舅");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_chenxi_异世界舅舅_09() {
        let path = PathBuf::from("[晨曦制作] 异世界舅舅 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 异世界舅舅 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "异世界舅舅");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_chenxi_异世界舅舅_10() {
        let path = PathBuf::from("[晨曦制作] 异世界舅舅 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 异世界舅舅 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "异世界舅舅");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_chenxi_间谍过家家_01() {
        let path = PathBuf::from("[晨曦制作] 间谍过家家 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 间谍过家家 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_chenxi_间谍过家家_02() {
        let path = PathBuf::from("[晨曦制作] 间谍过家家 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 间谍过家家 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_chenxi_间谍过家家_03() {
        let path = PathBuf::from("[晨曦制作] 间谍过家家 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 间谍过家家 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_chenxi_间谍过家家_04() {
        let path = PathBuf::from("[晨曦制作] 间谍过家家 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 间谍过家家 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_chenxi_间谍过家家_05() {
        let path = PathBuf::from("[晨曦制作] 间谍过家家 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 间谍过家家 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_chenxi_间谍过家家_06() {
        let path = PathBuf::from("[晨曦制作] 间谍过家家 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 间谍过家家 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_chenxi_间谍过家家_07() {
        let path = PathBuf::from("[晨曦制作] 间谍过家家 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 间谍过家家 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_chenxi_间谍过家家_08() {
        let path = PathBuf::from("[晨曦制作] 间谍过家家 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 间谍过家家 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_chenxi_间谍过家家_09() {
        let path = PathBuf::from("[晨曦制作] 间谍过家家 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 间谍过家家 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_chenxi_间谍过家家_10() {
        let path = PathBuf::from("[晨曦制作] 间谍过家家 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晨曦制作] 间谍过家家 format");
        assert_eq!(info.publisher, "晨曦制作");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_wanjie_契约之吻_01() {
        let path = PathBuf::from("[晚街与灯] 契约之吻 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 契约之吻 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "契约之吻");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_wanjie_契约之吻_02() {
        let path = PathBuf::from("[晚街与灯] 契约之吻 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 契约之吻 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "契约之吻");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_wanjie_契约之吻_03() {
        let path = PathBuf::from("[晚街与灯] 契约之吻 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 契约之吻 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "契约之吻");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_wanjie_契约之吻_04() {
        let path = PathBuf::from("[晚街与灯] 契约之吻 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 契约之吻 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "契约之吻");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_wanjie_契约之吻_05() {
        let path = PathBuf::from("[晚街与灯] 契约之吻 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 契约之吻 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "契约之吻");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_wanjie_契约之吻_06() {
        let path = PathBuf::from("[晚街与灯] 契约之吻 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 契约之吻 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "契约之吻");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_wanjie_契约之吻_07() {
        let path = PathBuf::from("[晚街与灯] 契约之吻 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 契约之吻 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "契约之吻");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_wanjie_契约之吻_08() {
        let path = PathBuf::from("[晚街与灯] 契约之吻 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 契约之吻 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "契约之吻");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_wanjie_契约之吻_09() {
        let path = PathBuf::from("[晚街与灯] 契约之吻 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 契约之吻 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "契约之吻");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_wanjie_契约之吻_10() {
        let path = PathBuf::from("[晚街与灯] 契约之吻 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 契约之吻 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "契约之吻");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_wanjie_Lycoris_Recoil_01() {
        let path = PathBuf::from("[晚街与灯] Lycoris Recoil - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] Lycoris Recoil format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "Lycoris Recoil");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_wanjie_Lycoris_Recoil_02() {
        let path = PathBuf::from("[晚街与灯] Lycoris Recoil - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] Lycoris Recoil format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "Lycoris Recoil");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_wanjie_Lycoris_Recoil_03() {
        let path = PathBuf::from("[晚街与灯] Lycoris Recoil - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] Lycoris Recoil format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "Lycoris Recoil");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_wanjie_Lycoris_Recoil_04() {
        let path = PathBuf::from("[晚街与灯] Lycoris Recoil - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] Lycoris Recoil format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "Lycoris Recoil");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_wanjie_Lycoris_Recoil_05() {
        let path = PathBuf::from("[晚街与灯] Lycoris Recoil - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] Lycoris Recoil format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "Lycoris Recoil");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_wanjie_Lycoris_Recoil_06() {
        let path = PathBuf::from("[晚街与灯] Lycoris Recoil - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] Lycoris Recoil format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "Lycoris Recoil");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_wanjie_Lycoris_Recoil_07() {
        let path = PathBuf::from("[晚街与灯] Lycoris Recoil - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] Lycoris Recoil format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "Lycoris Recoil");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_wanjie_Lycoris_Recoil_08() {
        let path = PathBuf::from("[晚街与灯] Lycoris Recoil - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] Lycoris Recoil format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "Lycoris Recoil");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_wanjie_Lycoris_Recoil_09() {
        let path = PathBuf::from("[晚街与灯] Lycoris Recoil - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] Lycoris Recoil format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "Lycoris Recoil");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_wanjie_Lycoris_Recoil_10() {
        let path = PathBuf::from("[晚街与灯] Lycoris Recoil - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] Lycoris Recoil format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "Lycoris Recoil");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_wanjie_白圣女与黑牧师_01() {
        let path = PathBuf::from("[晚街与灯] 白圣女与黑牧师 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 白圣女与黑牧师 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "白圣女与黑牧师");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_wanjie_白圣女与黑牧师_02() {
        let path = PathBuf::from("[晚街与灯] 白圣女与黑牧师 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 白圣女与黑牧师 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "白圣女与黑牧师");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_wanjie_白圣女与黑牧师_03() {
        let path = PathBuf::from("[晚街与灯] 白圣女与黑牧师 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 白圣女与黑牧师 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "白圣女与黑牧师");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_wanjie_白圣女与黑牧师_04() {
        let path = PathBuf::from("[晚街与灯] 白圣女与黑牧师 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 白圣女与黑牧师 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "白圣女与黑牧师");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_wanjie_白圣女与黑牧师_05() {
        let path = PathBuf::from("[晚街与灯] 白圣女与黑牧师 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 白圣女与黑牧师 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "白圣女与黑牧师");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_wanjie_白圣女与黑牧师_06() {
        let path = PathBuf::from("[晚街与灯] 白圣女与黑牧师 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 白圣女与黑牧师 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "白圣女与黑牧师");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_wanjie_白圣女与黑牧师_07() {
        let path = PathBuf::from("[晚街与灯] 白圣女与黑牧师 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 白圣女与黑牧师 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "白圣女与黑牧师");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_wanjie_白圣女与黑牧师_08() {
        let path = PathBuf::from("[晚街与灯] 白圣女与黑牧师 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 白圣女与黑牧师 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "白圣女与黑牧师");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_wanjie_白圣女与黑牧师_09() {
        let path = PathBuf::from("[晚街与灯] 白圣女与黑牧师 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 白圣女与黑牧师 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "白圣女与黑牧师");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_wanjie_白圣女与黑牧师_10() {
        let path = PathBuf::from("[晚街与灯] 白圣女与黑牧师 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 白圣女与黑牧师 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "白圣女与黑牧师");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_wanjie_别当欧尼酱了_01() {
        let path = PathBuf::from("[晚街与灯] 别当欧尼酱了 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 别当欧尼酱了 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "别当欧尼酱了");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_wanjie_别当欧尼酱了_02() {
        let path = PathBuf::from("[晚街与灯] 别当欧尼酱了 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 别当欧尼酱了 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "别当欧尼酱了");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_wanjie_别当欧尼酱了_03() {
        let path = PathBuf::from("[晚街与灯] 别当欧尼酱了 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 别当欧尼酱了 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "别当欧尼酱了");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_wanjie_别当欧尼酱了_04() {
        let path = PathBuf::from("[晚街与灯] 别当欧尼酱了 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 别当欧尼酱了 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "别当欧尼酱了");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_wanjie_别当欧尼酱了_05() {
        let path = PathBuf::from("[晚街与灯] 别当欧尼酱了 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 别当欧尼酱了 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "别当欧尼酱了");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_wanjie_别当欧尼酱了_06() {
        let path = PathBuf::from("[晚街与灯] 别当欧尼酱了 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 别当欧尼酱了 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "别当欧尼酱了");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_wanjie_别当欧尼酱了_07() {
        let path = PathBuf::from("[晚街与灯] 别当欧尼酱了 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 别当欧尼酱了 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "别当欧尼酱了");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_wanjie_别当欧尼酱了_08() {
        let path = PathBuf::from("[晚街与灯] 别当欧尼酱了 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 别当欧尼酱了 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "别当欧尼酱了");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_wanjie_别当欧尼酱了_09() {
        let path = PathBuf::from("[晚街与灯] 别当欧尼酱了 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 别当欧尼酱了 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "别当欧尼酱了");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_wanjie_别当欧尼酱了_10() {
        let path = PathBuf::from("[晚街与灯] 别当欧尼酱了 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [晚街与灯] 别当欧尼酱了 format");
        assert_eq!(info.publisher, "晚街与灯");
        assert_eq!(info.anime_name, "别当欧尼酱了");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_zhishi_Kemono_Friends_01() {
        let path = PathBuf::from("[芝士动物朋友] Kemono Friends - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] Kemono Friends format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "Kemono Friends");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_zhishi_Kemono_Friends_02() {
        let path = PathBuf::from("[芝士动物朋友] Kemono Friends - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] Kemono Friends format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "Kemono Friends");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_zhishi_Kemono_Friends_03() {
        let path = PathBuf::from("[芝士动物朋友] Kemono Friends - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] Kemono Friends format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "Kemono Friends");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_zhishi_Kemono_Friends_04() {
        let path = PathBuf::from("[芝士动物朋友] Kemono Friends - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] Kemono Friends format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "Kemono Friends");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_zhishi_Kemono_Friends_05() {
        let path = PathBuf::from("[芝士动物朋友] Kemono Friends - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] Kemono Friends format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "Kemono Friends");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_zhishi_Kemono_Friends_06() {
        let path = PathBuf::from("[芝士动物朋友] Kemono Friends - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] Kemono Friends format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "Kemono Friends");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_zhishi_Kemono_Friends_07() {
        let path = PathBuf::from("[芝士动物朋友] Kemono Friends - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] Kemono Friends format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "Kemono Friends");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_zhishi_Kemono_Friends_08() {
        let path = PathBuf::from("[芝士动物朋友] Kemono Friends - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] Kemono Friends format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "Kemono Friends");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_zhishi_Kemono_Friends_09() {
        let path = PathBuf::from("[芝士动物朋友] Kemono Friends - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] Kemono Friends format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "Kemono Friends");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_zhishi_Kemono_Friends_10() {
        let path = PathBuf::from("[芝士动物朋友] Kemono Friends - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] Kemono Friends format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "Kemono Friends");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_zhishi_动物朋友_01() {
        let path = PathBuf::from("[芝士动物朋友] 动物朋友 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 动物朋友 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "动物朋友");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_zhishi_动物朋友_02() {
        let path = PathBuf::from("[芝士动物朋友] 动物朋友 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 动物朋友 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "动物朋友");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_zhishi_动物朋友_03() {
        let path = PathBuf::from("[芝士动物朋友] 动物朋友 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 动物朋友 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "动物朋友");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_zhishi_动物朋友_04() {
        let path = PathBuf::from("[芝士动物朋友] 动物朋友 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 动物朋友 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "动物朋友");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_zhishi_动物朋友_05() {
        let path = PathBuf::from("[芝士动物朋友] 动物朋友 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 动物朋友 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "动物朋友");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_zhishi_动物朋友_06() {
        let path = PathBuf::from("[芝士动物朋友] 动物朋友 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 动物朋友 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "动物朋友");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_zhishi_动物朋友_07() {
        let path = PathBuf::from("[芝士动物朋友] 动物朋友 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 动物朋友 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "动物朋友");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_zhishi_动物朋友_08() {
        let path = PathBuf::from("[芝士动物朋友] 动物朋友 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 动物朋友 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "动物朋友");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_zhishi_动物朋友_09() {
        let path = PathBuf::from("[芝士动物朋友] 动物朋友 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 动物朋友 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "动物朋友");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_zhishi_动物朋友_10() {
        let path = PathBuf::from("[芝士动物朋友] 动物朋友 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 动物朋友 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "动物朋友");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_zhishi_小马宝莉_01() {
        let path = PathBuf::from("[芝士动物朋友] 小马宝莉 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 小马宝莉 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "小马宝莉");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_zhishi_小马宝莉_02() {
        let path = PathBuf::from("[芝士动物朋友] 小马宝莉 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 小马宝莉 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "小马宝莉");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_zhishi_小马宝莉_03() {
        let path = PathBuf::from("[芝士动物朋友] 小马宝莉 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 小马宝莉 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "小马宝莉");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_zhishi_小马宝莉_04() {
        let path = PathBuf::from("[芝士动物朋友] 小马宝莉 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 小马宝莉 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "小马宝莉");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_zhishi_小马宝莉_05() {
        let path = PathBuf::from("[芝士动物朋友] 小马宝莉 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 小马宝莉 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "小马宝莉");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_zhishi_小马宝莉_06() {
        let path = PathBuf::from("[芝士动物朋友] 小马宝莉 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 小马宝莉 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "小马宝莉");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_zhishi_小马宝莉_07() {
        let path = PathBuf::from("[芝士动物朋友] 小马宝莉 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 小马宝莉 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "小马宝莉");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_zhishi_小马宝莉_08() {
        let path = PathBuf::from("[芝士动物朋友] 小马宝莉 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 小马宝莉 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "小马宝莉");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_zhishi_小马宝莉_09() {
        let path = PathBuf::from("[芝士动物朋友] 小马宝莉 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 小马宝莉 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "小马宝莉");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_zhishi_小马宝莉_10() {
        let path = PathBuf::from("[芝士动物朋友] 小马宝莉 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 小马宝莉 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "小马宝莉");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_zhishi_摇曳露营_01() {
        let path = PathBuf::from("[芝士动物朋友] 摇曳露营 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 摇曳露营 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "摇曳露营");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_zhishi_摇曳露营_02() {
        let path = PathBuf::from("[芝士动物朋友] 摇曳露营 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 摇曳露营 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "摇曳露营");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_zhishi_摇曳露营_03() {
        let path = PathBuf::from("[芝士动物朋友] 摇曳露营 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 摇曳露营 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "摇曳露营");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_zhishi_摇曳露营_04() {
        let path = PathBuf::from("[芝士动物朋友] 摇曳露营 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 摇曳露营 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "摇曳露营");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_zhishi_摇曳露营_05() {
        let path = PathBuf::from("[芝士动物朋友] 摇曳露营 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 摇曳露营 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "摇曳露营");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_zhishi_摇曳露营_06() {
        let path = PathBuf::from("[芝士动物朋友] 摇曳露营 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 摇曳露营 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "摇曳露营");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_zhishi_摇曳露营_07() {
        let path = PathBuf::from("[芝士动物朋友] 摇曳露营 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 摇曳露营 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "摇曳露营");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_zhishi_摇曳露营_08() {
        let path = PathBuf::from("[芝士动物朋友] 摇曳露营 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 摇曳露营 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "摇曳露营");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_zhishi_摇曳露营_09() {
        let path = PathBuf::from("[芝士动物朋友] 摇曳露营 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 摇曳露营 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "摇曳露营");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_zhishi_摇曳露营_10() {
        let path = PathBuf::from("[芝士动物朋友] 摇曳露营 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [芝士动物朋友] 摇曳露营 format");
        assert_eq!(info.publisher, "芝士动物朋友");
        assert_eq!(info.anime_name, "摇曳露营");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_s1_snow_偶像活动_01() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] 偶像活动 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] 偶像活动 format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "偶像活动");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_s1_snow_偶像活动_02() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] 偶像活动 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] 偶像活动 format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "偶像活动");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_s1_snow_偶像活动_03() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] 偶像活动 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] 偶像活动 format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "偶像活动");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_s1_snow_偶像活动_04() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] 偶像活动 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] 偶像活动 format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "偶像活动");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_s1_snow_偶像活动_05() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] 偶像活动 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] 偶像活动 format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "偶像活动");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_s1_snow_偶像活动_06() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] 偶像活动 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] 偶像活动 format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "偶像活动");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_s1_snow_偶像活动_07() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] 偶像活动 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] 偶像活动 format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "偶像活动");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_s1_snow_偶像活动_08() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] 偶像活动 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] 偶像活动 format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "偶像活动");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_s1_snow_偶像活动_09() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] 偶像活动 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] 偶像活动 format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "偶像活动");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_s1_snow_偶像活动_10() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] 偶像活动 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] 偶像活动 format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "偶像活动");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_s1_snow_Deresute_01() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] Deresute - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] Deresute format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "Deresute");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_s1_snow_Deresute_02() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] Deresute - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] Deresute format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "Deresute");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_s1_snow_Deresute_03() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] Deresute - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] Deresute format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "Deresute");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_s1_snow_Deresute_04() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] Deresute - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] Deresute format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "Deresute");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_s1_snow_Deresute_05() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] Deresute - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] Deresute format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "Deresute");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_s1_snow_Deresute_06() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] Deresute - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] Deresute format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "Deresute");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_s1_snow_Deresute_07() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] Deresute - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] Deresute format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "Deresute");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_s1_snow_Deresute_08() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] Deresute - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] Deresute format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "Deresute");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_s1_snow_Deresute_09() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] Deresute - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] Deresute format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "Deresute");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_s1_snow_Deresute_10() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] Deresute - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] Deresute format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "Deresute");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_s1_snow_BanG_Dream_01() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] BanG Dream - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] BanG Dream format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "BanG Dream");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_s1_snow_BanG_Dream_02() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] BanG Dream - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] BanG Dream format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "BanG Dream");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_s1_snow_BanG_Dream_03() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] BanG Dream - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] BanG Dream format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "BanG Dream");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_s1_snow_BanG_Dream_04() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] BanG Dream - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] BanG Dream format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "BanG Dream");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_s1_snow_BanG_Dream_05() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] BanG Dream - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] BanG Dream format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "BanG Dream");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_s1_snow_BanG_Dream_06() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] BanG Dream - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] BanG Dream format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "BanG Dream");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_s1_snow_BanG_Dream_07() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] BanG Dream - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] BanG Dream format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "BanG Dream");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_s1_snow_BanG_Dream_08() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] BanG Dream - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] BanG Dream format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "BanG Dream");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_s1_snow_BanG_Dream_09() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] BanG Dream - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] BanG Dream format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "BanG Dream");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_s1_snow_BanG_Dream_10() {
        let path = PathBuf::from("[S1的大朋友们 x 雪飘工作室] BanG Dream - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [S1的大朋友们 x 雪飘工作室] BanG Dream format");
        assert_eq!(info.publisher, "S1的大朋友们 x 雪飘工作室");
        assert_eq!(info.anime_name, "BanG Dream");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_yyq_紫罗兰永恒花园_01() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 紫罗兰永恒花园 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_yyq_紫罗兰永恒花园_02() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 紫罗兰永恒花园 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_yyq_紫罗兰永恒花园_03() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 紫罗兰永恒花园 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_yyq_紫罗兰永恒花园_04() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 紫罗兰永恒花园 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_yyq_紫罗兰永恒花园_05() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 紫罗兰永恒花园 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_yyq_紫罗兰永恒花园_06() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 紫罗兰永恒花园 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_yyq_紫罗兰永恒花园_07() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 紫罗兰永恒花园 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_yyq_紫罗兰永恒花园_08() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 紫罗兰永恒花园 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_yyq_紫罗兰永恒花园_09() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 紫罗兰永恒花园 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_yyq_紫罗兰永恒花园_10() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 紫罗兰永恒花园 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_yyq_吹响吧上低音号_01() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 吹响吧上低音号 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_yyq_吹响吧上低音号_02() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 吹响吧上低音号 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_yyq_吹响吧上低音号_03() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 吹响吧上低音号 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_yyq_吹响吧上低音号_04() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 吹响吧上低音号 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_yyq_吹响吧上低音号_05() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 吹响吧上低音号 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_yyq_吹响吧上低音号_06() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 吹响吧上低音号 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_yyq_吹响吧上低音号_07() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 吹响吧上低音号 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_yyq_吹响吧上低音号_08() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 吹响吧上低音号 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_yyq_吹响吧上低音号_09() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 吹响吧上低音号 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_yyq_吹响吧上低音号_10() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 吹响吧上低音号 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_yyq_我的青春恋爱物语_01() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 我的青春恋爱物语 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [夜莺家族&YYQ字幕组] 我的青春恋爱物语 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_yyq_我的青春恋爱物语_02() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 我的青春恋爱物语 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [夜莺家族&YYQ字幕组] 我的青春恋爱物语 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_yyq_我的青春恋爱物语_03() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 我的青春恋爱物语 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [夜莺家族&YYQ字幕组] 我的青春恋爱物语 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_yyq_我的青春恋爱物语_04() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 我的青春恋爱物语 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [夜莺家族&YYQ字幕组] 我的青春恋爱物语 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_yyq_我的青春恋爱物语_05() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 我的青春恋爱物语 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [夜莺家族&YYQ字幕组] 我的青春恋爱物语 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_yyq_我的青春恋爱物语_06() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 我的青春恋爱物语 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [夜莺家族&YYQ字幕组] 我的青春恋爱物语 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_yyq_我的青春恋爱物语_07() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 我的青春恋爱物语 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [夜莺家族&YYQ字幕组] 我的青春恋爱物语 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_yyq_我的青春恋爱物语_08() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 我的青春恋爱物语 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [夜莺家族&YYQ字幕组] 我的青春恋爱物语 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_yyq_我的青春恋爱物语_09() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 我的青春恋爱物语 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [夜莺家族&YYQ字幕组] 我的青春恋爱物语 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_yyq_我的青春恋爱物语_10() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 我的青春恋爱物语 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info =
            result.expect("Parser should handle [夜莺家族&YYQ字幕组] 我的青春恋爱物语 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_sakura_间谍过家家_01() {
        let path = PathBuf::from("[桜都字幕组] 间谍过家家 - 01 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 间谍过家家 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_sakura_间谍过家家_02() {
        let path = PathBuf::from("[桜都字幕组] 间谍过家家 - 02 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 间谍过家家 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_sakura_间谍过家家_03() {
        let path = PathBuf::from("[桜都字幕组] 间谍过家家 - 03 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 间谍过家家 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_sakura_间谍过家家_04() {
        let path = PathBuf::from("[桜都字幕组] 间谍过家家 - 04 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 间谍过家家 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_sakura_间谍过家家_05() {
        let path = PathBuf::from("[桜都字幕组] 间谍过家家 - 05 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 间谍过家家 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_sakura_间谍过家家_06() {
        let path = PathBuf::from("[桜都字幕组] 间谍过家家 - 06 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 间谍过家家 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_sakura_间谍过家家_07() {
        let path = PathBuf::from("[桜都字幕组] 间谍过家家 - 07 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 间谍过家家 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_sakura_间谍过家家_08() {
        let path = PathBuf::from("[桜都字幕组] 间谍过家家 - 08 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 间谍过家家 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_sakura_间谍过家家_09() {
        let path = PathBuf::from("[桜都字幕组] 间谍过家家 - 09 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 间谍过家家 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_sakura_间谍过家家_10() {
        let path = PathBuf::from("[桜都字幕组] 间谍过家家 - 10 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 间谍过家家 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "间谍过家家");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_sakura_葬送的芙莉莲_01() {
        let path = PathBuf::from("[桜都字幕组] 葬送的芙莉莲 - 01 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_sakura_葬送的芙莉莲_02() {
        let path = PathBuf::from("[桜都字幕组] 葬送的芙莉莲 - 02 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_sakura_葬送的芙莉莲_03() {
        let path = PathBuf::from("[桜都字幕组] 葬送的芙莉莲 - 03 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_sakura_葬送的芙莉莲_04() {
        let path = PathBuf::from("[桜都字幕组] 葬送的芙莉莲 - 04 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_sakura_葬送的芙莉莲_05() {
        let path = PathBuf::from("[桜都字幕组] 葬送的芙莉莲 - 05 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_sakura_葬送的芙莉莲_06() {
        let path = PathBuf::from("[桜都字幕组] 葬送的芙莉莲 - 06 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_sakura_葬送的芙莉莲_07() {
        let path = PathBuf::from("[桜都字幕组] 葬送的芙莉莲 - 07 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_sakura_葬送的芙莉莲_08() {
        let path = PathBuf::from("[桜都字幕组] 葬送的芙莉莲 - 08 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_sakura_葬送的芙莉莲_09() {
        let path = PathBuf::from("[桜都字幕组] 葬送的芙莉莲 - 09 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_sakura_葬送的芙莉莲_10() {
        let path = PathBuf::from("[桜都字幕组] 葬送的芙莉莲 - 10 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_sakura_迷宫饭_01() {
        let path = PathBuf::from("[桜都字幕组] 迷宫饭 - 01 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 迷宫饭 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_sakura_迷宫饭_02() {
        let path = PathBuf::from("[桜都字幕组] 迷宫饭 - 02 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 迷宫饭 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_sakura_迷宫饭_03() {
        let path = PathBuf::from("[桜都字幕组] 迷宫饭 - 03 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 迷宫饭 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_sakura_迷宫饭_04() {
        let path = PathBuf::from("[桜都字幕组] 迷宫饭 - 04 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 迷宫饭 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_sakura_迷宫饭_05() {
        let path = PathBuf::from("[桜都字幕组] 迷宫饭 - 05 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 迷宫饭 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_sakura_迷宫饭_06() {
        let path = PathBuf::from("[桜都字幕组] 迷宫饭 - 06 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 迷宫饭 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_sakura_迷宫饭_07() {
        let path = PathBuf::from("[桜都字幕组] 迷宫饭 - 07 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 迷宫饭 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_sakura_迷宫饭_08() {
        let path = PathBuf::from("[桜都字幕组] 迷宫饭 - 08 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 迷宫饭 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_sakura_迷宫饭_09() {
        let path = PathBuf::from("[桜都字幕组] 迷宫饭 - 09 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 迷宫饭 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_sakura_迷宫饭_10() {
        let path = PathBuf::from("[桜都字幕组] 迷宫饭 - 10 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 迷宫饭 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "迷宫饭");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_sakura_蓝色监狱_01() {
        let path = PathBuf::from("[桜都字幕组] 蓝色监狱 - 01 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 蓝色监狱 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "蓝色监狱");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_sakura_蓝色监狱_02() {
        let path = PathBuf::from("[桜都字幕组] 蓝色监狱 - 02 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 蓝色监狱 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "蓝色监狱");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_sakura_蓝色监狱_03() {
        let path = PathBuf::from("[桜都字幕组] 蓝色监狱 - 03 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 蓝色监狱 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "蓝色监狱");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_sakura_蓝色监狱_04() {
        let path = PathBuf::from("[桜都字幕组] 蓝色监狱 - 04 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 蓝色监狱 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "蓝色监狱");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_sakura_蓝色监狱_05() {
        let path = PathBuf::from("[桜都字幕组] 蓝色监狱 - 05 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 蓝色监狱 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "蓝色监狱");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_sakura_蓝色监狱_06() {
        let path = PathBuf::from("[桜都字幕组] 蓝色监狱 - 06 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 蓝色监狱 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "蓝色监狱");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_sakura_蓝色监狱_07() {
        let path = PathBuf::from("[桜都字幕组] 蓝色监狱 - 07 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 蓝色监狱 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "蓝色监狱");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_sakura_蓝色监狱_08() {
        let path = PathBuf::from("[桜都字幕组] 蓝色监狱 - 08 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 蓝色监狱 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "蓝色监狱");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_sakura_蓝色监狱_09() {
        let path = PathBuf::from("[桜都字幕组] 蓝色监狱 - 09 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 蓝色监狱 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "蓝色监狱");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_sakura_蓝色监狱_10() {
        let path = PathBuf::from("[桜都字幕组] 蓝色监狱 - 10 [1080p][简繁内封].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [桜都字幕组] 蓝色监狱 format");
        assert_eq!(info.publisher, "桜都字幕组");
        assert_eq!(info.anime_name, "蓝色监狱");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_sweet_不起眼的魔王_01() {
        let path = PathBuf::from("[SweetSub] 不起眼的魔王 - 01 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 不起眼的魔王 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "不起眼的魔王");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_sweet_不起眼的魔王_02() {
        let path = PathBuf::from("[SweetSub] 不起眼的魔王 - 02 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 不起眼的魔王 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "不起眼的魔王");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_sweet_不起眼的魔王_03() {
        let path = PathBuf::from("[SweetSub] 不起眼的魔王 - 03 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 不起眼的魔王 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "不起眼的魔王");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_sweet_不起眼的魔王_04() {
        let path = PathBuf::from("[SweetSub] 不起眼的魔王 - 04 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 不起眼的魔王 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "不起眼的魔王");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_sweet_不起眼的魔王_05() {
        let path = PathBuf::from("[SweetSub] 不起眼的魔王 - 05 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 不起眼的魔王 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "不起眼的魔王");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_sweet_不起眼的魔王_06() {
        let path = PathBuf::from("[SweetSub] 不起眼的魔王 - 06 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 不起眼的魔王 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "不起眼的魔王");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_sweet_不起眼的魔王_07() {
        let path = PathBuf::from("[SweetSub] 不起眼的魔王 - 07 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 不起眼的魔王 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "不起眼的魔王");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_sweet_不起眼的魔王_08() {
        let path = PathBuf::from("[SweetSub] 不起眼的魔王 - 08 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 不起眼的魔王 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "不起眼的魔王");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_sweet_不起眼的魔王_09() {
        let path = PathBuf::from("[SweetSub] 不起眼的魔王 - 09 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 不起眼的魔王 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "不起眼的魔王");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_sweet_不起眼的魔王_10() {
        let path = PathBuf::from("[SweetSub] 不起眼的魔王 - 10 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 不起眼的魔王 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "不起眼的魔王");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_sweet_打工吧魔王大人_01() {
        let path = PathBuf::from("[SweetSub] 打工吧魔王大人 - 01 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 打工吧魔王大人 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "打工吧魔王大人");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_sweet_打工吧魔王大人_02() {
        let path = PathBuf::from("[SweetSub] 打工吧魔王大人 - 02 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 打工吧魔王大人 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "打工吧魔王大人");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_sweet_打工吧魔王大人_03() {
        let path = PathBuf::from("[SweetSub] 打工吧魔王大人 - 03 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 打工吧魔王大人 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "打工吧魔王大人");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_sweet_打工吧魔王大人_04() {
        let path = PathBuf::from("[SweetSub] 打工吧魔王大人 - 04 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 打工吧魔王大人 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "打工吧魔王大人");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_sweet_打工吧魔王大人_05() {
        let path = PathBuf::from("[SweetSub] 打工吧魔王大人 - 05 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 打工吧魔王大人 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "打工吧魔王大人");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_sweet_打工吧魔王大人_06() {
        let path = PathBuf::from("[SweetSub] 打工吧魔王大人 - 06 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 打工吧魔王大人 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "打工吧魔王大人");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_sweet_打工吧魔王大人_07() {
        let path = PathBuf::from("[SweetSub] 打工吧魔王大人 - 07 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 打工吧魔王大人 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "打工吧魔王大人");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_sweet_打工吧魔王大人_08() {
        let path = PathBuf::from("[SweetSub] 打工吧魔王大人 - 08 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 打工吧魔王大人 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "打工吧魔王大人");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_sweet_打工吧魔王大人_09() {
        let path = PathBuf::from("[SweetSub] 打工吧魔王大人 - 09 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 打工吧魔王大人 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "打工吧魔王大人");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_sweet_打工吧魔王大人_10() {
        let path = PathBuf::from("[SweetSub] 打工吧魔王大人 - 10 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 打工吧魔王大人 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "打工吧魔王大人");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_sweet_为美好的世界献上祝福_01() {
        let path = PathBuf::from("[SweetSub] 为美好的世界献上祝福 - 01 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 为美好的世界献上祝福 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "为美好的世界献上祝福");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_sweet_为美好的世界献上祝福_02() {
        let path = PathBuf::from("[SweetSub] 为美好的世界献上祝福 - 02 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 为美好的世界献上祝福 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "为美好的世界献上祝福");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_sweet_为美好的世界献上祝福_03() {
        let path = PathBuf::from("[SweetSub] 为美好的世界献上祝福 - 03 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 为美好的世界献上祝福 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "为美好的世界献上祝福");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_sweet_为美好的世界献上祝福_04() {
        let path = PathBuf::from("[SweetSub] 为美好的世界献上祝福 - 04 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 为美好的世界献上祝福 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "为美好的世界献上祝福");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_sweet_为美好的世界献上祝福_05() {
        let path = PathBuf::from("[SweetSub] 为美好的世界献上祝福 - 05 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 为美好的世界献上祝福 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "为美好的世界献上祝福");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_sweet_为美好的世界献上祝福_06() {
        let path = PathBuf::from("[SweetSub] 为美好的世界献上祝福 - 06 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 为美好的世界献上祝福 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "为美好的世界献上祝福");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_sweet_为美好的世界献上祝福_07() {
        let path = PathBuf::from("[SweetSub] 为美好的世界献上祝福 - 07 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 为美好的世界献上祝福 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "为美好的世界献上祝福");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_sweet_为美好的世界献上祝福_08() {
        let path = PathBuf::from("[SweetSub] 为美好的世界献上祝福 - 08 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 为美好的世界献上祝福 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "为美好的世界献上祝福");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_sweet_为美好的世界献上祝福_09() {
        let path = PathBuf::from("[SweetSub] 为美好的世界献上祝福 - 09 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 为美好的世界献上祝福 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "为美好的世界献上祝福");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_sweet_为美好的世界献上祝福_10() {
        let path = PathBuf::from("[SweetSub] 为美好的世界献上祝福 - 10 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 为美好的世界献上祝福 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "为美好的世界献上祝福");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_sweet_盾之勇者成名录_01() {
        let path = PathBuf::from("[SweetSub] 盾之勇者成名录 - 01 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 盾之勇者成名录 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "盾之勇者成名录");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_sweet_盾之勇者成名录_02() {
        let path = PathBuf::from("[SweetSub] 盾之勇者成名录 - 02 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 盾之勇者成名录 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "盾之勇者成名录");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_sweet_盾之勇者成名录_03() {
        let path = PathBuf::from("[SweetSub] 盾之勇者成名录 - 03 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 盾之勇者成名录 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "盾之勇者成名录");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_sweet_盾之勇者成名录_04() {
        let path = PathBuf::from("[SweetSub] 盾之勇者成名录 - 04 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 盾之勇者成名录 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "盾之勇者成名录");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_sweet_盾之勇者成名录_05() {
        let path = PathBuf::from("[SweetSub] 盾之勇者成名录 - 05 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 盾之勇者成名录 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "盾之勇者成名录");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_sweet_盾之勇者成名录_06() {
        let path = PathBuf::from("[SweetSub] 盾之勇者成名录 - 06 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 盾之勇者成名录 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "盾之勇者成名录");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_sweet_盾之勇者成名录_07() {
        let path = PathBuf::from("[SweetSub] 盾之勇者成名录 - 07 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 盾之勇者成名录 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "盾之勇者成名录");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_sweet_盾之勇者成名录_08() {
        let path = PathBuf::from("[SweetSub] 盾之勇者成名录 - 08 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 盾之勇者成名录 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "盾之勇者成名录");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_sweet_盾之勇者成名录_09() {
        let path = PathBuf::from("[SweetSub] 盾之勇者成名录 - 09 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 盾之勇者成名录 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "盾之勇者成名录");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_sweet_盾之勇者成名录_10() {
        let path = PathBuf::from("[SweetSub] 盾之勇者成名录 - 10 [WebRip].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [SweetSub] 盾之勇者成名录 format");
        assert_eq!(info.publisher, "SweetSub");
        assert_eq!(info.anime_name, "盾之勇者成名录");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_feiban_骸骨骑士_01() {
        let path = PathBuf::from("[沸班亚马制作组] 骸骨骑士 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 骸骨骑士 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "骸骨骑士");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_feiban_骸骨骑士_02() {
        let path = PathBuf::from("[沸班亚马制作组] 骸骨骑士 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 骸骨骑士 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "骸骨骑士");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_feiban_骸骨骑士_03() {
        let path = PathBuf::from("[沸班亚马制作组] 骸骨骑士 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 骸骨骑士 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "骸骨骑士");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_feiban_骸骨骑士_04() {
        let path = PathBuf::from("[沸班亚马制作组] 骸骨骑士 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 骸骨骑士 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "骸骨骑士");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_feiban_骸骨骑士_05() {
        let path = PathBuf::from("[沸班亚马制作组] 骸骨骑士 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 骸骨骑士 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "骸骨骑士");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_feiban_骸骨骑士_06() {
        let path = PathBuf::from("[沸班亚马制作组] 骸骨骑士 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 骸骨骑士 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "骸骨骑士");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_feiban_骸骨骑士_07() {
        let path = PathBuf::from("[沸班亚马制作组] 骸骨骑士 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 骸骨骑士 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "骸骨骑士");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_feiban_骸骨骑士_08() {
        let path = PathBuf::from("[沸班亚马制作组] 骸骨骑士 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 骸骨骑士 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "骸骨骑士");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_feiban_骸骨骑士_09() {
        let path = PathBuf::from("[沸班亚马制作组] 骸骨骑士 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 骸骨骑士 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "骸骨骑士");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_feiban_骸骨骑士_10() {
        let path = PathBuf::from("[沸班亚马制作组] 骸骨骑士 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 骸骨骑士 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "骸骨骑士");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_feiban_这个医师超麻烦_01() {
        let path = PathBuf::from("[沸班亚马制作组] 这个医师超麻烦 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 这个医师超麻烦 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "这个医师超麻烦");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_feiban_这个医师超麻烦_02() {
        let path = PathBuf::from("[沸班亚马制作组] 这个医师超麻烦 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 这个医师超麻烦 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "这个医师超麻烦");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_feiban_这个医师超麻烦_03() {
        let path = PathBuf::from("[沸班亚马制作组] 这个医师超麻烦 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 这个医师超麻烦 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "这个医师超麻烦");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_feiban_这个医师超麻烦_04() {
        let path = PathBuf::from("[沸班亚马制作组] 这个医师超麻烦 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 这个医师超麻烦 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "这个医师超麻烦");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_feiban_这个医师超麻烦_05() {
        let path = PathBuf::from("[沸班亚马制作组] 这个医师超麻烦 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 这个医师超麻烦 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "这个医师超麻烦");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_feiban_这个医师超麻烦_06() {
        let path = PathBuf::from("[沸班亚马制作组] 这个医师超麻烦 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 这个医师超麻烦 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "这个医师超麻烦");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_feiban_这个医师超麻烦_07() {
        let path = PathBuf::from("[沸班亚马制作组] 这个医师超麻烦 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 这个医师超麻烦 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "这个医师超麻烦");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_feiban_这个医师超麻烦_08() {
        let path = PathBuf::from("[沸班亚马制作组] 这个医师超麻烦 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 这个医师超麻烦 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "这个医师超麻烦");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_feiban_这个医师超麻烦_09() {
        let path = PathBuf::from("[沸班亚马制作组] 这个医师超麻烦 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 这个医师超麻烦 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "这个医师超麻烦");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_feiban_这个医师超麻烦_10() {
        let path = PathBuf::from("[沸班亚马制作组] 这个医师超麻烦 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 这个医师超麻烦 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "这个医师超麻烦");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_feiban_作为反派大小姐_01() {
        let path = PathBuf::from("[沸班亚马制作组] 作为反派大小姐 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 作为反派大小姐 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "作为反派大小姐");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_feiban_作为反派大小姐_02() {
        let path = PathBuf::from("[沸班亚马制作组] 作为反派大小姐 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 作为反派大小姐 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "作为反派大小姐");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_feiban_作为反派大小姐_03() {
        let path = PathBuf::from("[沸班亚马制作组] 作为反派大小姐 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 作为反派大小姐 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "作为反派大小姐");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_feiban_作为反派大小姐_04() {
        let path = PathBuf::from("[沸班亚马制作组] 作为反派大小姐 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 作为反派大小姐 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "作为反派大小姐");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_feiban_作为反派大小姐_05() {
        let path = PathBuf::from("[沸班亚马制作组] 作为反派大小姐 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 作为反派大小姐 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "作为反派大小姐");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_feiban_作为反派大小姐_06() {
        let path = PathBuf::from("[沸班亚马制作组] 作为反派大小姐 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 作为反派大小姐 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "作为反派大小姐");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_feiban_作为反派大小姐_07() {
        let path = PathBuf::from("[沸班亚马制作组] 作为反派大小姐 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 作为反派大小姐 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "作为反派大小姐");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_feiban_作为反派大小姐_08() {
        let path = PathBuf::from("[沸班亚马制作组] 作为反派大小姐 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 作为反派大小姐 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "作为反派大小姐");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_feiban_作为反派大小姐_09() {
        let path = PathBuf::from("[沸班亚马制作组] 作为反派大小姐 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 作为反派大小姐 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "作为反派大小姐");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_feiban_作为反派大小姐_10() {
        let path = PathBuf::from("[沸班亚马制作组] 作为反派大小姐 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 作为反派大小姐 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "作为反派大小姐");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_feiban_历史之眼_01() {
        let path = PathBuf::from("[沸班亚马制作组] 历史之眼 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 历史之眼 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "历史之眼");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_feiban_历史之眼_02() {
        let path = PathBuf::from("[沸班亚马制作组] 历史之眼 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 历史之眼 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "历史之眼");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_feiban_历史之眼_03() {
        let path = PathBuf::from("[沸班亚马制作组] 历史之眼 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 历史之眼 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "历史之眼");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_feiban_历史之眼_04() {
        let path = PathBuf::from("[沸班亚马制作组] 历史之眼 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 历史之眼 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "历史之眼");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_feiban_历史之眼_05() {
        let path = PathBuf::from("[沸班亚马制作组] 历史之眼 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 历史之眼 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "历史之眼");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_feiban_历史之眼_06() {
        let path = PathBuf::from("[沸班亚马制作组] 历史之眼 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 历史之眼 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "历史之眼");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_feiban_历史之眼_07() {
        let path = PathBuf::from("[沸班亚马制作组] 历史之眼 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 历史之眼 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "历史之眼");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_feiban_历史之眼_08() {
        let path = PathBuf::from("[沸班亚马制作组] 历史之眼 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 历史之眼 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "历史之眼");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_feiban_历史之眼_09() {
        let path = PathBuf::from("[沸班亚马制作组] 历史之眼 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 历史之眼 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "历史之眼");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_feiban_历史之眼_10() {
        let path = PathBuf::from("[沸班亚马制作组] 历史之眼 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 历史之眼 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "历史之眼");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战_01() {
        let path = PathBuf::from("[jibaketa] 夏日大作战 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战_02() {
        let path = PathBuf::from("[jibaketa] 夏日大作战 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战_03() {
        let path = PathBuf::from("[jibaketa] 夏日大作战 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战_04() {
        let path = PathBuf::from("[jibaketa] 夏日大作战 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战_05() {
        let path = PathBuf::from("[jibaketa] 夏日大作战 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战_06() {
        let path = PathBuf::from("[jibaketa] 夏日大作战 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战_07() {
        let path = PathBuf::from("[jibaketa] 夏日大作战 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战_08() {
        let path = PathBuf::from("[jibaketa] 夏日大作战 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战_09() {
        let path = PathBuf::from("[jibaketa] 夏日大作战 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战_10() {
        let path = PathBuf::from("[jibaketa] 夏日大作战 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_jibaketa_魔法少女小圆_01() {
        let path = PathBuf::from("[jibaketa] 魔法少女小圆 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 魔法少女小圆 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_jibaketa_魔法少女小圆_02() {
        let path = PathBuf::from("[jibaketa] 魔法少女小圆 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 魔法少女小圆 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_jibaketa_魔法少女小圆_03() {
        let path = PathBuf::from("[jibaketa] 魔法少女小圆 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 魔法少女小圆 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_jibaketa_魔法少女小圆_04() {
        let path = PathBuf::from("[jibaketa] 魔法少女小圆 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 魔法少女小圆 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_jibaketa_魔法少女小圆_05() {
        let path = PathBuf::from("[jibaketa] 魔法少女小圆 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 魔法少女小圆 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_jibaketa_魔法少女小圆_06() {
        let path = PathBuf::from("[jibaketa] 魔法少女小圆 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 魔法少女小圆 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_jibaketa_魔法少女小圆_07() {
        let path = PathBuf::from("[jibaketa] 魔法少女小圆 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 魔法少女小圆 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_jibaketa_魔法少女小圆_08() {
        let path = PathBuf::from("[jibaketa] 魔法少女小圆 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 魔法少女小圆 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_jibaketa_魔法少女小圆_09() {
        let path = PathBuf::from("[jibaketa] 魔法少女小圆 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 魔法少女小圆 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_jibaketa_魔法少女小圆_10() {
        let path = PathBuf::from("[jibaketa] 魔法少女小圆 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 魔法少女小圆 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_jibaketa_命运石之门_01() {
        let path = PathBuf::from("[jibaketa] 命运石之门 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 命运石之门 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "命运石之门");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_jibaketa_命运石之门_02() {
        let path = PathBuf::from("[jibaketa] 命运石之门 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 命运石之门 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "命运石之门");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_jibaketa_命运石之门_03() {
        let path = PathBuf::from("[jibaketa] 命运石之门 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 命运石之门 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "命运石之门");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_jibaketa_命运石之门_04() {
        let path = PathBuf::from("[jibaketa] 命运石之门 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 命运石之门 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "命运石之门");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_jibaketa_命运石之门_05() {
        let path = PathBuf::from("[jibaketa] 命运石之门 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 命运石之门 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "命运石之门");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_jibaketa_命运石之门_06() {
        let path = PathBuf::from("[jibaketa] 命运石之门 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 命运石之门 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "命运石之门");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_jibaketa_命运石之门_07() {
        let path = PathBuf::from("[jibaketa] 命运石之门 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 命运石之门 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "命运石之门");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_jibaketa_命运石之门_08() {
        let path = PathBuf::from("[jibaketa] 命运石之门 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 命运石之门 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "命运石之门");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_jibaketa_命运石之门_09() {
        let path = PathBuf::from("[jibaketa] 命运石之门 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 命运石之门 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "命运石之门");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_jibaketa_命运石之门_10() {
        let path = PathBuf::from("[jibaketa] 命运石之门 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 命运石之门 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "命运石之门");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_jibaketa_攻壳机动队_01() {
        let path = PathBuf::from("[jibaketa] 攻壳机动队 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 攻壳机动队 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "攻壳机动队");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_jibaketa_攻壳机动队_02() {
        let path = PathBuf::from("[jibaketa] 攻壳机动队 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 攻壳机动队 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "攻壳机动队");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_jibaketa_攻壳机动队_03() {
        let path = PathBuf::from("[jibaketa] 攻壳机动队 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 攻壳机动队 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "攻壳机动队");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_jibaketa_攻壳机动队_04() {
        let path = PathBuf::from("[jibaketa] 攻壳机动队 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 攻壳机动队 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "攻壳机动队");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_jibaketa_攻壳机动队_05() {
        let path = PathBuf::from("[jibaketa] 攻壳机动队 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 攻壳机动队 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "攻壳机动队");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_jibaketa_攻壳机动队_06() {
        let path = PathBuf::from("[jibaketa] 攻壳机动队 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 攻壳机动队 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "攻壳机动队");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_jibaketa_攻壳机动队_07() {
        let path = PathBuf::from("[jibaketa] 攻壳机动队 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 攻壳机动队 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "攻壳机动队");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_jibaketa_攻壳机动队_08() {
        let path = PathBuf::from("[jibaketa] 攻壳机动队 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 攻壳机动队 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "攻壳机动队");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_jibaketa_攻壳机动队_09() {
        let path = PathBuf::from("[jibaketa] 攻壳机动队 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 攻壳机动队 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "攻壳机动队");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_jibaketa_攻壳机动队_10() {
        let path = PathBuf::from("[jibaketa] 攻壳机动队 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 攻壳机动队 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "攻壳机动队");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_beiyuzhi_吹响吧上低音号_01() {
        let path = PathBuf::from("[北宇治字幕组] 吹响吧上低音号 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_beiyuzhi_吹响吧上低音号_02() {
        let path = PathBuf::from("[北宇治字幕组] 吹响吧上低音号 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_beiyuzhi_吹响吧上低音号_03() {
        let path = PathBuf::from("[北宇治字幕组] 吹响吧上低音号 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_beiyuzhi_吹响吧上低音号_04() {
        let path = PathBuf::from("[北宇治字幕组] 吹响吧上低音号 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_beiyuzhi_吹响吧上低音号_05() {
        let path = PathBuf::from("[北宇治字幕组] 吹响吧上低音号 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_beiyuzhi_吹响吧上低音号_06() {
        let path = PathBuf::from("[北宇治字幕组] 吹响吧上低音号 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_beiyuzhi_吹响吧上低音号_07() {
        let path = PathBuf::from("[北宇治字幕组] 吹响吧上低音号 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_beiyuzhi_吹响吧上低音号_08() {
        let path = PathBuf::from("[北宇治字幕组] 吹响吧上低音号 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_beiyuzhi_吹响吧上低音号_09() {
        let path = PathBuf::from("[北宇治字幕组] 吹响吧上低音号 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_beiyuzhi_吹响吧上低音号_10() {
        let path = PathBuf::from("[北宇治字幕组] 吹响吧上低音号 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 吹响吧上低音号 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "吹响吧上低音号");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_beiyuzhi_紫罗兰永恒花园_01() {
        let path = PathBuf::from("[北宇治字幕组] 紫罗兰永恒花园 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_beiyuzhi_紫罗兰永恒花园_02() {
        let path = PathBuf::from("[北宇治字幕组] 紫罗兰永恒花园 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_beiyuzhi_紫罗兰永恒花园_03() {
        let path = PathBuf::from("[北宇治字幕组] 紫罗兰永恒花园 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_beiyuzhi_紫罗兰永恒花园_04() {
        let path = PathBuf::from("[北宇治字幕组] 紫罗兰永恒花园 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_beiyuzhi_紫罗兰永恒花园_05() {
        let path = PathBuf::from("[北宇治字幕组] 紫罗兰永恒花园 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_beiyuzhi_紫罗兰永恒花园_06() {
        let path = PathBuf::from("[北宇治字幕组] 紫罗兰永恒花园 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_beiyuzhi_紫罗兰永恒花园_07() {
        let path = PathBuf::from("[北宇治字幕组] 紫罗兰永恒花园 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_beiyuzhi_紫罗兰永恒花园_08() {
        let path = PathBuf::from("[北宇治字幕组] 紫罗兰永恒花园 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_beiyuzhi_紫罗兰永恒花园_09() {
        let path = PathBuf::from("[北宇治字幕组] 紫罗兰永恒花园 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_beiyuzhi_紫罗兰永恒花园_10() {
        let path = PathBuf::from("[北宇治字幕组] 紫罗兰永恒花园 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 紫罗兰永恒花园 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "紫罗兰永恒花园");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_beiyuzhi_轻音少女_01() {
        let path = PathBuf::from("[北宇治字幕组] 轻音少女 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 轻音少女 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "轻音少女");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_beiyuzhi_轻音少女_02() {
        let path = PathBuf::from("[北宇治字幕组] 轻音少女 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 轻音少女 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "轻音少女");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_beiyuzhi_轻音少女_03() {
        let path = PathBuf::from("[北宇治字幕组] 轻音少女 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 轻音少女 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "轻音少女");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_beiyuzhi_轻音少女_04() {
        let path = PathBuf::from("[北宇治字幕组] 轻音少女 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 轻音少女 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "轻音少女");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_beiyuzhi_轻音少女_05() {
        let path = PathBuf::from("[北宇治字幕组] 轻音少女 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 轻音少女 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "轻音少女");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_beiyuzhi_轻音少女_06() {
        let path = PathBuf::from("[北宇治字幕组] 轻音少女 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 轻音少女 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "轻音少女");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_beiyuzhi_轻音少女_07() {
        let path = PathBuf::from("[北宇治字幕组] 轻音少女 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 轻音少女 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "轻音少女");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_beiyuzhi_轻音少女_08() {
        let path = PathBuf::from("[北宇治字幕组] 轻音少女 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 轻音少女 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "轻音少女");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_beiyuzhi_轻音少女_09() {
        let path = PathBuf::from("[北宇治字幕组] 轻音少女 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 轻音少女 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "轻音少女");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_beiyuzhi_轻音少女_10() {
        let path = PathBuf::from("[北宇治字幕组] 轻音少女 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 轻音少女 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "轻音少女");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_beiyuzhi_冰果_01() {
        let path = PathBuf::from("[北宇治字幕组] 冰果 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 冰果 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_beiyuzhi_冰果_02() {
        let path = PathBuf::from("[北宇治字幕组] 冰果 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 冰果 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_beiyuzhi_冰果_03() {
        let path = PathBuf::from("[北宇治字幕组] 冰果 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 冰果 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_beiyuzhi_冰果_04() {
        let path = PathBuf::from("[北宇治字幕组] 冰果 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 冰果 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_beiyuzhi_冰果_05() {
        let path = PathBuf::from("[北宇治字幕组] 冰果 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 冰果 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_beiyuzhi_冰果_06() {
        let path = PathBuf::from("[北宇治字幕组] 冰果 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 冰果 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_beiyuzhi_冰果_07() {
        let path = PathBuf::from("[北宇治字幕组] 冰果 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 冰果 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_beiyuzhi_冰果_08() {
        let path = PathBuf::from("[北宇治字幕组] 冰果 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 冰果 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_beiyuzhi_冰果_09() {
        let path = PathBuf::from("[北宇治字幕组] 冰果 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 冰果 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_beiyuzhi_冰果_10() {
        let path = PathBuf::from("[北宇治字幕组] 冰果 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [北宇治字幕组] 冰果 format");
        assert_eq!(info.publisher, "北宇治字幕组");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_liehu_跃动青春_01() {
        let path = PathBuf::from("[猎户压制部] 跃动青春 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 跃动青春 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "跃动青春");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_liehu_跃动青春_02() {
        let path = PathBuf::from("[猎户压制部] 跃动青春 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 跃动青春 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "跃动青春");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_liehu_跃动青春_03() {
        let path = PathBuf::from("[猎户压制部] 跃动青春 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 跃动青春 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "跃动青春");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_liehu_跃动青春_04() {
        let path = PathBuf::from("[猎户压制部] 跃动青春 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 跃动青春 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "跃动青春");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_liehu_跃动青春_05() {
        let path = PathBuf::from("[猎户压制部] 跃动青春 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 跃动青春 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "跃动青春");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_liehu_跃动青春_06() {
        let path = PathBuf::from("[猎户压制部] 跃动青春 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 跃动青春 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "跃动青春");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_liehu_跃动青春_07() {
        let path = PathBuf::from("[猎户压制部] 跃动青春 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 跃动青春 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "跃动青春");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_liehu_跃动青春_08() {
        let path = PathBuf::from("[猎户压制部] 跃动青春 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 跃动青春 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "跃动青春");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_liehu_跃动青春_09() {
        let path = PathBuf::from("[猎户压制部] 跃动青春 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 跃动青春 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "跃动青春");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_liehu_跃动青春_10() {
        let path = PathBuf::from("[猎户压制部] 跃动青春 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 跃动青春 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "跃动青春");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_liehu_逃避可耻_01() {
        let path = PathBuf::from("[猎户压制部] 逃避可耻 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 逃避可耻 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "逃避可耻");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_liehu_逃避可耻_02() {
        let path = PathBuf::from("[猎户压制部] 逃避可耻 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 逃避可耻 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "逃避可耻");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_liehu_逃避可耻_03() {
        let path = PathBuf::from("[猎户压制部] 逃避可耻 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 逃避可耻 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "逃避可耻");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_liehu_逃避可耻_04() {
        let path = PathBuf::from("[猎户压制部] 逃避可耻 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 逃避可耻 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "逃避可耻");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_liehu_逃避可耻_05() {
        let path = PathBuf::from("[猎户压制部] 逃避可耻 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 逃避可耻 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "逃避可耻");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_liehu_逃避可耻_06() {
        let path = PathBuf::from("[猎户压制部] 逃避可耻 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 逃避可耻 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "逃避可耻");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_liehu_逃避可耻_07() {
        let path = PathBuf::from("[猎户压制部] 逃避可耻 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 逃避可耻 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "逃避可耻");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_liehu_逃避可耻_08() {
        let path = PathBuf::from("[猎户压制部] 逃避可耻 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 逃避可耻 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "逃避可耻");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_liehu_逃避可耻_09() {
        let path = PathBuf::from("[猎户压制部] 逃避可耻 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 逃避可耻 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "逃避可耻");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_liehu_逃避可耻_10() {
        let path = PathBuf::from("[猎户压制部] 逃避可耻 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 逃避可耻 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "逃避可耻");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_liehu_四月是你的谎言_01() {
        let path = PathBuf::from("[猎户压制部] 四月是你的谎言 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 四月是你的谎言 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_liehu_四月是你的谎言_02() {
        let path = PathBuf::from("[猎户压制部] 四月是你的谎言 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 四月是你的谎言 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_liehu_四月是你的谎言_03() {
        let path = PathBuf::from("[猎户压制部] 四月是你的谎言 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 四月是你的谎言 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_liehu_四月是你的谎言_04() {
        let path = PathBuf::from("[猎户压制部] 四月是你的谎言 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 四月是你的谎言 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_liehu_四月是你的谎言_05() {
        let path = PathBuf::from("[猎户压制部] 四月是你的谎言 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 四月是你的谎言 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_liehu_四月是你的谎言_06() {
        let path = PathBuf::from("[猎户压制部] 四月是你的谎言 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 四月是你的谎言 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_liehu_四月是你的谎言_07() {
        let path = PathBuf::from("[猎户压制部] 四月是你的谎言 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 四月是你的谎言 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_liehu_四月是你的谎言_08() {
        let path = PathBuf::from("[猎户压制部] 四月是你的谎言 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 四月是你的谎言 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_liehu_四月是你的谎言_09() {
        let path = PathBuf::from("[猎户压制部] 四月是你的谎言 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 四月是你的谎言 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_liehu_四月是你的谎言_10() {
        let path = PathBuf::from("[猎户压制部] 四月是你的谎言 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 四月是你的谎言 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "四月是你的谎言");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_liehu_未闻花名_01() {
        let path = PathBuf::from("[猎户压制部] 未闻花名 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 未闻花名 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_liehu_未闻花名_02() {
        let path = PathBuf::from("[猎户压制部] 未闻花名 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 未闻花名 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_liehu_未闻花名_03() {
        let path = PathBuf::from("[猎户压制部] 未闻花名 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 未闻花名 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_liehu_未闻花名_04() {
        let path = PathBuf::from("[猎户压制部] 未闻花名 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 未闻花名 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_liehu_未闻花名_05() {
        let path = PathBuf::from("[猎户压制部] 未闻花名 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 未闻花名 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_liehu_未闻花名_06() {
        let path = PathBuf::from("[猎户压制部] 未闻花名 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 未闻花名 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_liehu_未闻花名_07() {
        let path = PathBuf::from("[猎户压制部] 未闻花名 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 未闻花名 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_liehu_未闻花名_08() {
        let path = PathBuf::from("[猎户压制部] 未闻花名 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 未闻花名 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_liehu_未闻花名_09() {
        let path = PathBuf::from("[猎户压制部] 未闻花名 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 未闻花名 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_liehu_未闻花名_10() {
        let path = PathBuf::from("[猎户压制部] 未闻花名 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 未闻花名 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "未闻花名");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_tsdm_Charlotte_01() {
        let path = PathBuf::from("[TSDM字幕组] Charlotte - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Charlotte format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Charlotte");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_tsdm_Charlotte_02() {
        let path = PathBuf::from("[TSDM字幕组] Charlotte - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Charlotte format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Charlotte");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_tsdm_Charlotte_03() {
        let path = PathBuf::from("[TSDM字幕组] Charlotte - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Charlotte format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Charlotte");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_tsdm_Charlotte_04() {
        let path = PathBuf::from("[TSDM字幕组] Charlotte - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Charlotte format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Charlotte");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_tsdm_Charlotte_05() {
        let path = PathBuf::from("[TSDM字幕组] Charlotte - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Charlotte format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Charlotte");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_tsdm_Charlotte_06() {
        let path = PathBuf::from("[TSDM字幕组] Charlotte - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Charlotte format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Charlotte");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_tsdm_Charlotte_07() {
        let path = PathBuf::from("[TSDM字幕组] Charlotte - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Charlotte format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Charlotte");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_tsdm_Charlotte_08() {
        let path = PathBuf::from("[TSDM字幕组] Charlotte - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Charlotte format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Charlotte");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_tsdm_Charlotte_09() {
        let path = PathBuf::from("[TSDM字幕组] Charlotte - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Charlotte format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Charlotte");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_tsdm_Charlotte_10() {
        let path = PathBuf::from("[TSDM字幕组] Charlotte - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Charlotte format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Charlotte");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_tsdm_Angel_Beats_01() {
        let path = PathBuf::from("[TSDM字幕组] Angel Beats - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Angel Beats format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Angel Beats");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_tsdm_Angel_Beats_02() {
        let path = PathBuf::from("[TSDM字幕组] Angel Beats - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Angel Beats format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Angel Beats");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_tsdm_Angel_Beats_03() {
        let path = PathBuf::from("[TSDM字幕组] Angel Beats - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Angel Beats format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Angel Beats");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_tsdm_Angel_Beats_04() {
        let path = PathBuf::from("[TSDM字幕组] Angel Beats - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Angel Beats format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Angel Beats");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_tsdm_Angel_Beats_05() {
        let path = PathBuf::from("[TSDM字幕组] Angel Beats - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Angel Beats format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Angel Beats");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_tsdm_Angel_Beats_06() {
        let path = PathBuf::from("[TSDM字幕组] Angel Beats - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Angel Beats format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Angel Beats");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_tsdm_Angel_Beats_07() {
        let path = PathBuf::from("[TSDM字幕组] Angel Beats - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Angel Beats format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Angel Beats");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_tsdm_Angel_Beats_08() {
        let path = PathBuf::from("[TSDM字幕组] Angel Beats - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Angel Beats format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Angel Beats");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_tsdm_Angel_Beats_09() {
        let path = PathBuf::from("[TSDM字幕组] Angel Beats - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Angel Beats format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Angel Beats");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_tsdm_Angel_Beats_10() {
        let path = PathBuf::from("[TSDM字幕组] Angel Beats - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Angel Beats format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Angel Beats");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_tsdm_Clannad_01() {
        let path = PathBuf::from("[TSDM字幕组] Clannad - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Clannad format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Clannad");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_tsdm_Clannad_02() {
        let path = PathBuf::from("[TSDM字幕组] Clannad - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Clannad format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Clannad");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_tsdm_Clannad_03() {
        let path = PathBuf::from("[TSDM字幕组] Clannad - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Clannad format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Clannad");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_tsdm_Clannad_04() {
        let path = PathBuf::from("[TSDM字幕组] Clannad - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Clannad format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Clannad");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_tsdm_Clannad_05() {
        let path = PathBuf::from("[TSDM字幕组] Clannad - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Clannad format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Clannad");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_tsdm_Clannad_06() {
        let path = PathBuf::from("[TSDM字幕组] Clannad - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Clannad format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Clannad");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_tsdm_Clannad_07() {
        let path = PathBuf::from("[TSDM字幕组] Clannad - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Clannad format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Clannad");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_tsdm_Clannad_08() {
        let path = PathBuf::from("[TSDM字幕组] Clannad - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Clannad format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Clannad");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_tsdm_Clannad_09() {
        let path = PathBuf::from("[TSDM字幕组] Clannad - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Clannad format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Clannad");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_tsdm_Clannad_10() {
        let path = PathBuf::from("[TSDM字幕组] Clannad - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Clannad format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Clannad");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_tsdm_Little_Busters_01() {
        let path = PathBuf::from("[TSDM字幕组] Little Busters - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Little Busters format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Little Busters");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_tsdm_Little_Busters_02() {
        let path = PathBuf::from("[TSDM字幕组] Little Busters - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Little Busters format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Little Busters");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_tsdm_Little_Busters_03() {
        let path = PathBuf::from("[TSDM字幕组] Little Busters - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Little Busters format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Little Busters");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_tsdm_Little_Busters_04() {
        let path = PathBuf::from("[TSDM字幕组] Little Busters - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Little Busters format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Little Busters");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_tsdm_Little_Busters_05() {
        let path = PathBuf::from("[TSDM字幕组] Little Busters - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Little Busters format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Little Busters");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_tsdm_Little_Busters_06() {
        let path = PathBuf::from("[TSDM字幕组] Little Busters - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Little Busters format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Little Busters");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_tsdm_Little_Busters_07() {
        let path = PathBuf::from("[TSDM字幕组] Little Busters - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Little Busters format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Little Busters");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_tsdm_Little_Busters_08() {
        let path = PathBuf::from("[TSDM字幕组] Little Busters - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Little Busters format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Little Busters");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_tsdm_Little_Busters_09() {
        let path = PathBuf::from("[TSDM字幕组] Little Busters - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Little Busters format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Little Busters");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_tsdm_Little_Busters_10() {
        let path = PathBuf::from("[TSDM字幕组] Little Busters - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] Little Busters format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "Little Busters");
        assert_eq!(info.episode, "10");
    }

    // =====================================================
    // VCB-Studio tests (NEW publisher - 10 tests)
    // =====================================================
    #[test]
    fn test_parse_vcb_某科学的超电磁炮_01() {
        let path = PathBuf::from("[VCB-Studio] 某科学的超电磁炮 - 01 [1080p][HEVC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [VCB-Studio] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "VCB-Studio");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_vcb_某科学的超电磁炮_02() {
        let path = PathBuf::from("[VCB-Studio] 某科学的超电磁炮 - 02 [1080p][HEVC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [VCB-Studio] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "VCB-Studio");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_vcb_某科学的超电磁炮_03() {
        let path = PathBuf::from("[VCB-Studio] 某科学的超电磁炮 - 03 [1080p][HEVC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [VCB-Studio] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "VCB-Studio");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_vcb_某科学的超电磁炮_04() {
        let path = PathBuf::from("[VCB-Studio] 某科学的超电磁炮 - 04 [1080p][HEVC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [VCB-Studio] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "VCB-Studio");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_vcb_某科学的超电磁炮_05() {
        let path = PathBuf::from("[VCB-Studio] 某科学的超电磁炮 - 05 [1080p][HEVC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [VCB-Studio] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "VCB-Studio");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_vcb_某科学的超电磁炮_06() {
        let path = PathBuf::from("[VCB-Studio] 某科学的超电磁炮 - 06 [1080p][HEVC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [VCB-Studio] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "VCB-Studio");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_vcb_某科学的超电磁炮_07() {
        let path = PathBuf::from("[VCB-Studio] 某科学的超电磁炮 - 07 [1080p][HEVC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [VCB-Studio] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "VCB-Studio");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_vcb_某科学的超电磁炮_08() {
        let path = PathBuf::from("[VCB-Studio] 某科学的超电磁炮 - 08 [1080p][HEVC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [VCB-Studio] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "VCB-Studio");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_vcb_某科学的超电磁炮_09() {
        let path = PathBuf::from("[VCB-Studio] 某科学的超电磁炮 - 09 [1080p][HEVC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [VCB-Studio] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "VCB-Studio");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_vcb_某科学的超电磁炮_10() {
        let path = PathBuf::from("[VCB-Studio] 某科学的超电磁炮 - 10 [1080p][HEVC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [VCB-Studio] 某科学的超电磁炮 format");
        assert_eq!(info.publisher, "VCB-Studio");
        assert_eq!(info.anime_name, "某科学的超电磁炮");
        assert_eq!(info.episode, "10");
    }

    // =====================================================
    // 极影字幕+轻之国度 tests (NEW publisher - 10 tests)
    // =====================================================
    #[test]
    fn test_parse_jijian_笨蛋测验召唤兽_01() {
        let path = PathBuf::from("[极影字幕+轻之国度] 笨蛋测验召唤兽2 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [极影字幕+轻之国度]笨蛋测验召唤兽2 format");
        assert_eq!(info.publisher, "极影字幕+轻之国度");
        assert_eq!(info.anime_name, "笨蛋测验召唤兽2");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_jijian_笨蛋测验召唤兽_02() {
        let path = PathBuf::from("[极影字幕+轻之国度] 笨蛋测验召唤兽2 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [极影字幕+轻之国度]笨蛋测验召唤兽2 format");
        assert_eq!(info.publisher, "极影字幕+轻之国度");
        assert_eq!(info.anime_name, "笨蛋测验召唤兽2");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_jijian_笨蛋测验召唤兽_03() {
        let path = PathBuf::from("[极影字幕+轻之国度] 笨蛋测验召唤兽2 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [极影字幕+轻之国度]笨蛋测验召唤兽2 format");
        assert_eq!(info.publisher, "极影字幕+轻之国度");
        assert_eq!(info.anime_name, "笨蛋测验召唤兽2");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_jijian_笨蛋测验召唤兽_04() {
        let path = PathBuf::from("[极影字幕+轻之国度] 笨蛋测验召唤兽2 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [极影字幕+轻之国度]笨蛋测验召唤兽2 format");
        assert_eq!(info.publisher, "极影字幕+轻之国度");
        assert_eq!(info.anime_name, "笨蛋测验召唤兽2");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_jijian_笨蛋测验召唤兽_05() {
        let path = PathBuf::from("[极影字幕+轻之国度] 笨蛋测验召唤兽2 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [极影字幕+轻之国度]笨蛋测验召唤兽2 format");
        assert_eq!(info.publisher, "极影字幕+轻之国度");
        assert_eq!(info.anime_name, "笨蛋测验召唤兽2");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_jijian_笨蛋测验召唤兽_06() {
        let path = PathBuf::from("[极影字幕+轻之国度] 笨蛋测验召唤兽2 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [极影字幕+轻之国度]笨蛋测验召唤兽2 format");
        assert_eq!(info.publisher, "极影字幕+轻之国度");
        assert_eq!(info.anime_name, "笨蛋测验召唤兽2");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_jijian_笨蛋测验召唤兽_07() {
        let path = PathBuf::from("[极影字幕+轻之国度] 笨蛋测验召唤兽2 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [极影字幕+轻之国度]笨蛋测验召唤兽2 format");
        assert_eq!(info.publisher, "极影字幕+轻之国度");
        assert_eq!(info.anime_name, "笨蛋测验召唤兽2");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_jijian_笨蛋测验召唤兽_08() {
        let path = PathBuf::from("[极影字幕+轻之国度] 笨蛋测验召唤兽2 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [极影字幕+轻之国度]笨蛋测验召唤兽2 format");
        assert_eq!(info.publisher, "极影字幕+轻之国度");
        assert_eq!(info.anime_name, "笨蛋测验召唤兽2");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_jijian_笨蛋测验召唤兽_09() {
        let path = PathBuf::from("[极影字幕+轻之国度] 笨蛋测验召唤兽2 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [极影字幕+轻之国度]笨蛋测验召唤兽2 format");
        assert_eq!(info.publisher, "极影字幕+轻之国度");
        assert_eq!(info.anime_name, "笨蛋测验召唤兽2");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_jijian_笨蛋测验召唤兽_10() {
        let path = PathBuf::from("[极影字幕+轻之国度] 笨蛋测验召唤兽2 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [极影字幕+轻之国度]笨蛋测验召唤兽2 format");
        assert_eq!(info.publisher, "极影字幕+轻之国度");
        assert_eq!(info.anime_name, "笨蛋测验召唤兽2");
        assert_eq!(info.episode, "10");
    }

    // =====================================================
    // jibaketa tests (NEW publisher - 10 tests)
    #[test]
    fn test_parse_jibaketa_夏日大作战2_01() {
        let path = PathBuf::from("[jibaketa] 夏日大作战2 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战2 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战2");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战2_02() {
        let path = PathBuf::from("[jibaketa] 夏日大作战2 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战2 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战2");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战2_03() {
        let path = PathBuf::from("[jibaketa] 夏日大作战2 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战2 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战2");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战2_04() {
        let path = PathBuf::from("[jibaketa] 夏日大作战2 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战2 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战2");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战2_05() {
        let path = PathBuf::from("[jibaketa] 夏日大作战2 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战2 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战2");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战2_06() {
        let path = PathBuf::from("[jibaketa] 夏日大作战2 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战2 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战2");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战2_07() {
        let path = PathBuf::from("[jibaketa] 夏日大作战2 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战2 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战2");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战2_08() {
        let path = PathBuf::from("[jibaketa] 夏日大作战2 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战2 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战2");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战2_09() {
        let path = PathBuf::from("[jibaketa] 夏日大作战2 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战2 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战2");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_jibaketa_夏日大作战2_10() {
        let path = PathBuf::from("[jibaketa] 夏日大作战2 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 夏日大作战2 format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "夏日大作战2");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_jibaketa_某科学的超电磁炮S_02() {
        let path = PathBuf::from("[jibaketa] 某科学的超电磁炮S - 02 [1080P].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 某科学的超电磁炮S format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "某科学的超电磁炮S");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_jibaketa_某科学的超电磁炮S_03() {
        let path = PathBuf::from("[jibaketa] 某科学的超电磁炮S - 03 [1080P].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 某科学的超电磁炮S format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "某科学的超电磁炮S");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_jibaketa_某科学的超电磁炮S_04() {
        let path = PathBuf::from("[jibaketa] 某科学的超电磁炮S - 04 [1080P].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 某科学的超电磁炮S format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "某科学的超电磁炮S");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_jibaketa_某科学的超电磁炮S_05() {
        let path = PathBuf::from("[jibaketa] 某科学的超电磁炮S - 05 [1080P].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 某科学的超电磁炮S format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "某科学的超电磁炮S");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_jibaketa_某科学的超电磁炮S_06() {
        let path = PathBuf::from("[jibaketa] 某科学的超电磁炮S - 06 [1080P].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 某科学的超电磁炮S format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "某科学的超电磁炮S");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_jibaketa_某科学的超电磁炮S_07() {
        let path = PathBuf::from("[jibaketa] 某科学的超电磁炮S - 07 [1080P].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 某科学的超电磁炮S format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "某科学的超电磁炮S");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_jibaketa_某科学的超电磁炮S_08() {
        let path = PathBuf::from("[jibaketa] 某科学的超电磁炮S - 08 [1080P].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 某科学的超电磁炮S format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "某科学的超电磁炮S");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_jibaketa_某科学的超电磁炮S_09() {
        let path = PathBuf::from("[jibaketa] 某科学的超电磁炮S - 09 [1080P].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 某科学的超电磁炮S format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "某科学的超电磁炮S");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_jibaketa_某科学的超电磁炮S_10() {
        let path = PathBuf::from("[jibaketa] 某科学的超电磁炮S - 10 [1080P].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [jibaketa] 某科学的超电磁炮S format");
        assert_eq!(info.publisher, "jibaketa");
        assert_eq!(info.anime_name, "某科学的超电磁炮S");
        assert_eq!(info.episode, "10");
    }

    // =====================================================
    // MagicStar tests (NEW publisher - 10 tests)
    // =====================================================
    #[test]
    fn test_parse_magicstar_异修罗_01() {
        let path = PathBuf::from("[MagicStar] 异修罗 - 01 [WebRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [MagicStar] 异修罗 format");
        assert_eq!(info.publisher, "MagicStar");
        assert_eq!(info.anime_name, "异修罗");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_magicstar_异修罗_02() {
        let path = PathBuf::from("[MagicStar] 异修罗 - 02 [WebRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [MagicStar] 异修罗 format");
        assert_eq!(info.publisher, "MagicStar");
        assert_eq!(info.anime_name, "异修罗");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_magicstar_异修罗_03() {
        let path = PathBuf::from("[MagicStar] 异修罗 - 03 [WebRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [MagicStar] 异修罗 format");
        assert_eq!(info.publisher, "MagicStar");
        assert_eq!(info.anime_name, "异修罗");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_magicstar_异修罗_04() {
        let path = PathBuf::from("[MagicStar] 异修罗 - 04 [WebRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [MagicStar] 异修罗 format");
        assert_eq!(info.publisher, "MagicStar");
        assert_eq!(info.anime_name, "异修罗");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_magicstar_异修罗_05() {
        let path = PathBuf::from("[MagicStar] 异修罗 - 05 [WebRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [MagicStar] 异修罗 format");
        assert_eq!(info.publisher, "MagicStar");
        assert_eq!(info.anime_name, "异修罗");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_magicstar_异修罗_06() {
        let path = PathBuf::from("[MagicStar] 异修罗 - 06 [WebRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [MagicStar] 异修罗 format");
        assert_eq!(info.publisher, "MagicStar");
        assert_eq!(info.anime_name, "异修罗");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_magicstar_异修罗_07() {
        let path = PathBuf::from("[MagicStar] 异修罗 - 07 [WebRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [MagicStar] 异修罗 format");
        assert_eq!(info.publisher, "MagicStar");
        assert_eq!(info.anime_name, "异修罗");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_magicstar_异修罗_08() {
        let path = PathBuf::from("[MagicStar] 异修罗 - 08 [WebRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [MagicStar] 异修罗 format");
        assert_eq!(info.publisher, "MagicStar");
        assert_eq!(info.anime_name, "异修罗");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_magicstar_异修罗_09() {
        let path = PathBuf::from("[MagicStar] 异修罗 - 09 [WebRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [MagicStar] 异修罗 format");
        assert_eq!(info.publisher, "MagicStar");
        assert_eq!(info.anime_name, "异修罗");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_magicstar_异修罗_10() {
        let path = PathBuf::from("[MagicStar] 异修罗 - 10 [WebRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [MagicStar] 异修罗 format");
        assert_eq!(info.publisher, "MagicStar");
        assert_eq!(info.anime_name, "异修罗");
        assert_eq!(info.episode, "10");
    }

    // =====================================================
    // 雪飘工作室 tests (NEW publisher - 10 tests)
    // =====================================================
    #[test]
    fn test_parse_xuepiao_冰果_01() {
        let path = PathBuf::from("[雪飘工作室] 冰果 - 01 [简繁外挂].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [雪飘工作室] 冰果 format");
        assert_eq!(info.publisher, "雪飘工作室");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_xuepiao_冰果_02() {
        let path = PathBuf::from("[雪飘工作室] 冰果 - 02 [简繁外挂].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [雪飘工作室] 冰果 format");
        assert_eq!(info.publisher, "雪飘工作室");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_xuepiao_冰果_03() {
        let path = PathBuf::from("[雪飘工作室] 冰果 - 03 [简繁外挂].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [雪飘工作室] 冰果 format");
        assert_eq!(info.publisher, "雪飘工作室");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_xuepiao_冰果_04() {
        let path = PathBuf::from("[雪飘工作室] 冰果 - 04 [简繁外挂].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [雪飘工作室] 冰果 format");
        assert_eq!(info.publisher, "雪飘工作室");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_xuepiao_冰果_05() {
        let path = PathBuf::from("[雪飘工作室] 冰果 - 05 [简繁外挂].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [雪飘工作室] 冰果 format");
        assert_eq!(info.publisher, "雪飘工作室");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_xuepiao_冰果_06() {
        let path = PathBuf::from("[雪飘工作室] 冰果 - 06 [简繁外挂].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [雪飘工作室] 冰果 format");
        assert_eq!(info.publisher, "雪飘工作室");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_xuepiao_冰果_07() {
        let path = PathBuf::from("[雪飘工作室] 冰果 - 07 [简繁外挂].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [雪飘工作室] 冰果 format");
        assert_eq!(info.publisher, "雪飘工作室");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_xuepiao_冰果_08() {
        let path = PathBuf::from("[雪飘工作室] 冰果 - 08 [简繁外挂].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [雪飘工作室] 冰果 format");
        assert_eq!(info.publisher, "雪飘工作室");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_xuepiao_冰果_09() {
        let path = PathBuf::from("[雪飘工作室] 冰果 - 09 [简繁外挂].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [雪飘工作室] 冰果 format");
        assert_eq!(info.publisher, "雪飘工作室");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_xuepiao_冰果_10() {
        let path = PathBuf::from("[雪飘工作室] 冰果 - 10 [简繁外挂].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [雪飘工作室] 冰果 format");
        assert_eq!(info.publisher, "雪飘工作室");
        assert_eq!(info.anime_name, "冰果");
        assert_eq!(info.episode, "10");
    }

    // =====================================================
    // 沸班亚马制作组 tests (NEW publisher - 10 tests)
    // =====================================================
    #[test]
    fn test_parse_feiban_异世界自杀小队_01() {
        let path =
            PathBuf::from("[沸班亚马制作组] 异世界自杀小队 - 01 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 异世界自杀小队 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "异世界自杀小队");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_feiban_异世界自杀小队_02() {
        let path =
            PathBuf::from("[沸班亚马制作组] 异世界自杀小队 - 02 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 异世界自杀小队 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "异世界自杀小队");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_feiban_异世界自杀小队_03() {
        let path =
            PathBuf::from("[沸班亚马制作组] 异世界自杀小队 - 03 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 异世界自杀小队 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "异世界自杀小队");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_feiban_异世界自杀小队_04() {
        let path =
            PathBuf::from("[沸班亚马制作组] 异世界自杀小队 - 04 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 异世界自杀小队 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "异世界自杀小队");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_feiban_异世界自杀小队_05() {
        let path =
            PathBuf::from("[沸班亚马制作组] 异世界自杀小队 - 05 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 异世界自杀小队 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "异世界自杀小队");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_feiban_异世界自杀小队_06() {
        let path =
            PathBuf::from("[沸班亚马制作组] 异世界自杀小队 - 06 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 异世界自杀小队 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "异世界自杀小队");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_feiban_异世界自杀小队_07() {
        let path =
            PathBuf::from("[沸班亚马制作组] 异世界自杀小队 - 07 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 异世界自杀小队 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "异世界自杀小队");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_feiban_异世界自杀小队_08() {
        let path =
            PathBuf::from("[沸班亚马制作组] 异世界自杀小队 - 08 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 异世界自杀小队 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "异世界自杀小队");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_feiban_异世界自杀小队_09() {
        let path =
            PathBuf::from("[沸班亚马制作组] 异世界自杀小队 - 09 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 异世界自杀小队 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "异世界自杀小队");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_feiban_异世界自杀小队_10() {
        let path =
            PathBuf::from("[沸班亚马制作组] 异世界自杀小队 - 10 [WebRip 1080p HEVC-10bit AAC].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [沸班亚马制作组] 异世界自杀小队 format");
        assert_eq!(info.publisher, "沸班亚马制作组");
        assert_eq!(info.anime_name, "异世界自杀小队");
        assert_eq!(info.episode, "10");
    }

    // =====================================================
    // 猎户压制部 tests (NEW publisher - 10 tests)
    // =====================================================
    #[test]
    fn test_parse_liehu_葬送的芙莉莲_01() {
        let path = PathBuf::from("[猎户压制部] 葬送的芙莉莲 - 01 [BDRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_liehu_葬送的芙莉莲_02() {
        let path = PathBuf::from("[猎户压制部] 葬送的芙莉莲 - 02 [BDRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_liehu_葬送的芙莉莲_03() {
        let path = PathBuf::from("[猎户压制部] 葬送的芙莉莲 - 03 [BDRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_liehu_葬送的芙莉莲_04() {
        let path = PathBuf::from("[猎户压制部] 葬送的芙莉莲 - 04 [BDRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_liehu_葬送的芙莉莲_05() {
        let path = PathBuf::from("[猎户压制部] 葬送的芙莉莲 - 05 [BDRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_liehu_葬送的芙莉莲_06() {
        let path = PathBuf::from("[猎户压制部] 葬送的芙莉莲 - 06 [BDRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_liehu_葬送的芙莉莲_07() {
        let path = PathBuf::from("[猎户压制部] 葬送的芙莉莲 - 07 [BDRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_liehu_葬送的芙莉莲_08() {
        let path = PathBuf::from("[猎户压制部] 葬送的芙莉莲 - 08 [BDRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_liehu_葬送的芙莉莲_09() {
        let path = PathBuf::from("[猎户压制部] 葬送的芙莉莲 - 09 [BDRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_liehu_葬送的芙莉莲_10() {
        let path = PathBuf::from("[猎户压制部] 葬送的芙莉莲 - 10 [BDRip 1080p].mkv");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [猎户压制部] 葬送的芙莉莲 format");
        assert_eq!(info.publisher, "猎户压制部");
        assert_eq!(info.anime_name, "葬送的芙莉莲");
        assert_eq!(info.episode, "10");
    }

    // =====================================================
    // TSDM字幕组 tests (NEW publisher - 10 tests)
    // =====================================================
    #[test]
    fn test_parse_tsdm_以死亡游戏为生_01() {
        let path = PathBuf::from("[TSDM字幕组] 以死亡游戏为生 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] 以死亡游戏为生 format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "以死亡游戏为生");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_tsdm_以死亡游戏为生_02() {
        let path = PathBuf::from("[TSDM字幕组] 以死亡游戏为生 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] 以死亡游戏为生 format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "以死亡游戏为生");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_tsdm_以死亡游戏为生_03() {
        let path = PathBuf::from("[TSDM字幕组] 以死亡游戏为生 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] 以死亡游戏为生 format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "以死亡游戏为生");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_tsdm_以死亡游戏为生_04() {
        let path = PathBuf::from("[TSDM字幕组] 以死亡游戏为生 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] 以死亡游戏为生 format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "以死亡游戏为生");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_tsdm_以死亡游戏为生_05() {
        let path = PathBuf::from("[TSDM字幕组] 以死亡游戏为生 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] 以死亡游戏为生 format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "以死亡游戏为生");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_tsdm_以死亡游戏为生_06() {
        let path = PathBuf::from("[TSDM字幕组] 以死亡游戏为生 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] 以死亡游戏为生 format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "以死亡游戏为生");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_tsdm_以死亡游戏为生_07() {
        let path = PathBuf::from("[TSDM字幕组] 以死亡游戏为生 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] 以死亡游戏为生 format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "以死亡游戏为生");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_tsdm_以死亡游戏为生_08() {
        let path = PathBuf::from("[TSDM字幕组] 以死亡游戏为生 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] 以死亡游戏为生 format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "以死亡游戏为生");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_tsdm_以死亡游戏为生_09() {
        let path = PathBuf::from("[TSDM字幕组] 以死亡游戏为生 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] 以死亡游戏为生 format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "以死亡游戏为生");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_tsdm_以死亡游戏为生_10() {
        let path = PathBuf::from("[TSDM字幕组] 以死亡游戏为生 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [TSDM字幕组] 以死亡游戏为生 format");
        assert_eq!(info.publisher, "TSDM字幕组");
        assert_eq!(info.anime_name, "以死亡游戏为生");
        assert_eq!(info.episode, "10");
    }

    // =====================================================
    // 夜莺家族&YYQ字幕组 tests (NEW publisher - 10 tests)
    // =====================================================
    #[test]
    fn test_parse_yyq_魔法少女小圆_01() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 魔法少女小圆 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 魔法少女小圆 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_yyq_魔法少女小圆_02() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 魔法少女小圆 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 魔法少女小圆 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_yyq_魔法少女小圆_03() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 魔法少女小圆 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 魔法少女小圆 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_yyq_魔法少女小圆_04() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 魔法少女小圆 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 魔法少女小圆 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_yyq_魔法少女小圆_05() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 魔法少女小圆 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 魔法少女小圆 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_yyq_魔法少女小圆_06() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 魔法少女小圆 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 魔法少女小圆 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_yyq_魔法少女小圆_07() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 魔法少女小圆 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 魔法少女小圆 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_yyq_魔法少女小圆_08() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 魔法少女小圆 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 魔法少女小圆 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_yyq_魔法少女小圆_09() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 魔法少女小圆 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 魔法少女小圆 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_yyq_魔法少女小圆_10() {
        let path = PathBuf::from("[夜莺家族&YYQ字幕组] 魔法少女小圆 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [夜莺家族&YYQ字幕组] 魔法少女小圆 format");
        assert_eq!(info.publisher, "夜莺家族&YYQ字幕组");
        assert_eq!(info.anime_name, "魔法少女小圆");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_樱花字幕组_我的青春恋爱物语_01() {
        let path = PathBuf::from("[樱花字幕组] 我的青春恋爱物语 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [樱花字幕组] format");
        assert_eq!(info.publisher, "樱花字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_樱花字幕组_我的青春恋爱物语_02() {
        let path = PathBuf::from("[樱花字幕组] 我的青春恋爱物语 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [樱花字幕组] format");
        assert_eq!(info.publisher, "樱花字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_樱花字幕组_我的青春恋爱物语_03() {
        let path = PathBuf::from("[樱花字幕组] 我的青春恋爱物语 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [樱花字幕组] format");
        assert_eq!(info.publisher, "樱花字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_樱花字幕组_我的青春恋爱物语_04() {
        let path = PathBuf::from("[樱花字幕组] 我的青春恋爱物语 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [樱花字幕组] format");
        assert_eq!(info.publisher, "樱花字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_樱花字幕组_我的青春恋爱物语_05() {
        let path = PathBuf::from("[樱花字幕组] 我的青春恋爱物语 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [樱花字幕组] format");
        assert_eq!(info.publisher, "樱花字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_樱花字幕组_我的青春恋爱物语_06() {
        let path = PathBuf::from("[樱花字幕组] 我的青春恋爱物语 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [樱花字幕组] format");
        assert_eq!(info.publisher, "樱花字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_樱花字幕组_我的青春恋爱物语_07() {
        let path = PathBuf::from("[樱花字幕组] 我的青春恋爱物语 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [樱花字幕组] format");
        assert_eq!(info.publisher, "樱花字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_樱花字幕组_我的青春恋爱物语_08() {
        let path = PathBuf::from("[樱花字幕组] 我的青春恋爱物语 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [樱花字幕组] format");
        assert_eq!(info.publisher, "樱花字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_樱花字幕组_我的青春恋爱物语_09() {
        let path = PathBuf::from("[樱花字幕组] 我的青春恋爱物语 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [樱花字幕组] format");
        assert_eq!(info.publisher, "樱花字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_樱花字幕组_我的青春恋爱物语_10() {
        let path = PathBuf::from("[樱花字幕组] 我的青春恋爱物语 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [樱花字幕组] format");
        assert_eq!(info.publisher, "樱花字幕组");
        assert_eq!(info.anime_name, "我的青春恋爱物语");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_萌菇字幕组_路人超能100_01() {
        let path = PathBuf::from("[萌菇字幕组] 路人超能100 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [萌菇字幕组] format");
        assert_eq!(info.publisher, "萌菇字幕组");
        assert_eq!(info.anime_name, "路人超能100");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_萌菇字幕组_路人超能100_02() {
        let path = PathBuf::from("[萌菇字幕组] 路人超能100 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [萌菇字幕组] format");
        assert_eq!(info.publisher, "萌菇字幕组");
        assert_eq!(info.anime_name, "路人超能100");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_萌菇字幕组_路人超能100_03() {
        let path = PathBuf::from("[萌菇字幕组] 路人超能100 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [萌菇字幕组] format");
        assert_eq!(info.publisher, "萌菇字幕组");
        assert_eq!(info.anime_name, "路人超能100");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_萌菇字幕组_路人超能100_04() {
        let path = PathBuf::from("[萌菇字幕组] 路人超能100 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [萌菇字幕组] format");
        assert_eq!(info.publisher, "萌菇字幕组");
        assert_eq!(info.anime_name, "路人超能100");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_萌菇字幕组_路人超能100_05() {
        let path = PathBuf::from("[萌菇字幕组] 路人超能100 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [萌菇字幕组] format");
        assert_eq!(info.publisher, "萌菇字幕组");
        assert_eq!(info.anime_name, "路人超能100");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_萌菇字幕组_路人超能100_06() {
        let path = PathBuf::from("[萌菇字幕组] 路人超能100 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [萌菇字幕组] format");
        assert_eq!(info.publisher, "萌菇字幕组");
        assert_eq!(info.anime_name, "路人超能100");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_萌菇字幕组_路人超能100_07() {
        let path = PathBuf::from("[萌菇字幕组] 路人超能100 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [萌菇字幕组] format");
        assert_eq!(info.publisher, "萌菇字幕组");
        assert_eq!(info.anime_name, "路人超能100");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_萌菇字幕组_路人超能100_08() {
        let path = PathBuf::from("[萌菇字幕组] 路人超能100 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [萌菇字幕组] format");
        assert_eq!(info.publisher, "萌菇字幕组");
        assert_eq!(info.anime_name, "路人超能100");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_萌菇字幕组_路人超能100_09() {
        let path = PathBuf::from("[萌菇字幕组] 路人超能100 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [萌菇字幕组] format");
        assert_eq!(info.publisher, "萌菇字幕组");
        assert_eq!(info.anime_name, "路人超能100");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_萌菇字幕组_路人超能100_10() {
        let path = PathBuf::from("[萌菇字幕组] 路人超能100 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [萌菇字幕组] format");
        assert_eq!(info.publisher, "萌菇字幕组");
        assert_eq!(info.anime_name, "路人超能100");
        assert_eq!(info.episode, "10");
    }

    #[test]
    fn test_parse_肥宅字幕组_银魂_01() {
        let path = PathBuf::from("[肥宅字幕组] 银魂 - 01 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [肥宅字幕组] format");
        assert_eq!(info.publisher, "肥宅字幕组");
        assert_eq!(info.anime_name, "银魂");
        assert_eq!(info.episode, "01");
    }

    #[test]
    fn test_parse_肥宅字幕组_银魂_02() {
        let path = PathBuf::from("[肥宅字幕组] 银魂 - 02 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [肥宅字幕组] format");
        assert_eq!(info.publisher, "肥宅字幕组");
        assert_eq!(info.anime_name, "银魂");
        assert_eq!(info.episode, "02");
    }

    #[test]
    fn test_parse_肥宅字幕组_银魂_03() {
        let path = PathBuf::from("[肥宅字幕组] 银魂 - 03 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [肥宅字幕组] format");
        assert_eq!(info.publisher, "肥宅字幕组");
        assert_eq!(info.anime_name, "银魂");
        assert_eq!(info.episode, "03");
    }

    #[test]
    fn test_parse_肥宅字幕组_银魂_04() {
        let path = PathBuf::from("[肥宅字幕组] 银魂 - 04 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [肥宅字幕组] format");
        assert_eq!(info.publisher, "肥宅字幕组");
        assert_eq!(info.anime_name, "银魂");
        assert_eq!(info.episode, "04");
    }

    #[test]
    fn test_parse_肥宅字幕组_银魂_05() {
        let path = PathBuf::from("[肥宅字幕组] 银魂 - 05 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [肥宅字幕组] format");
        assert_eq!(info.publisher, "肥宅字幕组");
        assert_eq!(info.anime_name, "银魂");
        assert_eq!(info.episode, "05");
    }

    #[test]
    fn test_parse_肥宅字幕组_银魂_06() {
        let path = PathBuf::from("[肥宅字幕组] 银魂 - 06 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [肥宅字幕组] format");
        assert_eq!(info.publisher, "肥宅字幕组");
        assert_eq!(info.anime_name, "银魂");
        assert_eq!(info.episode, "06");
    }

    #[test]
    fn test_parse_肥宅字幕组_银魂_07() {
        let path = PathBuf::from("[肥宅字幕组] 银魂 - 07 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [肥宅字幕组] format");
        assert_eq!(info.publisher, "肥宅字幕组");
        assert_eq!(info.anime_name, "银魂");
        assert_eq!(info.episode, "07");
    }

    #[test]
    fn test_parse_肥宅字幕组_银魂_08() {
        let path = PathBuf::from("[肥宅字幕组] 银魂 - 08 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [肥宅字幕组] format");
        assert_eq!(info.publisher, "肥宅字幕组");
        assert_eq!(info.anime_name, "银魂");
        assert_eq!(info.episode, "08");
    }

    #[test]
    fn test_parse_肥宅字幕组_银魂_09() {
        let path = PathBuf::from("[肥宅字幕组] 银魂 - 09 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [肥宅字幕组] format");
        assert_eq!(info.publisher, "肥宅字幕组");
        assert_eq!(info.anime_name, "银魂");
        assert_eq!(info.episode, "09");
    }

    #[test]
    fn test_parse_肥宅字幕组_银魂_10() {
        let path = PathBuf::from("[肥宅字幕组] 银魂 - 10 [1080P].mp4");
        let result = FilenameParser::parse(&path);
        let info = result.expect("Parser should handle [肥宅字幕组] format");
        assert_eq!(info.publisher, "肥宅字幕组");
        assert_eq!(info.anime_name, "银魂");
        assert_eq!(info.episode, "10");
    }
}
