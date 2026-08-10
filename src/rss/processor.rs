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
use crate::rss::proxy::{build_http_client, ProxyConfig};
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
        let mut failed = 0;
        let mut first_error = None;

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
                Ok(info_hash) => {
                    db.record_submitted_item(
                        subscription_id,
                        &item_hash,
                        &item.title,
                        info_hash.as_deref(),
                    )?;
                    submitted += 1;
                    info!("已提交: {}", item.title);
                }
                Err(e) => {
                    warn!("处理失败 '{}': {e}", item.title);
                    failed += 1;
                    first_error.get_or_insert_with(|| format!("'{}': {e}", item.title));
                }
            }
        }

        if let Some(error) = first_error {
            return Err(AppError::MetadataFetchError(format!(
                "{failed} 个 RSS 条目处理失败；首个错误: {error}"
            )));
        }

        Ok(submitted)
    }

    async fn resolve_and_submit(
        &self,
        item: &RssItem,
        target_folder: &str,
        verbose: bool,
    ) -> Result<Option<String>> {
        let magnet = if let Some(ref magnet) = item.magnet {
            magnet.clone()
        } else if let Some(ref torrent_url) = item.torrent_url {
            if verbose {
                info!("下载 .torrent: {}", torrent_url);
            }
            let proxy_config = ProxyConfig::from_env();
            let client =
                build_http_client(&proxy_config).unwrap_or_else(|_| reqwest::Client::new());
            download_torrent_to_magnet(&client, torrent_url).await?
        } else {
            return Err(AppError::MetadataFetchError(format!(
                "RSS 条目 '{}' 没有 magnet 或 torrent URL",
                item.title
            )));
        };

        if verbose {
            info!("提交 magnet 到 CloudDrive2");
        }

        self.cd_client
            .add_offline_files(vec![magnet.clone()], target_folder)
            .await?;
        Ok(normalize_magnet_info_hash(&magnet))
    }
}

fn normalize_magnet_info_hash(magnet: &str) -> Option<String> {
    let url = url::Url::parse(magnet).ok()?;
    url.query_pairs().find_map(|(key, value)| {
        key.eq_ignore_ascii_case("xt")
            .then(|| {
                value
                    .get(..9)
                    .filter(|prefix| prefix.eq_ignore_ascii_case("urn:btih:"))
            })
            .flatten()
            .and_then(|_| value.get(9..))
            .and_then(normalize_btih)
    })
}

fn normalize_btih(value: &str) -> Option<String> {
    if value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Some(value.to_ascii_lowercase());
    }
    if value.len() != 32 {
        return None;
    }

    let mut output = Vec::with_capacity(20);
    let mut bits = 0u8;
    let mut buffer = 0u32;
    for byte in value.bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a',
            b'2'..=b'7' => byte - b'2' + 26,
            _ => return None,
        };
        buffer = (buffer << 5) | u32::from(value);
        bits += 5;
        while bits >= 8 {
            bits -= 8;
            output.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    (output.len() == 20).then(|| output.iter().map(|byte| format!("{byte:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::normalize_magnet_info_hash;

    #[test]
    fn normalizes_hex_magnet_btih_without_retaining_trackers() {
        assert_eq!(
            normalize_magnet_info_hash(
                "magnet:?dn=Episode&xt=urn%3Abtih%3AABCDEF1234567890ABCDEF1234567890ABCDEF12&tr=https%3A%2F%2Fsecret.example"
            )
            .as_deref(),
            Some("abcdef1234567890abcdef1234567890abcdef12")
        );
        assert!(normalize_magnet_info_hash("magnet:?xt=urn:btih:not-a-hash").is_none());
        assert_eq!(
            normalize_magnet_info_hash("magnet:?xt=urn:btih:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
                .as_deref(),
            Some("0000000000000000000000000000000000000000")
        );
    }
}
