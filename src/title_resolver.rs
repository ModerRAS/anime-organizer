use anime_organizer::parser::split_series_and_season;
use quick_xml::de::from_str;
use regex::Regex;
use serde::Deserialize;
use std::sync::LazyLock;
use std::time::Duration;

const DMHY_RSS_URL: &str = "https://share.dmhy.org/topics/rss/rss.xml";
const DMHY_SEARCH_URL: &str = "https://share.dmhy.org/topics/list";
const ANILIST_GRAPHQL_URL: &str = "https://graphql.anilist.co";
const ANILIST_QUERY: &str = r#"
query ($search: String) {
  Page(page: 1, perPage: 5) {
    media(search: $search, type: ANIME, sort: SEARCH_MATCH) {
      title { romaji english native }
      synonyms
      format
    }
  }
}
"#;

static DMHY_TOPIC_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<a\b[^>]*href=["']/topics/view/[^"']+["'][^>]*>(?P<title>.*?)</a>"#)
        .expect("DMHY topic link regex should compile")
});
static HTML_TAG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<[^>]+>").expect("HTML tag regex should compile"));
static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(concat!(
            "ModerRAS/anime-organizer/",
            env!("CARGO_PKG_VERSION"),
            " (https://github.com/ModerRAS/anime-organizer)"
        ))
        .timeout(Duration::from_secs(15))
        .build()
        .expect("title resolver HTTP client should build")
});

pub(crate) struct ResolvedTitle {
    pub(crate) titles: Vec<String>,
    pub(crate) source: &'static str,
}

pub(crate) fn is_latin_title(value: &str) -> bool {
    let letters = value
        .chars()
        .filter(|ch| ch.is_alphabetic())
        .collect::<Vec<_>>();
    !letters.is_empty() && letters.iter().all(|ch| ch.is_ascii_alphabetic())
}

pub(crate) async fn resolve_latin_title(
    title: &str,
    publisher: Option<&str>,
    season_hint: Option<u32>,
    verbose: bool,
) -> Option<ResolvedTitle> {
    if !is_latin_title(title) {
        return None;
    }

    if publisher.is_some_and(is_lolihouse) {
        match resolve_from_dmhy(title, season_hint).await {
            Ok(Some(titles)) => {
                return Some(ResolvedTitle {
                    titles,
                    source: "动漫花园",
                });
            }
            Ok(None) => {}
            Err(error) if verbose => eprintln!("动漫花园标题解析失败: {error}"),
            Err(_) => {}
        }
    }

    match resolve_from_anilist(title, season_hint).await {
        Ok(Some(titles)) => Some(ResolvedTitle {
            titles,
            source: "AniList",
        }),
        Ok(None) => None,
        Err(error) => {
            if verbose {
                eprintln!("AniList 标题解析失败: {error}");
            }
            None
        }
    }
}

fn is_lolihouse(publisher: &str) -> bool {
    publisher.to_ascii_lowercase().contains("lolihouse")
}

fn dmhy_search_title(title: &str) -> String {
    split_series_and_season(title).0
}

async fn resolve_from_dmhy(
    title: &str,
    season_hint: Option<u32>,
) -> Result<Option<Vec<String>>, String> {
    let keyword = dmhy_search_title(title);
    let mut rss_url = reqwest::Url::parse(DMHY_RSS_URL).map_err(|error| error.to_string())?;
    rss_url.query_pairs_mut().append_pair("keyword", &keyword);

    let rss_error = match fetch_text(rss_url).await {
        Ok(xml) => match select_dmhy_title(&xml, title, season_hint) {
            Ok(Some(title)) => return Ok(Some(title)),
            Ok(None) => None,
            Err(error) => Some(error.to_string()),
        },
        Err(error) => Some(error),
    };

    let mut search_url = reqwest::Url::parse(DMHY_SEARCH_URL).map_err(|error| error.to_string())?;
    search_url
        .query_pairs_mut()
        .append_pair("keyword", &keyword)
        .append_pair("sort_id", "2")
        .append_pair("team_id", "0")
        .append_pair("order", "date-desc");
    let html = fetch_text(search_url).await.map_err(|error| {
        rss_error.map_or(error.clone(), |rss| format!("RSS: {rss}; HTML: {error}"))
    })?;
    Ok(select_dmhy_html_title(&html, title, season_hint))
}

async fn fetch_text(url: reqwest::Url) -> Result<String, String> {
    HTTP_CLIENT
        .get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .text()
        .await
        .map_err(|error| error.to_string())
}

fn select_dmhy_title(
    xml: &str,
    query: &str,
    season_hint: Option<u32>,
) -> Result<Option<Vec<String>>, quick_xml::DeError> {
    let feed: DmhyRss = from_str(xml)?;
    Ok(select_dmhy_topic_titles(
        feed.channel.items.into_iter().map(|item| item.title),
        query,
        season_hint,
    ))
}

fn select_dmhy_html_title(
    html: &str,
    query: &str,
    season_hint: Option<u32>,
) -> Option<Vec<String>> {
    let titles = DMHY_TOPIC_LINK_REGEX
        .captures_iter(html)
        .filter_map(|captures| captures.name("title"))
        .map(|title| decode_html_title(title.as_str()));
    select_dmhy_topic_titles(titles, query, season_hint)
}

fn select_dmhy_topic_titles(
    titles: impl IntoIterator<Item = String>,
    query: &str,
    season_hint: Option<u32>,
) -> Option<Vec<String>> {
    let expected_season = season_hint.unwrap_or(1);
    let query_series = normalized_series_title(query);

    titles.into_iter().find_map(|title| {
        if !is_lolihouse(&title) {
            return None;
        }

        let topic = strip_leading_groups(&title);
        let parts = topic
            .split(['/', '／'])
            .map(clean_topic_part)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        if !parts.iter().any(|part| {
            is_latin_title(part)
                && titles_match(&query_series, &normalized_series_title(part))
                && split_series_and_season(part).1.unwrap_or(1) == expected_season
        }) {
            return None;
        }

        let aliases = parts
            .into_iter()
            .filter(|part| contains_han(part))
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        (!aliases.is_empty()).then_some(aliases)
    })
}

fn decode_html_title(value: &str) -> String {
    HTML_TAG_REGEX
        .replace_all(value, " ")
        .replace("&amp;", "&")
        .replace("&#39;", "'")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

async fn resolve_from_anilist(
    title: &str,
    season_hint: Option<u32>,
) -> Result<Option<Vec<String>>, String> {
    let response = HTTP_CLIENT
        .post(ANILIST_GRAPHQL_URL)
        .json(&serde_json::json!({
            "query": ANILIST_QUERY,
            "variables": { "search": title },
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let response: AniListResponse = response.json().await.map_err(|error| error.to_string())?;
    Ok(select_anilist_title(&response, title, season_hint))
}

fn select_anilist_title(
    response: &AniListResponse,
    query: &str,
    season_hint: Option<u32>,
) -> Option<Vec<String>> {
    let expected_season = season_hint.unwrap_or(1);
    let query_lower = query.to_ascii_lowercase();

    response
        .data
        .as_ref()?
        .page
        .media
        .iter()
        .filter(|media| {
            !matches!(media.format.as_deref(), Some("MUSIC" | "SPECIAL" | "OVA"))
                || media
                    .format
                    .as_deref()
                    .is_some_and(|format| query_lower.contains(&format.to_ascii_lowercase()))
        })
        .filter_map(|media| {
            let titles = media.all_titles();
            let candidate_season = titles
                .iter()
                .find_map(|title| split_series_and_season(title).1)
                .unwrap_or(1);
            if candidate_season != expected_season {
                return None;
            }

            let score = titles
                .iter()
                .map(|candidate| latin_title_match_score(query, candidate))
                .fold(0.0, f32::max);
            (score >= 0.75).then_some((media, score))
        })
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(media, _)| {
            let mut titles = Vec::new();
            if let Some(native) = media.title.native.as_ref() {
                titles.push(native.clone());
            }
            for candidate in media.all_titles() {
                if is_latin_title(candidate)
                    && latin_title_match_score(query, candidate) >= 0.75
                    && !titles.iter().any(|title| title == candidate)
                {
                    titles.push(candidate.to_string());
                }
            }
            titles
        })
}

fn latin_title_match_score(query: &str, candidate: &str) -> f32 {
    let query = normalized_series_title(query);
    let candidate = normalized_series_title(candidate);
    if query.is_empty() || candidate.is_empty() {
        return 0.0;
    }
    if query == candidate {
        return 1.0;
    }

    let shorter = query.len().min(candidate.len());
    let longer = query.len().max(candidate.len());
    if query.starts_with(&candidate) || candidate.starts_with(&query) {
        let coverage = shorter as f32 / longer as f32;
        if shorter >= 12 {
            return 0.9 + coverage * 0.1;
        }
        if shorter >= 6 && coverage >= 0.6 {
            return 0.8 + coverage * 0.2;
        }
    }

    0.0
}

fn titles_match(query: &str, candidate: &str) -> bool {
    query == candidate
        || query.len().min(candidate.len()) >= 8
            && (query.starts_with(candidate) || candidate.starts_with(query))
            && query.len().min(candidate.len()) * 5 >= query.len().max(candidate.len()) * 3
}

fn normalized_series_title(value: &str) -> String {
    split_series_and_season(clean_topic_part(value))
        .0
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn strip_leading_groups(mut value: &str) -> &str {
    loop {
        let trimmed = value.trim_start();
        let Some(rest) = trimmed.strip_prefix('[') else {
            return trimmed;
        };
        let Some(end) = rest.find(']') else {
            return trimmed;
        };
        value = &rest[end + 1..];
    }
}

fn clean_topic_part(value: &str) -> &str {
    let mut value = value.trim();
    if let Some(index) = value.find(" [") {
        value = &value[..index];
    }
    if let Some(index) = value.rfind(" - ") {
        let suffix = value[index + 3..].trim_start();
        if suffix.starts_with(|ch: char| ch.is_ascii_digit()) {
            value = &value[..index];
        }
    }
    value.trim()
}

fn contains_han(value: &str) -> bool {
    value
        .chars()
        .any(|ch| matches!(ch, '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}'))
}

#[derive(Deserialize)]
struct DmhyRss {
    channel: DmhyChannel,
}

#[derive(Deserialize)]
struct DmhyChannel {
    #[serde(default, rename = "item")]
    items: Vec<DmhyItem>,
}

#[derive(Deserialize)]
struct DmhyItem {
    title: String,
}

#[derive(Deserialize)]
struct AniListResponse {
    data: Option<AniListData>,
}

#[derive(Deserialize)]
struct AniListData {
    #[serde(rename = "Page")]
    page: AniListPage,
}

#[derive(Deserialize)]
struct AniListPage {
    #[serde(default)]
    media: Vec<AniListMedia>,
}

#[derive(Deserialize)]
struct AniListMedia {
    title: AniListTitles,
    #[serde(default)]
    synonyms: Vec<String>,
    format: Option<String>,
}

impl AniListMedia {
    fn all_titles(&self) -> Vec<&str> {
        self.title
            .romaji
            .iter()
            .chain(self.title.english.iter())
            .chain(self.title.native.iter())
            .chain(self.synonyms.iter())
            .map(String::as_str)
            .collect()
    }
}

#[derive(Deserialize)]
struct AniListTitles {
    romaji: Option<String>,
    english: Option<String>,
    native: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_latin_titles_and_lolihouse_publishers() {
        assert!(is_latin_title("Yomi no Tsugai"));
        assert!(is_lolihouse("Nekomoe kissaten&LoliHouse"));
        assert_eq!(
            dmhy_search_title("Gaikotsu Kishi-sama, Tadaima Isekai e Odekakechuu II"),
            "Gaikotsu Kishi-sama, Tadaima Isekai e Odekakechuu"
        );
        assert!(!is_latin_title("黄泉使者"));
        assert!(!is_lolihouse("ANi"));
    }

    #[test]
    fn dmhy_rss_recovers_chinese_title_and_checks_season() {
        let xml = r#"
<rss><channel>
  <item><title><![CDATA[[LoliHouse] 关于我在无意间被隔壁的天使变成废柴这件事 第二季 / Otonari no Tenshi-sama S2 - 12 [1080p]]]></title></item>
  <item><title><![CDATA[[LoliHouse] 关于我在无意间被隔壁的天使变成废柴这件事 / Otonari no Tenshi-sama - 12 [1080p]]]></title></item>
</channel></rss>"#;

        assert_eq!(
            select_dmhy_title(xml, "Otonari no Tenshi-sama S2", Some(2)).unwrap(),
            Some(vec![
                "关于我在无意间被隔壁的天使变成废柴这件事 第二季".to_string()
            ])
        );
        assert_eq!(
            select_dmhy_title(xml, "Otonari no Tenshi-sama", None).unwrap(),
            Some(vec!["关于我在无意间被隔壁的天使变成废柴这件事".to_string()])
        );
    }

    #[test]
    fn dmhy_html_fallback_recovers_punctuated_title() {
        let html = r#"
<a href="/topics/view/722722_example.html">[LoliHouse] 骸骨骑士大人异世界冒险中II / Gaikotsu Kishi-sama, Tadaima Isekai e Odekakechuu II - 02 [1080p]</a>"#;

        assert_eq!(
            select_dmhy_html_title(
                html,
                "Gaikotsu Kishi-sama, Tadaima Isekai e Odekakechuu II",
                Some(2)
            ),
            Some(vec!["骸骨骑士大人异世界冒险中II".to_string()])
        );
    }

    #[test]
    fn dmhy_rss_accepts_real_kill_blue_topic() {
        let xml = r#"
<rss><channel><item><title><![CDATA[[喵萌奶茶屋&LoliHouse] 杀手青春 / KILL BLUE / Kill Ao - 05 [WebRip 1080p]]]></title></item></channel></rss>"#;

        assert_eq!(
            select_dmhy_title(xml, "KILL BLUE", None).unwrap(),
            Some(vec!["杀手青春".to_string()])
        );
    }

    #[test]
    fn dmhy_returns_all_cjk_aliases() {
        let xml = r#"
<rss><channel><item><title><![CDATA[[喵萌奶茶屋&LoliHouse] 灰原君的青春二周目 / 灰原同學重返過去，開啟所向無敵的第二輪青春遊戲 / 灰原くんの強くて青春ニューゲーム / Haibara-kun no Tsuyokute Seishun New Game - 07 [1080p]]]></title></item></channel></rss>"#;

        assert_eq!(
            select_dmhy_title(xml, "Haibara-kun no Tsuyokute Seishun New Game", None).unwrap(),
            Some(vec![
                "灰原君的青春二周目".to_string(),
                "灰原同學重返過去，開啟所向無敵的第二輪青春遊戲".to_string(),
                "灰原くんの強くて青春ニューゲーム".to_string(),
            ])
        );
    }

    #[test]
    fn anilist_skips_ova_and_returns_native_title() {
        let response: AniListResponse = serde_json::from_str(
            r#"{
  "data": {"Page": {"media": [
    {"title":{"romaji":"Honzuki no Gekokujou OVA","english":null,"native":"本好きの下剋上 OVA"},"synonyms":["Honzuki no Gekokujou"],"format":"OVA"},
    {"title":{"romaji":"Honzuki no Gekokujou","english":"Ascendance of a Bookworm","native":"本好きの下剋上"},"synonyms":[],"format":"TV"}
  ]}}
}"#,
        )
        .unwrap();

        assert_eq!(
            select_anilist_title(&response, "Honzuki no Gekokujou", None),
            Some(vec![
                "本好きの下剋上".to_string(),
                "Honzuki no Gekokujou".to_string(),
            ])
        );
    }

    #[test]
    fn anilist_accepts_expanded_romaji_title() {
        let response: AniListResponse = serde_json::from_str(
            r#"{
  "data": {"Page": {"media": [
    {"title":{"romaji":"Rakudai Kenja no Gakuin Musou: Nidome no Tensei, S-Rank Cheat Majutsushi Bouken-roku","english":null,"native":"落第賢者の学院無双"},"synonyms":[],"format":"TV"}
  ]}}
}"#,
        )
        .unwrap();

        assert_eq!(
            select_anilist_title(&response, "Rakudai Kenja no Gakuin Musou", None),
            Some(vec![
                "落第賢者の学院無双".to_string(),
                "Rakudai Kenja no Gakuin Musou: Nidome no Tensei, S-Rank Cheat Majutsushi Bouken-roku".to_string(),
            ])
        );
    }

    #[test]
    fn anilist_keeps_latin_candidates_for_cross_script_bangumi_search() {
        let response: AniListResponse = serde_json::from_str(
            r#"{
  "data": {"Page": {"media": [
    {"title":{"romaji":"BanG Dream! Yume∞Mita","english":"BanG Dream! YUME∞MITA","native":"バンドリ！ ゆめ∞みた"},"synonyms":[],"format":"TV"}
  ]}}
}"#,
        )
        .unwrap();

        assert_eq!(
            select_anilist_title(&response, "BanG Dream Yumemita", None),
            Some(vec![
                "バンドリ！ ゆめ∞みた".to_string(),
                "BanG Dream! Yume∞Mita".to_string(),
                "BanG Dream! YUME∞MITA".to_string(),
            ])
        );
    }

    #[test]
    fn anilist_rejects_wrong_season() {
        let response: AniListResponse = serde_json::from_str(
            r#"{
  "data": {"Page": {"media": [
    {"title":{"romaji":"Otonari no Tenshi-sama","english":null,"native":"お隣の天使様"},"synonyms":[],"format":"TV"}
  ]}}
}"#,
        )
        .unwrap();

        assert_eq!(
            select_anilist_title(&response, "Otonari no Tenshi-sama S2", Some(2)),
            None
        );
    }
}
