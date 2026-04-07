//! RSS Item 处理逻辑模块
//!
//! 协调整个处理流程：区分 magnet/torrent 类型，调用对应模块完成下载和提交。

use tracing::{info, warn};

use crate::error::{AppError, Result};
use crate::rss::client::CloudDriveClient;
use crate::rss::db::{compute_item_hash, RssDatabase};
use crate::rss::filter::{matches_filter, RssFilter};
use crate::rss::parser::{parse_rss, RssItem};
use crate::rss::torrent::download_torrent_to_magnet;

/// RSS 处理器
///
/// 协调从 RSS 抓取到离线下载提交的完整流水线。
pub struct RssProcessor {
    http_client: reqwest::Client,
    cd_client: CloudDriveClient,
}

impl RssProcessor {
    pub fn new(http_client: reqwest::Client, cd_client: CloudDriveClient) -> Self {
        Self {
            http_client,
            cd_client,
        }
    }

    /// 处理单个订阅：抓取 RSS → 过滤 → 解析 → 提交离线下载
    ///
    /// 返回本次新提交的项目数量。
    pub async fn process_subscription(
        &self,
        db: &RssDatabase,
        subscription_id: i64,
        rss_url: &str,
        filter: &Option<RssFilter>,
        target_folder: &str,
        verbose: bool,
    ) -> Result<usize> {
        // 1. 抓取 RSS XML
        let xml = self
            .http_client
            .get(rss_url)
            .send()
            .await
            .map_err(|e| AppError::MetadataFetchError(format!("抓取 RSS 失败: {e}")))?
            .text()
            .await
            .map_err(|e| AppError::MetadataFetchError(format!("读取 RSS 内容失败: {e}")))?;

        // 2. 解析 RSS
        let items = parse_rss(&xml)?;
        if verbose {
            info!("从 {} 获取到 {} 个 RSS 项", rss_url, items.len());
        }

        let mut submitted = 0;

        for item in &items {
            // 3. 过滤
            if !matches_filter(&item.title, filter) {
                if verbose {
                    info!("跳过（不匹配过滤器）: {}", item.title);
                }
                continue;
            }

            // 4. 去重
            let item_hash = compute_item_hash(item.guid.as_deref(), &item.title);
            if db.is_item_processed(subscription_id, &item_hash)? {
                if verbose {
                    info!("跳过（已处理）: {}", item.title);
                }
                continue;
            }

            // 5. 解析 magnet / torrent
            match self.resolve_and_submit(item, target_folder, verbose).await {
                Ok(()) => {
                    // 6. 记录已处理
                    db.mark_item_processed(subscription_id, &item_hash, &item.title)?;
                    db.save_download_task(subscription_id, &item_hash)?;
                    submitted += 1;
                    info!("已提交: {}", item.title);
                }
                Err(e) => {
                    warn!("处理失败 '{}': {e}", item.title);
                }
            }
        }

        Ok(submitted)
    }

    /// 解析 magnet 或 torrent URL，然后提交到 CloudDrive2
    async fn resolve_and_submit(
        &self,
        item: &RssItem,
        target_folder: &str,
        verbose: bool,
    ) -> Result<()> {
        let magnet = if let Some(ref magnet) = item.magnet {
            magnet.clone()
        } else if let Some(ref torrent_url) = item.torrent_url {
            if verbose {
                info!("下载 .torrent: {}", torrent_url);
            }
            download_torrent_to_magnet(&self.http_client, torrent_url).await?
        } else {
            return Err(AppError::MetadataFetchError(format!(
                "RSS 条目 '{}' 没有 magnet 或 torrent URL",
                item.title
            )));
        };

        if verbose {
            info!(
                "提交 magnet 到 CloudDrive2: {}...",
                &magnet[..magnet.len().min(80)]
            );
        }

        self.cd_client
            .add_offline_files(vec![magnet], target_folder)
            .await
    }
}
