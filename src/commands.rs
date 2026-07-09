use crate::cli::*;
use anime_organizer::error::AppError;
#[cfg(feature = "metadata")]
use anime_organizer::metadata::AliasLookup;
#[cfg(feature = "scraper")]
use anime_organizer::scraper::{
    db_builder::build_bangumi_db,
    matcher::{format_github_output, match_aliases},
    ScrapedAnime, Scraper,
};
#[cfg(feature = "scraper")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "scraper")]
use std::io::Write;
#[cfg(feature = "scraper")]
use std::path::PathBuf;

#[cfg(any(
    feature = "scraper",
    feature = "clouddrive",
    feature = "torrent-scraper"
))]
pub(crate) fn run_command(command: Commands) -> Result<(), AppError> {
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
    use anime_organizer::torrent::nyaa;

    let pages = args.pages.clamp(1, 2000);
    if args.headed {
        eprintln!("--headed 仅适用于旧 Playwright 后端；当前使用 HTTP 抓取");
    }

    let all_titles = match args.source {
        TorrentSource::Dmhy => {
            println!("正在从 DMHY 爬取种子文件列表 ({} 页)...", pages);
            dmhy::scrape_dmhy(pages).await?
        }
        TorrentSource::Nyaa => {
            if let Some(query) = args.query.as_deref() {
                println!(
                    "正在从 Nyaa 搜索 '{}' 并爬取种子文件列表 ({} 页)...",
                    query, pages
                );
                nyaa::scrape_search(query, pages).await?
            } else {
                println!("正在从 Nyaa 首页爬取最新种子文件列表 ({} 页)...", pages);
                nyaa::scrape_recent(pages).await?
            }
        }
        TorrentSource::All => {
            println!("正在从 DMHY 爬取 ({} 页)...", pages);
            let mut titles = dmhy::scrape_dmhy(pages).await?;

            println!("正在从 Nyaa 首页爬取最新种子 ({} 页)...", pages);
            titles.extend(nyaa::scrape_recent(pages).await?);
            titles
        }
    };

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
