//! # anime-organizer
//!
//! 轻量级、跨平台命令行工具，专为动漫收藏者批量整理视频文件。
//!
//! ## 功能特性
//!
//! - **智能解析**: 自动识别 `[发布组] 动漫名 - 集数 [标签].ext` 格式
//! - **灵活整理**: 重构为 `动漫名/集数 [标签].ext` 结构
//! - **多种模式**: 支持移动、复制、硬链接三种操作模式
//! - **跨平台**: 支持 Windows、Linux、macOS
//! - **零依赖运行**: 单文件部署，无需外部配置
//!
//! ## 快速开始
//!
//! ```bash
//! # 移动模式（默认）
//! aniorg --source="/path/to/downloads"
//!
//! # 硬链接模式（推荐，零额外空间）
//! aniorg --source="/path/to/downloads" --mode=link --target="/path/to/anime"
//!
//! # 预览模式
//! aniorg --source="/path/to/downloads" --dry-run --verbose
//! ```
//!
//! ## 模块结构
//!
//! - [`parser`]: 文件名解析模块
//! - [`organizer`]: 文件整理模块
//! - [`error`]: 错误处理模块

pub mod error;
pub mod organizer;
pub mod parser;

pub use error::{AppError, Result};
pub use organizer::{FileOrganizer, OperationMode};
pub use parser::{AnimeFileInfo, FilenameParser};
