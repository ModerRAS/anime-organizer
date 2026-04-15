use crate::error::{AppError, Result};
use crate::torrent::types::{ScrapedTitle, TorrentSource};
use std::collections::HashSet;

const NYAA_BASE_URL: &str = "https://nyaa.si";

pub async fn scrape_recent(pages: u32) -> Result<Vec<ScrapedTitle>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| AppError::TorrentFetchError(format!("创建 HTTP 客户端失败: {e}")))?;

    let mut all_titles = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for page in 0..pages {
        let url = format!("{}/?f=0&c=1_2&o=1&p={}", NYAA_BASE_URL, page + 1);

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("请求 Nyaa 页面失败: {e}")))?;

        let html = resp
            .text()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("读取 Nyaa 响应失败: {e}")))?;

        let view_urls = extract_view_urls(&html);

        for view_url in view_urls {
            if let Some(files) = fetch_nyaa_torrent_files(&client, &view_url).await? {
                for file in files {
                    if !seen.insert(file.title.clone()) {
                        continue;
                    }
                    all_titles.push(file);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    Ok(all_titles)
}

pub async fn scrape_search(query: &str, pages: u32) -> Result<Vec<ScrapedTitle>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| AppError::TorrentFetchError(format!("创建 HTTP 客户端失败: {e}")))?;

    let encoded = urlencoding::encode(query);
    let mut all_titles = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for page in 0..pages {
        let url = format!(
            "{}/?f=0&c=1_2&q={}&o=1&p={}",
            NYAA_BASE_URL,
            encoded,
            page + 1
        );

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("请求 Nyaa 搜索页面失败: {e}")))?;

        let html = resp
            .text()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("读取 Nyaa 响应失败: {e}")))?;

        let view_urls = extract_view_urls(&html);

        for view_url in view_urls {
            if let Some(files) = fetch_nyaa_torrent_files(&client, &view_url).await? {
                for file in files {
                    if !seen.insert(file.title.clone()) {
                        continue;
                    }
                    all_titles.push(file);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    Ok(all_titles)
}

async fn fetch_nyaa_torrent_files(
    client: &reqwest::Client,
    view_url: &str,
) -> Result<Option<Vec<ScrapedTitle>>> {
    let full_url = if view_url.starts_with("http") {
        view_url.to_string()
    } else if view_url.starts_with('/') {
        format!("https://nyaa.si{}", view_url)
    } else {
        format!("https://nyaa.si/view/{}", view_url)
    };

    let resp = match client.get(&full_url).send().await {
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

    let files = parse_file_list_from_html(&html)?;
    Ok(Some(files))
}

fn extract_view_urls(html: &str) -> Vec<String> {
    let mut urls = Vec::new();

    for line in html.lines() {
        if line.contains("href=\"/view/") {
            if let Some(start) = line.find("href=\"/view/") {
                let after_href = &line[start + 12..];
                if let Some(end) = after_href.find('"') {
                    let id = &after_href[..end];
                    urls.push(id.to_string());
                }
            }
        }
    }

    urls
}

fn parse_file_list_from_html(html: &str) -> Result<Vec<ScrapedTitle>> {
    let mut files = Vec::new();

    let file_list_start = match html.find("File list") {
        Some(pos) => pos,
        None => return Ok(files),
    };

    let content = &html[file_list_start..];

    let list_start = match content.find("<ul") {
        Some(pos) => pos,
        None => return Ok(files),
    };
    let list_content = &content[list_start..];
    let list_end = list_content[4..]
        .find("</ul>")
        .map(|p| p + 8)
        .unwrap_or(list_content.len());
    let list_section = &list_content[..list_end.min(list_content.len())];

    for line in list_section.lines() {
        if line.contains(".mkv")
            || line.contains(".mp4")
            || line.contains(".avi")
            || line.contains(".wmv")
            || line.contains(".mov")
        {
            if let Some(file_name) = extract_file_name_from_line(line) {
                files.push(ScrapedTitle {
                    title: file_name,
                    source: TorrentSource::Nyaa,
                    url: None,
                });
            }
        }
    }

    Ok(files)
}

fn extract_file_name_from_line(line: &str) -> Option<String> {
    let line = line.trim();

    for ext in [".mkv", ".mp4", ".avi", ".wmv", ".mov", ".flv", ".rmvb"] {
        if let Some(pos) = line.to_lowercase().find(ext) {
            let start = if let Some(paren) = line[..pos].rfind('>') {
                paren + 1
            } else {
                0
            };

            let end = pos + ext.len();
            let name = line[start..end].trim();

            if !name.is_empty() && name.contains('.') {
                // 去掉文件名后的文件大小部分，如 "(569.0 MiB)" 或 " (500 MiB)"
                let name = if let Some(size_pos) = name.rfind("MiB)") {
                    if size_pos > 0
                        && (name.as_bytes()[size_pos - 1] == b' '
                            || name.as_bytes()[size_pos - 1] == b'(')
                    {
                        name[..size_pos - 1].trim().to_string()
                    } else {
                        name.to_string()
                    }
                } else if let Some(size_pos) = name.rfind(" MiB") {
                    name[..size_pos].trim().to_string()
                } else {
                    name.to_string()
                };

                if !name.is_empty() && name.contains('.') {
                    return Some(name);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_file_name_from_line() {
        let line = r#"<li><span></span> Witch Hat Atelier - S01E03 - The Dadah Range Test.mkv (569.0 MiB)</li>"#;
        let result = extract_file_name_from_line(line);
        assert_eq!(
            result,
            Some("Witch Hat Atelier - S01E03 - The Dadah Range Test.mkv".to_string())
        );
    }

    #[test]
    fn test_extract_file_name_mp4() {
        let line = "<li>Some Anime - 01 [1080p].mp4 (300 MiB)</li>";
        let result = extract_file_name_from_line(line);
        assert_eq!(result, Some("Some Anime - 01 [1080p].mp4".to_string()));
    }

    #[test]
    fn test_extract_file_name_no_match() {
        let line = "Some random text without extension";
        let result = extract_file_name_from_line(line);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_file_list_from_html() {
        let html = r#"<h3>File list</h3>
<ul>
<li><span></span> Anime - S01E01.mkv (500 MiB)</li>
<li><span></span> Anime - S01E02.mkv (600 MiB)</li>
</ul>"#;

        let results = parse_file_list_from_html(html).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Anime - S01E01.mkv");
        assert_eq!(results[1].title, "Anime - S01E02.mkv");
    }

    #[test]
    fn test_extract_view_urls() {
        let html = r#"<a href="/view/123456">Link 1</a>
        <a href="/view/789012">Link 2</a>"#;

        let urls = extract_view_urls(html);
        assert_eq!(urls, vec!["123456", "789012"]);
    }
}
