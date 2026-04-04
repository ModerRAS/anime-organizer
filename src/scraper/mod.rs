//! 刮削模块
//!
//! 从多个数据源定期采集最新动画信息，用于别名库的自动更新。
//!
//! ## 数据源
//!
//! - Bangumi Archive：最权威的动画元数据
//! - TMDB：用于补充图片和国际化信息
//! - DMHY（动漫花园）：获取字幕组发布的最新番剧名称
//!
//! ## 功能特性
//!
//! 需要启用 `scraper` feature：
//! ```toml
//! anime-organizer = { features = ["scraper"] }
//! ```

pub mod matcher;
pub mod sources;

pub use matcher::{MatchConfidence, MatchResult, Proposal};
pub use sources::{ScrapedAnime, ScrapedSource, Scraper};
