use crate::error::{AppError, Result};
use crate::torrent::types::{ScrapedTitle, TorrentSource};
use playwright::Playwright;
use std::collections::HashSet;

const DMHY_LIST_URL: &str = "https://share.dmhy.org/topics/list/sort_id/2/page/";

#[derive(Debug, Clone, Default)]
pub struct ScrapeOptions {
    pub headed: bool,
}

impl ScrapeOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_headed(mut self, headed: bool) -> Self {
        self.headed = headed;
        self
    }
}

pub async fn scrape_dmhy_with_playwright(pages: u32) -> Result<Vec<ScrapedTitle>> {
    scrape_dmhy_with_playwright_opts(pages, ScrapeOptions::new()).await
}

pub async fn scrape_dmhy_with_playwright_opts(
    pages: u32,
    opts: ScrapeOptions,
) -> Result<Vec<ScrapedTitle>> {
    eprintln!("[DMHY/Playwright] 正在初始化 Playwright...");

    let playwright = Playwright::initialize()
        .await
        .map_err(|e| AppError::TorrentFetchError(format!("初始化 Playwright 失败: {e}")))?;

    playwright
        .prepare()
        .map_err(|e| AppError::TorrentFetchError(format!("准备 Playwright 失败: {e}")))?;

    let mode_str = if opts.headed {
        "headed (可见窗口)"
    } else {
        "headless (无头)"
    };
    eprintln!("[DMHY/Playwright] 启动浏览器 (模式: {})...", mode_str);

    let browser = if opts.headed {
        playwright.chromium().launcher().launch().await
    } else {
        playwright
            .chromium()
            .launcher()
            .headless(true)
            .launch()
            .await
    }
    .map_err(|e| AppError::TorrentFetchError(format!("启动浏览器失败: {e}")))?;

    let context = browser
        .context_builder()
        .build()
        .await
        .map_err(|e| AppError::TorrentFetchError(format!("创建浏览器上下文失败: {e}")))?;

    let page = context
        .new_page()
        .await
        .map_err(|e| AppError::TorrentFetchError(format!("创建页面失败: {e}")))?;

    let mut all_titles = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for page_num in 1..=pages {
        eprintln!(
            "[DMHY/Playwright] 正在爬取第 {} / {} 页...",
            page_num, pages
        );

        let url = format!("{}{}", DMHY_LIST_URL, page_num);
        eprintln!("[DMHY/Playwright] 导航到: {}", url);

        page.goto_builder(&url)
            .goto()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("导航到 DMHY 失败: {e}")))?;

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        let html = page
            .content()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("获取页面内容失败: {e}")))?;

        eprintln!("[DMHY/Playwright] HTML 长度: {} 字节", html.len());

        if html.len() < 1000 {
            eprintln!("[DMHY/Playwright] 警告: HTML 太短，可能是被阻止了");
        }

        let view_links = extract_view_links(&html);
        eprintln!(
            "[DMHY/Playwright] 第 {} 页找到 {} 个种子链接",
            page_num,
            view_links.len()
        );

        for (i, link) in view_links.iter().enumerate() {
            if let Some(files) = fetch_torrent_files(&page, link).await? {
                for file in files {
                    if !seen.insert(file.title.clone()) {
                        continue;
                    }
                    all_titles.push(file);
                }
            }

            if (i + 1) % 10 == 0 || i == view_links.len().saturating_sub(1) {
                eprintln!(
                    "[DMHY/Playwright] 已处理 {}/{}, 当前累计: {} 个文件名",
                    i + 1,
                    view_links.len(),
                    all_titles.len()
                );
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
    }

    browser.close().await.ok();

    eprintln!(
        "[DMHY/Playwright] 爬取完成，共获取 {} 个文件名",
        all_titles.len()
    );
    Ok(all_titles)
}

async fn fetch_torrent_files(
    page: &playwright::api::Page,
    link: &str,
) -> Result<Option<Vec<ScrapedTitle>>> {
    let detail_url = if link.starts_with("http") {
        link.to_string()
    } else if link.starts_with('/') {
        format!("https://share.dmhy.org{}", link)
    } else {
        format!("https://share.dmhy.org/topics/view/{}", link)
    };

    match page.goto_builder(&detail_url).timeout(60000.0).goto().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("[DMHY/Playwright] 详情页加载失败 (继续): {}", e);
            return Ok(None);
        }
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let html = match page.content().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("[DMHY/Playwright] 获取页面内容失败 (继续): {}", e);
            return Ok(None);
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_file_name_from_line() {
        let line = r#"[ANi] 溜掉的大魚比不上自己釣到的魚 - 03 [1080P][Baha][WEB-DL][AAC AVC][CHT].mp4 420.5MB"#;
        let result = extract_file_name_from_line(line).unwrap();
        assert!(result.contains(".mp4"));
    }

    #[test]
    fn test_parse_file_list_from_html() {
        let html = r#"BT列表
[ANi] Anime - 01 [1080P].mp4 500MB
[ANi] Anime - 02 [1080P].mkv 600MB
查看評論"#;

        let results = parse_file_list_from_html(html).unwrap();
        assert!(results.len() >= 1);
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
