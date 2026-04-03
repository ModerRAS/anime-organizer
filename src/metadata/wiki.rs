//! Wiki Infobox 解析模块
//!
//! 解析 Bangumi 的 Wiki Infobox 格式，提取动漫的详细信息。

use regex::Regex;
use std::sync::LazyLock;

/// Infobox 解析结果
#[derive(Debug, Clone, Default)]
pub struct AnimeInfobox {
    /// 中文名
    pub name_cn: Option<String>,
    /// 集数
    pub episode_count: Option<u32>,
    /// 放送开始日期
    pub air_date_start: Option<String>,
    /// 放送结束日期
    pub air_date_end: Option<String>,
    /// 导演
    pub director: Option<String>,
    /// 动画制作公司
    pub studio: Option<String>,
    /// 音乐
    pub music: Option<String>,
    /// 系列构成
    pub series_composition: Option<String>,
    /// 原作
    pub original_work: Option<String>,
    /// 脚本
    pub script: Option<String>,
}

/// Wiki Infobox 解析器
pub struct WikiParser;

// 预编译正则表达式
static RE_FIELD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\|(.+?)=(.+?)(?:\r?\n|\||\}|$)").expect("正则表达式编译失败"));

static RE_EPISODE_COUNT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"话数=(\d+)").expect("正则表达式编译失败"));

static RE_SIMPLE_FIELD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\|([^=]+)=(.+)$").expect("正则表达式编译失败"));

impl WikiParser {
    /// 解析 Wiki Infobox 文本
    ///
    /// 支持 `{{Infobox animanga/TVAnime|...}}` 等格式。
    ///
    /// # 参数
    ///
    /// * `infobox` - Wiki Infobox 原始文本
    ///
    /// # 返回值
    ///
    /// 返回解析后的 `AnimeInfobox` 结构体。
    pub fn parse(infobox: &str) -> AnimeInfobox {
        let mut result = AnimeInfobox::default();

        // 解析各字段
        let fields = Self::extract_fields(infobox);

        for (key, value) in &fields {
            let key_trimmed = key.trim();
            let value_trimmed = value.trim();

            if value_trimmed.is_empty() {
                continue;
            }

            match key_trimmed {
                "中文名" => result.name_cn = Some(value_trimmed.to_string()),
                "话数" => {
                    result.episode_count = value_trimmed.parse::<u32>().ok();
                }
                "放送开始" => result.air_date_start = Some(Self::clean_date(value_trimmed)),
                "放送结束" => result.air_date_end = Some(Self::clean_date(value_trimmed)),
                "导演" | "監督" => result.director = Some(value_trimmed.to_string()),
                "动画制作" | "アニメーション制作" => {
                    result.studio = Some(value_trimmed.to_string());
                }
                "音乐" | "音楽" => result.music = Some(value_trimmed.to_string()),
                "系列构成" | "シリーズ構成" => {
                    result.series_composition = Some(value_trimmed.to_string());
                }
                "原作" => result.original_work = Some(value_trimmed.to_string()),
                "脚本" => result.script = Some(value_trimmed.to_string()),
                _ => {}
            }
        }

        result
    }

    /// 提取 Infobox 中所有键值对
    fn extract_fields(infobox: &str) -> Vec<(String, String)> {
        let mut fields = Vec::new();

        // 尝试行级解析：每行 |key=value 格式
        for cap in RE_SIMPLE_FIELD.captures_iter(infobox) {
            if let (Some(key), Some(value)) = (cap.get(1), cap.get(2)) {
                fields.push((key.as_str().to_string(), value.as_str().to_string()));
            }
        }

        // 如果行级解析无结果，尝试内联解析
        if fields.is_empty() {
            for cap in RE_FIELD.captures_iter(infobox) {
                if let (Some(key), Some(value)) = (cap.get(1), cap.get(2)) {
                    fields.push((key.as_str().to_string(), value.as_str().to_string()));
                }
            }
        }

        // 还有话数的特殊解析
        if !fields.iter().any(|(k, _)| k.trim() == "话数") {
            if let Some(cap) = RE_EPISODE_COUNT.captures(infobox) {
                if let Some(count) = cap.get(1) {
                    fields.push(("话数".to_string(), count.as_str().to_string()));
                }
            }
        }

        fields
    }

    /// 清理日期字符串
    ///
    /// 去除 "日" 等后缀，统一格式。
    fn clean_date(date: &str) -> String {
        date.trim_end_matches('日').trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tv_anime_infobox() {
        let infobox = r#"{{Infobox animanga/TVAnime
|中文名=Code Geass 反叛的鲁路修R2
|话数=25
|放送开始=2008-04-06日
|放送结束=2008-09-28日
|导演=谷口悟朗
|动画制作=サンライズ
|音乐=中川幸太郎
|原作=大河内一楼 / 谷口悟朗
}}"#;
        let result = WikiParser::parse(infobox);
        assert_eq!(
            result.name_cn.as_deref(),
            Some("Code Geass 反叛的鲁路修R2")
        );
        assert_eq!(result.episode_count, Some(25));
        assert_eq!(result.air_date_start.as_deref(), Some("2008-04-06"));
        assert_eq!(result.director.as_deref(), Some("谷口悟朗"));
        assert_eq!(result.studio.as_deref(), Some("サンライズ"));
        assert_eq!(result.music.as_deref(), Some("中川幸太郎"));
    }

    #[test]
    fn test_parse_missing_fields() {
        let infobox = r#"{{Infobox animanga/TVAnime
|中文名=测试动画
}}"#;
        let result = WikiParser::parse(infobox);
        assert_eq!(result.name_cn.as_deref(), Some("测试动画"));
        assert_eq!(result.episode_count, None);
        assert_eq!(result.director, None);
        assert_eq!(result.studio, None);
    }

    #[test]
    fn test_parse_ova_infobox() {
        let infobox = r#"{{Infobox animanga/OVA
|中文名=某OVA
|话数=3
|导演=某导演
|动画制作=A-1 Pictures
}}"#;
        let result = WikiParser::parse(infobox);
        assert_eq!(result.name_cn.as_deref(), Some("某OVA"));
        assert_eq!(result.episode_count, Some(3));
        assert_eq!(result.director.as_deref(), Some("某导演"));
        assert_eq!(result.studio.as_deref(), Some("A-1 Pictures"));
    }

    #[test]
    fn test_parse_japanese_field_names() {
        let infobox = r#"{{Infobox animanga/TVAnime
|中文名=测试
|監督=新房昭之
|アニメーション制作=シャフト
|音楽=梶浦由記
|シリーズ構成=虚淵玄
}}"#;
        let result = WikiParser::parse(infobox);
        assert_eq!(result.director.as_deref(), Some("新房昭之"));
        assert_eq!(result.studio.as_deref(), Some("シャフト"));
        assert_eq!(result.music.as_deref(), Some("梶浦由記"));
        assert_eq!(result.series_composition.as_deref(), Some("虚淵玄"));
    }

    #[test]
    fn test_parse_empty_infobox() {
        let result = WikiParser::parse("");
        assert!(result.name_cn.is_none());
        assert!(result.episode_count.is_none());
    }

    #[test]
    fn test_clean_date() {
        assert_eq!(WikiParser::clean_date("2008-04-06日"), "2008-04-06");
        assert_eq!(WikiParser::clean_date("2008-04-06"), "2008-04-06");
    }
}
