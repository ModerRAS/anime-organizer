//! Wiki Infobox 解析器
//!
//! 解析 Bangumi 的 Wiki Infobox 格式，提取结构化的动画信息。
//!
//! ## Infobox 格式
//!
//! Bangumi 的 Wiki Infobox 使用自定义格式：
//! ```text
//! {{Infobox animanga/TVAnime
//! |中文名= 孤独摇滚！
//! |别名={
//! [ぼっち・ざ・ろっく！]
//! [Bocchi the Rock!]
//! }
//! |话数= 12
//! |放送开始= 2022年10月8日
//! |动画制作= CloverWorks
//! |导演= 斎藤圭一郎
//! }}
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// 动画 Infobox 结构化信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnimeInfobox {
    /// Infobox 类型（如 TVAnime, OVA, Movie 等）
    pub infobox_type: Option<String>,
    /// 中文名
    pub name_cn: Option<String>,
    /// 别名列表
    pub aliases: Vec<String>,
    /// 话数
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
    /// 原始键值对（所有字段）
    pub raw: HashMap<String, String>,
}

/// Wiki Infobox 解析器
///
/// 将 Bangumi Wiki Infobox 文本解析为键值对映射。
pub struct WikiParser;

impl WikiParser {
    /// 创建新的解析器实例
    pub fn new() -> Self {
        Self
    }

    /// 解析 Wiki Infobox 文本为原始键值对
    ///
    /// # 参数
    ///
    /// - `infobox` - Wiki Infobox 原始文本
    ///
    /// # 返回
    ///
    /// 键值对映射，其中多值字段以 ` / ` 连接。
    pub fn parse(&self, infobox: &str) -> Result<HashMap<String, String>> {
        let mut result = HashMap::new();
        let mut current_key: Option<String> = None;
        let mut multi_values: Vec<String> = Vec::new();
        let mut in_multi_value = false;

        for line in infobox.lines() {
            let line = line.trim();

            // 跳过 Infobox 开头和结尾
            if line.starts_with("{{") || line == "}}" {
                // 保存最后一个多值字段
                if in_multi_value {
                    if let Some(ref key) = current_key {
                        if !multi_values.is_empty() {
                            result.insert(key.clone(), multi_values.join(" / "));
                        }
                    }
                }
                continue;
            }

            // 多值块结束
            if line == "}" {
                in_multi_value = false;
                if let Some(ref key) = current_key {
                    if !multi_values.is_empty() {
                        result.insert(key.clone(), multi_values.join(" / "));
                    }
                }
                multi_values.clear();
                continue;
            }

            // 多值块中的条目
            if in_multi_value {
                let value = line.trim_start_matches('[').trim_end_matches(']').trim();
                if !value.is_empty() {
                    multi_values.push(value.to_string());
                }
                continue;
            }

            // 键值对行
            if let Some(stripped) = line.strip_prefix('|') {
                if let Some((key, value)) = stripped.split_once('=') {
                    let key = key.trim().to_string();
                    let value = value.trim();

                    current_key = Some(key.clone());

                    if value == "{" {
                        // 开始多值块
                        in_multi_value = true;
                        multi_values.clear();
                    } else if !value.is_empty() {
                        result.insert(key, value.to_string());
                    }
                }
            }
        }

        if result.is_empty() {
            return Err(AppError::WikiParseError("Infobox 解析结果为空".to_string()));
        }

        Ok(result)
    }

    /// 解析 Wiki Infobox 并提取结构化的动画信息
    ///
    /// # 示例
    ///
    /// ```
    /// use anime_organizer::metadata::wiki::WikiParser;
    ///
    /// let infobox = r#"{{Infobox animanga/TVAnime
    /// |中文名= 孤独摇滚！
    /// |话数= 12
    /// |放送开始= 2022年10月8日
    /// |动画制作= CloverWorks
    /// |导演= 斎藤圭一郎
    /// }}"#;
    ///
    /// let parser = WikiParser::new();
    /// let info = parser.parse_anime_infobox(infobox).unwrap();
    /// assert_eq!(info.episode_count, Some(12));
    /// assert_eq!(info.studio.as_deref(), Some("CloverWorks"));
    /// ```
    pub fn parse_anime_infobox(&self, infobox: &str) -> Result<AnimeInfobox> {
        let raw = self.parse(infobox)?;
        let mut info = AnimeInfobox {
            raw: raw.clone(),
            ..Default::default()
        };

        // 提取 Infobox 类型
        for line in infobox.lines() {
            let line = line.trim();
            if line.starts_with("{{Infobox") {
                let type_str = line
                    .trim_start_matches("{{Infobox")
                    .trim_start_matches(" animanga/")
                    .trim_start_matches(' ');
                if !type_str.is_empty() {
                    info.infobox_type = Some(type_str.to_string());
                }
                break;
            }
        }

        // 提取已知字段
        info.name_cn = raw.get("中文名").cloned();
        info.director = raw.get("导演").cloned();
        info.studio = raw.get("动画制作").cloned();
        info.music = raw.get("音乐").cloned();
        info.series_composition = raw.get("系列构成").cloned();
        info.air_date_start = raw.get("放送开始").cloned();
        info.air_date_end = raw.get("放送结束").cloned();

        // 解析话数（数字提取）
        if let Some(eps_str) = raw.get("话数") {
            info.episode_count = eps_str
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .ok();
        }

        // 解析别名（多值字段已用 " / " 连接）
        if let Some(aliases) = raw.get("别名") {
            info.aliases = aliases.split(" / ").map(|s| s.trim().to_string()).collect();
        }

        Ok(info)
    }
}

impl Default for WikiParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tv_anime_infobox() {
        let infobox = r#"{{Infobox animanga/TVAnime
|中文名= 孤独摇滚！
|别名={
[ぼっち・ざ・ろっく！]
[Bocchi the Rock!]
}
|话数= 12
|放送开始= 2022年10月8日
|放送结束= 2022年12月24日
|动画制作= CloverWorks
|导演= 斎藤圭一郎
|系列构成= 吉田恵里香
|音乐= 菊谷知樹
}}"#;

        let parser = WikiParser::new();
        let info = parser.parse_anime_infobox(infobox).unwrap();

        assert_eq!(info.infobox_type.as_deref(), Some("TVAnime"));
        assert_eq!(info.name_cn.as_deref(), Some("孤独摇滚！"));
        assert_eq!(info.episode_count, Some(12));
        assert_eq!(info.air_date_start.as_deref(), Some("2022年10月8日"));
        assert_eq!(info.air_date_end.as_deref(), Some("2022年12月24日"));
        assert_eq!(info.studio.as_deref(), Some("CloverWorks"));
        assert_eq!(info.director.as_deref(), Some("斎藤圭一郎"));
        assert_eq!(info.series_composition.as_deref(), Some("吉田恵里香"));
        assert_eq!(info.music.as_deref(), Some("菊谷知樹"));
        assert_eq!(info.aliases.len(), 2);
        assert!(info.aliases.contains(&"ぼっち・ざ・ろっく！".to_string()));
        assert!(info.aliases.contains(&"Bocchi the Rock!".to_string()));
    }

    #[test]
    fn test_parse_code_geass_r2_infobox() {
        let infobox = r#"{{Infobox animanga/TVAnime
|中文名= Code Geass 反叛的鲁路修R2
|话数= 25
|放送开始= 2008-04-06日
|导演= 谷口悟朗
|动画制作= サンライズ
}}"#;

        let parser = WikiParser::new();
        let info = parser.parse_anime_infobox(infobox).unwrap();

        assert_eq!(info.episode_count, Some(25));
        assert_eq!(info.director.as_deref(), Some("谷口悟朗"));
        assert_eq!(info.studio.as_deref(), Some("サンライズ"));
    }

    #[test]
    fn test_parse_ova_infobox() {
        let infobox = r#"{{Infobox animanga/OVA
|中文名= 某OVA作品
|话数= 2
|动画制作= MAPPA
}}"#;

        let parser = WikiParser::new();
        let info = parser.parse_anime_infobox(infobox).unwrap();

        assert_eq!(info.infobox_type.as_deref(), Some("OVA"));
        assert_eq!(info.episode_count, Some(2));
        assert_eq!(info.studio.as_deref(), Some("MAPPA"));
    }

    #[test]
    fn test_parse_movie_infobox() {
        let infobox = r#"{{Infobox animanga/Movie
|中文名= 铃芽之旅
|导演= 新海诚
|动画制作= CoMix Wave Films
}}"#;

        let parser = WikiParser::new();
        let info = parser.parse_anime_infobox(infobox).unwrap();

        assert_eq!(info.infobox_type.as_deref(), Some("Movie"));
        assert_eq!(info.name_cn.as_deref(), Some("铃芽之旅"));
        assert_eq!(info.director.as_deref(), Some("新海诚"));
        assert_eq!(info.episode_count, None);
    }

    #[test]
    fn test_parse_missing_fields() {
        let infobox = r#"{{Infobox animanga/TVAnime
|中文名= 最简信息
}}"#;

        let parser = WikiParser::new();
        let info = parser.parse_anime_infobox(infobox).unwrap();

        assert_eq!(info.name_cn.as_deref(), Some("最简信息"));
        assert_eq!(info.episode_count, None);
        assert_eq!(info.director, None);
        assert_eq!(info.studio, None);
        assert_eq!(info.music, None);
    }

    #[test]
    fn test_parse_empty_infobox() {
        let parser = WikiParser::new();
        let result = parser.parse_anime_infobox("{{Infobox animanga/TVAnime\n}}");
        assert!(result.is_err());
    }

    #[test]
    fn test_raw_parse_preserves_all_fields() {
        let infobox = r#"{{Infobox animanga/TVAnime
|中文名= 测试
|原作= 某作者
|脚本= 某脚本
}}"#;

        let parser = WikiParser::new();
        let raw = parser.parse(infobox).unwrap();

        assert_eq!(raw.get("原作").map(|s| s.as_str()), Some("某作者"));
        assert_eq!(raw.get("脚本").map(|s| s.as_str()), Some("某脚本"));
    }
}
