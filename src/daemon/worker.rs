use super::model::{JobResult, JobSpec};
use super::queue::QueueRepository;
#[cfg(feature = "clouddrive")]
use super::rss_schedule::RssRuntime;
#[cfg(feature = "torrent-scraper")]
use crate::cli::TorrentSource;
#[cfg(feature = "clouddrive")]
use anime_organizer::rss::{
    proxy::{build_http_client, ProxyConfig},
    torrent::download_torrent_to_magnet,
};
#[cfg(feature = "torrent-scraper")]
use anime_organizer::torrent::ScrapedTitle;
#[cfg(all(feature = "torrent-scraper", test))]
use anime_organizer::torrent::TorrentSource as ScrapedTorrentSource;
#[cfg(any(feature = "scraper", feature = "torrent-scraper"))]
use serde_json::{json, Value};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Default, Clone)]
pub(crate) struct WorkerSnapshot {
    pub(crate) state: String,
    pub(crate) current_job_id: Option<i64>,
}

pub(crate) struct WorkerHandle {
    thread: Option<thread::JoinHandle<()>>,
}

impl WorkerHandle {
    pub(crate) fn join(mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub(crate) fn start(
    queue: QueueRepository,
    wake_rx: mpsc::Receiver<()>,
    snapshot: Arc<Mutex<WorkerSnapshot>>,
    shutting_down: Arc<AtomicBool>,
    #[cfg(feature = "clouddrive")] rss_runtime: RssRuntime,
) -> WorkerHandle {
    let thread = thread::Builder::new()
        .name("anime-organizer-daemon-worker".to_string())
        .spawn(move || {
            worker_loop(
                queue,
                wake_rx,
                snapshot,
                shutting_down,
                #[cfg(feature = "clouddrive")]
                rss_runtime,
            )
        })
        .expect("failed to start daemon worker thread");
    WorkerHandle {
        thread: Some(thread),
    }
}

fn worker_loop(
    queue: QueueRepository,
    wake_rx: mpsc::Receiver<()>,
    snapshot: Arc<Mutex<WorkerSnapshot>>,
    shutting_down: Arc<AtomicBool>,
    #[cfg(feature = "clouddrive")] rss_runtime: RssRuntime,
) {
    set_snapshot(&snapshot, "idle", None);
    loop {
        if shutting_down.load(Ordering::Acquire) {
            set_snapshot(&snapshot, "stopping", None);
            return;
        }

        match queue.claim_next() {
            Ok(Some(job)) => {
                set_snapshot(&snapshot, "validating", Some(job.id));
                let _ = queue.set_progress(job.id, "validating");
                let _ = queue.append_log(job.id, "info", "Worker started the job");
                let outcome = catch_unwind(AssertUnwindSafe(|| {
                    execute(
                        &queue,
                        &job,
                        #[cfg(feature = "clouddrive")]
                        &rss_runtime,
                    )
                }));
                match outcome {
                    Ok(Ok(result)) => {
                        let json = serde_json::to_string(&result)
                            .unwrap_or_else(|_| "{\"summary\":\"succeeded\"}".to_string());
                        let _ = queue.append_log(job.id, "info", &result.summary);
                        let _ = queue.mark_succeeded(job.id, &json);
                    }
                    Ok(Err(error)) => {
                        let _ = queue.append_log(job.id, "error", &error);
                        let _ = queue.mark_failed(job.id, &error);
                    }
                    Err(panic) => {
                        let message = panic_message(panic);
                        let error = format!("worker panic: {message}");
                        let _ = queue.append_log(job.id, "error", &error);
                        let _ = queue.mark_failed(job.id, &error);
                    }
                }
                set_snapshot(&snapshot, "idle", None);
            }
            Ok(None) => {
                let _ = wake_rx.recv_timeout(Duration::from_millis(500));
            }
            Err(error) => {
                set_snapshot(&snapshot, "error", None);
                thread::sleep(Duration::from_millis(500));
                if shutting_down.load(Ordering::Acquire) {
                    return;
                }
                eprintln!("daemon worker queue error: {error}");
            }
        }
    }
}

fn execute(
    queue: &QueueRepository,
    job: &super::model::StoredJob,
    #[cfg(feature = "clouddrive")] rss_runtime: &RssRuntime,
) -> Result<JobResult, String> {
    let spec: JobSpec = serde_json::from_str(&job.request_json)
        .map_err(|error| format!("invalid stored job request: {error}"))?;
    let _ = queue.set_progress(job.id, "running");
    #[allow(unreachable_patterns)]
    match &spec {
        JobSpec::Organize(args) => crate::run_organize_entry_with_log(args.clone(), &|message| {
            let _ = queue.append_log(job.id, "info", message);
        })
        .map(|_| JobResult {
            summary: "organize completed".to_string(),
            data: serde_json::json!({}),
            artifacts: Vec::new(),
        })
        .map_err(|error| error.to_string()),
        #[cfg(feature = "clouddrive")]
        JobSpec::RssPoll { .. } | JobSpec::RssPollAll => {
            let runtime = tokio::runtime::Runtime::new()
                .map_err(|error| format!("failed to create RSS runtime: {error}"))?;
            runtime.block_on(super::rss_schedule::execute(spec.clone(), rss_runtime))
        }
        #[cfg(feature = "clouddrive")]
        JobSpec::CloudAddOffline(args) => execute_cloud_add_offline(args, rss_runtime),
        #[cfg(feature = "scraper")]
        JobSpec::Scrape(args) => execute_scrape(queue, job.id, args),
        #[cfg(feature = "scraper")]
        JobSpec::MatchAliases(args) => execute_match_aliases(queue, job.id, args),
        #[cfg(feature = "scraper")]
        JobSpec::BuildBangumiDb(args) => execute_build_db(queue, job.id, args),
        #[cfg(feature = "scraper")]
        JobSpec::ExtractAliases(args) => execute_extract_aliases(queue, job.id, args),
        #[cfg(feature = "scraper")]
        JobSpec::MergeAliases(args) => execute_merge_aliases(args),
        #[cfg(feature = "scraper")]
        JobSpec::ApplyMatches(args) => execute_apply_matches(args),
        #[cfg(feature = "scraper")]
        JobSpec::CreateAliasIssues(args) => execute_create_alias_issues(args),
        #[cfg(feature = "torrent-scraper")]
        JobSpec::TorrentScrape(args) => execute_torrent_scrape(queue, job.id, args),
        _ => Err(format!(
            "job type '{}' has no registered executor",
            job.kind
        )),
    }
}

#[cfg(feature = "clouddrive")]
fn execute_cloud_add_offline(
    args: &super::model::CloudAddOfflineJobArgs,
    runtime: &RssRuntime,
) -> Result<JobResult, String> {
    let connection = runtime
        .cloud
        .repository
        .get(args.connection_id)
        .map_err(|error| error.to_string())?;
    let async_runtime = tokio::runtime::Runtime::new()
        .map_err(|error| format!("failed to create CloudDrive runtime: {error}"))?;
    async_runtime.block_on(async {
        let client = runtime
            .cloud
            .authenticated_client(&connection)
            .await
            .map_err(|error| error.to_string())?;
        let offline_url = resolve_cloud_offline_url(&args.url).await?;
        tokio::time::timeout(
            Duration::from_secs(super::cloud::CLOUD_OPERATION_TIMEOUT_SECS),
            client.add_offline_files(vec![offline_url], &args.target),
        )
        .await
        .map_err(|_| "CloudDrive offline submission timed out".to_string())?
        .map_err(|error| error.to_string())?;
        Ok(JobResult {
            summary: "CloudDrive offline URL submitted".to_string(),
            data: serde_json::json!({
                "connection_id": args.connection_id,
                "target": args.target,
            }),
            artifacts: Vec::new(),
        })
    })
}

#[cfg(feature = "clouddrive")]
async fn resolve_cloud_offline_url(url: &str) -> Result<String, String> {
    if !is_torrent_url(url) {
        return Ok(url.to_string());
    }

    let client = build_http_client(&ProxyConfig::from_env()).map_err(|error| error.to_string())?;
    download_torrent_to_magnet(&client, url)
        .await
        .map_err(|error| error.to_string())
}

#[cfg(feature = "clouddrive")]
fn is_torrent_url(url: &str) -> bool {
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    path.ends_with(".torrent") || path.ends_with("%2etorrent")
}

#[cfg(feature = "scraper")]
const MAX_INLINE_RESULT_BYTES: usize = 64 * 1024;

#[cfg(any(feature = "scraper", feature = "torrent-scraper"))]
fn artifact_descriptor(
    queue: &QueueRepository,
    job_id: i64,
    name: &str,
    content_type: &str,
    bytes: &[u8],
) -> Result<Value, String> {
    let (id, size) = queue
        .store_artifact(job_id, name, content_type, bytes)
        .map_err(|error| error.to_string())?;
    Ok(json!({
        "id": id,
        "name": name,
        "content_type": content_type,
        "size": size,
        "download_url": format!("/api/v1/jobs/{job_id}/artifacts/{id}"),
    }))
}

#[cfg(feature = "scraper")]
fn execute_scrape(
    queue: &QueueRepository,
    job_id: i64,
    args: &crate::cli::ScrapeArgs,
) -> Result<JobResult, String> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| format!("failed to create scraper runtime: {error}"))?;
    let scraped = runtime
        .block_on(crate::commands::scrape_result(args))
        .map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec(&scraped).map_err(|error| error.to_string())?;
    if bytes.len() <= MAX_INLINE_RESULT_BYTES {
        let data = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        return Ok(JobResult {
            summary: format!("scrape completed: {} item(s)", scraped.len()),
            data,
            artifacts: Vec::new(),
        });
    }

    let artifact = artifact_descriptor(queue, job_id, "scrape.json", "application/json", &bytes)?;
    Ok(JobResult {
        summary: format!(
            "scrape completed: {} item(s); result stored as artifact",
            scraped.len()
        ),
        data: json!({ "count": scraped.len(), "inline": false }),
        artifacts: vec![artifact],
    })
}

#[cfg(feature = "scraper")]
fn execute_build_db(
    queue: &QueueRepository,
    job_id: i64,
    args: &crate::cli::BuildDbArgs,
) -> Result<JobResult, String> {
    let stats = crate::commands::build_db_result(args).map_err(|error| error.to_string())?;
    let mut artifacts = Vec::new();
    if let Ok(bytes) = std::fs::read(&args.output) {
        artifacts.push(artifact_descriptor(
            queue,
            job_id,
            "bangumi.db",
            "application/vnd.sqlite3",
            &bytes,
        )?);
    }
    Ok(JobResult {
        summary: format!(
            "build_bangumi_db completed: {} subjects, {} aliases",
            stats.subjects_count, stats.aliases_count
        ),
        data: json!({
            "subjects_count": stats.subjects_count,
            "episodes_count": stats.episodes_count,
            "aliases_count": stats.aliases_count,
            "relations_count": stats.relations_count,
            "db_size": stats.db_size,
            "processing_time_ms": stats.processing_time_ms,
        }),
        artifacts,
    })
}

#[cfg(feature = "scraper")]
fn execute_extract_aliases(
    queue: &QueueRepository,
    job_id: i64,
    args: &crate::cli::ExtractAliasesArgs,
) -> Result<JobResult, String> {
    let aliases =
        crate::commands::extract_aliases_result(args).map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec(&aliases).map_err(|error| error.to_string())?;
    if let Some(path) = &args.output {
        std::fs::write(path, &bytes)
            .map_err(|error| format!("failed to write alias output: {error}"))?;
    }
    let artifact = artifact_descriptor(queue, job_id, "aliases.json", "application/json", &bytes)?;
    Ok(JobResult {
        summary: format!("extract_aliases completed: {} aliases", aliases.len()),
        data: json!({ "count": aliases.len() }),
        artifacts: vec![artifact],
    })
}

#[cfg(feature = "scraper")]
fn execute_merge_aliases(args: &crate::cli::MergeAliasesArgs) -> Result<JobResult, String> {
    let result = crate::commands::merge_aliases_result(args).map_err(|error| error.to_string())?;
    Ok(JobResult {
        summary: format!("merge_aliases completed: {} aliases added", result.added),
        data: serde_json::to_value(result).map_err(|error| error.to_string())?,
        artifacts: Vec::new(),
    })
}

#[cfg(feature = "scraper")]
fn execute_apply_matches(args: &crate::cli::ApplyMatchesArgs) -> Result<JobResult, String> {
    let result = crate::commands::apply_matches_result(args).map_err(|error| error.to_string())?;
    Ok(JobResult {
        summary: format!("apply_matches completed: {} aliases added", result.added),
        data: serde_json::to_value(result).map_err(|error| error.to_string())?,
        artifacts: Vec::new(),
    })
}

#[cfg(feature = "scraper")]
fn execute_create_alias_issues(
    args: &crate::cli::CreateAliasIssuesArgs,
) -> Result<JobResult, String> {
    let results =
        crate::commands::create_alias_issues_result(args).map_err(|error| error.to_string())?;
    let created = results.iter().filter(|result| result.success).count();
    Ok(JobResult {
        summary: format!(
            "create_alias_issues completed: {created}/{} issues created",
            results.len()
        ),
        data: serde_json::to_value(results).map_err(|error| error.to_string())?,
        artifacts: Vec::new(),
    })
}

#[cfg(feature = "torrent-scraper")]
const MAX_TORRENT_PREVIEW_LINES: usize = 20;
#[cfg(feature = "torrent-scraper")]
const MAX_TORRENT_PREVIEW_CHARS: usize = 256;

#[cfg(feature = "torrent-scraper")]
fn execute_torrent_scrape(
    queue: &QueueRepository,
    job_id: i64,
    args: &crate::cli::TorrentScrapeArgs,
) -> Result<JobResult, String> {
    let pages = anime_organizer::torrent::clamp_pages(args.pages);
    let source = args.source;
    let query = args.query.clone();
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| format!("failed to create torrent scraper runtime: {error}"))?;
    let titles = runtime
        .block_on(async move {
            match source {
                TorrentSource::Dmhy => anime_organizer::torrent::dmhy::scrape_dmhy(pages).await,
                TorrentSource::Nyaa => match query.as_deref() {
                    Some(query) => {
                        anime_organizer::torrent::nyaa::scrape_search(query, pages).await
                    }
                    None => anime_organizer::torrent::nyaa::scrape_recent(pages).await,
                },
                TorrentSource::All => {
                    let mut titles = anime_organizer::torrent::dmhy::scrape_dmhy(pages).await?;
                    titles.extend(anime_organizer::torrent::nyaa::scrape_recent(pages).await?);
                    Ok(titles)
                }
            }
        })
        .map_err(|error| error.to_string())?;
    torrent_result(queue, job_id, args, &titles, pages)
}

#[cfg(feature = "torrent-scraper")]
fn torrent_result(
    queue: &QueueRepository,
    job_id: i64,
    args: &crate::cli::TorrentScrapeArgs,
    titles: &[ScrapedTitle],
    pages: u32,
) -> Result<JobResult, String> {
    let lines = anime_organizer::torrent::sorted_unique_title_lines(titles);
    let text = anime_organizer::torrent::sorted_unique_title_text(titles);
    if let Some(output) = &args.output {
        std::fs::write(output, &text)
            .map_err(|error| format!("failed to write torrent output: {error}"))?;
    }
    let artifact = artifact_descriptor(
        queue,
        job_id,
        "torrent-titles.txt",
        "text/plain; charset=utf-8",
        text.as_bytes(),
    )?;
    let preview = lines
        .iter()
        .take(MAX_TORRENT_PREVIEW_LINES)
        .map(|line| {
            line.chars()
                .take(MAX_TORRENT_PREVIEW_CHARS)
                .collect::<String>()
        })
        .collect::<Vec<_>>();
    Ok(JobResult {
        summary: format!("torrent_scrape completed: {} unique title(s)", lines.len()),
        data: json!({
            "count": lines.len(),
            "pages": pages,
            "preview": preview,
            "preview_truncated": lines.len() > MAX_TORRENT_PREVIEW_LINES,
        }),
        artifacts: vec![artifact],
    })
}

#[cfg(feature = "scraper")]
fn execute_match_aliases(
    queue: &QueueRepository,
    job_id: i64,
    args: &crate::cli::MatchArgs,
) -> Result<JobResult, String> {
    let result = crate::commands::match_result(args).map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec(&result).map_err(|error| error.to_string())?;
    let mut artifacts = Vec::new();
    let data = if bytes.len() <= MAX_INLINE_RESULT_BYTES {
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?
    } else {
        artifacts.push(artifact_descriptor(
            queue,
            job_id,
            "match.json",
            "application/json",
            &bytes,
        )?);
        json!({
            "confident_count": result.confident.len(),
            "uncertain_count": result.uncertain.len(),
            "inline": false,
        })
    };

    if matches!(args.format, crate::cli::MatchOutputFormat::Github) {
        let github = anime_organizer::scraper::matcher::format_github_output(&result);
        artifacts.push(artifact_descriptor(
            queue,
            job_id,
            "match-github.txt",
            "text/plain; charset=utf-8",
            github.as_bytes(),
        )?);
    }

    Ok(JobResult {
        summary: format!(
            "match_aliases completed: {} confident, {} uncertain",
            result.confident.len(),
            result.uncertain.len()
        ),
        data,
        artifacts,
    })
}

fn set_snapshot(snapshot: &Arc<Mutex<WorkerSnapshot>>, state: &str, job_id: Option<i64>) {
    if let Ok(mut value) = snapshot.lock() {
        value.state = state.to_string();
        value.current_job_id = job_id;
    }
}

fn panic_message(panic: Box<dyn std::any::Any + Send>) -> String {
    panic
        .downcast_ref::<&str>()
        .map(|value| (*value).to_string())
        .or_else(|| panic.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic payload".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "clouddrive")]
    use anime_organizer::error::Result;
    #[cfg(feature = "clouddrive")]
    use anime_organizer::rss::client::{proto::CloudDriveFile, CloudDriveClientTrait};
    #[cfg(feature = "clouddrive")]
    use async_trait::async_trait;
    use std::sync::atomic::AtomicUsize;

    #[cfg(feature = "clouddrive")]
    #[derive(Clone)]
    struct CountingCloud(Arc<AtomicUsize>);

    #[cfg(feature = "clouddrive")]
    #[async_trait]
    impl CloudDriveClientTrait for CountingCloud {
        async fn login(&mut self, _: &str, _: &str) -> Result<String> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok("token".to_string())
        }

        async fn add_offline_files(&self, _: Vec<String>, _: &str) -> Result<()> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn list_folder(&self, _: &str) -> Result<Vec<CloudDriveFile>> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn panic_payload_is_recordable() {
        let message = catch_unwind(AssertUnwindSafe(|| panic!("boom"))).unwrap_err();
        assert_eq!(panic_message(message), "boom");
    }

    #[test]
    fn test_only_serial_executor_counter_can_be_shared() {
        let active = Arc::new(AtomicUsize::new(0));
        let previous = active.fetch_add(1, Ordering::SeqCst);
        assert_eq!(previous, 0);
        active.fetch_sub(1, Ordering::SeqCst);
    }

    #[cfg(feature = "clouddrive")]
    #[test]
    fn cloud_offline_torrent_urls_are_detected_without_affecting_regular_urls() {
        assert!(is_torrent_url("https://example.com/show.torrent"));
        assert!(is_torrent_url(
            "https://example.com/show%2Etorrent?token=secret"
        ));
        assert!(!is_torrent_url("https://example.com/video.mkv"));
        assert!(!is_torrent_url("magnet:?xt=urn:btih:test"));
    }

    #[cfg(feature = "clouddrive")]
    #[test]
    fn cloud_offline_executor_submits_once() {
        use crate::daemon::cloud::{
            CloudConnectionRepository, CloudConnectionRequest, CloudDriveClientFactory,
            CloudDriveState,
        };

        let directory = tempfile::tempdir().unwrap();
        let repository =
            CloudConnectionRepository::new(&directory.path().join("daemon.db")).unwrap();
        let connection = repository
            .create(
                &CloudConnectionRequest {
                    name: "test".to_string(),
                    url: "http://localhost:19798".to_string(),
                    token: Some("token".to_string()),
                    username: None,
                    password: None,
                }
                .normalize()
                .unwrap(),
            )
            .unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let client_calls = calls.clone();
        let factory: CloudDriveClientFactory = Arc::new(move |_| {
            Ok(Box::new(CountingCloud(client_calls.clone())) as Box<dyn CloudDriveClientTrait>)
        });
        let runtime = RssRuntime {
            cloud: CloudDriveState::with_factory(repository, factory),
            rss_db_path: directory.path().join("rss.db"),
        };
        let result = execute_cloud_add_offline(
            &super::super::model::CloudAddOfflineJobArgs {
                connection_id: connection.id,
                url: "magnet:?xt=test".to_string(),
                target: "/anime".to_string(),
            },
            &runtime,
        )
        .unwrap();

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(result.data["target"], "/anime");
    }

    #[cfg(feature = "clouddrive")]
    #[test]
    fn cloud_offline_executor_logs_in_when_token_is_absent() {
        use crate::daemon::cloud::{
            CloudConnectionRepository, CloudConnectionRequest, CloudDriveClientFactory,
            CloudDriveState,
        };

        let directory = tempfile::tempdir().unwrap();
        let repository =
            CloudConnectionRepository::new(&directory.path().join("daemon.db")).unwrap();
        let connection = repository
            .create(
                &CloudConnectionRequest {
                    name: "login".to_string(),
                    url: "http://localhost:19798".to_string(),
                    token: None,
                    username: Some("user".to_string()),
                    password: Some("password".to_string()),
                }
                .normalize()
                .unwrap(),
            )
            .unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let client_calls = calls.clone();
        let factory: CloudDriveClientFactory = Arc::new(move |_| {
            Ok(Box::new(CountingCloud(client_calls.clone())) as Box<dyn CloudDriveClientTrait>)
        });
        let runtime = RssRuntime {
            cloud: CloudDriveState::with_factory(repository, factory),
            rss_db_path: directory.path().join("rss.db"),
        };

        execute_cloud_add_offline(
            &super::super::model::CloudAddOfflineJobArgs {
                connection_id: connection.id,
                url: "magnet:?xt=test".to_string(),
                target: "/anime".to_string(),
            },
            &runtime,
        )
        .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[cfg(feature = "torrent-scraper")]
    #[test]
    fn torrent_pages_preserve_the_one_to_2000_clamp() {
        assert_eq!(anime_organizer::torrent::clamp_pages(0), 1);
        assert_eq!(anime_organizer::torrent::clamp_pages(2000), 2000);
        assert_eq!(anime_organizer::torrent::clamp_pages(2001), 2000);
    }

    #[cfg(feature = "torrent-scraper")]
    #[test]
    fn mocked_dmhy_nyaa_titles_are_sorted_deduplicated_and_bounded() {
        let directory = tempfile::tempdir().unwrap();
        let queue = QueueRepository::new(&directory.path().join("daemon.db")).unwrap();
        let args = crate::cli::TorrentScrapeArgs {
            source: TorrentSource::All,
            query: None,
            pages: 0,
            output: None,
            headed: false,
        };
        let request = super::super::model::EnqueueRequest {
            idempotency_key: None,
            origin: super::super::model::JobOrigin::Manual,
            confirmed: false,
            job: JobSpec::TorrentScrape(args.clone()),
        };
        let job = queue.enqueue(&request).unwrap().job;
        let mut titles = vec![
            ScrapedTitle {
                title: "Nyaa - 02.mkv".to_string(),
                source: ScrapedTorrentSource::Nyaa,
                url: None,
            },
            ScrapedTitle {
                title: "DMHY - 01.mkv".to_string(),
                source: ScrapedTorrentSource::Dmhy,
                url: None,
            },
            ScrapedTitle {
                title: "Nyaa - 02.mkv".to_string(),
                source: ScrapedTorrentSource::Nyaa,
                url: None,
            },
        ];
        titles.extend((0..25).map(|index| ScrapedTitle {
            title: format!("title-{index:02}.mkv"),
            source: ScrapedTorrentSource::Dmhy,
            url: None,
        }));
        let result = torrent_result(
            &queue,
            job.id,
            &args,
            &titles,
            anime_organizer::torrent::clamp_pages(args.pages),
        )
        .unwrap();
        let artifact_id = result.artifacts[0]["id"].as_i64().unwrap();
        let artifact = queue.get_artifact(job.id, artifact_id).unwrap();
        assert_eq!(std::fs::read_to_string(artifact.path).unwrap(), "DMHY - 01.mkv\nNyaa - 02.mkv\ntitle-00.mkv\ntitle-01.mkv\ntitle-02.mkv\ntitle-03.mkv\ntitle-04.mkv\ntitle-05.mkv\ntitle-06.mkv\ntitle-07.mkv\ntitle-08.mkv\ntitle-09.mkv\ntitle-10.mkv\ntitle-11.mkv\ntitle-12.mkv\ntitle-13.mkv\ntitle-14.mkv\ntitle-15.mkv\ntitle-16.mkv\ntitle-17.mkv\ntitle-18.mkv\ntitle-19.mkv\ntitle-20.mkv\ntitle-21.mkv\ntitle-22.mkv\ntitle-23.mkv\ntitle-24.mkv");
        let data = &result.data;
        assert_eq!(data["count"], 27);
        assert_eq!(data["pages"], 1);
        assert_eq!(
            data["preview"].as_array().unwrap().len(),
            MAX_TORRENT_PREVIEW_LINES
        );
        assert_eq!(data["preview_truncated"], true);
    }
}
