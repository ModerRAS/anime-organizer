#[derive(Debug, Default)]
pub struct InfoboxFields {
    pub studio: Option<String>,
    pub director: Option<String>,
    pub animation_production: Option<String>,
    pub series_composition: Option<String>,
    pub music: Option<String>,
    pub episode_count: Option<String>,
    pub aliases: Vec<String>,
}

impl InfoboxFields {
    pub fn parse(wiki_text: &str) -> Self {
        parse_infobox(wiki_text)
    }

    pub fn extract_aliases(&self) -> Vec<String> {
        self.aliases.clone()
    }
}

fn parse_infobox(text: &str) -> InfoboxFields {
    let mut fields = InfoboxFields::default();

    if text.is_empty() {
        return fields;
    }

    let text = text.trim();
    if !text.starts_with("{{Infobox") {
        return fields;
    }

    if !text.ends_with("}}") {
        return fields;
    }

    let mut in_array = false;
    let mut current_array_aliases: Vec<String> = Vec::new();

    for line in text.split('\n') {
        let line = line.trim();
        if line.is_empty() || line == "{{Infobox" || line == "}}" {
            continue;
        }

        let line = line.strip_prefix('|').unwrap_or(line);

        if line == "{" {
            in_array = true;
            current_array_aliases.clear();
            continue;
        }

        if line == "}" {
            if in_array && !current_array_aliases.is_empty() {
                fields.aliases.append(&mut current_array_aliases);
            }
            in_array = false;
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            if value.is_empty() {
                continue;
            }

            match key {
                "制作公司" => fields.studio = Some(cleanup_value(value)),
                "导演" | "监督" => {
                    if fields.director.is_none() {
                        fields.director = Some(cleanup_value(value));
                    }
                }
                "动画制作" => fields.animation_production = Some(cleanup_value(value)),
                "系列构成" => fields.series_composition = Some(cleanup_value(value)),
                "音乐" => fields.music = Some(cleanup_value(value)),
                "话数" => fields.episode_count = Some(cleanup_value(value)),
                "别名" | "中文名" | "简体中文名" | "繁体中文名" | "英文名" | "日文名" => {
                    if in_array {
                        current_array_aliases.push(cleanup_value(value));
                    } else if value != "{" {
                        fields.aliases.push(cleanup_value(value));
                    }
                }
                _ => {}
            }
        }
    }

    fields
}

pub(crate) fn cleanup_value(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }

    if value.starts_with("[[") && value.ends_with("]]") {
        let inner = &value[2..value.len() - 2];
        if let Some((_, rest)) = inner.split_once('|') {
            return rest.trim().to_string();
        }
        return inner.trim().to_string();
    }

    if value.starts_with('[') && value.ends_with(']') {
        if let Some((_, rest)) = value.split_once('|') {
            if let Some((first, _)) = rest.split_once('[') {
                if !first.is_empty() {
                    return first.trim().to_string();
                }
            }
            let inner = &value[1..value.len() - 1];
            if let Some((_, rest)) = inner.split_once('|') {
                return rest.trim().to_string();
            }
            return inner.trim().to_string();
        }
        return value[1..value.len() - 1].trim().to_string();
    }

    if value.starts_with("{{") && value.ends_with("}}") {
        let inner = &value[2..value.len() - 2];
        if let Some((_, rest)) = inner.split_once('|') {
            return rest.trim().to_string();
        }
        return inner.trim().to_string();
    }

    value.to_string()
}

pub fn parse_infobox_field(wiki_text: &str, field_name: &str) -> Option<String> {
    if wiki_text.is_empty() {
        return None;
    }

    let wiki_text = wiki_text.trim();
    if !wiki_text.starts_with("{{Infobox") || !wiki_text.ends_with("}}") {
        return None;
    }

    let search = format!("|{}=", field_name);
    if let Some(start) = wiki_text.find(&search) {
        let after_equals = &wiki_text[start + search.len()..];
        if let Some(end) = after_equals.find('|') {
            let value = &after_equals[..end];
            let cleaned = cleanup_value(value.trim());
            if !cleaned.is_empty() {
                return Some(cleaned);
            }
        } else {
            let value = after_equals.trim_end_matches("}}").trim();
            let cleaned = cleanup_value(value);
            if !cleaned.is_empty() {
                return Some(cleaned);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_infobox() {
        let wiki = "{{Infobox animanga/Anime\n|type=动画\n|话数=12\n|制作公司=Studio Pierrot\n|导演=村野佑太\n}}";
        let fields = InfoboxFields::parse(wiki);
        assert_eq!(fields.studio.as_deref(), Some("Studio Pierrot"));
        assert_eq!(fields.director.as_deref(), Some("村野佑太"));
        assert_eq!(fields.episode_count.as_deref(), Some("12"));
    }

    #[test]
    fn test_parse_with_arrays() {
        let wiki = "{{Infobox\n|type=动画\n|制作公司={{Studio|Pierrot}}\n|导演=[村野佑太]\n}}";
        let fields = InfoboxFields::parse(wiki);
        assert_eq!(fields.studio.as_deref(), Some("Pierrot"));
        assert_eq!(fields.director.as_deref(), Some("村野佑太"));
    }

    #[test]
    fn test_parse_empty() {
        let fields = InfoboxFields::parse("");
        assert!(fields.studio.is_none());
        assert!(fields.director.is_none());
    }

    #[test]
    fn test_parse_invalid_format() {
        let fields = InfoboxFields::parse("not a wiki");
        assert!(fields.studio.is_none());
    }

    #[test]
    fn test_parse_with_unknown_fields() {
        let wiki = "{{Infobox\n|unknown_field=test\n|制作公司=Pierrot\n}}";
        let fields = InfoboxFields::parse(wiki);
        assert_eq!(fields.studio.as_deref(), Some("Pierrot"));
        assert!(fields.director.is_none());
    }

    #[test]
    fn test_parse_supervision() {
        let wiki = "{{Infobox\n|监督=新房昭之\n}}";
        let fields = InfoboxFields::parse(wiki);
        assert_eq!(fields.director.as_deref(), Some("新房昭之"));
    }

    #[test]
    fn test_cleanup_value_wiki_link() {
        assert_eq!(cleanup_value("[[鲁路修]]"), "鲁路修");
        assert_eq!(cleanup_value("[[鲁路修|ルルーシュ]]"), "ルルーシュ");
        assert_eq!(cleanup_value("[[鲁路修|]]"), "");
    }

    #[test]
    fn test_cleanup_value_bracket() {
        assert_eq!(cleanup_value("[鲁路修]"), "鲁路修");
        assert_eq!(cleanup_value("[鲁路修|ルルーシュ]"), "ルルーシュ");
        assert_eq!(
            cleanup_value("[ぼっち・ざ・ろっく！]"),
            "ぼっち・ざ・ろっく！"
        );
        assert_eq!(cleanup_value("[Bocchi the Rock!]"), "Bocchi the Rock!");
    }

    #[test]
    fn test_cleanup_value_template() {
        assert_eq!(cleanup_value("{{Studio|Pierrot}}"), "Pierrot");
        assert_eq!(cleanup_value("{{导演|痞子}}"), "痞子");
        assert_eq!(cleanup_value("{{CloverWorks}}"), "CloverWorks");
    }

    #[test]
    fn test_cleanup_value_mixed() {
        assert_eq!(cleanup_value("[[鲁路修]]"), cleanup_value("[鲁路修]"));
        assert_eq!(cleanup_value("  [[鲁路修]]  "), "鲁路修");
        assert_eq!(cleanup_value(""), "");
        assert_eq!(cleanup_value("普通文本"), "普通文本");
    }

    #[test]
    fn test_cleanup_value_complex_nested() {
        assert_eq!(
            cleanup_value("[[鲁路修|ルルーシュ·ランペルージ]]"),
            "ルルーシュ·ランペルージ"
        );
    }

    #[test]
    fn test_parse_field_helper() {
        let wiki = "{{Infobox\n|制作公司=Pierrot\n}}";
        assert_eq!(
            parse_infobox_field(wiki, "制作公司"),
            Some("Pierrot".to_string())
        );
        assert_eq!(parse_infobox_field(wiki, "导演"), None);
    }
}
