use crate::error::{AppError, Result};
use crate::torrent::types::{ScrapedTitle, TorrentSource};
use std::collections::HashSet;

const DMHY_RSS_URL: &str = "https://share.dmhy.org/topics/rss/rss.xml";

pub async fn scrape_dmhy(pages: u32) -> Result<Vec<ScrapedTitle>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| AppError::TorrentFetchError(format!("创建 HTTP 客户端失败: {e}")))?;

    let mut all_titles = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for page in 0..pages {
        let url = if page == 0 {
            DMHY_RSS_URL.to_string()
        } else {
            format!("{}?page={}", DMHY_RSS_URL, page + 1)
        };

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("请求 DMHY RSS 失败: {e}")))?;

        let text = resp
            .text()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("读取 DMHY RSS 响应失败: {e}")))?;

        let entries = parse_dmhy_rss(&text)?;

        for entry in entries {
            let topic_url = entry.url.clone().unwrap_or_default();

            if let Some(files) = fetch_torrent_files(&client, &topic_url).await? {
                for file in files {
                    if !seen.insert(file.title.clone()) {
                        continue;
                    }
                    all_titles.push(file);
                }
            }
        }
    }

    Ok(all_titles)
}

async fn fetch_torrent_files(
    client: &reqwest::Client,
    topic_url: &str,
) -> Result<Option<Vec<ScrapedTitle>>> {
    let detail_url = if topic_url.contains("view/") {
        topic_url.to_string()
    } else if topic_url.starts_with('/') {
        format!("https://share.dmhy.org{}", topic_url)
    } else {
        format!("https://share.dmhy.org/topics/view/{}", topic_url)
    };

    let resp = match client.get(&detail_url).send().await {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    if !resp.status().is_success() {
        return Ok(None);
    }

    let html = match resp.text().await {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };

    let files = parse_torrent_files_from_html(&html)?;
    Ok(Some(files))
}

fn parse_torrent_files_from_html(html: &str) -> Result<Vec<ScrapedTitle>> {
    let mut files = Vec::new();

    let tabs_start = match html.find("id=\"tabs-1\"") {
        Some(pos) => pos,
        None => match html.find("id='tabs-1'") {
            Some(pos) => pos,
            None => return Ok(files),
        },
    };

    let tabs_content = &html[tabs_start..];

    let list_start = match tabs_content.find("<ul") {
        Some(pos) => pos,
        None => return Ok(files),
    };
    let list_content = &tabs_content[list_start..];
    let list_end = match list_content.find("</ul>") {
        Some(pos) => pos + 5,
        None => list_content.len(),
    };
    let list_section = &list_content[..list_end];

    for line in list_section.lines() {
        let line = line.trim();
        if !line.contains(".mkv")
            && !line.contains(".mp4")
            && !line.contains(".avi")
            && !line.contains(".wmv")
            && !line.contains(".mov")
            && !line.contains(".flv")
            && !line.contains(".rmvb")
        {
            continue;
        }

        if let Some(file_name) = extract_file_name_from_line(line) {
            files.push(ScrapedTitle {
                title: file_name,
                source: TorrentSource::Dmhy,
                url: None,
            });
        }
    }

    Ok(files)
}

fn extract_file_name_from_line(line: &str) -> Option<String> {
    let line = line.trim();

    let extensions = ["mkv", "mp4", "avi", "wmv", "mov", "flv", "rmvb"];
    let line_lower = line.to_lowercase();

    for ext in &extensions {
        let ext_with_dot = format!(".{}", ext);
        if let Some(ext_pos) = line_lower.find(&ext_with_dot) {
            let ext_end = ext_pos + ext_with_dot.len();
            let slice = &line[..ext_end];
            let start = slice.find('[').unwrap_or(0);

            let name = slice[start..].trim();
            if !name.is_empty() && name.contains('.') {
                return Some(name.to_string());
            }
        }
    }
    None
}

fn parse_dmhy_rss(xml: &str) -> Result<Vec<ScrapedTitle>> {
    let mut results = Vec::new();

    for segment in xml.split("<item>").skip(1) {
        let title = extract_xml_value(segment, "title").unwrap_or_default();
        let link = extract_xml_value(segment, "link");

        results.push(ScrapedTitle {
            title,
            source: TorrentSource::Dmhy,
            url: link,
        });
    }

    Ok(results)
}

fn extract_xml_value(segment: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");

    let start = segment.find(&start_tag)?;
    let after_start = start + start_tag.len();
    let end = segment[after_start..].find(&end_tag)?;

    Some(segment[after_start..after_start + end].trim().to_string())
}

pub fn titles_to_text(titles: &[ScrapedTitle]) -> String {
    titles
        .iter()
        .map(|t| t.title.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn write_titles_to_file(titles: &[ScrapedTitle], path: &std::path::Path) -> Result<usize> {
    let text = titles_to_text(titles);
    std::fs::write(path, text)
        .map_err(|e| AppError::Io(std::io::Error::other(format!("写入文件失败: {e}"))))?;
    Ok(titles.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_file_name_from_line() {
        let line = r#"[ANi] 出租女友 第五季 - 02 [1080P][Baha][WEB-DL][AAC AVC][CHT].mp4 468.4MB"#;
        let result = extract_file_name_from_line(line);
        assert_eq!(
            result,
            Some("[ANi] 出租女友 第五季 - 02 [1080P][Baha][WEB-DL][AAC AVC][CHT].mp4".to_string())
        );
    }

    #[test]
    fn test_extract_file_name_mkv() {
        let line = "[LoliHouse] Anime - 01 [WebRip 1080p HEVC-10bit AAC].mkv 300MB";
        let result = extract_file_name_from_line(line);
        assert_eq!(
            result,
            Some("[LoliHouse] Anime - 01 [WebRip 1080p HEVC-10bit AAC].mkv".to_string())
        );
    }

    #[test]
    fn test_extract_file_name_no_match() {
        let line = "Some random text without extension";
        let result = extract_file_name_from_line(line);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_torrent_files_from_html() {
        let html = r#"<div id="tabs-1">
<ul>
<li><img src="mp4">[ANi] Anime - 01 [1080P].mp4 500MB</li>
<li><img src="mkv">[ANi] Anime - 02 [1080P].mkv 600MB</li>
</ul>
</div>"#;

        let results = parse_torrent_files_from_html(html).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "[ANi] Anime - 01 [1080P].mp4");
        assert_eq!(results[1].title, "[ANi] Anime - 02 [1080P].mkv");
    }

    #[test]
    fn test_titles_to_text() {
        let titles = vec![
            ScrapedTitle {
                title: "Title 1".to_string(),
                source: TorrentSource::Dmhy,
                url: None,
            },
            ScrapedTitle {
                title: "Title 2".to_string(),
                source: TorrentSource::Dmhy,
                url: None,
            },
        ];

        let text = titles_to_text(&titles);
        assert_eq!(text, "Title 1\nTitle 2");
    }
}
