use crate::error::{AppError, Result};
use crate::torrent::types::{ScrapedTitle, TorrentSource};
use std::collections::HashSet;

const NYAA_BASE_URL: &str = "https://nyaa.si";

pub async fn scrape_recent(pages: u32) -> Result<Vec<ScrapedTitle>> {
    eprintln!("[Nyaa] 正在爬取最新种子 ({} 页)...", pages);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::TorrentFetchError(format!("创建 HTTP 客户端失败: {e}")))?;

    let mut all_titles = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for page_num in 1..=pages {
        eprintln!("[Nyaa] 正在爬取第 {} / {} 页...", page_num, pages);

        let url = format!(
            "{}/?c=1_2&s=seeders&o=desc{}",
            NYAA_BASE_URL,
            if page_num > 1 {
                format!("&page={}", page_num)
            } else {
                String::new()
            }
        );

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("请求 Nyaa 页面失败: {e}")))?;

        let html = resp
            .text()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("读取 Nyaa 页面失败: {e}")))?;

        eprintln!("[Nyaa] HTML 长度: {} 字节", html.len());

        let view_urls = extract_view_urls_from_html(&html);
        eprintln!(
            "[Nyaa] 第 {} 页找到 {} 个种子链接",
            page_num,
            view_urls.len()
        );

        if view_urls.is_empty() && page_num == 1 {
            eprintln!("[Nyaa] 前 500 字符: {}", &html[..html.len().min(500)]);
        }

        if view_urls.is_empty() && page_num == 1 {
            eprintln!("[Nyaa] 警告: 未能提取种子链接");
        }

        for (i, view_url) in view_urls.iter().enumerate() {
            if let Some(files) = fetch_torrent_details(&client, view_url).await? {
                for file in files {
                    if !seen.insert(file.title.clone()) {
                        continue;
                    }
                    all_titles.push(file);
                }
            }

            if (i + 1) % 10 == 0 {
                eprintln!(
                    "[Nyaa] 已处理 {}/{}, 当前累计: {} 个文件名",
                    i + 1,
                    view_urls.len(),
                    all_titles.len()
                );
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    eprintln!("[Nyaa] 爬取完成，共获取 {} 个文件名", all_titles.len());
    Ok(all_titles)
}

async fn fetch_torrent_details(
    client: &reqwest::Client,
    view_url: &str,
) -> Result<Option<Vec<ScrapedTitle>>> {
    let full_url = format!("https://nyaa.si{}", view_url);

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

    let mut files = Vec::new();

    if let Some(file_list_start) = html.find("File list") {
        let content = &html[file_list_start..];

        if let Some(list_start) = content.find("<ul") {
            let list_content = &content[list_start..];
            let list_end = list_content[4..]
                .find("</ul>")
                .map(|p| p + 8)
                .unwrap_or(list_content.len());
            let list_section = &list_content[..list_end.min(list_content.len())];

            for line in list_section.lines() {
                if let Some(file_name) = extract_file_name_from_line(line) {
                    files.push(ScrapedTitle {
                        title: file_name,
                        source: TorrentSource::Nyaa,
                        url: Some(full_url.clone()),
                    });
                }
            }
        }
    }

    if files.is_empty() {
        return Ok(None);
    }

    Ok(Some(files))
}

fn extract_view_urls_from_html(html: &str) -> Vec<String> {
    let mut urls = Vec::new();

    for line in html.lines() {
        if line.contains("href=\"/view/") {
            if let Some(start) = line.find("href=\"/view/") {
                let after_href = &line[start + 12..];
                if let Some(end) = after_href.find('"') {
                    let id = &after_href[..end];
                    urls.push(format!("/view/{}", id));
                }
            }
        }
    }

    urls
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
                let name = if let Some(size_pos) = name.rfind("MiB)") {
                    if size_pos > 0
                        && (name.as_bytes()[size_pos - 1] == b' '
                            || name.as_bytes()[size_pos - 1] == b'(')
                    {
                        name[..size_pos - 1].trim().to_string()
                    } else {
                        name.to_string()
                    }
                } else if let Some(size_pos) = name.rfind("GiB)") {
                    if size_pos > 0
                        && (name.as_bytes()[size_pos - 1] == b' '
                            || name.as_bytes()[size_pos - 1] == b'(')
                    {
                        name[..size_pos - 1].trim().to_string()
                    } else {
                        name.to_string()
                    }
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

pub async fn scrape_search(query: &str, pages: u32) -> Result<Vec<ScrapedTitle>> {
    eprintln!("[Nyaa] 正在搜索 '{}' ({} 页)...", query, pages);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::TorrentFetchError(format!("创建 HTTP 客户端失败: {e}")))?;

    let encoded_query = urlencoding::encode(query);

    let mut all_titles = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for page_num in 1..=pages {
        eprintln!(
            "[Nyaa] 正在爬取搜索 '{}' 第 {} / {} 页...",
            query, page_num, pages
        );

        let url = format!(
            "{}/?c=1_2&q={}&s=seeders&o=desc{}",
            NYAA_BASE_URL,
            encoded_query,
            if page_num > 1 {
                format!("&page={}", page_num)
            } else {
                String::new()
            }
        );

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("请求 Nyaa 搜索页失败: {e}")))?;

        let html = resp
            .text()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("读取 Nyaa 搜索页失败: {e}")))?;

        let view_urls = extract_view_urls_from_html(&html);
        eprintln!(
            "[Nyaa] 第 {} 页找到 {} 个种子链接",
            page_num,
            view_urls.len()
        );

        for (i, view_url) in view_urls.iter().enumerate() {
            if let Some(files) = fetch_torrent_details(&client, view_url).await? {
                for file in files {
                    if !seen.insert(file.title.clone()) {
                        continue;
                    }
                    all_titles.push(file);
                }
            }

            if (i + 1) % 10 == 0 {
                eprintln!(
                    "[Nyaa] 已处理 {}/{}, 当前累计: {} 个文件名",
                    i + 1,
                    view_urls.len(),
                    all_titles.len()
                );
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    eprintln!("[Nyaa] 爬取完成，共获取 {} 个文件名", all_titles.len());
    Ok(all_titles)
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
    fn test_extract_view_urls_from_html() {
        let html = r#"<a href="/view/123456">Link 1</a>
        <a href="/view/789012">Link 2</a>"#;
        let urls = extract_view_urls_from_html(html);
        assert_eq!(urls, vec!["/view/123456", "/view/789012"]);
    }
}
