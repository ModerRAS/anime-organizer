//! anime-organizer 命令行入口
//!
//! 提供默认的文件整理模式，以及用于自动化工作流的 scraper 子命令。

mod cli;
mod commands;
#[cfg(feature = "metadata")]
mod mlip;
#[cfg(feature = "metadata")]
mod title_resolver;

use crate::cli::*;
#[cfg(any(
    feature = "scraper",
    feature = "clouddrive",
    feature = "torrent-scraper"
))]
use crate::commands::run_command;
#[cfg(feature = "metadata")]
use crate::mlip::{
    anime_group_min_episode, apply_bangumi_episode_details, create_episode_nfo, download_images,
    fetch_anime_metadata, fetch_bangumi_episodes_cached, min_episode_by_series, MetadataLookup,
};
#[cfg(feature = "metadata")]
use anime_organizer::library_index::{Artwork, ArtworkKind};
use anime_organizer::{
    error::AppError, AnimeFileInfo, FileOrganizer, FilenameParser, LibraryExtraRecord,
    LibraryIndex, LibraryIndexRecord, OperationMode,
};
#[cfg(feature = "metadata")]
use anime_organizer::{
    metadata::{bangumi::BangumiEpisode, AliasLookup, BangumiClient, TmdbClient},
    nfo::{NfoWriter, TvShowNfo},
    AnimeMetadata,
};
use clap::Parser;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// 默认支持的视频扩展名
const DEFAULT_EXTENSIONS: &[&str] = &[".mp4", ".mkv", ".avi", ".mov", ".wmv", ".flv", ".rmvb"];
#[cfg(feature = "metadata")]
const ANIMEATLAS_SQLITE_FILENAME: &str = "animeatlas.sqlite";
#[cfg(feature = "metadata")]
const ANIMEATLAS_SQLITE_URL: &str =
    "https://github.com/ModerRAS/AnimeAtlas/releases/latest/download/animeatlas.sqlite";
#[cfg(feature = "metadata")]
const HTTP_USER_AGENT: &str = concat!(
    "ModerRAS/anime-organizer/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/ModerRAS/anime-organizer)"
);
static ANIFILEBERT_AUTO_WARNED: std::sync::OnceLock<()> = std::sync::OnceLock::new();

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
    if args.scrape_metadata || args.mlip {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| AppError::MetadataFetchError(format!("创建异步运行时失败: {e}")))?;
        runtime.block_on(run_with_metadata(args))
    } else {
        run_organize(args)
    }
}

#[cfg(not(feature = "metadata"))]
fn run_organize_entry(args: OrganizeArgs) -> Result<(), AppError> {
    if args.scrape_metadata || args.mlip {
        return Err(AppError::MetadataFetchError(
            "元数据功能未启用，请使用 --features metadata 编译".to_string(),
        ));
    }

    run_organize(args)
}

/// 仅文件整理流程（无元数据）
fn run_organize(args: OrganizeArgs) -> Result<(), AppError> {
    validate_library_index_args(&args)?;
    validate_filename_parser_args(&args)?;
    let (source, target) = resolve_source_and_target(&args)?;
    let fallback_mode = args
        .fallback_on_link_failure
        .map(FallbackMode::to_operation_mode);
    let extensions = build_extensions(&args.include_ext);
    let probe_runtime = runtime_probe_enabled(&args);

    let mut processed = 0;
    let mut succeeded = 0;
    let mut failed = 0;
    let mut library_records = Vec::new();

    for entry in WalkDir::new(&source)
        .into_iter()
        .filter_map(|item| item.ok())
        .filter(|item| item.file_type().is_file())
    {
        let path = entry.path();
        if !has_valid_extension(path, &extensions) {
            continue;
        }

        let anime_file = match parse_anime_file(path, args.filename_parser, args.verbose)? {
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
            Ok(target_path) => {
                succeeded += 1;
                if args.writes_library_index() {
                    if let Some(mut record) =
                        LibraryIndexRecord::from_target_path(&target, &target_path)?
                    {
                        apply_runtime_probe(&mut record, &target, probe_runtime, args.verbose);
                        library_records.push(record);
                    }
                }
            }
            Err(_) => failed += 1,
        }
    }

    println!("处理完成：总计{processed}个文件，成功{succeeded}个，失败{failed}个");
    finish_library_index(&args, &target, &extensions, &library_records)?;
    Ok(())
}

/// 带元数据刮削的流程
#[cfg(feature = "metadata")]
async fn run_with_metadata(args: OrganizeArgs) -> Result<(), AppError> {
    validate_library_index_args(&args)?;
    validate_filename_parser_args(&args)?;
    let (source, target) = resolve_source_and_target(&args)?;
    let fallback_mode = args
        .fallback_on_link_failure
        .map(FallbackMode::to_operation_mode);
    let extensions = build_extensions(&args.include_ext);
    let bangumi =
        BangumiClient::with_source(args.bangumi_cache.clone(), args.metadata_source.clone());
    let alias_db_path =
        resolve_alias_db_path(&bangumi, args.metadata_source.as_deref(), args.verbose).await;
    let alias_lookup =
        AliasLookup::load_from_sources(alias_db_path.as_deref(), args.alias_file.as_deref())?;
    if args.verbose {
        if alias_lookup.is_empty() {
            eprintln!("未找到可用别名库，将仅使用 Bangumi 名称和搜索匹配");
        } else {
            eprintln!("已加载 {} 条别名", alias_lookup.len());
        }
    }

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

    let allow_online_title_resolution = args.metadata_source.is_none();
    let tmdb = args.tmdb_api_key.clone().map(TmdbClient::new);
    if !args.no_images && tmdb.is_none() && args.verbose {
        eprintln!("未提供 TMDB API Key，将跳过 TMDB 图片下载");
    }
    let probe_runtime = runtime_probe_enabled(&args);

    let anime_groups =
        collect_anime_groups(&source, &extensions, args.filename_parser, args.verbose)?;
    let mut processed = 0;
    let mut succeeded = 0;
    let mut failed = 0;
    let mut metadata_cache: HashMap<(String, u32), Option<AnimeMetadata>> = HashMap::new();
    let mut episode_cache: HashMap<u32, Option<Vec<BangumiEpisode>>> = HashMap::new();
    let mut library_records = Vec::new();

    for (anime_name, files) in anime_groups {
        let Some(first_file) = files.first() else {
            continue;
        };

        let series_name = first_file.series_name();
        let season_number = first_file.season_number().unwrap_or(1);
        let anime_root = target.join(&series_name);

        let cache_key = (anime_name.clone(), season_number);
        let metadata = if let Some(cached) = metadata_cache.get(&cache_key) {
            cached.clone()
        } else {
            let fetched = fetch_anime_metadata(
                MetadataLookup {
                    anime_name: &anime_name,
                    series_name: &series_name,
                    publisher_hint: Some(&first_file.publisher),
                    season_hint: first_file.season_number(),
                    allow_online_title_resolution,
                },
                &alias_lookup,
                &bangumi,
                tmdb.as_ref(),
                args.verbose,
            )
            .await;
            metadata_cache.insert(cache_key, fetched.clone());
            fetched
        };

        let episodes = if !args.no_episode_metadata {
            if let Some(ref meta) = metadata {
                fetch_bangumi_episodes_cached(
                    meta.bangumi_id,
                    &bangumi,
                    &mut episode_cache,
                    args.verbose,
                )
                .await
            } else {
                None
            }
        } else {
            None
        };
        let group_min_episode = anime_group_min_episode(&files);

        if let Some(ref meta) = metadata {
            if args.scrape_metadata {
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
            }

            if !args.no_images && !args.dry_run {
                download_images(
                    meta,
                    &anime_root,
                    season_number,
                    &bangumi,
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

                    if args.writes_library_index() {
                        if let Some(mut record) =
                            LibraryIndexRecord::from_target_path(&target, &target_path)?
                        {
                            if let Some(ref meta) = metadata {
                                record.apply_metadata(meta);
                                apply_bangumi_episode_details(
                                    &mut record,
                                    episodes.as_deref(),
                                    group_min_episode,
                                );
                                add_metadata_artwork(
                                    &mut record,
                                    &target,
                                    &anime_root,
                                    season_number,
                                );
                            }
                            apply_runtime_probe(&mut record, &target, probe_runtime, args.verbose);
                            library_records.push(record);
                        }
                    }

                    if args.scrape_metadata {
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

    finish_library_index_with_metadata(
        &args,
        &target,
        &extensions,
        &library_records,
        MetadataIndexContext {
            metadata_cache: &mut metadata_cache,
            episode_cache: &mut episode_cache,
            alias_lookup: &alias_lookup,
            bangumi: &bangumi,
            tmdb: tmdb.as_ref(),
            download_images: !args.no_images,
            force_overwrite: args.force_overwrite,
            fetch_episode_metadata: !args.no_episode_metadata,
            allow_online_title_resolution,
            probe_runtime,
            verbose: args.verbose,
        },
    )
    .await?;

    Ok(())
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

#[derive(Debug, Clone, Copy)]
enum LibraryIndexWriteMode {
    Initialize,
    Incremental,
    Rebuild,
}

impl LibraryIndexWriteMode {
    fn label(self) -> &'static str {
        match self {
            Self::Initialize => "初始化",
            Self::Incremental => "增量更新",
            Self::Rebuild => "重建",
        }
    }
}

fn validate_library_index_args(args: &OrganizeArgs) -> Result<(), AppError> {
    if args.rebuild_library_index && !args.writes_library_index() {
        return Err(AppError::ParseError(
            "--rebuild-library-index 必须与 --library-index 或 --mlip 一起使用".to_string(),
        ));
    }
    Ok(())
}

fn validate_filename_parser_args(args: &OrganizeArgs) -> Result<(), AppError> {
    let _ = args;
    #[cfg(not(feature = "anifilebert"))]
    if args.filename_parser == FilenameParserMode::Anifilebert {
        return Err(AppError::ParseError(
            "--filename-parser anifilebert 需要使用 --features anifilebert 编译".to_string(),
        ));
    }

    Ok(())
}

fn resolve_library_index_mode(args: &OrganizeArgs, target: &Path) -> Option<LibraryIndexWriteMode> {
    if !args.writes_library_index() {
        return None;
    }

    if args.rebuild_library_index {
        return Some(LibraryIndexWriteMode::Rebuild);
    }

    if LibraryIndex::database_path(target).exists() {
        Some(LibraryIndexWriteMode::Incremental)
    } else {
        Some(LibraryIndexWriteMode::Initialize)
    }
}

fn finish_library_index(
    args: &OrganizeArgs,
    target: &Path,
    extensions: &HashSet<String>,
    current_records: &[LibraryIndexRecord],
) -> Result<(), AppError> {
    let Some(mode) = resolve_library_index_mode(args, target) else {
        return Ok(());
    };

    if args.dry_run {
        let scan_count = if matches!(
            mode,
            LibraryIndexWriteMode::Initialize | LibraryIndexWriteMode::Rebuild
        ) {
            let records = collect_target_library_records(target, extensions, args.verbose)?;
            let extras = collect_target_library_extras(target, extensions, &records)?;
            records.len() + extras.len()
        } else {
            current_records.len()
        };
        eprintln!(
            "[dry-run] MLIP {}: {} ({} 条记录)",
            mode.label(),
            LibraryIndex::database_path(target).display(),
            scan_count
        );
        return Ok(());
    }

    let stats = match mode {
        LibraryIndexWriteMode::Initialize | LibraryIndexWriteMode::Rebuild => {
            let mut records = collect_target_library_records(target, extensions, args.verbose)?;
            let probe_runtime = runtime_probe_enabled(args);
            apply_runtime_probe_to_records(&mut records, target, probe_runtime, args.verbose);
            let extras = collect_target_library_extras(target, extensions, &records)?;
            LibraryIndex::rebuild_with_extras(target, &records, &extras)?
        }
        LibraryIndexWriteMode::Incremental => LibraryIndex::update(target, current_records)?,
    };

    println!(
        "媒体库索引{}完成：{} 部作品，{} 集，{} 个文件，{} 个特典 ({})",
        mode.label(),
        stats.series,
        stats.episodes,
        stats.media_files,
        stats.extras,
        LibraryIndex::database_path(target).display()
    );
    Ok(())
}

fn collect_target_library_records(
    target: &Path,
    extensions: &HashSet<String>,
    verbose: bool,
) -> Result<Vec<LibraryIndexRecord>, AppError> {
    let mut records = Vec::new();

    for entry in WalkDir::new(target)
        .into_iter()
        .filter_map(|item| item.ok())
        .filter(|item| item.file_type().is_file())
    {
        let path = entry.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("library.db"))
        {
            continue;
        }
        if !has_valid_extension(path, extensions) {
            continue;
        }

        match LibraryIndexRecord::from_target_path(target, path)? {
            Some(record) => records.push(record),
            None if verbose => eprintln!(
                "跳过：无法加入媒体库索引 {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ),
            None => {}
        }
    }

    Ok(records)
}

fn collect_target_library_extras(
    target: &Path,
    extensions: &HashSet<String>,
    records: &[LibraryIndexRecord],
) -> Result<Vec<LibraryExtraRecord>, AppError> {
    let mut titles_by_root: HashMap<&str, HashSet<&str>> = HashMap::new();
    for record in records {
        let Some(root) = record.relative_path.split('/').next() else {
            continue;
        };
        titles_by_root
            .entry(root)
            .or_default()
            .insert(&record.series_title);
    }
    let owners: HashMap<&str, &str> = titles_by_root
        .into_iter()
        .filter_map(|(root, titles)| {
            (titles.len() == 1).then(|| (root, *titles.iter().next().expect("one title")))
        })
        .collect();
    let mut extras = Vec::new();
    for entry in WalkDir::new(target)
        .into_iter()
        .filter_map(|item| item.ok())
        .filter(|item| item.file_type().is_file())
    {
        let path = entry.path();
        if !has_valid_extension(path, extensions) {
            continue;
        }
        let Some(root) = path
            .strip_prefix(target)
            .ok()
            .and_then(|relative| relative.components().next())
            .and_then(|component| component.as_os_str().to_str())
        else {
            continue;
        };
        let Some(series_title) = owners.get(root) else {
            continue;
        };
        if let Some(extra) = LibraryExtraRecord::from_target_path(target, path, *series_title)? {
            extras.push(extra);
        }
    }
    Ok(extras)
}

fn runtime_probe_enabled(args: &OrganizeArgs) -> bool {
    if !args.probe_runtime || args.dry_run || !args.writes_library_index() {
        return false;
    }

    match Command::new("ffprobe").arg("-version").output() {
        Ok(output) if output.status.success() => true,
        _ => {
            eprintln!("--probe-runtime 需要 PATH 中存在 ffprobe，将跳过时长探测");
            false
        }
    }
}

fn apply_runtime_probe_to_records(
    records: &mut [LibraryIndexRecord],
    target: &Path,
    probe_runtime: bool,
    verbose: bool,
) {
    if !probe_runtime {
        return;
    }

    for record in records {
        apply_runtime_probe(record, target, true, verbose);
    }
}

fn apply_runtime_probe(
    record: &mut LibraryIndexRecord,
    target: &Path,
    probe_runtime: bool,
    verbose: bool,
) {
    if !probe_runtime {
        return;
    }

    let media_path = target.join(&record.relative_path);
    if let Some(seconds) = probe_media_runtime_seconds(&media_path, verbose) {
        record.runtime = Some(seconds);
    }
}

fn probe_media_runtime_seconds(path: &Path, verbose: bool) -> Option<i64> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(path)
        .output();

    let output = match output {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            if verbose {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("ffprobe 读取时长失败 {}: {}", path.display(), stderr.trim());
            }
            return None;
        }
        Err(error) => {
            if verbose {
                eprintln!("ffprobe 启动失败 {}: {error}", path.display());
            }
            return None;
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let duration = stdout
        .lines()
        .find_map(|line| line.trim().parse::<f64>().ok())?;
    if !duration.is_finite() || duration <= 0.0 || duration > i64::MAX as f64 {
        return None;
    }

    Some(duration.round() as i64)
}

#[cfg(feature = "metadata")]
struct MetadataIndexContext<'a> {
    metadata_cache: &'a mut HashMap<(String, u32), Option<AnimeMetadata>>,
    episode_cache: &'a mut HashMap<u32, Option<Vec<BangumiEpisode>>>,
    alias_lookup: &'a AliasLookup,
    bangumi: &'a BangumiClient,
    tmdb: Option<&'a TmdbClient>,
    download_images: bool,
    force_overwrite: bool,
    fetch_episode_metadata: bool,
    allow_online_title_resolution: bool,
    probe_runtime: bool,
    verbose: bool,
}

#[cfg(feature = "metadata")]
async fn finish_library_index_with_metadata(
    args: &OrganizeArgs,
    target: &Path,
    extensions: &HashSet<String>,
    current_records: &[LibraryIndexRecord],
    mut context: MetadataIndexContext<'_>,
) -> Result<(), AppError> {
    let Some(mode) = resolve_library_index_mode(args, target) else {
        return Ok(());
    };

    if args.dry_run {
        let scan_count = if matches!(
            mode,
            LibraryIndexWriteMode::Initialize | LibraryIndexWriteMode::Rebuild
        ) {
            let records = collect_target_library_records(target, extensions, args.verbose)?;
            let extras = collect_target_library_extras(target, extensions, &records)?;
            records.len() + extras.len()
        } else {
            current_records.len()
        };
        eprintln!(
            "[dry-run] MLIP {}: {} ({} 条记录)",
            mode.label(),
            LibraryIndex::database_path(target).display(),
            scan_count
        );
        return Ok(());
    }

    let stats = match mode {
        LibraryIndexWriteMode::Initialize | LibraryIndexWriteMode::Rebuild => {
            let mut records = collect_target_library_records(target, extensions, args.verbose)?;
            enrich_library_index_records(&mut records, target, &mut context).await;
            let extras = collect_target_library_extras(target, extensions, &records)?;
            LibraryIndex::rebuild_with_extras(target, &records, &extras)?
        }
        LibraryIndexWriteMode::Incremental => LibraryIndex::update(target, current_records)?,
    };

    println!(
        "媒体库索引{}完成：{} 部作品，{} 集，{} 个文件，{} 个特典 ({})",
        mode.label(),
        stats.series,
        stats.episodes,
        stats.media_files,
        stats.extras,
        LibraryIndex::database_path(target).display()
    );
    Ok(())
}

#[cfg(feature = "metadata")]
async fn enrich_library_index_records(
    records: &mut [LibraryIndexRecord],
    target: &Path,
    context: &mut MetadataIndexContext<'_>,
) {
    let min_episodes = min_episode_by_series(records);

    for record in records {
        let lookup_title = record.series_title.clone();
        let season = u32::try_from(record.season).unwrap_or(1);
        let season_hint = (season > 1).then_some(season);
        let cache_key = (lookup_title.clone(), season);
        let metadata = if let Some(cached) = context.metadata_cache.get(&cache_key) {
            cached.clone()
        } else {
            let publisher = FilenameParser::parse(target.join(&record.relative_path))
                .map(|file| file.publisher);
            let fetched = fetch_anime_metadata(
                MetadataLookup {
                    anime_name: &lookup_title,
                    series_name: &lookup_title,
                    publisher_hint: publisher.as_deref(),
                    season_hint,
                    allow_online_title_resolution: context.allow_online_title_resolution,
                },
                context.alias_lookup,
                context.bangumi,
                context.tmdb,
                context.verbose,
            )
            .await;
            context.metadata_cache.insert(cache_key, fetched.clone());
            fetched
        };

        if let Some(ref meta) = metadata {
            let episodes = if context.fetch_episode_metadata {
                fetch_bangumi_episodes_cached(
                    meta.bangumi_id,
                    context.bangumi,
                    context.episode_cache,
                    context.verbose,
                )
                .await
            } else {
                None
            };
            let anime_root = library_series_root(target, record, &lookup_title);
            if context.download_images {
                download_images(
                    meta,
                    &anime_root,
                    season.max(1),
                    context.bangumi,
                    context.tmdb,
                    context.force_overwrite,
                    context.verbose,
                )
                .await;
            }
            record.apply_metadata(meta);
            apply_bangumi_episode_details(
                record,
                episodes.as_deref(),
                min_episodes
                    .get(&(lookup_title.clone(), i64::from(season)))
                    .copied(),
            );
            add_metadata_artwork(record, target, &anime_root, season.max(1));
        }
        apply_runtime_probe(record, target, context.probe_runtime, context.verbose);
    }
}

#[cfg(feature = "metadata")]
fn library_series_root(
    target: &Path,
    record: &LibraryIndexRecord,
    fallback_title: &str,
) -> PathBuf {
    let directory = record
        .relative_path
        .split('/')
        .next()
        .filter(|component| !component.is_empty())
        .unwrap_or(fallback_title);
    target.join(directory)
}

#[cfg(feature = "metadata")]
fn add_metadata_artwork(
    record: &mut LibraryIndexRecord,
    target: &Path,
    anime_root: &Path,
    season_number: u32,
) {
    add_series_artwork_if_exists(
        record,
        target,
        ArtworkKind::Poster,
        &anime_root.join("poster.jpg"),
    );
    add_series_artwork_if_exists(
        record,
        target,
        ArtworkKind::Fanart,
        &anime_root.join("fanart.jpg"),
    );
    add_series_artwork_if_exists(
        record,
        target,
        ArtworkKind::SeasonPoster,
        &anime_root.join(format!("season{season_number:02}-poster.jpg")),
    );
}

#[cfg(feature = "metadata")]
fn add_series_artwork_if_exists(
    record: &mut LibraryIndexRecord,
    target: &Path,
    kind: ArtworkKind,
    path: &Path,
) {
    if !path.exists() {
        return;
    }

    if let Some(relative) = path.strip_prefix(target).ok().map(normalized_relative_path) {
        record.series_artwork.push(Artwork::new(kind, relative));
    }
}

#[cfg(feature = "metadata")]
fn normalized_relative_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => value.to_str().map(ToOwned::to_owned),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn parse_anime_file(
    path: &Path,
    mode: FilenameParserMode,
    verbose: bool,
) -> Result<Option<AnimeFileInfo>, AppError> {
    match mode {
        FilenameParserMode::Rules => Ok(FilenameParser::parse(path)),
        FilenameParserMode::Anifilebert => parse_anifilebert(path, true, verbose),
        FilenameParserMode::Auto => match FilenameParser::parse(path) {
            Some(info) => Ok(Some(info)),
            None => parse_anifilebert(path, false, verbose),
        },
    }
}

#[cfg(feature = "anifilebert")]
fn parse_anifilebert(
    path: &Path,
    required: bool,
    verbose: bool,
) -> Result<Option<AnimeFileInfo>, AppError> {
    match anime_organizer::anifilebert::parse_path(path) {
        Ok(info) => Ok(info),
        Err(error) if required => Err(AppError::ParseError(format!(
            "AniFileBERT 解析器初始化或推理失败: {error}"
        ))),
        Err(error) => {
            if verbose && ANIFILEBERT_AUTO_WARNED.set(()).is_ok() {
                eprintln!("AniFileBERT 回退不可用，将继续使用规则解析器: {error}");
            }
            Ok(None)
        }
    }
}

#[cfg(not(feature = "anifilebert"))]
fn parse_anifilebert(
    _path: &Path,
    required: bool,
    verbose: bool,
) -> Result<Option<AnimeFileInfo>, AppError> {
    if required {
        return Err(AppError::ParseError(
            "AniFileBERT 解析器未启用，请使用 --features anifilebert 编译".to_string(),
        ));
    }
    if verbose && ANIFILEBERT_AUTO_WARNED.set(()).is_ok() {
        eprintln!("AniFileBERT 回退未启用，将继续使用规则解析器");
    }
    Ok(None)
}

fn collect_anime_groups(
    source: &Path,
    extensions: &HashSet<String>,
    filename_parser: FilenameParserMode,
    verbose: bool,
) -> Result<HashMap<String, Vec<AnimeFileInfo>>, AppError> {
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

        if let Some(info) = parse_anime_file(path, filename_parser, verbose)? {
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

    Ok(groups)
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

#[cfg(feature = "metadata")]
async fn resolve_alias_db_path(
    bangumi: &BangumiClient,
    metadata_source: Option<&Path>,
    verbose: bool,
) -> Option<PathBuf> {
    if let Some(path) = find_metadata_source_alias_db(metadata_source) {
        return Some(path);
    }

    let animeatlas_path = bangumi.cache_dir().join(ANIMEATLAS_SQLITE_FILENAME);
    let bangumi_path = bangumi.cache_dir().join("bangumi.db");
    if metadata_source.is_some() {
        return animeatlas_path
            .is_file()
            .then_some(animeatlas_path)
            .or_else(|| bangumi_path.is_file().then_some(bangumi_path));
    }

    match download_animeatlas_alias_db(bangumi.cache_dir()).await {
        Ok(path) => {
            if verbose {
                eprintln!("AnimeAtlas 别名库已就绪: {}", path.display());
            }
            Some(path)
        }
        Err(error) => {
            if verbose {
                eprintln!("AnimeAtlas 别名库下载失败，将回退到本地缓存、本地 bangumi.db 或在线搜索: {error}");
            }
            animeatlas_path
                .is_file()
                .then_some(animeatlas_path)
                .or_else(|| bangumi_path.is_file().then_some(bangumi_path))
        }
    }
}

#[cfg(feature = "metadata")]
fn find_metadata_source_alias_db(metadata_source: Option<&Path>) -> Option<PathBuf> {
    let source = metadata_source?;
    let parent = if source.is_dir() {
        source
    } else {
        source.parent()?
    };
    [ANIMEATLAS_SQLITE_FILENAME, "bangumi.db"]
        .into_iter()
        .map(|name| parent.join(name))
        .find(|path| path.is_file())
}

#[cfg(feature = "metadata")]
async fn download_animeatlas_alias_db(cache_dir: &Path) -> Result<PathBuf, AppError> {
    std::fs::create_dir_all(cache_dir)
        .map_err(|e| AppError::MetadataFetchError(format!("创建缓存目录失败: {e}")))?;

    let db_path = cache_dir.join(ANIMEATLAS_SQLITE_FILENAME);

    let client = reqwest::Client::builder()
        .user_agent(HTTP_USER_AGENT)
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::MetadataFetchError(format!("创建 HTTP 客户端失败: {e}")))?;
    let resp = client
        .get(ANIMEATLAS_SQLITE_URL)
        .send()
        .await
        .map_err(|e| AppError::MetadataFetchError(format!("下载 AnimeAtlas 别名库失败: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::MetadataFetchError(format!(
            "下载 AnimeAtlas 别名库失败 (HTTP {})",
            resp.status()
        )));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| AppError::MetadataFetchError(format!("读取 AnimeAtlas 别名库失败: {e}")))?;
    let tmp_path = db_path.with_extension("sqlite.tmp");
    std::fs::write(&tmp_path, &bytes)
        .map_err(|e| AppError::MetadataFetchError(format!("写入 AnimeAtlas 临时文件失败: {e}")))?;
    if db_path.is_file() {
        std::fs::remove_file(&db_path).map_err(|e| {
            AppError::MetadataFetchError(format!("替换旧 AnimeAtlas 别名库失败: {e}"))
        })?;
    }
    std::fs::rename(&tmp_path, &db_path)
        .map_err(|e| AppError::MetadataFetchError(format!("保存 AnimeAtlas 别名库失败: {e}")))?;

    Ok(db_path)
}

fn has_valid_extension(path: &Path, extensions: &HashSet<String>) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.contains(&format!(".{}", ext.to_lowercase())))
        .unwrap_or(false)
}

#[cfg(all(test, feature = "metadata"))]
mod tests {
    use super::*;

    #[test]
    fn library_artwork_uses_physical_series_directory() {
        let record = LibraryIndexRecord::new(
            "Parsed Title".to_string(),
            1,
            1.0,
            "Physical Folder/[ANi] Parsed Title - 01.mkv".to_string(),
            Path::new("unused.mkv"),
        );

        assert_eq!(
            library_series_root(Path::new("library"), &record, "Parsed Title"),
            Path::new("library").join("Physical Folder")
        );
    }
}
