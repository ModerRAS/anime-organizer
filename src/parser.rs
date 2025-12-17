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
    ///
    /// # 示例
    ///
    /// ```
    /// use anime_organizer::parser::AnimeFileInfo;
    ///
    /// let info = AnimeFileInfo {
    ///     publisher: "ANi".to_string(),
    ///     anime_name: "测试".to_string(),
    ///     episode: "01".to_string(),
    ///     tags: "[1080P]".to_string(),
    ///     extension: ".mp4".to_string(),
    ///     original_path: "/path/to/file".to_string(),
    /// };
    /// assert_eq!(info.target_filename(), "01 [1080P].mp4");
    /// ```
    #[must_use]
    pub fn target_filename(&self) -> String {
        format!("{} {}{}", self.episode, self.tags, self.extension)
    }
}

/// 文件名解析器
///
/// 使用正则表达式解析符合特定格式的动漫文件名。
pub struct FilenameParser;

/// 预编译的正则表达式
static ANIME_FILE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^\[(?P<publisher>[^\]]+)\]\s+(?P<anime>.+?)\s+-\s+(?P<episode>\d+)\s+(?P<tags>\[.+\])(?P<ext>\.\w+)$"
    ).expect("正则表达式编译失败")
});

impl FilenameParser {
    /// 解析文件路径，提取动漫文件信息
    ///
    /// # 参数
    ///
    /// * `file_path` - 文件路径
    ///
    /// # 返回值
    ///
    /// 如果文件名符合格式，返回 `Some(AnimeFileInfo)`；否则返回 `None`。
    ///
    /// # 示例
    ///
    /// ```
    /// use anime_organizer::parser::FilenameParser;
    /// use std::path::Path;
    ///
    /// let path = Path::new("/downloads/[SubsPlease] 间谍过家家 - 12 [1080p].mkv");
    /// let info = FilenameParser::parse(path).unwrap();
    ///
    /// assert_eq!(info.publisher, "SubsPlease");
    /// assert_eq!(info.anime_name, "间谍过家家");
    /// assert_eq!(info.episode, "12");
    /// assert_eq!(info.tags, "[1080p]");
    /// assert_eq!(info.extension, ".mkv");
    /// ```
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
        let path = PathBuf::from("test/[ANi] 妖怪旅館營業中 貳 - 07 [1080P][Baha][WEB-DL][AAC AVC][CHT].mp4");
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
        let path = PathBuf::from("test/[EMBER] 进击的巨人 The Final Season - 01 [1080p][Multiple Subtitle].avi");
        let result = FilenameParser::parse(&path).unwrap();

        assert_eq!(result.publisher, "EMBER");
        assert_eq!(result.anime_name, "进击的巨人 The Final Season");
        assert_eq!(result.episode, "01");
        assert_eq!(result.tags, "[1080p][Multiple Subtitle]");
        assert_eq!(result.extension, ".avi");
    }

    #[test]
    fn test_parse_single_digit_episode_pads_with_zero() {
        let test_cases = vec![
            ("[ANi] 测试 - 1 [Tag].mp4", "01"),
            ("[ANi] 测试 - 5 [Tag].mp4", "05"),
            ("[ANi] 测试 - 9 [Tag].mp4", "09"),
            ("[ANi] 测试 - 10 [Tag].mp4", "10"),
        ];

        for (filename, expected_episode) in test_cases {
            let path = PathBuf::from(format!("test/{}", filename));
            let result = FilenameParser::parse(&path).unwrap();
            assert_eq!(result.episode, expected_episode, "文件名: {}", filename);
        }
    }

    #[test]
    fn test_parse_invalid_filename_returns_none() {
        let invalid_filenames = vec![
            "测试 - 01.mp4",
            "[ANi] 测试.mp4",
            "测试 - 01 [Tag].mp4",
            "[ANi] 测试 - 01 Tag.mp4",
            "",
            "random_file.txt",
        ];

        for filename in invalid_filenames {
            let path = PathBuf::from(format!("test/{}", filename));
            let result = FilenameParser::parse(&path);
            assert!(result.is_none(), "应返回 None: {}", filename);
        }
    }

    #[test]
    fn test_parse_extension_normalized_to_lowercase() {
        let test_cases = vec![
            ("[ANi] 测试 - 01 [Tag].MP4", ".mp4"),
            ("[ANi] 测试 - 01 [Tag].Mp4", ".mp4"),
            ("[ANi] 测试 - 01 [Tag].MKV", ".mkv"),
        ];

        for (filename, expected_ext) in test_cases {
            let path = PathBuf::from(format!("test/{}", filename));
            let result = FilenameParser::parse(&path).unwrap();
            assert_eq!(result.extension, expected_ext, "文件名: {}", filename);
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
}
