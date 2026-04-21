use crate::error::{AppError, Result};
use crate::torrent::types::{ScrapedTitle, TorrentSource};
use std::collections::HashSet;

const DMHY_LIST_URL: &str = "https://share.dmhy.org/topics/list/sort_id/2/page/";

pub async fn scrape_dmhy(pages: u32) -> Result<Vec<ScrapedTitle>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::TorrentFetchError(format!("创建 HTTP 客户端失败: {e}")))?;

    let mut all_titles = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for page in 1..=pages {
        eprintln!("[DMHY] 正在爬取第 {} / {} 页...", page, pages);
        let url = format!("{}{}", DMHY_LIST_URL, page);

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("请求 DMHY 页面失败: {e}")))?;

        let html = resp
            .text()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("读取 DMHY 页面失败: {e}")))?;

        let view_links = extract_view_links(&html);
        eprintln!("[DMHY] 第 {} 页找到 {} 个种子", page, view_links.len());

        for (i, link) in view_links.iter().enumerate() {
            if let Some(files) = fetch_torrent_files(&client, link).await? {
                for file in files {
                    if !seen.insert(file.title.clone()) {
                        continue;
                    }
                    all_titles.push(file);
                }
            }

            if (i + 1) % 10 == 0 || i == view_links.len() - 1 {
                eprintln!(
                    "[DMHY] 已处理 {}/{}, 当前累计: {} 个文件名",
                    i + 1,
                    view_links.len(),
                    all_titles.len()
                );
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
    }

    eprintln!("[DMHY] 爬取完成，共获取 {} 个文件名", all_titles.len());
    Ok(all_titles)
}

async fn fetch_torrent_files(
    client: &reqwest::Client,
    link: &str,
) -> Result<Option<Vec<ScrapedTitle>>> {
    let detail_url = if link.starts_with("http") {
        link.to_string()
    } else if link.starts_with('/') {
        format!("https://share.dmhy.org{}", link)
    } else {
        format!("https://share.dmhy.org/topics/view/{}", link)
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

    let files = parse_file_list_from_html(&html)?;
    if files.is_empty() {
        return Ok(None);
    }

    Ok(Some(files))
}

fn parse_file_list_from_html(html: &str) -> Result<Vec<ScrapedTitle>> {
    let mut files = Vec::new();

    let bt_section_start = match html.find("BT列表") {
        Some(pos) => pos,
        None => return Ok(files),
    };

    let bt_section = &html[bt_section_start..];

    for line in bt_section.lines() {
        let line = line.trim();

        let file_name = extract_file_name_from_line(line)?;
        if !file_name.is_empty() {
            files.push(ScrapedTitle {
                title: file_name,
                source: TorrentSource::Dmhy,
                url: None,
            });
        }

        if line.contains("查看評論") || line.contains("Powered by") {
            break;
        }
    }

    Ok(files)
}

fn extract_file_name_from_line(line: &str) -> Result<String> {
    let line = line.trim();

    let extensions = [".mkv", ".mp4", ".avi", ".wmv", ".mov", ".flv", ".rmvb"];
    let line_lower = line.to_lowercase();

    for ext in extensions.iter() {
        if let Some(ext_pos) = line_lower.find(ext) {
            let ext_end = ext_pos + ext.len();
            let before_ext = &line[..ext_end];

            let start = match before_ext.rfind("]>") {
                Some(pos) => pos + 2,
                None => before_ext
                    .rfind('>')
                    .map(|p| p + 1)
                    .unwrap_or_else(|| before_ext.rfind('[').unwrap_or(0)),
            };

            let file_name = line[start..ext_end].trim();

            if file_name.contains('.') && file_name.len() > 4 {
                return Ok(file_name.to_string());
            }
        }
    }

    Ok(String::new())
}

fn extract_view_links(html: &str) -> Vec<String> {
    let mut links = Vec::new();

    for line in html.lines() {
        if line.contains("href=\"/topics/view/") {
            if let Some(start) = line.find("href=\"/topics/view/") {
                let after_href = &line[start + 19..];
                if let Some(end) = after_href.find('"') {
                    let link = &after_href[..end];
                    if !link.is_empty() {
                        links.push(format!("/topics/view/{}", link));
                    }
                }
            }
        }
    }

    links
}

pub fn titles_to_text(titles: &[ScrapedTitle]) -> String {
    crate::torrent::sorted_unique_title_text(titles)
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
        // DMHY format: filename.ext SIZE (e.g., "420.5MB")
        let line = r#"[ANi] 溜掉的大魚比不上自己釣到的魚 - 03 [1080P][Baha][WEB-DL][AAC AVC][CHT].mp4 420.5MB"#;
        let result = extract_file_name_from_line(line).unwrap();
        // Should extract up to .mp4
        assert!(result.contains(".mp4"));
    }

    #[test]
    fn test_parse_file_list_from_html() {
        let html = r#"BT列表
[ANi] Anime - 01 [1080P].mp4 500MB
[ANi] Anime - 02 [1080P].mkv 600MB
查看評論"#;

        let results = parse_file_list_from_html(html).unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_extract_view_links() {
        let html = r#"<a href="/topics/view/123_test.html">Link 1</a>
<a href="/topics/view/456_test.html">Link 2</a>"#;
        let links = extract_view_links(html);
        assert_eq!(links.len(), 2);
        assert!(links[0].contains("/topics/view/"));
        assert!(links[1].contains("/topics/view/"));
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
