//! RSS + CloudDrive2 模块
//!
//! 提供 RSS 订阅监控和 CloudDrive2 离线下载功能。
//!
//! ## 功能特性
//!
//! 需要启用 `clouddrive` feature：
//! ```toml
//! anime-organizer = { features = ["clouddrive"] }
//! ```
//!
//! ## 子模块
//!
//! - [`parser`] - RSS 2.0 XML 解析
//! - [`client`] - CloudDrive2 gRPC 客户端
//! - [`scheduler`] - 定时调度器
//! - [`db`] - SQLite 数据库操作
//! - [`proxy`] - HTTP 代理配置
//! - [`filter`] - 正则表达式过滤
//! - [`torrent`] - .torrent 下载和 magnet 转换
//! - [`processor`] - RSS item 处理流程

#[cfg(feature = "clouddrive")]
pub mod client;
#[cfg(feature = "clouddrive")]
pub mod db;
#[cfg(feature = "clouddrive")]
pub mod filter;
#[cfg(feature = "clouddrive")]
pub mod parser;
#[cfg(feature = "clouddrive")]
pub mod processor;
#[cfg(feature = "clouddrive")]
pub mod proxy;
#[cfg(feature = "clouddrive")]
pub mod scheduler;
#[cfg(feature = "clouddrive")]
pub mod torrent;
