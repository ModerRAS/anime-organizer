//! RSS + CloudDrive2 模块
//!
//! 提供 RSS 订阅监控和 CloudDrive2 离线下载功能。

#[cfg(feature = "clouddrive")]
pub mod client;

#[cfg(feature = "clouddrive")]
pub use client::CloudDriveClientTrait;
#[cfg(feature = "clouddrive")]
pub mod db;
#[cfg(feature = "clouddrive")]
pub mod filter;
#[cfg(feature = "clouddrive")]
pub mod http_client;
#[cfg(feature = "clouddrive")]
pub use http_client::{HttpClient, HttpClientTrait};
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
