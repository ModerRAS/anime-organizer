//! anime-organizer 命令行入口
//!
//! 提供命令行界面，用于批量整理动漫视频文件。

use anime_organizer::{error::AppError, FileOrganizer, FilenameParser, OperationMode};
#[cfg(feature = "metadata")]
use anime_organizer::{
    metadata::{AliasLookup, BangumiClient, TmdbClient},
    nfo::{EpisodeNfo, NfoWriter, TvShowNfo, UniqueId},
    AnimeMetadata,
};
use clap::{Parser, ValueEnum};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
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

    /// 启用元数据刮削（生成 NFO 文件和下载封面图片）
    #[arg(long)]
    scrape_metadata: bool,

    /// TMDB API Key（用于下载封面图片）
    #[arg(long, value_name = "KEY")]
    tmdb_api_key: Option<String>,

    /// 自定义别名文件路径（覆盖内置别名库）
    #[arg(long, value_name = "PATH")]
    alias_file: Option<PathBuf>,

    /// 跳过图片下载
    #[arg(long)]
    no_images: bool,

    /// 覆盖已有的 NFO 和图片文件
    #[arg(long)]
    force_overwrite: bool,

    /// Bangumi dump 缓存目录
    #[arg(long, value_name = "PATH")]
    bangumi_cache: Option<PathBuf>,
}

/// 程序入口
fn main() {
    if let Err(e) = run() {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
}

/// 主运行逻辑
#[cfg(feature = "metadata")]
fn run() -> Result<(), AppError> {
    let cli = Cli::parse();
    if cli.scrape_metadata {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| AppError::MetadataFetchError(format!("创建异步运行时失败: {e}")))?;
        rt.block_on(run_with_metadata(cli))
    } else {
        run_organize(cli)
    }
}

#[cfg(not(feature = "metadata"))]
fn run() -> Result<(), AppError> {
    let cli = Cli::parse();
    if cli.scrape_metadata {
        return Err(AppError::MetadataFetchError(
            "元数据功能未启用，请使用 --features metadata 编译".to_string(),
        ));
    }
    run_organize(cli)
}

/// 仅文件整理流程（无元数据）
fn run_organize(cli: Cli) -> Result<(), AppError> {
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

    let extensions = build_extensions(&cli.include_ext);
    let mut processed = 0;
    let mut succeeded = 0;
    let mut failed = 0;

    for entry in WalkDir::new(&cli.source)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if !has_valid_extension(path, &extensions) {
            continue;
        }

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
        match organize_file(
            &anime_file,
            &target,
            cli.mode,
            cli.dry_run,
            fallback_mode,
            cli.verbose,
        ) {
            Ok(()) => succeeded += 1,
            Err(_) => failed += 1,
        }
    }

    println!("处理完成：总计{processed}个文件，成功{succeeded}个，失败{failed}个");
    Ok(())
}

/// 带元数据刮削的流程
#[cfg(feature = "metadata")]
async fn run_with_metadata(cli: Cli) -> Result<(), AppError> {
    let fallback_mode = cli
        .fallback_on_link_failure
        .as_ref()
        .map(FallbackMode::to_operation_mode);

    if !cli.source.exists() {
        return Err(AppError::SourceNotFound(cli.source));
    }

    let target = cli.target.unwrap_or_else(|| cli.source.clone());
    if !target.exists() {
        return Err(AppError::TargetNotFound(target));
    }

    let extensions = build_extensions(&cli.include_ext);

    // 加载别名库
    let alias_lookup = AliasLookup::load(cli.alias_file.as_deref())?;
    if cli.verbose {
        eprintln!("已加载 {} 条别名", alias_lookup.len());
    }

    // 初始化 Bangumi 客户端
    let bangumi = BangumiClient::new(cli.bangumi_cache);

    // 初始化 TMDB 客户端（可选）
    let tmdb = cli
        .tmdb_api_key
        .as_ref()
        .map(|key| TmdbClient::new(key.clone()));

    // 收集同名动画文件，按动画名分组
    let mut anime_groups: HashMap<String, Vec<anime_organizer::AnimeFileInfo>> = HashMap::new();
    for entry in WalkDir::new(&cli.source)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if !has_valid_extension(path, &extensions) {
            continue;
        }
        if let Some(info) = FilenameParser::parse(path) {
            anime_groups
                .entry(info.anime_name.clone())
                .or_default()
                .push(info);
        } else if cli.verbose {
            eprintln!(
                "跳过：无法解析文件名 {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            );
        }
    }

    let mut processed = 0;
    let mut succeeded = 0;
    let mut failed = 0;
    // 元数据缓存（避免重复查询同一动画）
    let mut metadata_cache: HashMap<String, Option<AnimeMetadata>> = HashMap::new();

    for (anime_name, files) in &anime_groups {
        // 查找别名 → 获取元数据
        if !metadata_cache.contains_key(anime_name) {
            let meta = fetch_anime_metadata(
                anime_name,
                &alias_lookup,
                &bangumi,
                tmdb.as_ref(),
                cli.verbose,
            )
            .await;
            metadata_cache.insert(anime_name.clone(), meta);
        }

        let meta = metadata_cache.get(anime_name).and_then(|m| m.as_ref());

        // 生成 tvshow.nfo（每个动画只生成一次）
        if let Some(meta) = meta {
            let anime_dir = target.join(anime_name);
            let tvshow_nfo_path = anime_dir.join("tvshow.nfo");

            if cli.force_overwrite || !tvshow_nfo_path.exists() {
                let nfo = TvShowNfo::from(meta);
                if cli.dry_run {
                    if cli.verbose {
                        eprintln!("[dry-run] 生成 tvshow.nfo: {}", tvshow_nfo_path.display());
                    }
                } else {
                    match NfoWriter::write_tvshow(&anime_dir, &nfo) {
                        Ok(()) => {
                            if cli.verbose {
                                eprintln!("已生成 tvshow.nfo: {}", tvshow_nfo_path.display());
                            }
                        }
                        Err(e) => eprintln!("生成 tvshow.nfo 失败: {e}"),
                    }
                }
            }

            // 下载图片
            if !cli.no_images && !cli.dry_run {
                download_images(
                    meta,
                    &anime_dir,
                    tmdb.as_ref(),
                    cli.force_overwrite,
                    cli.verbose,
                )
                .await;
            }
        }

        // 整理每个文件并生成 episode.nfo
        for file in files {
            processed += 1;

            match organize_file(
                file,
                &target,
                cli.mode,
                cli.dry_run,
                fallback_mode,
                cli.verbose,
            ) {
                Ok(()) => {
                    succeeded += 1;

                    // 生成 episode.nfo
                    if let Some(m) = meta {
                        let episode_nfo_path = target
                            .join(anime_name)
                            .join(file.target_filename())
                            .with_extension("nfo");

                        if cli.force_overwrite || !episode_nfo_path.exists() {
                            let ep_nfo = create_episode_nfo(file, m);
                            if cli.dry_run {
                                if cli.verbose {
                                    eprintln!(
                                        "[dry-run] 生成 episode.nfo: {}",
                                        episode_nfo_path.display()
                                    );
                                }
                            } else if let Err(e) =
                                NfoWriter::write_episode(&episode_nfo_path, &ep_nfo)
                            {
                                eprintln!("生成 episode.nfo 失败: {e}");
                            }
                        }
                    }
                }
                Err(_) => failed += 1,
            }
        }
    }

    println!("处理完成：总计{processed}个文件，成功{succeeded}个，失败{failed}个");
    if !metadata_cache.is_empty() {
        let matched = metadata_cache.values().filter(|v| v.is_some()).count();
        println!("元数据匹配：{matched}/{} 部动画", metadata_cache.len());
    }

    Ok(())
}

/// 获取动画元数据
#[cfg(feature = "metadata")]
async fn fetch_anime_metadata(
    anime_name: &str,
    alias_lookup: &AliasLookup,
    bangumi: &BangumiClient,
    tmdb: Option<&TmdbClient>,
    verbose: bool,
) -> Option<AnimeMetadata> {
    // 1. 精确查找别名
    let alias = alias_lookup
        .find(anime_name)
        .or_else(|| alias_lookup.find_fuzzy(anime_name));

    if let Some(entry) = alias {
        if verbose {
            eprintln!(
                "别名匹配: {} → {} (bangumi_id={})",
                anime_name, entry.name, entry.bangumi_id
            );
        }

        // 2. 从 Bangumi Archive 获取元数据
        match bangumi.fetch_metadata(entry.bangumi_id).await {
            Ok(mut meta) => {
                meta.tmdb_id = entry.tmdb_id;
                meta.anidb_id = entry.anidb_id;

                // 3. 如果有 TMDB ID 且有 TMDB 客户端，尝试获取额外信息
                if let (Some(tmdb_id), Some(tmdb_client)) = (entry.tmdb_id, tmdb) {
                    if let Ok(show) = tmdb_client.find_by_tmdb_id(tmdb_id).await {
                        if verbose {
                            eprintln!("TMDB 匹配: {} (tmdb_id={})", show.name, tmdb_id);
                        }
                    }
                }

                return Some(meta);
            }
            Err(e) => {
                if verbose {
                    eprintln!("Bangumi 获取失败 {}: {e}", entry.bangumi_id);
                }
            }
        }
    } else if verbose {
        eprintln!("未找到别名: {anime_name}");
    }

    None
}

/// 下载封面图片
#[cfg(feature = "metadata")]
async fn download_images(
    meta: &AnimeMetadata,
    anime_dir: &Path,
    tmdb: Option<&TmdbClient>,
    force: bool,
    verbose: bool,
) {
    let Some(tmdb_client) = tmdb else { return };
    let Some(tmdb_id) = meta.tmdb_id else { return };

    // 尝试获取 TMDB 详情和图片
    match tmdb_client.find_by_tmdb_id(tmdb_id).await {
        Ok(show) => {
            // 下载海报
            let poster_path = anime_dir.join("poster.jpg");
            if force || !poster_path.exists() {
                if let Some(url) = tmdb_client.poster_url(&show) {
                    match tmdb_client.download_image(&url, &poster_path).await {
                        Ok(()) => {
                            if verbose {
                                eprintln!("已下载海报: {}", poster_path.display());
                            }
                        }
                        Err(e) => eprintln!("海报下载失败: {e}"),
                    }
                }
            }

            // 下载背景图
            let fanart_path = anime_dir.join("fanart.jpg");
            if force || !fanart_path.exists() {
                if let Some(url) = tmdb_client.backdrop_url(&show) {
                    match tmdb_client.download_image(&url, &fanart_path).await {
                        Ok(()) => {
                            if verbose {
                                eprintln!("已下载背景图: {}", fanart_path.display());
                            }
                        }
                        Err(e) => eprintln!("背景图下载失败: {e}"),
                    }
                }
            }
        }
        Err(e) => {
            if verbose {
                eprintln!("TMDB 获取失败 (tmdb_id={tmdb_id}): {e}");
            }
        }
    }
}

/// 创建单集 NFO
fn create_episode_nfo(file: &anime_organizer::AnimeFileInfo, meta: &AnimeMetadata) -> EpisodeNfo {
    let episode_num: u32 = file.episode.trim().parse().unwrap_or(0);

    EpisodeNfo {
        title: format!("Episode {}", file.episode.trim()),
        season: 1,
        episode: episode_num,
        plot: None,
        aired: meta.air_date.clone(),
        runtime: None,
        displayseason: None,
        displayepisode: None,
        uniqueid: vec![UniqueId {
            id_type: "bangumi".to_string(),
            default: true,
            value: meta.bangumi_id.to_string(),
        }],
        credits: Vec::new(),
        director: meta.director.iter().cloned().collect(),
        actor: Vec::new(),
    }
}

/// 整理单个文件（含回退逻辑）
fn organize_file(
    anime_file: &anime_organizer::AnimeFileInfo,
    target: &Path,
    mode: OperationMode,
    dry_run: bool,
    fallback_mode: Option<OperationMode>,
    verbose: bool,
) -> Result<(), AppError> {
    match FileOrganizer::organize(anime_file, target, mode, dry_run) {
        Ok(()) => {
            if verbose && !dry_run {
                println!(
                    "成功: {} -> {}/{}",
                    anime_file.original_path,
                    anime_file.anime_name,
                    anime_file.target_filename()
                );
            }
            Ok(())
        }
        Err(e) => {
            if mode == OperationMode::Link {
                if let Some(fb_mode) = fallback_mode {
                    if matches!(
                        e,
                        AppError::CrossDeviceLink | AppError::HardLinkNotSupported
                    ) {
                        if verbose {
                            eprintln!(
                                "硬链接失败，回退为 {}: {}",
                                fb_mode, anime_file.original_path
                            );
                        }
                        return FileOrganizer::organize(anime_file, target, fb_mode, dry_run)
                            .map_err(|e2| {
                                eprintln!("处理文件失败 {}: {e2}", anime_file.original_path);
                                e2
                            });
                    }
                }
            }
            eprintln!("处理文件失败 {}: {e}", anime_file.original_path);
            Err(e)
        }
    }
}

/// 构建扩展名集合
fn build_extensions(include_ext: &Option<Vec<String>>) -> HashSet<String> {
    match include_ext {
        Some(exts) => exts
            .iter()
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
    }
}

/// 检查文件是否有有效扩展名
fn has_valid_extension(path: &Path, extensions: &HashSet<String>) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| extensions.contains(&format!(".{}", e.to_lowercase())))
        .unwrap_or(false)
}
