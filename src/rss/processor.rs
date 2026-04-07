//! RSS Item 处理逻辑模块
//!
//! 协调整个处理流程：区分 magnet/torrent 类型，调用对应模块完成下载和提交。

use std::sync::Arc;
use tracing::{info, warn};

use crate::error::{AppError, Result};
use crate::rss::client::CloudDriveClientTrait;
use crate::rss::db::{compute_item_hash, RssDatabase};
use crate::rss::filter::{matches_filter, RssFilter};
use crate::rss::http_client::HttpClientTrait;
use crate::rss::parser::{parse_rss, RssItem};
use crate::rss::torrent::download_torrent_to_magnet;

pub struct RssProcessor {
    http_client: Arc<dyn HttpClientTrait>,
    cd_client: Arc<dyn CloudDriveClientTrait>,
}

impl RssProcessor {
    pub fn new(
        http_client: Arc<dyn HttpClientTrait>,
        cd_client: Arc<dyn CloudDriveClientTrait>,
    ) -> Self {
        Self {
            http_client,
            cd_client,
        }
    }

    pub async fn process_subscription(
        &self,
        db: &RssDatabase,
        subscription_id: i64,
        rss_url: &str,
        filter: &Option<RssFilter>,
        target_folder: &str,
        verbose: bool,
    ) -> Result<usize> {
        let xml = self.http_client.get(rss_url).await?;

        let items = parse_rss(&xml)?;
        if verbose {
            info!("从 {} 获取到 {} 个 RSS 项", rss_url, items.len());
        }

        let mut submitted = 0;

        for item in &items {
            if !matches_filter(&item.title, filter) {
                if verbose {
                    info!("跳过（不匹配过滤器）: {}", item.title);
                }
                continue;
            }

            let item_hash = compute_item_hash(item.guid.as_deref(), &item.title);
            if db.is_item_processed(subscription_id, &item_hash)? {
                if verbose {
                    info!("跳过（已处理）: {}", item.title);
                }
                continue;
            }

            match self.resolve_and_submit(item, target_folder, verbose).await {
                Ok(()) => {
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
            let client = reqwest::Client::new();
            download_torrent_to_magnet(&client, torrent_url).await?
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
