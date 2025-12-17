//! anime-organizer 命令行入口
//!
//! 提供命令行界面，用于批量整理动漫视频文件。

use anime_organizer::{error::AppError, FileOrganizer, FilenameParser, OperationMode};
use clap::{Parser, ValueEnum};
use std::collections::HashSet;
use std::path::PathBuf;
use walkdir::WalkDir;

/// 默认支持的视频扩展名
const DEFAULT_EXTENSIONS: &[&str] = &[".mp4", ".mkv", ".avi", ".mov", ".wmv", ".flv", ".rmvb"];

/// 跨平台动漫文件整理工具
///
/// 自动识别并整理符合特定格式的动漫文件，支持移动、复制和硬链接模式。
#[derive(Clone, Debug, ValueEnum)]
enum FallbackMode {
    /// 移动文件
    Move,
    /// 复制文件
    Copy,
}

impl FallbackMode {
    fn to_operation_mode(&self) -> OperationMode {
        match self {
            Self::Move => OperationMode::Move,
            Self::Copy => OperationMode::Copy,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "aniorg")]
#[command(version = "1.0.0")]
#[command(about = "轻量级、跨平台命令行工具，专为动漫收藏者批量整理视频文件")]
#[command(long_about = r#"AnimeOrganizer v1.0.0 - 跨平台动漫文件整理工具

用法: aniorg --source=<路径> [选项]

硬链接说明：
    使用 --mode=link 可创建硬链接，几乎不占用额外空间，但要求源和目标在同一文件系统。

示例:
    aniorg --source="D:\Downloads" --mode=link --target="E:\Anime"
    aniorg --source="/media/下载" --dry-run --verbose"#)]
struct Cli {
    /// 源目录路径（必填）
    #[arg(short, long, value_name = "PATH")]
    source: PathBuf,

    /// 目标根目录（默认：与源目录相同）
    #[arg(short, long, value_name = "PATH")]
    target: Option<PathBuf>,

    /// 操作模式：move（移动）、copy（复制）、link（硬链接）
    #[arg(short, long, value_enum, default_value = "link")]
    mode: OperationMode,

    /// 硬链接失败时的回退模式：move 或 copy（默认不回退）
    #[arg(long, value_enum, value_name = "MODE")]
    fallback_on_link_failure: Option<FallbackMode>,

    /// 仅预览不执行
    #[arg(long)]
    dry_run: bool,

    /// 包含的扩展名（逗号分隔，默认：mp4,mkv,avi,mov,wmv,flv,rmvb）
    #[arg(long, value_name = "EXT", value_delimiter = ',')]
    include_ext: Option<Vec<String>>,

    /// 显示详细日志
    #[arg(short, long)]
    verbose: bool,
}

/// 程序入口
fn main() {
    if let Err(e) = run() {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
}

/// 主运行逻辑
fn run() -> Result<(), AppError> {
    let cli = Cli::parse();

    let fallback_mode = cli
        .fallback_on_link_failure
        .as_ref()
        .map(FallbackMode::to_operation_mode);

    // 验证源目录
    if !cli.source.exists() {
        return Err(AppError::SourceNotFound(cli.source));
    }

    // 确定目标目录
    let target = cli.target.unwrap_or_else(|| cli.source.clone());
    if !target.exists() {
        return Err(AppError::TargetNotFound(target));
    }

    // 构建扩展名集合
    let extensions: HashSet<String> = match cli.include_ext {
        Some(exts) => exts
            .into_iter()
            .map(|ext| {
                if ext.starts_with('.') {
                    ext.to_lowercase()
                } else {
                    format!(".{}", ext.to_lowercase())
                }
            })
            .collect(),
        None => DEFAULT_EXTENSIONS
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
    };

    // 统计计数
    let mut processed = 0;
    let mut succeeded = 0;
    let mut failed = 0;

    // 遍历源目录
    for entry in WalkDir::new(&cli.source)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // 检查扩展名
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e.to_lowercase()));

        if let Some(ref ext) = extension {
            if !extensions.contains(ext) {
                continue;
            }
        } else {
            continue;
        }

        // 解析文件名
        let anime_file = match FilenameParser::parse(path) {
            Some(info) => info,
            None => {
                if cli.verbose {
                    eprintln!(
                        "跳过：无法解析文件名 {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    );
                }
                continue;
            }
        };

        processed += 1;

        // 整理文件
        match FileOrganizer::organize(&anime_file, &target, cli.mode, cli.dry_run) {
            Ok(()) => {
                succeeded += 1;
                if cli.verbose && !cli.dry_run {
                    println!(
                        "成功: {} -> {}/{}",
                        anime_file.original_path,
                        anime_file.anime_name,
                        anime_file.target_filename()
                    );
                }
            }
            Err(e) => {
                let mut handled = false;

                if cli.mode == OperationMode::Link {
                    if let Some(fallback_mode) = fallback_mode {
                        if matches!(e, AppError::CrossDeviceLink | AppError::HardLinkNotSupported)
                        {
                            if cli.verbose {
                                eprintln!(
                                    "硬链接失败，回退为 {}: {}",
                                    fallback_mode,
                                    anime_file.original_path
                                );
                            }

                            match FileOrganizer::organize(
                                &anime_file,
                                &target,
                                fallback_mode,
                                cli.dry_run,
                            ) {
                                Ok(()) => {
                                    succeeded += 1;
                                    handled = true;
                                }
                                Err(e2) => {
                                    failed += 1;
                                    handled = true;
                                    eprintln!("处理文件失败 {}: {e2}", anime_file.original_path);
                                }
                            }
                        }
                    }
                }

                if !handled {
                    failed += 1;
                    eprintln!("处理文件失败 {}: {e}", anime_file.original_path);
                }
            }
        }
    }

    // 输出统计
    println!("处理完成：总计{processed}个文件，成功{succeeded}个，失败{failed}个");

    Ok(())
}
