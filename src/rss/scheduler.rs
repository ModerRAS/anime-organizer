//! 调度器模块
//!
//! 提供 Daemon 模式下的定时调度功能，定期检查 RSS 更新。

use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::sync::watch;
use tracing::{error, info};

use crate::error::Result;
use crate::rss::client::CloudDriveClientTrait;
use crate::rss::db::{RssDatabase, Subscription};
use crate::rss::filter::RssFilter;
use crate::rss::http_client::HttpClientTrait;
use crate::rss::processor::RssProcessor;

pub struct RssScheduler {
    db: RssDatabase,
    processor: RssProcessor,
    verbose: bool,
}

impl RssScheduler {
    pub fn new(
        db: RssDatabase,
        http_client: Arc<dyn HttpClientTrait>,
        cd_client: Arc<dyn CloudDriveClientTrait>,
        verbose: bool,
    ) -> Self {
        let processor = RssProcessor::new(http_client, cd_client);
        Self {
            db,
            processor,
            verbose,
        }
    }

    /// 单次执行：处理所有订阅一遍后退出
    pub async fn run_once(&self) -> Result<()> {
        let subs = self.db.list_subscriptions()?;
        if subs.is_empty() {
            info!("没有已启用的 RSS 订阅");
            return Ok(());
        }

        let mut total = 0usize;
        for sub in &subs {
            match self.process_subscription(sub).await {
                Ok(n) => total += n,
                Err(e) => error!("处理订阅 '{}' 失败: {e}", sub.url),
            }
        }

        info!("单次执行完成，共提交 {total} 个新项目");
        Ok(())
    }

    /// 单次执行：处理指定 URL（非数据库订阅）
    pub async fn run_once_url(
        &self,
        rss_url: &str,
        filter_pattern: Option<&str>,
        target_folder: &str,
        interval_secs: u64,
    ) -> Result<()> {
        // 确保数据库中有该订阅
        let sub_id = self.db.add_subscription(
            rss_url,
            filter_pattern,
            target_folder,
            interval_secs as i64,
        )?;

        let filter = if let Some(pattern) = filter_pattern {
            Some(RssFilter::new(pattern)?)
        } else {
            None
        };

        let n = self
            .processor
            .process_subscription(
                &self.db,
                sub_id,
                rss_url,
                &filter,
                target_folder,
                self.verbose,
            )
            .await?;

        info!("单次执行完成，共提交 {n} 个新项目");
        Ok(())
    }

    /// Daemon 模式：持续运行，按间隔轮询
    pub async fn run_daemon(&self, default_interval: Duration) -> Result<()> {
        info!(
            "Daemon 模式启动，默认间隔 {} 秒",
            default_interval.as_secs()
        );

        // Ctrl+C shutdown channel
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

        tokio::spawn(async move {
            let _ = signal::ctrl_c().await;
            info!("收到 Ctrl+C，正在优雅关闭...");
            let _ = shutdown_tx.send(true);
        });

        loop {
            // 检查 shutdown
            if *shutdown_rx.borrow() {
                info!("调度器已关闭");
                return Ok(());
            }

            self.tick().await;

            // 等待间隔或 shutdown 信号
            tokio::select! {
                _ = tokio::time::sleep(default_interval) => {},
                _ = shutdown_rx.changed() => {
                    info!("调度器已关闭");
                    return Ok(());
                }
            }
        }
    }

    /// Daemon 模式：持续运行指定 URL
    pub async fn run_daemon_url(
        &self,
        rss_url: &str,
        filter_pattern: Option<&str>,
        target_folder: &str,
        interval: Duration,
    ) -> Result<()> {
        info!(
            "Daemon 模式启动 (URL: {})，间隔 {} 秒",
            rss_url,
            interval.as_secs()
        );

        let sub_id = self.db.add_subscription(
            rss_url,
            filter_pattern,
            target_folder,
            interval.as_secs() as i64,
        )?;

        let filter = if let Some(pattern) = filter_pattern {
            Some(RssFilter::new(pattern)?)
        } else {
            None
        };

        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

        tokio::spawn(async move {
            let _ = signal::ctrl_c().await;
            info!("收到 Ctrl+C，正在优雅关闭...");
            let _ = shutdown_tx.send(true);
        });

        loop {
            if *shutdown_rx.borrow() {
                info!("调度器已关闭");
                return Ok(());
            }

            match self
                .processor
                .process_subscription(
                    &self.db,
                    sub_id,
                    rss_url,
                    &filter,
                    target_folder,
                    self.verbose,
                )
                .await
            {
                Ok(n) => {
                    if n > 0 {
                        info!("本轮提交 {n} 个新项目");
                    }
                }
                Err(e) => error!("处理订阅 '{}' 失败: {e}", rss_url),
            }

            tokio::select! {
                _ = tokio::time::sleep(interval) => {},
                _ = shutdown_rx.changed() => {
                    info!("调度器已关闭");
                    return Ok(());
                }
            }
        }
    }

    /// 处理单个订阅（从数据库记录）
    async fn process_subscription(&self, sub: &Subscription) -> Result<usize> {
        let filter = if let Some(ref pattern) = sub.filter_regex {
            Some(RssFilter::new(pattern)?)
        } else {
            None
        };

        self.processor
            .process_subscription(
                &self.db,
                sub.id,
                &sub.url,
                &filter,
                &sub.target_folder,
                self.verbose,
            )
            .await
    }

    /// 执行一次 tick：遍历所有订阅各处理一遍
    async fn tick(&self) {
        let subs = match self.db.list_subscriptions() {
            Ok(s) => s,
            Err(e) => {
                error!("读取订阅列表失败: {e}");
                return;
            }
        };

        for sub in &subs {
            match self.process_subscription(sub).await {
                Ok(n) => {
                    if n > 0 {
                        info!("订阅 '{}' 本轮提交 {n} 个新项目", sub.url);
                    }
                }
                Err(e) => error!("处理订阅 '{}' 失败: {e}", sub.url),
            }
        }
    }
}
