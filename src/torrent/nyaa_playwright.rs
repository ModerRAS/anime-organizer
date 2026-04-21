use crate::error::{AppError, Result};
use crate::torrent::types::{ScrapedTitle, TorrentSource};
use playwright::Playwright;
use std::collections::HashSet;

const NYAA_BASE_URL: &str = "https://nyaa.si";

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

pub async fn scrape_recent_with_playwright(pages: u32) -> Result<Vec<ScrapedTitle>> {
    scrape_recent_with_playwright_opts(pages, ScrapeOptions::new()).await
}

pub async fn scrape_recent_with_playwright_opts(
    pages: u32,
    opts: ScrapeOptions,
) -> Result<Vec<ScrapedTitle>> {
    eprintln!("[Nyaa/Playwright] 正在初始化 Playwright...");

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
    eprintln!("[Nyaa/Playwright] 启动浏览器 (模式: {})...", mode_str);

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
            "[Nyaa/Playwright] 正在爬取第 {} / {} 页...",
            page_num, pages
        );

        let url = if page_num == 1 {
            format!("{}/?c=1_2&s=seeders&o=desc", NYAA_BASE_URL)
        } else {
            format!(
                "{}/?c=1_2&s=seeders&o=desc&page={}",
                NYAA_BASE_URL, page_num
            )
        };

        eprintln!("[Nyaa/Playwright] 导航到: {}", url);

        page.goto_builder(&url)
            .goto()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("导航到 Nyaa 失败: {e}")))?;

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        let html = page
            .content()
            .await
            .map_err(|e| AppError::TorrentFetchError(format!("获取页面内容失败: {e}")))?;

        eprintln!("[Nyaa/Playwright] HTML 长度: {} 字节", html.len());

        if html.len() < 1000 {
            eprintln!("[Nyaa/Playwright] 警告: HTML 太短，可能是被阻止了");
        }

        let view_urls = extract_view_urls_from_html(&html);
        eprintln!(
            "[Nyaa/Playwright] 第 {} 页找到 {} 个种子链接",
            page_num,
            view_urls.len()
        );

        for (i, view_url) in view_urls.iter().enumerate() {
            if let Some(files) = scrape_torrent_details(&page, view_url).await? {
                for file in files {
                    if !seen.insert(file.title.clone()) {
                        continue;
                    }
                    all_titles.push(file);
                }
            }

            if (i + 1) % 10 == 0 {
                eprintln!(
                    "[Nyaa/Playwright] 已处理 {}/{}, 当前累计: {} 个文件名",
                    i + 1,
                    view_urls.len(),
                    all_titles.len()
                );
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    browser.close().await.ok();

    eprintln!(
        "[Nyaa/Playwright] 爬取完成，共获取 {} 个文件名",
        all_titles.len()
    );
    Ok(all_titles)
}

async fn scrape_torrent_details(
    page: &playwright::api::Page,
    view_url: &str,
) -> Result<Option<Vec<ScrapedTitle>>> {
    let full_url = format!("https://nyaa.si{}", view_url);

    match page.goto_builder(&full_url).timeout(60000.0).goto().await {
        Ok(_) => {}
        Err(_) => {
            return Ok(None);
        }
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let html = match page.content().await {
        Ok(h) => h,
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
