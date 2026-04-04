//! LLM 辅助别名匹配
//!
//! 将最近刮削的动画数据 + 现有别名库 + Bangumi 参考数据发送给 LLM，
//! 让 LLM 推理出新的别名关系。
//!
//! ## 工作流
//!
//! 1. 收集多数据源的最近动画列表
//! 2. 加载现有别名库
//! 3. 构造 prompt 发送给 LLM
//! 4. 解析 LLM 返回的 JSON，分为确信匹配和待审核匹配

use serde::{Deserialize, Serialize};

use super::sources::ScrapedAnime;
use crate::error::{AppError, Result};
use crate::metadata::alias::AliasEntry;

/// 匹配置信度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MatchConfidence {
    /// 高置信度（>90%），可直接创建 PR
    High,
    /// 中置信度（50-90%），需要人工审核
    Medium,
    /// 低置信度（<50%），创建 Issue 讨论
    Low,
}

impl std::fmt::Display for MatchConfidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Low => write!(f, "low"),
        }
    }
}

/// 别名匹配提案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// 字幕组/粉丝翻译名称
    pub fan_translation: String,
    /// 匹配到的别名条目
    pub alias_entry: AliasEntry,
    /// 置信度
    pub confidence: MatchConfidence,
    /// LLM 推理理由
    pub reasoning: String,
    /// 数据来源
    pub source: String,
}

/// 匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    /// 高置信度匹配（可直接 PR）
    pub confident: Vec<Proposal>,
    /// 待审核匹配（需创建 Issue）
    pub uncertain: Vec<Proposal>,
}

impl MatchResult {
    /// 创建空的匹配结果
    pub fn empty() -> Self {
        Self {
            confident: Vec::new(),
            uncertain: Vec::new(),
        }
    }

    /// 是否没有任何匹配
    pub fn is_empty(&self) -> bool {
        self.confident.is_empty() && self.uncertain.is_empty()
    }
}

/// 构建 LLM prompt
///
/// 将刮削数据、现有别名、Bangumi 参考数据组装成结构化 prompt。
pub fn build_prompt(
    scraped: &[ScrapedAnime],
    existing_aliases: &std::collections::HashMap<String, AliasEntry>,
) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are an anime alias matching assistant. Your job is to find new aliases for anime that are not yet in the alias library.\n\n");

    prompt.push_str("## Existing Aliases\n\n");
    for (key, entry) in existing_aliases.iter().take(100) {
        prompt.push_str(&format!(
            "- \"{}\" → bangumi_id={}, name=\"{}\"\n",
            key, entry.bangumi_id, entry.name
        ));
    }

    prompt.push_str("\n## Recently Scraped Anime\n\n");
    for anime in scraped.iter().take(50) {
        prompt.push_str(&format!(
            "- title=\"{}\", title_cn={}, source={}, bangumi_id={}, tmdb_id={}\n",
            anime.title,
            anime.title_cn.as_deref().unwrap_or("N/A"),
            anime.source,
            anime
                .bangumi_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            anime
                .tmdb_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "N/A".to_string()),
        ));
    }

    prompt.push_str("\n## Task\n\n");
    prompt.push_str("Find entries in the scraped list where the fan translation name (usually from DMHY) differs from the standard Bangumi name, and IS NOT already in the existing aliases list.\n\n");
    prompt.push_str("Return a JSON array of objects with these fields:\n");
    prompt.push_str("- fan_translation: the fan/subtitle group name\n");
    prompt.push_str("- bangumi_id: the Bangumi subject ID\n");
    prompt.push_str("- name: the standard Japanese/Bangumi name\n");
    prompt.push_str("- confidence: \"high\" (>90%), \"medium\" (50-90%), or \"low\" (<50%)\n");
    prompt.push_str("- reasoning: brief explanation\n\n");
    prompt.push_str("Return ONLY the JSON array, no other text.\n");

    prompt
}

/// 解析 LLM 返回的 JSON 为匹配结果
pub fn parse_llm_response(response: &str) -> Result<MatchResult> {
    // 提取 JSON 数组（LLM 可能会包含额外文本）
    let json_str = extract_json_array(response)
        .ok_or_else(|| AppError::BangumiParseError("LLM 响应中未找到 JSON 数组".to_string()))?;

    let proposals: Vec<serde_json::Value> = serde_json::from_str(&json_str)
        .map_err(|e| AppError::BangumiParseError(format!("LLM JSON 解析失败: {e}")))?;

    let mut result = MatchResult::empty();

    for item in proposals {
        let fan_translation = item
            .get("fan_translation")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let bangumi_id = item.get("bangumi_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let confidence_str = item
            .get("confidence")
            .and_then(|v| v.as_str())
            .unwrap_or("low");

        let confidence = match confidence_str {
            "high" => MatchConfidence::High,
            "medium" => MatchConfidence::Medium,
            _ => MatchConfidence::Low,
        };

        let reasoning = item
            .get("reasoning")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let proposal = Proposal {
            fan_translation,
            alias_entry: AliasEntry {
                bangumi_id,
                name,
                tmdb_id: None,
                anidb_id: None,
            },
            confidence: confidence.clone(),
            reasoning,
            source: "LLM".to_string(),
        };

        match confidence {
            MatchConfidence::High => result.confident.push(proposal),
            _ => result.uncertain.push(proposal),
        }
    }

    Ok(result)
}

/// 从文本中提取 JSON 数组
fn extract_json_array(text: &str) -> Option<String> {
    let start = text.find('[')?;
    let mut depth = 0;
    let mut end = start;

    for (i, ch) in text[start..].char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    end = start + i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    if depth == 0 && end > start {
        Some(text[start..end].to_string())
    } else {
        None
    }
}

/// 将匹配结果格式化为 GitHub 格式输出
///
/// 用于 GitHub Actions 中设置 output 变量。
pub fn format_github_output(result: &MatchResult) -> String {
    let mut output = String::new();

    output.push_str(&format!("confident_count={}\n", result.confident.len()));
    output.push_str(&format!("uncertain_count={}\n", result.uncertain.len()));

    if !result.confident.is_empty() {
        let json = serde_json::to_string_pretty(&result.confident).unwrap_or_default();
        output.push_str(&format!("confident_proposals={json}\n"));
    }

    if !result.uncertain.is_empty() {
        let json = serde_json::to_string_pretty(&result.uncertain).unwrap_or_default();
        output.push_str(&format!("uncertain_proposals={json}\n"));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_array() {
        let text = r#"Here are the results: [{"a": 1}, {"b": 2}] done"#;
        let json = extract_json_array(text).unwrap();
        assert_eq!(json, r#"[{"a": 1}, {"b": 2}]"#);
    }

    #[test]
    fn test_extract_json_array_nested() {
        let text = r#"[{"a": [1, 2]}, {"b": 3}]"#;
        let json = extract_json_array(text).unwrap();
        assert_eq!(json, text);
    }

    #[test]
    fn test_extract_json_array_none() {
        assert!(extract_json_array("no json here").is_none());
    }

    #[test]
    fn test_parse_llm_response() {
        let response = r#"[
            {
                "fan_translation": "孤独摇滚",
                "bangumi_id": 378862,
                "name": "ぼっち・ざ・ろっく！",
                "confidence": "high",
                "reasoning": "孤独摇滚 is the Chinese fan translation of Bocchi the Rock"
            },
            {
                "fan_translation": "某新番",
                "bangumi_id": 99999,
                "name": "某アニメ",
                "confidence": "medium",
                "reasoning": "Uncertain match"
            }
        ]"#;

        let result = parse_llm_response(response).unwrap();
        assert_eq!(result.confident.len(), 1);
        assert_eq!(result.uncertain.len(), 1);
        assert_eq!(result.confident[0].fan_translation, "孤独摇滚");
        assert_eq!(result.confident[0].alias_entry.bangumi_id, 378862);
        assert_eq!(result.confident[0].confidence, MatchConfidence::High);
    }

    #[test]
    fn test_parse_llm_response_empty() {
        let result = parse_llm_response("[]").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_match_result_empty() {
        let result = MatchResult::empty();
        assert!(result.is_empty());
        assert_eq!(result.confident.len(), 0);
        assert_eq!(result.uncertain.len(), 0);
    }

    #[test]
    fn test_build_prompt_contains_structure() {
        let scraped = vec![ScrapedAnime {
            title: "Test Anime".to_string(),
            title_cn: Some("测试动画".to_string()),
            date: Some("2024-01-01".to_string()),
            source: super::super::sources::ScrapedSource::Bangumi,
            source_url: None,
            bangumi_id: Some(12345),
            tmdb_id: None,
        }];

        let aliases = std::collections::HashMap::new();
        let prompt = build_prompt(&scraped, &aliases);

        assert!(prompt.contains("Existing Aliases"));
        assert!(prompt.contains("Recently Scraped Anime"));
        assert!(prompt.contains("Test Anime"));
        assert!(prompt.contains("测试动画"));
    }

    #[test]
    fn test_format_github_output() {
        let result = MatchResult {
            confident: vec![Proposal {
                fan_translation: "test".to_string(),
                alias_entry: AliasEntry {
                    bangumi_id: 1,
                    name: "テスト".to_string(),
                    tmdb_id: None,
                    anidb_id: None,
                },
                confidence: MatchConfidence::High,
                reasoning: "test".to_string(),
                source: "LLM".to_string(),
            }],
            uncertain: Vec::new(),
        };

        let output = format_github_output(&result);
        assert!(output.contains("confident_count=1"));
        assert!(output.contains("uncertain_count=0"));
    }

    #[test]
    fn test_match_confidence_display() {
        assert_eq!(MatchConfidence::High.to_string(), "high");
        assert_eq!(MatchConfidence::Medium.to_string(), "medium");
        assert_eq!(MatchConfidence::Low.to_string(), "low");
    }
}
