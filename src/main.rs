//! anime-organizer 命令行入口
//!
//! 提供默认的文件整理模式，以及用于自动化工作流的 scraper 子命令。

#[cfg(feature = "scraper")]
use anime_organizer::scraper::{
    db_builder::build_bangumi_db,
    matcher::{format_github_output, match_aliases},
    ScrapedAnime, Scraper,
};
use anime_organizer::{
    error::AppError, AnimeFileInfo, FileOrganizer, FilenameParser, OperationMode,
};
#[cfg(feature = "metadata")]
use anime_organizer::{
    metadata::{tmdb::TmdbTvShow, AliasLookup, BangumiClient, TmdbClient},
    nfo::{EpisodeNfo, NfoWriter, TvShowNfo, UniqueId},
    AnimeMetadata,
};
#[cfg(any(
    feature = "scraper",
    feature = "clouddrive",
    feature = "torrent-scraper"
))]
use clap::Subcommand;
use clap::{Args, Parser, ValueEnum};
#[cfg(feature = "scraper")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
#[cfg(feature = "scraper")]
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 默认支持的视频扩展名
const DEFAULT_EXTENSIONS: &[&str] = &[".mp4", ".mkv", ".avi", ".mov", ".wmv", ".flv", ".rmvb"];
const APP_LONG_ABOUT: &str = concat!(
    "AnimeOrganizer v",
    env!("CARGO_PKG_VERSION"),
    " - 跨平台动漫文件整理工具\n\n",
    "默认模式用于批量整理动漫文件：\n",
    "    aniorg --source=\"D:\\Downloads\" --target=\"E:\\Anime\"\n\n",
    "启用元数据刮削：\n",
    "    aniorg --source=\"D:\\Downloads\" --scrape-metadata --tmdb-api-key=\"...\"\n\n",
    "启用 scraper 子命令（需以 --features scraper 编译）：\n",
    "    aniorg scrape --days 7 --format json\n",
    "    aniorg match --input scraped.json --format github\n"
);

/// 跨平台动漫文件整理工具
#[derive(Parser, Debug)]
#[command(name = "aniorg")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "轻量级、跨平台动漫文件整理工具")]
#[command(long_about = APP_LONG_ABOUT)]
struct Cli {
    #[cfg(any(
        feature = "scraper",
        feature = "clouddrive",
        feature = "torrent-scraper"
    ))]
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    organize: OrganizeArgs,
}

#[derive(Args, Debug, Clone)]
struct OrganizeArgs {
    /// 源目录路径（整理模式必填）
    #[arg(short, long, value_name = "PATH")]
    source: Option<PathBuf>,

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
    #[arg(long = "scrape-metadata", visible_alias = "刮削")]
    scrape_metadata: bool,

    /// TMDB API Key（用于下载封面图片）
    #[arg(long, value_name = "KEY")]
    tmdb_api_key: Option<String>,

    /// 跳过图片下载
    #[arg(long)]
    no_images: bool,

    /// 覆盖已有的 NFO 和图片文件
    #[arg(long)]
    force_overwrite: bool,

    /// Bangumi 缓存目录
    #[arg(long, value_name = "PATH")]
    bangumi_cache: Option<PathBuf>,

    /// Bangumi 元数据源路径（subject.jsonlines 或包含该文件的目录）
    #[arg(long, value_name = "PATH")]
    metadata_source: Option<PathBuf>,

    /// 启用分季模式：按 `番名/Season N/` 结构整理文件
    #[arg(long = "season-mode", visible_alias = "分季")]
    season_mode: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum FallbackMode {
    /// 移动文件
    Move,
    /// 复制文件
    Copy,
}

impl FallbackMode {
    fn to_operation_mode(self) -> OperationMode {
        match self {
            Self::Move => OperationMode::Move,
            Self::Copy => OperationMode::Copy,
        }
    }
}

#[cfg(any(
    feature = "scraper",
    feature = "clouddrive",
    feature = "torrent-scraper"
))]
#[derive(Subcommand, Debug)]
enum Commands {
    #[cfg(feature = "scraper")]
    Scrape(ScrapeArgs),
    #[cfg(feature = "scraper")]
    /// 根据刮削结果生成别名提案
    Match(MatchArgs),
    #[cfg(feature = "scraper")]
    /// 从 Bangumi Archive 构建 SQLite 数据库
    BuildDb(BuildDbArgs),
    #[cfg(feature = "scraper")]
    /// 从 Bangumi dump 中提取别名信息
    ExtractAliases(ExtractAliasesArgs),
    #[cfg(feature = "scraper")]
    /// Merge new aliases from input JSON into database
    MergeAliases(MergeAliasesArgs),
    #[cfg(feature = "scraper")]
    /// Apply confident match proposals to database
    ApplyMatches(ApplyMatchesArgs),
    #[cfg(feature = "scraper")]
    /// Create GitHub issues for uncertain alias match proposals
    CreateAliasIssues(CreateAliasIssuesArgs),
    #[cfg(feature = "clouddrive")]
    /// RSS 订阅管理
    Rss(RssArgs),
    #[cfg(feature = "clouddrive")]
    /// 直接提交 magnet/torrent URL 到 115 网盘离线下载
    AddOffline(AddOfflineArgs),
    #[cfg(feature = "clouddrive")]
    /// 列出云盘目录内容
    ListFolder(ListFolderArgs),
    #[cfg(feature = "torrent-scraper")]
    /// 爬取 DMHY/Nyaa 的番剧种子文件名
    TorrentScrape(TorrentScrapeArgs),
}

#[cfg(feature = "scraper")]
#[derive(Args, Debug, Clone)]
struct ScrapeArgs {
    /// 向前回溯的天数
    #[arg(long, default_value_t = 7)]
    days: u32,

    /// 输出格式
    #[arg(long, value_enum, default_value = "json")]
    format: ScrapeOutputFormat,

    /// TMDB API Key；未传时尝试读取环境变量 TMDB_API_KEY
    #[arg(long, value_name = "KEY")]
    tmdb_api_key: Option<String>,
}

#[cfg(feature = "scraper")]
#[derive(Clone, Copy, Debug, ValueEnum)]
enum ScrapeOutputFormat {
    Json,
    Pretty,
}

#[cfg(feature = "scraper")]
#[derive(Args, Debug, Clone)]
struct MatchArgs {
    /// scrape 子命令生成的 JSON 文件
    #[arg(long, value_name = "PATH")]
    input: PathBuf,

    /// 输出格式
    #[arg(long, value_enum, default_value = "github")]
    format: MatchOutputFormat,
}

#[cfg(feature = "scraper")]
#[derive(Clone, Copy, Debug, ValueEnum)]
enum MatchOutputFormat {
    Json,
    Github,
}

#[cfg(feature = "scraper")]
#[derive(Args, Debug, Clone)]
struct BuildDbArgs {
    #[arg(long, value_name = "PATH")]
    output: PathBuf,

    #[arg(long, default_value = "false")]
    include_relations: bool,

    #[arg(long, short, default_value = "false")]
    verbose: bool,
}

#[cfg(feature = "scraper")]
#[derive(Args, Debug, Clone)]
struct ExtractAliasesArgs {
    /// 本地 subject.jsonlines 文件路径
    #[arg(long, value_name = "PATH")]
    input: Option<PathBuf>,

    /// 从 Bangumi Archive 下载最新的 dump
    #[arg(long)]
    download: bool,

    /// 输出文件路径（默认stdout）
    #[arg(long, value_name = "PATH")]
    output: Option<PathBuf>,
}

#[cfg(feature = "scraper")]
#[derive(Args, Debug, Clone)]
struct MergeAliasesArgs {
    /// JSON file containing new aliases to merge
    #[arg(long, value_name = "PATH")]
    input: PathBuf,

    /// Target database file (default: bangumi.db in current directory)
    #[arg(long, value_name = "PATH")]
    target: Option<PathBuf>,
}

#[cfg(feature = "scraper")]
#[derive(Args, Debug, Clone)]
struct ApplyMatchesArgs {
    /// JSON file containing confident match proposals
    #[arg(long, value_name = "PATH")]
    input: PathBuf,

    /// Target database file (default: bangumi.db in current directory)
    #[arg(long, value_name = "PATH")]
    target: Option<PathBuf>,
}

#[cfg(feature = "scraper")]
#[derive(Args, Debug, Clone)]
struct CreateAliasIssuesArgs {
    /// JSON file containing uncertain match proposals
    #[arg(long, value_name = "PATH")]
    input: PathBuf,

    /// Repository owner/name (e.g., ModerRAS/anime-organizer)
    #[arg(long, value_name = "REPO")]
    repo: Option<String>,
}

#[cfg(feature = "clouddrive")]
#[derive(Args, Debug, Clone)]
struct RssArgs {
    /// 持续运行的 Daemon 模式
    #[arg(long)]
    daemon: bool,

    /// 单次执行模式
    #[arg(long)]
    single_shot: bool,

    /// RSS 订阅 URL
    #[arg(long, value_name = "URL")]
    rss_url: Option<String>,

    /// 正则过滤表达式
    #[arg(long, value_name = "REGEX")]
    rss_filter: Option<String>,

    /// 轮询间隔（秒）
    #[arg(long, default_value_t = 300, value_name = "SECS")]
    rss_interval: u64,

    /// 115网盘目标目录
    #[arg(long, value_name = "PATH")]
    rss_target: Option<String>,

    /// CloudDrive2 服务地址（如 http://localhost:19798）
    #[arg(long, value_name = "URL")]
    clouddrive_url: Option<String>,

    /// CloudDrive2 JWT 令牌（已有令牌时直接使用）
    #[arg(long, value_name = "TOKEN")]
    clouddrive_token: Option<String>,

    /// CloudDrive2 用户名（用于登录获取令牌）
    #[arg(long, value_name = "USER")]
    clouddrive_user: Option<String>,

    /// CloudDrive2 密码
    #[arg(long, value_name = "PASS")]
    clouddrive_pass: Option<String>,

    /// 添加 RSS 订阅到数据库
    #[arg(long)]
    add_subscription: bool,

    /// 列出所有已保存的订阅
    #[arg(long)]
    list_subscriptions: bool,
}

#[cfg(feature = "clouddrive")]
#[derive(Args, Debug, Clone)]
struct AddOfflineArgs {
    /// magnet 链接或 .torrent 文件 URL
    #[arg(value_name = "MAGNET_OR_URL")]
    url: String,

    /// 115网盘目标目录
    #[arg(long, short = 't', value_name = "PATH")]
    target: String,

    /// CloudDrive2 服务地址
    #[arg(long, value_name = "URL")]
    clouddrive_url: String,

    /// CloudDrive2 JWT 令牌
    #[arg(long, value_name = "TOKEN")]
    clouddrive_token: String,
}

#[cfg(feature = "clouddrive")]
#[derive(Args, Debug, Clone)]
struct ListFolderArgs {
    /// CloudDrive2 服务地址
    #[arg(long, value_name = "URL")]
    clouddrive_url: String,

    /// CloudDrive2 JWT 令牌
    #[arg(long, value_name = "TOKEN")]
    clouddrive_token: String,

    /// 要浏览的目录路径（默认根目录）
    #[arg(value_name = "PATH", default_value = "/")]
    path: String,
}

#[cfg(feature = "torrent-scraper")]
#[derive(Args, Debug, Clone)]
struct TorrentScrapeArgs {
    /// 数据来源：dmhy、nyaa 或 all
    #[arg(long, default_value = "all")]
    source: TorrentSource,

    /// Nyaa 搜索关键词（仅用于 nyaa 源）
    #[arg(long, value_name = "KEYWORD")]
    query: Option<String>,

    /// 爬取页数（每页约 75 条）
    #[arg(long, default_value_t = 1)]
    pages: u32,

    /// 输出文件路径
    #[arg(long, short = 'o', value_name = "PATH")]
    output: Option<PathBuf>,

    /// 显示浏览器窗口（用于调试）
    #[arg(long)]
    headed: bool,
}

#[cfg(feature = "torrent-scraper")]
#[derive(Clone, Copy, Debug, ValueEnum)]
enum TorrentSource {
    Dmhy,
    Nyaa,
    All,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("错误: {error}");
        std::process::exit(1);
    }
}

#[cfg(any(
    feature = "scraper",
    feature = "clouddrive",
    feature = "torrent-scraper"
))]
fn run() -> Result<(), AppError> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        return run_command(command);
    }

    run_organize_entry(cli.organize)
}

#[cfg(not(any(
    feature = "scraper",
    feature = "clouddrive",
    feature = "torrent-scraper"
)))]
fn run() -> Result<(), AppError> {
    let cli = Cli::parse();
    run_organize_entry(cli.organize)
}

#[cfg(feature = "metadata")]
fn run_organize_entry(args: OrganizeArgs) -> Result<(), AppError> {
    if args.scrape_metadata {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| AppError::MetadataFetchError(format!("创建异步运行时失败: {e}")))?;
        runtime.block_on(run_with_metadata(args))
    } else {
        run_organize(args)
    }
}

#[cfg(not(feature = "metadata"))]
fn run_organize_entry(args: OrganizeArgs) -> Result<(), AppError> {
    if args.scrape_metadata {
        return Err(AppError::MetadataFetchError(
            "元数据功能未启用，请使用 --features metadata 编译".to_string(),
        ));
    }

    run_organize(args)
}

#[cfg(any(
    feature = "scraper",
    feature = "clouddrive",
    feature = "torrent-scraper"
))]
fn run_command(command: Commands) -> Result<(), AppError> {
    match command {
        #[cfg(feature = "scraper")]
        Commands::Scrape(args) => {
            let runtime = tokio::runtime::Runtime::new()
                .map_err(|e| AppError::MetadataFetchError(format!("创建异步运行时失败: {e}")))?;
            runtime.block_on(run_scrape(args))
        }
        #[cfg(feature = "scraper")]
        Commands::Match(args) => run_match(args),
        #[cfg(feature = "scraper")]
        Commands::BuildDb(args) => run_build_db(args),
        #[cfg(feature = "scraper")]
        Commands::ExtractAliases(args) => run_extract_aliases(args),
        #[cfg(feature = "scraper")]
        Commands::MergeAliases(args) => run_merge_aliases(args),
        #[cfg(feature = "scraper")]
        Commands::ApplyMatches(args) => run_apply_matches(args),
        #[cfg(feature = "scraper")]
        Commands::CreateAliasIssues(args) => run_create_alias_issues(args),
        #[cfg(feature = "clouddrive")]
        Commands::Rss(args) => run_rss(args),
        #[cfg(feature = "clouddrive")]
        Commands::AddOffline(args) => run_add_offline(args),
        #[cfg(feature = "clouddrive")]
        Commands::ListFolder(args) => run_list_folder(args),
        #[cfg(feature = "torrent-scraper")]
        Commands::TorrentScrape(args) => {
            let runtime = tokio::runtime::Runtime::new()
                .map_err(|e| AppError::MetadataFetchError(format!("创建异步运行时失败: {e}")))?;
            runtime.block_on(run_torrent_scrape(args))
        }
    }
}

/// 仅文件整理流程（无元数据）
fn run_organize(args: OrganizeArgs) -> Result<(), AppError> {
    let (source, target) = resolve_source_and_target(&args)?;
    let fallback_mode = args
        .fallback_on_link_failure
        .map(FallbackMode::to_operation_mode);
    let extensions = build_extensions(&args.include_ext);

    let mut processed = 0;
    let mut succeeded = 0;
    let mut failed = 0;

    for entry in WalkDir::new(&source)
        .into_iter()
        .filter_map(|item| item.ok())
        .filter(|item| item.file_type().is_file())
    {
        let path = entry.path();
        if !has_valid_extension(path, &extensions) {
            continue;
        }

        let anime_file = match FilenameParser::parse(path) {
            Some(info) => info,
            None => {
                if args.verbose {
                    eprintln!(
                        "跳过：无法解析文件名 {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    );
                }
                continue;
            }
        };

        processed += 1;
        let target_dir = if args.season_mode {
            target
                .join(anime_file.series_name())
                .join(anime_file.season_dir_name())
        } else {
            target.join(&anime_file.anime_name)
        };
        match organize_file_to_dir(
            &anime_file,
            &target_dir,
            args.mode,
            args.dry_run,
            fallback_mode,
            args.verbose,
        ) {
            Ok(_) => succeeded += 1,
            Err(_) => failed += 1,
        }
    }

    println!("处理完成：总计{processed}个文件，成功{succeeded}个，失败{failed}个");
    Ok(())
}

/// 带元数据刮削的流程
#[cfg(feature = "metadata")]
async fn run_with_metadata(args: OrganizeArgs) -> Result<(), AppError> {
    let (source, target) = resolve_source_and_target(&args)?;
    let fallback_mode = args
        .fallback_on_link_failure
        .map(FallbackMode::to_operation_mode);
    let extensions = build_extensions(&args.include_ext);

    let cache_dir = args
        .bangumi_cache
        .clone()
        .unwrap_or_else(std::env::temp_dir);
    let db_path = cache_dir.join("bangumi.db");
    let alias_lookup = AliasLookup::load(&db_path)?;
    if args.verbose {
        eprintln!("已加载 {} 条别名", alias_lookup.len());
    }

    let bangumi =
        BangumiClient::with_source(args.bangumi_cache.clone(), args.metadata_source.clone());
    match bangumi.prepare_index().await {
        Ok(count) => {
            if args.verbose {
                eprintln!("Bangumi 索引已就绪，共 {} 条动画条目", count);
            }
        }
        Err(error) => {
            if args.verbose {
                eprintln!("Bangumi 本地索引准备失败，将回退到在线查询: {error}");
            }
        }
    }

    let tmdb = args.tmdb_api_key.clone().map(TmdbClient::new);
    if args.scrape_metadata && !args.no_images && tmdb.is_none() && args.verbose {
        eprintln!("未提供 TMDB API Key，将跳过 TMDB 图片下载，仅保留 NFO 生成");
    }

    let anime_groups = collect_anime_groups(&source, &extensions, args.verbose);
    let mut processed = 0;
    let mut succeeded = 0;
    let mut failed = 0;
    let mut metadata_cache: HashMap<String, Option<AnimeMetadata>> = HashMap::new();

    for (anime_name, files) in anime_groups {
        let Some(first_file) = files.first() else {
            continue;
        };

        let series_name = first_file.series_name();
        let season_number = first_file.season_number().unwrap_or(1);
        let anime_root = target.join(&series_name);

        let metadata = if let Some(cached) = metadata_cache.get(&anime_name) {
            cached.clone()
        } else {
            let fetched = fetch_anime_metadata(
                &anime_name,
                &series_name,
                &alias_lookup,
                &bangumi,
                tmdb.as_ref(),
                args.verbose,
            )
            .await;
            metadata_cache.insert(anime_name.clone(), fetched.clone());
            fetched
        };

        if let Some(ref meta) = metadata {
            let tvshow_nfo_path = anime_root.join("tvshow.nfo");
            if args.force_overwrite || !tvshow_nfo_path.exists() {
                let nfo = TvShowNfo::from(meta);
                if args.dry_run {
                    if args.verbose {
                        eprintln!("[dry-run] 生成 tvshow.nfo: {}", tvshow_nfo_path.display());
                    }
                } else {
                    NfoWriter::write_tvshow(&anime_root, &nfo)?;
                    if args.verbose {
                        eprintln!("已生成 tvshow.nfo: {}", tvshow_nfo_path.display());
                    }
                }
            }

            if !args.no_images && !args.dry_run {
                download_images(
                    meta,
                    &anime_root,
                    season_number,
                    tmdb.as_ref(),
                    args.force_overwrite,
                    args.verbose,
                )
                .await;
            }
        }

        for file in files {
            let season_dir = target.join(file.series_name()).join(file.season_dir_name());
            processed += 1;

            match organize_file_to_dir(
                &file,
                &season_dir,
                args.mode,
                args.dry_run,
                fallback_mode,
                args.verbose,
            ) {
                Ok(target_path) => {
                    succeeded += 1;

                    if let Some(ref meta) = metadata {
                        let episode_nfo_path = target_path.with_extension("nfo");
                        if args.force_overwrite || !episode_nfo_path.exists() {
                            let episode_nfo = create_episode_nfo(&file, meta);
                            if args.dry_run {
                                if args.verbose {
                                    eprintln!(
                                        "[dry-run] 生成 episode.nfo: {}",
                                        episode_nfo_path.display()
                                    );
                                }
                            } else if let Err(error) =
                                NfoWriter::write_episode(&episode_nfo_path, &episode_nfo)
                            {
                                eprintln!("生成 episode.nfo 失败: {error}");
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
        let matched = metadata_cache
            .values()
            .filter(|item| item.is_some())
            .count();
        println!("元数据匹配：{matched}/{} 部动画", metadata_cache.len());
    }

    Ok(())
}

#[cfg(feature = "metadata")]
async fn fetch_anime_metadata(
    anime_name: &str,
    series_name: &str,
    alias_lookup: &AliasLookup,
    bangumi: &BangumiClient,
    tmdb: Option<&TmdbClient>,
    verbose: bool,
) -> Option<AnimeMetadata> {
    let mut metadata = None;
    let mut anidb_id = None;

    let alias = [anime_name, series_name].into_iter().find_map(|query| {
        alias_lookup
            .find(query)
            .or_else(|| alias_lookup.find_fuzzy(query))
    });

    if let Some(entry) = alias {
        if verbose {
            eprintln!(
                "别名匹配: {} -> {} (bangumi_id={})",
                anime_name, entry.name, entry.bangumi_id
            );
        }

        match bangumi.fetch_metadata(entry.bangumi_id).await {
            Ok(mut meta) => {
                meta.tmdb_id = entry.tmdb_id;
                meta.anidb_id = entry.anidb_id;
                metadata = Some(meta);
                anidb_id = entry.anidb_id;
            }
            Err(error) => {
                if verbose {
                    eprintln!("Bangumi 获取失败 {}: {error}", entry.bangumi_id);
                }
            }
        }
    }

    if metadata.is_none() {
        let subject = bangumi
            .find_by_name(anime_name)
            .ok()
            .flatten()
            .or_else(|| bangumi.find_by_name(series_name).ok().flatten())
            .or_else(|| {
                bangumi
                    .search(series_name)
                    .ok()
                    .and_then(|mut matches| matches.drain(..).next())
            });

        if let Some(subject) = subject {
            if verbose {
                eprintln!("Bangumi 名称匹配: {} -> {}", anime_name, subject.name);
            }
            if let Ok(meta) = bangumi.fetch_metadata(subject.id).await {
                metadata = Some(meta);
            }
        }
    }

    let mut metadata = metadata?;
    if metadata.anidb_id.is_none() {
        metadata.anidb_id = anidb_id;
    }

    if metadata.tmdb_id.is_none() {
        if let Some(tmdb_client) = tmdb {
            let year = metadata.air_date.as_deref().and_then(parse_year);
            for candidate in unique_titles(
                &metadata.title,
                metadata.title_cn.as_deref(),
                Some(&metadata.original_title),
            ) {
                match tmdb_client.find_by_title(&candidate, year).await {
                    Ok(Some(show)) => {
                        metadata.tmdb_id = Some(show.id);
                        if verbose {
                            eprintln!(
                                "TMDB 搜索匹配: {} -> {} (tmdb_id={})",
                                candidate, show.name, show.id
                            );
                        }
                        break;
                    }
                    Ok(None) => continue,
                    Err(error) => {
                        if verbose {
                            eprintln!("TMDB 搜索失败 {}: {error}", candidate);
                        }
                    }
                }
            }
        }
    }

    Some(metadata)
}

#[cfg(feature = "metadata")]
async fn download_images(
    meta: &AnimeMetadata,
    anime_root: &Path,
    season_number: u32,
    tmdb: Option<&TmdbClient>,
    force: bool,
    verbose: bool,
) {
    let root_poster_path = anime_root.join("poster.jpg");
    let season_poster_path = anime_root.join(format!("season{season_number:02}-poster.jpg"));
    let fanart_path = anime_root.join("fanart.jpg");
    let mut poster_written = false;

    if let Some(tmdb_client) = tmdb {
        if let Some(show) = resolve_tmdb_show(meta, tmdb_client, verbose).await {
            if let Ok(Some(url)) = tmdb_client.best_poster_url(&show).await {
                let needs_root = force || !root_poster_path.exists();
                let needs_season = force || !season_poster_path.exists();
                if needs_root || needs_season {
                    match tmdb_client.download_image_bytes(&url).await {
                        Ok(bytes) => {
                            if needs_root {
                                if let Err(error) =
                                    NfoWriter::write_image(&root_poster_path, &bytes)
                                {
                                    eprintln!("海报写入失败: {error}");
                                } else if verbose {
                                    eprintln!("已下载海报: {}", root_poster_path.display());
                                }
                            }

                            if needs_season {
                                if let Err(error) =
                                    NfoWriter::write_image(&season_poster_path, &bytes)
                                {
                                    eprintln!("季海报写入失败: {error}");
                                } else if verbose {
                                    eprintln!("已下载季海报: {}", season_poster_path.display());
                                }
                            }

                            poster_written = true;
                        }
                        Err(error) => eprintln!("海报下载失败: {error}"),
                    }
                } else {
                    poster_written = true;
                }
            }

            if force || !fanart_path.exists() {
                match tmdb_client.best_backdrop_url(&show).await {
                    Ok(Some(url)) => match tmdb_client.download_image_bytes(&url).await {
                        Ok(bytes) => {
                            if let Err(error) = NfoWriter::write_image(&fanart_path, &bytes) {
                                eprintln!("背景图写入失败: {error}");
                            } else if verbose {
                                eprintln!("已下载背景图: {}", fanart_path.display());
                            }
                        }
                        Err(error) => eprintln!("背景图下载失败: {error}"),
                    },
                    Ok(None) => {}
                    Err(error) => eprintln!("背景图获取失败: {error}"),
                }
            }
        }
    }

    if !poster_written && (force || !root_poster_path.exists()) {
        if let (Some(tmdb_client), Some(anidb_id)) = (tmdb, meta.anidb_id) {
            match tmdb_client
                .download_anidb_poster(anidb_id, &root_poster_path)
                .await
            {
                Ok(()) => {
                    if verbose {
                        eprintln!("已从 AniDB 下载海报: {}", root_poster_path.display());
                    }

                    if force || !season_poster_path.exists() {
                        match std::fs::read(&root_poster_path) {
                            Ok(bytes) => {
                                if let Err(error) =
                                    NfoWriter::write_image(&season_poster_path, &bytes)
                                {
                                    eprintln!("季海报写入失败: {error}");
                                }
                            }
                            Err(error) => eprintln!("读取 AniDB 海报失败: {error}"),
                        }
                    }
                }
                Err(error) => {
                    if verbose {
                        eprintln!("AniDB 回退失败 (aid={anidb_id}): {error}");
                    }
                }
            }
        }
    }
}

#[cfg(feature = "metadata")]
async fn resolve_tmdb_show(
    meta: &AnimeMetadata,
    tmdb_client: &TmdbClient,
    verbose: bool,
) -> Option<TmdbTvShow> {
    if let Some(tmdb_id) = meta.tmdb_id {
        match tmdb_client.find_by_tmdb_id(tmdb_id).await {
            Ok(show) => return Some(show),
            Err(error) => {
                if verbose {
                    eprintln!("TMDB 详情获取失败 (tmdb_id={tmdb_id}): {error}");
                }
            }
        }
    }

    let year = meta.air_date.as_deref().and_then(parse_year);
    for title in unique_titles(
        &meta.title,
        meta.title_cn.as_deref(),
        Some(&meta.original_title),
    ) {
        match tmdb_client.find_by_title(&title, year).await {
            Ok(Some(show)) => return Some(show),
            Ok(None) => continue,
            Err(error) => {
                if verbose {
                    eprintln!("TMDB 搜索失败 {}: {error}", title);
                }
            }
        }
    }

    None
}

#[cfg(feature = "metadata")]
fn create_episode_nfo(file: &AnimeFileInfo, meta: &AnimeMetadata) -> EpisodeNfo {
    let episode_number = file.episode.trim().parse().unwrap_or(0);

    EpisodeNfo {
        title: format!("Episode {}", file.episode.trim()),
        season: file.season_number().unwrap_or(1),
        episode: episode_number,
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
        tagline: None,
        playcount: None,
        lastplayed: None,
    }
}

#[allow(dead_code)]
fn organize_file(
    anime_file: &AnimeFileInfo,
    target: &Path,
    mode: OperationMode,
    dry_run: bool,
    fallback_mode: Option<OperationMode>,
    verbose: bool,
) -> Result<(), AppError> {
    let target_dir = target.join(&anime_file.anime_name);
    organize_file_to_dir(
        anime_file,
        &target_dir,
        mode,
        dry_run,
        fallback_mode,
        verbose,
    )
    .map(|_| ())
}

fn organize_file_to_dir(
    anime_file: &AnimeFileInfo,
    target_dir: &Path,
    mode: OperationMode,
    dry_run: bool,
    fallback_mode: Option<OperationMode>,
    verbose: bool,
) -> Result<PathBuf, AppError> {
    match FileOrganizer::organize_to_dir(anime_file, target_dir, mode, dry_run) {
        Ok(target_path) => {
            if verbose && !dry_run {
                println!(
                    "成功: {} -> {}",
                    anime_file.original_path,
                    target_path.display()
                );
            }
            Ok(target_path)
        }
        Err(error) => {
            if mode == OperationMode::Link {
                if let Some(fallback) = fallback_mode {
                    if matches!(
                        error,
                        AppError::CrossDeviceLink | AppError::HardLinkNotSupported
                    ) {
                        if verbose {
                            eprintln!(
                                "硬链接失败，回退为 {}: {}",
                                fallback, anime_file.original_path
                            );
                        }

                        return FileOrganizer::organize_to_dir(
                            anime_file, target_dir, fallback, dry_run,
                        )
                        .map_err(|fallback_error| {
                            eprintln!(
                                "处理文件失败 {}: {fallback_error}",
                                anime_file.original_path
                            );
                            fallback_error
                        });
                    }
                }
            }

            eprintln!("处理文件失败 {}: {error}", anime_file.original_path);
            Err(error)
        }
    }
}

fn collect_anime_groups(
    source: &Path,
    extensions: &HashSet<String>,
    verbose: bool,
) -> HashMap<String, Vec<AnimeFileInfo>> {
    let mut groups: HashMap<String, Vec<AnimeFileInfo>> = HashMap::new();

    for entry in WalkDir::new(source)
        .into_iter()
        .filter_map(|item| item.ok())
        .filter(|item| item.file_type().is_file())
    {
        let path = entry.path();
        if !has_valid_extension(path, extensions) {
            continue;
        }

        if let Some(info) = FilenameParser::parse(path) {
            groups
                .entry(info.anime_name.clone())
                .or_default()
                .push(info);
        } else if verbose {
            eprintln!(
                "跳过：无法解析文件名 {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            );
        }
    }

    groups
}

fn resolve_source_and_target(args: &OrganizeArgs) -> Result<(PathBuf, PathBuf), AppError> {
    let source = args.source.clone().ok_or_else(|| {
        AppError::ParseError("整理模式下必须提供 --source；若要使用工作流子命令，请执行 aniorg scrape 或 aniorg match".to_string())
    })?;

    if !source.exists() {
        return Err(AppError::SourceNotFound(source));
    }

    let target = args.target.clone().unwrap_or_else(|| source.clone());
    if !target.exists() {
        return Err(AppError::TargetNotFound(target));
    }

    Ok((source, target))
}

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
            .map(|ext| (*ext).to_string())
            .collect(),
    }
}

fn has_valid_extension(path: &Path, extensions: &HashSet<String>) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.contains(&format!(".{}", ext.to_lowercase())))
        .unwrap_or(false)
}

fn parse_year(value: &str) -> Option<i32> {
    value.get(0..4)?.parse().ok()
}

fn unique_titles(primary: &str, secondary: Option<&str>, tertiary: Option<&str>) -> Vec<String> {
    let mut titles = Vec::new();

    for value in [Some(primary), secondary, tertiary].into_iter().flatten() {
        let trimmed = value.trim();
        if !trimmed.is_empty() && !titles.iter().any(|item| item == trimmed) {
            titles.push(trimmed.to_string());
        }
    }

    titles
}

#[cfg(feature = "scraper")]
async fn run_scrape(args: ScrapeArgs) -> Result<(), AppError> {
    let scraper = Scraper::new();
    let tmdb_api_key = args
        .tmdb_api_key
        .clone()
        .or_else(|| std::env::var("TMDB_API_KEY").ok());
    let scraped = scraper
        .scrape_all(args.days, tmdb_api_key.as_deref())
        .await?;

    match args.format {
        ScrapeOutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&scraped).map_err(
                    |e| AppError::MetadataFetchError(format!("序列化刮削结果失败: {e}"))
                )?
            );
        }
        ScrapeOutputFormat::Pretty => {
            for anime in scraped {
                println!(
                    "{} | {} | {} | {}",
                    anime.source,
                    anime.date.unwrap_or_else(|| "N/A".to_string()),
                    anime.title,
                    anime.title_cn.unwrap_or_else(|| "-".to_string())
                );
            }
        }
    }

    Ok(())
}

#[cfg(feature = "scraper")]
fn run_match(args: MatchArgs) -> Result<(), AppError> {
    let input = std::fs::read_to_string(&args.input)
        .map_err(|e| AppError::MetadataFetchError(format!("读取刮削输入失败: {e}")))?;
    let scraped: Vec<ScrapedAnime> = serde_json::from_str(&input)
        .map_err(|e| AppError::MetadataFetchError(format!("解析刮削输入失败: {e}")))?;

    let db_path = PathBuf::from("./bangumi.db");
    let aliases = AliasLookup::load(&db_path)?;
    let result = match_aliases(&scraped, aliases.entries());

    match args.format {
        MatchOutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&result).map_err(|e| AppError::MetadataFetchError(
                    format!("序列化匹配结果失败: {e}")
                ))?
            );
        }
        MatchOutputFormat::Github => {
            println!("{}", format_github_output(&result));
        }
    }

    Ok(())
}

#[cfg(feature = "scraper")]
fn run_build_db(args: BuildDbArgs) -> Result<(), AppError> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| AppError::MetadataFetchError(format!("创建异步运行时失败: {e}")))?;
    let stats = runtime.block_on(build_bangumi_db(
        &args.output,
        args.include_relations,
        args.verbose,
    ))?;

    println!("=== Database Build Stats ===");
    println!("Subjects: {}", stats.subjects_count);
    println!("Episodes: {}", stats.episodes_count);
    println!("Aliases: {}", stats.aliases_count);
    println!("Relations: {}", stats.relations_count);
    println!(
        "Database size: {} bytes ({} KB, {} MB)",
        stats.db_size,
        stats.db_size / 1024,
        stats.db_size / (1024 * 1024)
    );
    println!(
        "Processing time: {} ms ({} s)",
        stats.processing_time_ms,
        stats.processing_time_ms as f64 / 1000.0
    );

    Ok(())
}

#[cfg(feature = "scraper")]
fn run_extract_aliases(args: ExtractAliasesArgs) -> Result<(), AppError> {
    use anime_organizer::metadata::wiki::WikiParser;
    use std::collections::HashMap;
    use std::io::{BufRead, BufReader};

    let dump_path = if args.download {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| AppError::MetadataFetchError(format!("创建异步运行时失败: {e}")))?;
        runtime.block_on(async {
            let client = anime_organizer::metadata::BangumiClient::new(None);
            client.download_dump().await
        })?
    } else if let Some(ref path) = args.input {
        path.clone()
    } else {
        let client = anime_organizer::metadata::BangumiClient::new(None);
        client.resolve_existing_dump_path().ok_or_else(|| {
            AppError::MetadataFetchError(
                "未找到本地 dump，请使用 --download 下载或 --input 指定路径".to_string(),
            )
        })?
    };

    let file = std::fs::File::open(&dump_path)
        .map_err(|e| AppError::BangumiParseError(format!("打开 dump 文件失败: {e}")))?;
    let reader = BufReader::new(file);

    let mut aliases_map: HashMap<String, AliasEntry> = HashMap::new();
    let parser = WikiParser::new();

    for line in reader.lines() {
        let line = line.map_err(|e| AppError::BangumiParseError(format!("读取行失败: {e}")))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let subject: anime_organizer::metadata::bangumi::BangumiSubject =
            match serde_json::from_str(line) {
                Ok(s) => s,
                Err(_) => continue,
            };

        if subject.subject_type != 2 {
            continue;
        }

        let Some(ref infobox) = subject.infobox else {
            continue;
        };

        let anime_info = match parser.parse_anime_infobox(infobox) {
            Ok(info) => info,
            Err(_) => continue,
        };

        if anime_info.aliases.is_empty() {
            continue;
        }

        let entry = AliasEntry {
            bangumi_id: subject.id,
            name: subject.name,
            name_cn: subject.name_cn,
        };

        for alias in anime_info.aliases {
            if !alias.is_empty() {
                aliases_map.insert(alias, entry.clone());
            }
        }
    }

    let output: Box<dyn std::io::Write> = if let Some(ref path) = args.output {
        Box::new(
            std::fs::File::create(path)
                .map_err(|e| AppError::MetadataFetchError(format!("创建输出文件失败: {e}")))?,
        )
    } else {
        Box::new(std::io::stdout())
    };

    let mut writer = std::io::BufWriter::new(output);
    serde_json::to_writer(&mut writer, &aliases_map)
        .map_err(|e| AppError::MetadataFetchError(format!("序列化别名失败: {e}")))?;
    writer
        .flush()
        .map_err(|e| AppError::MetadataFetchError(format!("写入输出失败: {e}")))?;

    eprintln!("成功提取 {} 个别名", aliases_map.len());

    Ok(())
}

#[cfg(feature = "scraper")]
fn run_merge_aliases(args: MergeAliasesArgs) -> Result<(), AppError> {
    use rusqlite::Connection;

    let db_path = PathBuf::from("bangumi.db");
    if !db_path.exists() {
        return Err(AppError::AliasLoadError(
            "数据库不存在，请先运行 build-db".to_string(),
        ));
    }

    let conn = Connection::open(&db_path)
        .map_err(|e| AppError::AliasLoadError(format!("打开数据库失败: {e}")))?;

    let content = std::fs::read_to_string(&args.input)
        .map_err(|e| AppError::AliasLoadError(format!("读取输入文件失败: {e}")))?;
    let aliases: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&content)
        .map_err(|e| AppError::AliasLoadError(format!("解析输入文件失败: {e}")))?;

    let mut new_count = 0;
    for (alias, value) in aliases {
        if alias.is_empty() {
            continue;
        }
        let bangumi_id = value
            .get("bangumi_id")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        if bangumi_id == 0 {
            continue;
        }
        let result = conn.execute(
            "INSERT OR IGNORE INTO aliases (subject_id, alias) VALUES (?1, ?2)",
            rusqlite::params![bangumi_id, alias],
        );
        if result.is_ok() {
            new_count += 1;
        }
    }

    println!("新增 {} 个别名到数据库", new_count);
    Ok(())
}

#[cfg(feature = "scraper")]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AliasEntry {
    bangumi_id: u32,
    name: String,
    name_cn: Option<String>,
}

#[cfg(feature = "scraper")]
fn run_apply_matches(args: ApplyMatchesArgs) -> Result<(), AppError> {
    use anime_organizer::scraper::matcher::Proposal;
    use rusqlite::Connection;

    let db_path = PathBuf::from("bangumi.db");
    if !db_path.exists() {
        return Err(AppError::AliasLoadError(
            "数据库不存在，请先运行 build-db".to_string(),
        ));
    }

    let conn = Connection::open(&db_path)
        .map_err(|e| AppError::AliasLoadError(format!("打开数据库失败: {e}")))?;

    let content = std::fs::read_to_string(&args.input)
        .map_err(|e| AppError::AliasLoadError(format!("读取提案文件失败: {e}")))?;
    let proposals: Vec<Proposal> = serde_json::from_str(&content)
        .map_err(|e| AppError::AliasLoadError(format!("解析提案文件失败: {e}")))?;

    let mut new_count = 0;
    for proposal in proposals {
        let alias = proposal.fan_translation;
        if alias.is_empty() {
            continue;
        }
        let subject_id = proposal.alias_entry.bangumi_id;
        let result = conn.execute(
            "INSERT OR IGNORE INTO aliases (subject_id, alias) VALUES (?1, ?2)",
            rusqlite::params![subject_id, alias],
        );
        if result.is_ok() {
            new_count += 1;
        }
    }

    println!("新增 {} 个别名到数据库", new_count);
    Ok(())
}

#[cfg(feature = "scraper")]
fn run_create_alias_issues(args: CreateAliasIssuesArgs) -> Result<(), AppError> {
    use anime_organizer::scraper::matcher::Proposal;
    use std::process::Command;

    let content = std::fs::read_to_string(&args.input)
        .map_err(|e| AppError::AliasLoadError(format!("读取提案文件失败: {e}")))?;
    let proposals: Vec<Proposal> = serde_json::from_str(&content)
        .map_err(|e| AppError::AliasLoadError(format!("解析提案文件失败: {e}")))?;

    let repo = args.repo.unwrap_or_else(|| {
        std::env::var("GITHUB_REPOSITORY")
            .unwrap_or_else(|_| "ModerRAS/anime-organizer".to_string())
    });

    let mut created = 0;
    for proposal in proposals {
        let fan = &proposal.fan_translation;
        if fan.is_empty() {
            continue;
        }
        let bgm_id = proposal.alias_entry.bangumi_id;
        let name = &proposal.alias_entry.name;
        let confidence = proposal.confidence.to_string();
        let reasoning = &proposal.reasoning;

        let title = format!("[Alias Request] {} -> {} (bgm:{})", fan, name, bgm_id);
        let body = format!(
            "## Anime Information\n\n- **Bangumi ID**: {}\n- **Bangumi Name**: {}\n- **Fan Translation**: {}\n\n## LLM Analysis\n\n- **Confidence**: {}\n- **Reasoning**: {}\n\n## User Action Required\n\nReply with:\n- `confirm` - approve as-is\n- `correct: {{...}}` - provide correction\n- `reject` - discard",
            bgm_id, name, fan, confidence, reasoning
        );

        let output = Command::new("gh")
            .args([
                "issue",
                "create",
                "--repo",
                &repo,
                "--title",
                &title,
                "--body",
                &body,
                "--label",
                "alias-request",
            ])
            .output()
            .map_err(|e| AppError::AliasLoadError(format!("执行 gh 命令失败: {e}")))?;

        if output.status.success() {
            created += 1;
        }
    }

    println!("创建了 {} 个 issue", created);
    Ok(())
}

#[cfg(feature = "clouddrive")]
fn run_rss(args: RssArgs) -> Result<(), AppError> {
    use anime_organizer::rss::{
        client::{CloudDriveClient, CloudDriveClientTrait},
        db::{default_db_path, RssDatabase},
        http_client::HttpClient,
        proxy::{build_http_client, ProxyConfig},
        scheduler::RssScheduler,
    };
    use std::sync::Arc;

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| AppError::MetadataFetchError(format!("创建异步运行时失败: {e}")))?;

    runtime.block_on(async {
        // 初始化 tracing
        tracing_subscriber::fmt::init();

        // 初始化数据库
        let db_path = default_db_path();
        let db = RssDatabase::new(&db_path)?;

        // --list-subscriptions：列出订阅后退出
        if args.list_subscriptions {
            let subs = db.list_all_subscriptions()?;
            if subs.is_empty() {
                println!("暂无 RSS 订阅");
            } else {
                println!(
                    "{:<4} {:<60} {:<20} {:<10} 目标目录",
                    "ID", "URL", "过滤器", "间隔(s)"
                );
                println!("{}", "-".repeat(120));
                for sub in &subs {
                    println!(
                        "{:<4} {:<60} {:<20} {:<10} {}",
                        sub.id,
                        sub.url,
                        sub.filter_regex.as_deref().unwrap_or("-"),
                        sub.interval_secs,
                        sub.target_folder
                    );
                }
            }
            return Ok(());
        }

        // --add-subscription：添加订阅后退出
        if args.add_subscription {
            let rss_url = args.rss_url.as_deref().ok_or_else(|| {
                AppError::MetadataFetchError("添加订阅需要 --rss-url 参数".to_string())
            })?;
            let target = args.rss_target.as_deref().ok_or_else(|| {
                AppError::MetadataFetchError("添加订阅需要 --rss-target 参数".to_string())
            })?;
            let id = db.add_subscription(
                rss_url,
                args.rss_filter.as_deref(),
                target,
                args.rss_interval as i64,
            )?;
            println!("已添加/更新订阅 (id={id}): {rss_url}");
            return Ok(());
        }

        // 构建 CloudDrive2 客户端
        let cd_url = args.clouddrive_url.as_deref().ok_or_else(|| {
            AppError::MetadataFetchError("需要 --clouddrive-url 参数".to_string())
        })?;
        let mut cd_client = CloudDriveClient::new(cd_url, args.clouddrive_token.clone())?;

        // 如果没有 token 但有用户名/密码，则登录
        if cd_client.get_token_value().is_none() {
            let user = args.clouddrive_user.as_deref().ok_or_else(|| {
                AppError::MetadataFetchError(
                    "需要 --clouddrive-token 或 --clouddrive-user + --clouddrive-pass".to_string(),
                )
            })?;
            let pass = args.clouddrive_pass.as_deref().ok_or_else(|| {
                AppError::MetadataFetchError("需要 --clouddrive-pass 参数".to_string())
            })?;
            let token = cd_client.login(user, pass).await?;
            tracing::info!("CloudDrive2 登录成功，获取到令牌");
            let _ = token;
        }

        let proxy_config = ProxyConfig::from_env();
        let http_client = build_http_client(&proxy_config)?;
        let scheduler = RssScheduler::new(
            db,
            Arc::new(HttpClient::new(http_client)),
            Arc::new(cd_client),
            args.daemon || args.single_shot,
        );

        // 根据模式选择执行路径
        if args.daemon {
            let interval = std::time::Duration::from_secs(args.rss_interval);

            if let Some(ref rss_url) = args.rss_url {
                let target = args.rss_target.as_deref().ok_or_else(|| {
                    AppError::MetadataFetchError("daemon 模式需要 --rss-target 参数".to_string())
                })?;
                scheduler
                    .run_daemon_url(rss_url, args.rss_filter.as_deref(), target, interval)
                    .await
            } else {
                scheduler.run_daemon(interval).await
            }
        } else {
            // 默认单次执行
            if let Some(ref rss_url) = args.rss_url {
                let target = args.rss_target.as_deref().ok_or_else(|| {
                    AppError::MetadataFetchError("单次执行需要 --rss-target 参数".to_string())
                })?;
                scheduler
                    .run_once_url(
                        rss_url,
                        args.rss_filter.as_deref(),
                        target,
                        args.rss_interval,
                    )
                    .await
            } else {
                scheduler.run_once().await
            }
        }
    })
}

#[cfg(feature = "clouddrive")]
fn run_add_offline(args: AddOfflineArgs) -> Result<(), AppError> {
    use anime_organizer::rss::client::{CloudDriveClient, CloudDriveClientTrait};

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| AppError::MetadataFetchError(format!("创建异步运行时失败: {e}")))?;

    runtime.block_on(async {
        let client = CloudDriveClient::new(&args.clouddrive_url, Some(args.clouddrive_token))?;

        println!("提交离线下载...");
        println!("  目标: {}", args.target);
        println!("  URL: {}...", &args.url[..args.url.len().min(60)]);

        client
            .add_offline_files(vec![args.url], &args.target)
            .await?;

        println!("✅ 离线下载提交成功！");
        Ok(())
    })
}

#[cfg(feature = "clouddrive")]
fn run_list_folder(args: ListFolderArgs) -> Result<(), AppError> {
    use anime_organizer::rss::client::{CloudDriveClient, CloudDriveClientTrait};

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| AppError::MetadataFetchError(format!("创建异步运行时失败: {e}")))?;

    runtime.block_on(async {
        let client = CloudDriveClient::new(&args.clouddrive_url, Some(args.clouddrive_token))?;

        println!("浏览目录: {}", args.path);
        println!("{}", "=".repeat(80));

        let files = client.list_folder(&args.path).await?;

        if files.is_empty() {
            println!("(空目录)");
            return Ok(());
        }

        println!("{:<6} {:<50} {:>12}  云盘", "类型", "名称", "大小");
        println!("{}", "-".repeat(80));

        for file in &files {
            let file_type = if file.is_directory {
                "[目录]"
            } else {
                "[文件]"
            };
            let size = if file.is_directory {
                "-".to_string()
            } else {
                format_size(file.size)
            };
            let name = if file.name.is_empty() {
                file.full_path_name.clone()
            } else {
                file.name.clone()
            };
            let cloud = file
                .cloud_api
                .as_ref()
                .map(|c| c.name.as_str())
                .unwrap_or("-");
            println!("{:<6} {:<50} {:>12}  {}", file_type, name, size, cloud);
        }

        println!("{}", "-".repeat(80));
        println!("共 {} 项", files.len());
        Ok(())
    })
}

#[allow(dead_code)]
fn format_size(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(feature = "torrent-scraper")]
async fn run_torrent_scrape(args: TorrentScrapeArgs) -> Result<(), AppError> {
    use anime_organizer::torrent::dmhy;
    use anime_organizer::torrent::dmhy_playwright;
    use anime_organizer::torrent::nyaa;
    use anime_organizer::torrent::nyaa_playwright;

    let pages = args.pages.clamp(1, 2000);
    let mut all_titles = Vec::new();

    let opts = dmhy_playwright::ScrapeOptions::new().with_headed(args.headed);
    let nyaa_opts = nyaa_playwright::ScrapeOptions::new().with_headed(args.headed);

    match args.source {
        TorrentSource::Dmhy => {
            println!(
                "正在从 DMHY (Playwright) 爬取种子文件列表 ({} 页)...",
                pages
            );
            all_titles = match dmhy_playwright::scrape_dmhy_with_playwright_opts(pages, opts).await
            {
                Ok(titles) => titles,
                Err(err) => {
                    eprintln!("[Torrent] DMHY Playwright 抓取失败，回退到 HTTP 抓取: {err}");
                    dmhy::scrape_dmhy(pages).await?
                }
            };
        }
        TorrentSource::Nyaa => {
            if let Some(query) = args.query.as_deref() {
                println!(
                    "正在从 Nyaa 搜索 '{}' 并爬取种子文件列表 ({} 页)...",
                    query, pages
                );
                all_titles = nyaa::scrape_search(query, pages).await?;
            } else {
                println!(
                    "正在从 Nyaa (Playwright) 首页爬取最新种子文件列表 ({} 页)...",
                    pages
                );
                all_titles =
                    match nyaa_playwright::scrape_recent_with_playwright_opts(pages, nyaa_opts)
                        .await
                    {
                        Ok(titles) => titles,
                        Err(err) => {
                            eprintln!(
                                "[Torrent] Nyaa Playwright 抓取失败，回退到 HTTP 抓取: {err}"
                            );
                            nyaa::scrape_recent(pages).await?
                        }
                    };
            }
        }
        TorrentSource::All => {
            println!("正在从 DMHY (Playwright) 爬取 ({} 页)...", pages);
            match dmhy_playwright::scrape_dmhy_with_playwright_opts(pages, opts.clone()).await {
                Ok(dmhy_titles) => all_titles.extend(dmhy_titles),
                Err(err) => {
                    eprintln!("[Torrent] DMHY Playwright 抓取失败，回退到 HTTP 抓取: {err}");
                    all_titles.extend(dmhy::scrape_dmhy(pages).await?);
                }
            }

            println!(
                "正在从 Nyaa (Playwright) 首页爬取最新种子 ({} 页)...",
                pages
            );
            match nyaa_playwright::scrape_recent_with_playwright_opts(pages, nyaa_opts).await {
                Ok(nyaa_titles) => all_titles.extend(nyaa_titles),
                Err(err) => {
                    eprintln!("[Torrent] Nyaa Playwright 抓取失败，回退到 HTTP 抓取: {err}");
                    all_titles.extend(nyaa::scrape_recent(pages).await?);
                }
            }
        }
    }

    if all_titles.is_empty() {
        println!("未爬取到任何文件");
        return Ok(());
    }

    let unique_lines = anime_organizer::torrent::sorted_unique_title_lines(&all_titles);
    let text = unique_lines.join("\n");

    if let Some(ref output_path) = args.output {
        std::fs::write(output_path, &text)
            .map_err(|e| AppError::Io(std::io::Error::other(format!("写入文件失败: {e}"))))?;
        println!("已保存到: {}", output_path.display());
    } else {
        print!("{}", text);
    }

    println!("\n共爬取 {} 个文件名", unique_lines.len());
    Ok(())
}
