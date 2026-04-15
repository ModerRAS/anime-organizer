//! Torrent 标题爬取模块
//!
//! 从 DMHY 和 Nyaa.si 爬取番剧种子文件的原始发布标题，
//! 用于多字幕组格式识别的训练数据收集。
//!
//! ## 数据源
//!
//! - DMHY (动漫花园): `https://share.dmhy.org` - 通过 RSS 获取最新发布
//! - Nyaa.si: `https://nyaa.si` - 通过 HTML 解析搜索结果
//!
//! ## 功能特性
//!
//! 需要启用 `torrent-scraper` feature：
//! ```toml
//! anime-organizer = { features = ["torrent-scraper"] }
//! ```

pub mod dmhy;
pub mod nyaa;
pub mod types;

pub use types::{ScrapedTitle, TorrentSource};
