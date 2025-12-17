//! 错误处理模块
//!
//! 定义了 `anime-organizer` 中使用的错误类型。

use std::path::PathBuf;
use thiserror::Error;

/// 应用程序错误类型
///
/// 包含所有可能在文件解析和整理过程中发生的错误。
#[derive(Error, Debug)]
pub enum AppError {
    /// IO 操作错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    /// 文件名解析失败
    #[error("无法解析文件名: {0}")]
    ParseError(String),

    /// 源目录不存在
    #[error("源目录不存在: {0}")]
    SourceNotFound(PathBuf),

    /// 目标目录不存在
    #[error("目标目录不存在: {0}")]
    TargetNotFound(PathBuf),

    /// 硬链接跨设备错误
    #[error("硬链接失败：源文件和目标必须在同一文件系统")]
    CrossDeviceLink,

    /// 硬链接不支持
    #[error("当前系统不支持硬链接")]
    HardLinkNotSupported,

    /// 文件操作失败
    #[error("文件操作失败 {path}: {message}")]
    FileOperation {
        /// 操作的文件路径
        path: PathBuf,
        /// 错误信息
        message: String,
    },
}

/// 应用程序结果类型别名
pub type Result<T> = std::result::Result<T, AppError>;
