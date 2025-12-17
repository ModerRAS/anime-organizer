//! 文件整理模块
//!
//! 该模块负责将解析后的动漫文件整理到目标目录。
//!
//! # 操作模式
//!
//! - **Move**: 移动文件到目标目录
//! - **Copy**: 复制文件到目标目录
//! - **Link**: 创建硬链接（零额外空间占用）
//!
//! # 示例
//!
//! ```no_run
//! use anime_organizer::organizer::{FileOrganizer, OperationMode};
//! use anime_organizer::parser::AnimeFileInfo;
//!
//! let info = AnimeFileInfo {
//!     publisher: "ANi".to_string(),
//!     anime_name: "测试".to_string(),
//!     episode: "01".to_string(),
//!     tags: "[1080P]".to_string(),
//!     extension: ".mp4".to_string(),
//!     original_path: "/downloads/[ANi] 测试 - 01 [1080P].mp4".to_string(),
//! };
//!
//! let result = FileOrganizer::organize(&info, "/anime", OperationMode::Copy, false);
//! ```

use crate::error::{AppError, Result};
use crate::parser::AnimeFileInfo;
use std::fs;
use std::path::Path;

/// 文件操作模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum OperationMode {
    /// 移动文件
    #[value(name = "move")]
    #[default]
    Move,
    /// 复制文件
    #[value(name = "copy")]
    Copy,
    /// 创建硬链接
    #[value(name = "link")]
    Link,
}

impl std::fmt::Display for OperationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Move => write!(f, "move"),
            Self::Copy => write!(f, "copy"),
            Self::Link => write!(f, "link"),
        }
    }
}

/// 文件整理器
///
/// 提供文件整理的静态方法。
pub struct FileOrganizer;

impl FileOrganizer {
    /// 整理单个动漫文件
    ///
    /// 根据指定的操作模式将文件整理到目标目录。
    ///
    /// # 参数
    ///
    /// * `anime_file` - 动漫文件信息
    /// * `target_root` - 目标根目录
    /// * `mode` - 操作模式
    /// * `dry_run` - 是否为预览模式
    ///
    /// # 返回值
    ///
    /// 成功返回 `Ok(())`，失败返回相应的错误。
    ///
    /// # 错误
    ///
    /// - `AppError::Io` - IO 操作失败
    /// - `AppError::CrossDeviceLink` - 硬链接跨设备
    /// - `AppError::HardLinkNotSupported` - 硬链接不支持
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use anime_organizer::organizer::{FileOrganizer, OperationMode};
    /// use anime_organizer::parser::AnimeFileInfo;
    ///
    /// let info = AnimeFileInfo {
    ///     publisher: "ANi".to_string(),
    ///     anime_name: "测试".to_string(),
    ///     episode: "01".to_string(),
    ///     tags: "[1080P]".to_string(),
    ///     extension: ".mp4".to_string(),
    ///     original_path: "/downloads/test.mp4".to_string(),
    /// };
    ///
    /// FileOrganizer::organize(&info, "/anime", OperationMode::Copy, false)?;
    /// # Ok::<(), anime_organizer::error::AppError>(())
    /// ```
    pub fn organize<P: AsRef<Path>>(
        anime_file: &AnimeFileInfo,
        target_root: P,
        mode: OperationMode,
        dry_run: bool,
    ) -> Result<()> {
        let target_dir = target_root.as_ref().join(&anime_file.anime_name);
        let target_filename = anime_file.target_filename();
        let target_path = target_dir.join(&target_filename);
        let source_path = Path::new(&anime_file.original_path);

        if dry_run {
            println!(
                "[DRY-RUN] {} -> {}",
                anime_file.original_path,
                target_path.display()
            );
            return Ok(());
        }

        // 创建目标目录
        fs::create_dir_all(&target_dir)?;

        // 如果目标文件已存在，先删除
        if target_path.exists() {
            fs::remove_file(&target_path)?;
        }

        match mode {
            OperationMode::Move => {
                // 尝试重命名（同一文件系统）
                if fs::rename(source_path, &target_path).is_err() {
                    // 如果重命名失败（可能跨设备），则复制后删除
                    fs::copy(source_path, &target_path)?;
                    fs::remove_file(source_path)?;
                }
            }
            OperationMode::Copy => {
                fs::copy(source_path, &target_path)?;
            }
            OperationMode::Link => {
                Self::create_hard_link(source_path, &target_path)?;
            }
        }

        Ok(())
    }

    /// 创建硬链接
    ///
    /// 在不同平台上创建硬链接。
    ///
    /// # 参数
    ///
    /// * `source` - 源文件路径
    /// * `target` - 目标路径
    ///
    /// # 错误
    ///
    /// - `AppError::CrossDeviceLink` - 源和目标不在同一文件系统
    /// - `AppError::HardLinkNotSupported` - 当前系统不支持硬链接
    fn create_hard_link<P: AsRef<Path>, Q: AsRef<Path>>(source: P, target: Q) -> Result<()> {
        let result = fs::hard_link(source.as_ref(), target.as_ref());

        match result {
            Ok(()) => Ok(()),
            Err(e) => {
                // 检查是否为跨设备错误
                let error_code = e.raw_os_error();

                // Windows: ERROR_NOT_SAME_DEVICE = 17
                // Unix: EXDEV = 18
                if error_code == Some(17) || error_code == Some(18) {
                    Err(AppError::CrossDeviceLink)
                } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Err(AppError::HardLinkNotSupported)
                } else {
                    Err(AppError::Io(e))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, filename: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.join(filename);
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file_path
    }

    fn create_test_anime_info(source_path: &Path) -> AnimeFileInfo {
        AnimeFileInfo {
            publisher: "ANi".to_string(),
            anime_name: "测试".to_string(),
            episode: "01".to_string(),
            tags: "[1080P]".to_string(),
            extension: ".mp4".to_string(),
            original_path: source_path.to_string_lossy().to_string(),
        }
    }

    #[test]
    fn test_organize_move_mode() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        let source_file = create_test_file(source_dir.path(), "test.mp4", "test content");
        let anime_info = create_test_anime_info(&source_file);

        let result =
            FileOrganizer::organize(&anime_info, target_dir.path(), OperationMode::Move, false);

        assert!(result.is_ok());
        let expected_path = target_dir.path().join("测试").join("01 [1080P].mp4");
        assert!(expected_path.exists());
        assert!(!source_file.exists());
        assert_eq!(fs::read_to_string(&expected_path).unwrap(), "test content");
    }

    #[test]
    fn test_organize_copy_mode() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        let source_file = create_test_file(source_dir.path(), "test.mp4", "test content");
        let anime_info = create_test_anime_info(&source_file);

        let result =
            FileOrganizer::organize(&anime_info, target_dir.path(), OperationMode::Copy, false);

        assert!(result.is_ok());
        let expected_path = target_dir.path().join("测试").join("01 [1080P].mp4");
        assert!(expected_path.exists());
        assert!(source_file.exists());
        assert_eq!(fs::read_to_string(&expected_path).unwrap(), "test content");
        assert_eq!(fs::read_to_string(&source_file).unwrap(), "test content");
    }

    #[test]
    fn test_organize_dry_run_does_not_modify_files() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        let source_file = create_test_file(source_dir.path(), "test.mp4", "test content");
        let anime_info = create_test_anime_info(&source_file);

        let result =
            FileOrganizer::organize(&anime_info, target_dir.path(), OperationMode::Move, true);

        assert!(result.is_ok());
        assert!(source_file.exists());
        assert!(!target_dir.path().join("测试").exists());
    }

    #[test]
    fn test_organize_creates_target_directory() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();
        let nested_target = target_dir.path().join("sub").join("dir");

        let source_file = create_test_file(source_dir.path(), "test.mp4", "test content");
        let anime_info = create_test_anime_info(&source_file);

        let result =
            FileOrganizer::organize(&anime_info, &nested_target, OperationMode::Copy, false);

        assert!(result.is_ok());
        assert!(nested_target.join("测试").exists());
    }

    #[test]
    fn test_organize_overwrites_existing_file() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        let source_file = create_test_file(source_dir.path(), "test.mp4", "new content");
        let anime_info = create_test_anime_info(&source_file);

        // 创建已存在的目标文件
        let target_anime_dir = target_dir.path().join("测试");
        fs::create_dir_all(&target_anime_dir).unwrap();
        create_test_file(&target_anime_dir, "01 [1080P].mp4", "old content");

        let result =
            FileOrganizer::organize(&anime_info, target_dir.path(), OperationMode::Copy, false);

        assert!(result.is_ok());
        let expected_path = target_dir.path().join("测试").join("01 [1080P].mp4");
        assert_eq!(fs::read_to_string(&expected_path).unwrap(), "new content");
    }

    #[test]
    fn test_operation_mode_display() {
        assert_eq!(format!("{}", OperationMode::Move), "move");
        assert_eq!(format!("{}", OperationMode::Copy), "copy");
        assert_eq!(format!("{}", OperationMode::Link), "link");
    }

    #[test]
    fn test_operation_mode_default() {
        assert_eq!(OperationMode::default(), OperationMode::Move);
    }
}
