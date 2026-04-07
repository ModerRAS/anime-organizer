//! 正则表达式过滤模块
//!
//! 提供 RSS item 标题的过滤匹配功能。

use regex::{Regex, RegexBuilder};

use crate::error::{AppError, Result};

/// RSS 标题过滤器
///
/// 使用编译后的正则表达式对 RSS 条目标题进行匹配。
/// 默认大小写不敏感。
#[derive(Debug, Clone)]
pub struct RssFilter {
    regex: Regex,
}

impl RssFilter {
    /// 创建新的 RSS 过滤器
    ///
    /// # 参数
    /// * `pattern` - 正则表达式模式
    ///
    /// # 返回
    /// * 成功返回包含编译后正则表达式的 `RssFilter`
    /// * 失败返回 `AppError::ParseError`
    ///
    /// # 示例
    /// ```
    /// use anime_organizer::rss::filter::RssFilter;
    ///
    /// let filter = RssFilter::new(r"\[ANi\]").unwrap();
    /// assert!(filter.matches("[ANi] 某动漫 - 01"));
    /// ```
    pub fn new(pattern: &str) -> Result<Self> {
        let regex = RegexBuilder::new(pattern)
            .case_insensitive(true)
            .build()
            .map_err(|e| AppError::ParseError(format!("无效的正则表达式 '{}': {}", pattern, e)))?;
        Ok(Self { regex })
    }

    /// 检查标题是否匹配过滤器
    ///
    /// # 参数
    /// * `title` - RSS 条目标题
    ///
    /// # 返回
    /// * 标题匹配正则表达式返回 `true`
    /// * 否则返回 `false`
    pub fn matches(&self, title: &str) -> bool {
        self.regex.is_match(title)
    }
}

/// 检查标题是否匹配过滤器（便捷函数）
///
/// 如果没有设置过滤器，则返回 `true`（表示不过滤）。
///
/// # 参数
/// * `title` - RSS 条目标题
/// * `filter` - 可选的 RSS 过滤器引用
///
/// # 返回
/// * 没有过滤器时始终返回 `true`
/// * 有过滤器时返回标题是否匹配
pub fn matches_filter(title: &str, filter: &Option<RssFilter>) -> bool {
    match filter {
        None => true,
        Some(f) => f.matches(title),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_matches_expected_title() {
        let filter = RssFilter::new(r"\[ANi\]").unwrap();
        assert!(filter.matches("[ANi] 妖怪旅馆营业中 - 01"));
        assert!(filter.matches("[ani] 妖怪旅馆营业中 - 01")); // 大小写不敏感
    }

    #[test]
    fn test_filter_rejects_non_matching_title() {
        let filter = RssFilter::new(r"\[ANi\]").unwrap();
        assert!(!filter.matches("[SubsPlease] 某动漫 - 01"));
        assert!(!filter.matches("普通文件名.mp4"));
    }

    #[test]
    fn test_no_filter_matches_everything() {
        let title = "[ANi] 某动漫 - 01 [1080P].mkv";
        assert!(matches_filter(title, &None));
    }

    #[test]
    fn test_invalid_regex_returns_error() {
        let result = RssFilter::new(r"["); // 无效的正则表达式
        assert!(result.is_err());
        if let Err(AppError::ParseError(msg)) = result {
            assert!(msg.contains("无效的正则表达式"));
        } else {
            panic!("期望 ParseError 错误类型");
        }
    }

    #[test]
    fn test_filter_with_complex_pattern() {
        let filter = RssFilter::new(r"\d{2}\s*\[.*?\]").unwrap();
        assert!(filter.matches("01 [1080P]"));
        assert!(filter.matches("12  [720P]"));
        assert!(!filter.matches("no episode number"));
    }

    #[test]
    fn test_matches_filter_with_filter() {
        let filter = RssFilter::new(r"\[ANi\]").unwrap();
        let filter_clone = filter.clone();
        assert!(matches_filter("[ANi] 某动漫 - 01", &Some(filter)));
        assert!(!matches_filter(
            "[SubsPlease] 某动漫 - 01",
            &Some(filter_clone)
        ));
    }
}
