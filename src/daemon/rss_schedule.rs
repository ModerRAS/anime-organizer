use super::cloud::CloudDriveState;
use super::model::{
    EnqueueRequest, JobOrigin, JobResult, JobSpec, StorageEndpointArgs, StorageOrganizeJobArgs,
    StorageOrganizeMode,
};
use super::queue::QueueRepository;
use anime_organizer::rss::client::CloudDriveClientTrait;
use anime_organizer::rss::db::RssDatabase;
use anime_organizer::rss::filter::RssFilter;
use anime_organizer::rss::http_client::HttpClient;
use anime_organizer::rss::processor::RssProcessor;
use anime_organizer::rss::proxy::{build_http_client, ProxyConfig};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) const SCHEDULER_INTERVAL: Duration = Duration::from_secs(30);

type ProgressReporter<'a> = dyn Fn(&str, Option<usize>, Option<usize>, &str) + 'a;

#[derive(Clone)]
pub(crate) struct RssRuntime {
    pub(crate) cloud: CloudDriveState,
    pub(crate) rss_db_path: PathBuf,
}

pub(crate) async fn execute(
    spec: JobSpec,
    runtime: &RssRuntime,
) -> std::result::Result<JobResult, String> {
    execute_with_progress(spec, runtime, &|_, _, _, _| {}).await
}

pub(crate) async fn execute_with_progress(
    spec: JobSpec,
    runtime: &RssRuntime,
    progress: &ProgressReporter<'_>,
) -> std::result::Result<JobResult, String> {
    let db = RssDatabase::new(&runtime.rss_db_path).map_err(|error| error.to_string())?;
    let submitted = match spec {
        JobSpec::RssPoll { subscription_id } => poll_one(&db, subscription_id, runtime).await?,
        JobSpec::RssPollAll => {
            let subscriptions = db.list_subscriptions().map_err(|error| error.to_string())?;
            let mut total = 0usize;
            let mut failures = Vec::new();
            for subscription in subscriptions {
                match poll_subscription(&db, &subscription, runtime).await {
                    Ok(submitted) => total += submitted,
                    Err(error) => failures.push(format!("{}: {error}", subscription.id)),
                }
            }
            if !failures.is_empty() {
                return Err(format!(
                    "RSS poll-all submitted {total} new item(s); {} subscription(s) failed: {}",
                    failures.len(),
                    failures.join("; ")
                ));
            }
            total
        }
        JobSpec::RemoteRssOrganize { subscription_id } => {
            progress(
                "info",
                None,
                None,
                &format!("Loading RSS subscription {subscription_id} and storage connections"),
            );
            let subscription = db
                .get_subscription(subscription_id)
                .map_err(|error| error.to_string())?
                .ok_or_else(|| format!("RSS subscription {subscription_id} was not found"))?;
            if !subscription.enabled || !subscription.auto_organize {
                return Err(format!(
                    "RSS subscription {subscription_id} is not auto-organize enabled"
                ));
            }
            let connection_id = subscription.connection_id.ok_or_else(|| {
                format!("RSS subscription {subscription_id} has no CloudDrive connection")
            })?;
            let source_connection = runtime
                .cloud
                .repository
                .get(connection_id)
                .map_err(|error| error.to_string())?;
            let source_client = runtime
                .cloud
                .authenticated_client(&source_connection)
                .await
                .map_err(|error| error.to_string())?;
            let target_connection_id = subscription
                .organize_target_connection_id
                .unwrap_or(connection_id);
            let target_client = if target_connection_id == connection_id {
                None
            } else {
                let target_connection = runtime
                    .cloud
                    .repository
                    .get(target_connection_id)
                    .map_err(|error| error.to_string())?;
                Some(
                    runtime
                        .cloud
                        .authenticated_client(&target_connection)
                        .await
                        .map_err(|error| error.to_string())?,
                )
            };
            progress(
                "info",
                None,
                None,
                &format!(
                    "Authenticated RSS storage connections: source={connection_id}, target={target_connection_id}; source='{}', target='{}', mode={}, remote_mlip={}, remove_empty_dirs={}",
                    subscription.target_folder,
                    subscription.organize_target_folder.as_deref().unwrap_or("<missing>"),
                    subscription.organize_mode,
                    subscription.remote_mlip,
                    subscription.remove_empty_dirs
                ),
            );
            let target_client = target_client.as_deref().unwrap_or(&*source_client);
            let summary = super::remote_organize::organize_subscription_between_with_progress(
                &db,
                &subscription,
                &*source_client,
                target_client,
                super::remote_organize::RemoteOrganizeOptions {
                    same_connection: target_connection_id == connection_id,
                    use_native_transfers: target_connection_id == connection_id,
                    move_sources: subscription.remote_mlip
                        || subscription.organize_mode != "original",
                    source_root_as_bundle: false,
                },
                progress,
            )
            .await?;
            return Ok(JobResult {
                summary: format!(
                    "Remote RSS organize moved {} media file(s), copied {} media file(s), removed {} empty source folder(s), skipped {} conflict(s), left {} uncorrelated legacy task(s)",
                    summary.moved_media,
                    summary.copied_media,
                    summary.removed_empty_directories,
                    summary.skipped_conflicts,
                    summary.uncorrelated_legacy_tasks
                ),
                data: serde_json::to_value(summary).map_err(|error| error.to_string())?,
                artifacts: Vec::new(),
            });
        }
        JobSpec::CleanupEmptyDirs {
            subscription_id,
            dry_run,
        } => {
            let subscription = db
                .get_subscription(subscription_id)
                .map_err(|error| error.to_string())?
                .ok_or_else(|| format!("RSS subscription {subscription_id} was not found"))?;
            let connection_id = subscription.connection_id.ok_or_else(|| {
                format!("RSS subscription {subscription_id} has no CloudDrive connection")
            })?;
            let connection = runtime
                .cloud
                .repository
                .get(connection_id)
                .map_err(|error| error.to_string())?;
            let client = runtime
                .cloud
                .authenticated_client(&connection)
                .await
                .map_err(|error| error.to_string())?;
            let offline = client
                .list_offline_files_by_path(&subscription.target_folder)
                .await
                .map_err(|error| error.to_string())?;
            if offline.iter().any(|file| {
                matches!(
                    file.status,
                    value if value == anime_organizer::rss::client::proto::OfflineFileStatus::OfflineInit as i32
                        || value == anime_organizer::rss::client::proto::OfflineFileStatus::OfflineDownloading as i32
                )
            }) {
                return Err(format!(
                    "RSS subscription {subscription_id} still has active offline downloads; empty-directory cleanup was not started"
                ));
            }
            let removed = cleanup_remote_empty_directories(
                &*client,
                &subscription.target_folder,
                dry_run,
                progress,
            )
            .await?;
            return Ok(JobResult {
                summary: if dry_run {
                    format!("Found {} removable empty source folder(s)", removed.len())
                } else {
                    format!("Removed {} empty source folder(s)", removed.len())
                },
                data: serde_json::json!({
                    "dry_run": dry_run,
                    "root": subscription.target_folder,
                    "directories": removed,
                }),
                artifacts: Vec::new(),
            });
        }
        _ => return Err("not an RSS job".to_string()),
    };
    Ok(JobResult {
        summary: format!("RSS poll submitted {submitted} new item(s)"),
        data: serde_json::json!({ "submitted": submitted }),
        artifacts: Vec::new(),
    })
}

pub(crate) async fn execute_storage_organize(
    args: &StorageOrganizeJobArgs,
    runtime: &RssRuntime,
    progress: &ProgressReporter<'_>,
) -> std::result::Result<JobResult, String> {
    validate_local_endpoint_relationship(&args.source, &args.target)?;
    let temporary = tempfile::NamedTempFile::new()
        .map_err(|error| format!("create storage organize planning database failed: {error}"))?;
    let db = RssDatabase::new(temporary.path()).map_err(|error| error.to_string())?;
    let source_path = endpoint_logical_path(&args.source);
    let target_path = endpoint_logical_path(&args.target);
    let subscription_id = db
        .add_subscription(
            "https://storage-organize.invalid/plan",
            None,
            source_path,
            300,
        )
        .map_err(|error| error.to_string())?;
    db.update_subscription_organization_settings_with_mlip_and_mode(
        subscription_id,
        true,
        Some(target_path),
        args.season_mode,
        args.remove_empty_dirs,
        args.mlip,
        "original",
    )
    .map_err(|error| error.to_string())?;
    let subscription = db
        .get_subscription(subscription_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "temporary storage organization plan disappeared".to_string())?;

    let source_client = endpoint_client(&args.source, runtime).await?;
    let same_connection = matches!(
        (&args.source, &args.target),
        (
            StorageEndpointArgs::Connection { connection_id: source, .. },
            StorageEndpointArgs::Connection { connection_id: target, .. }
        ) if source == target
    );
    let use_native_transfers = if same_connection {
        let StorageEndpointArgs::Connection { connection_id, .. } = &args.source else {
            unreachable!("same_connection only matches connection endpoints")
        };
        runtime
            .cloud
            .repository
            .get(*connection_id)
            .map_err(|error| error.to_string())?
            .kind
            == "clouddrive"
    } else {
        false
    };
    let target_client = if same_connection {
        None
    } else {
        Some(endpoint_client(&args.target, runtime).await?)
    };
    let target_client = target_client.as_deref().unwrap_or(&*source_client);
    let mut summary = super::remote_organize::organize_subscription_between_with_progress(
        &db,
        &subscription,
        &*source_client,
        target_client,
        super::remote_organize::RemoteOrganizeOptions {
            same_connection,
            use_native_transfers,
            move_sources: args.mode == StorageOrganizeMode::Move,
            source_root_as_bundle: true,
        },
        progress,
    )
    .await?;
    if args.remove_empty_dirs {
        let removed =
            cleanup_remote_empty_directories(&*source_client, source_path, false, progress).await?;
        summary.removed_empty_directories += removed.len();
    }
    Ok(JobResult {
        summary: format!(
            "Storage organize moved {} media file(s), copied {} media file(s), removed {} empty source folder(s), skipped {} conflict(s)",
            summary.moved_media,
            summary.copied_media,
            summary.removed_empty_directories,
            summary.skipped_conflicts
        ),
        data: serde_json::to_value(summary).map_err(|error| error.to_string())?,
        artifacts: Vec::new(),
    })
}

fn validate_local_endpoint_relationship(
    source: &StorageEndpointArgs,
    target: &StorageEndpointArgs,
) -> std::result::Result<(), String> {
    let (StorageEndpointArgs::Local { path: source }, StorageEndpointArgs::Local { path: target }) =
        (source, target)
    else {
        return Ok(());
    };
    let source = source
        .canonicalize()
        .map_err(|error| format!("resolve local storage source failed: {error}"))?;
    let target = target
        .canonicalize()
        .map_err(|error| format!("resolve local storage target failed: {error}"))?;
    if source == target || source.starts_with(&target) || target.starts_with(&source) {
        return Err("local storage source and target must not be equal or nested".to_string());
    }
    Ok(())
}

fn endpoint_logical_path(endpoint: &StorageEndpointArgs) -> &str {
    match endpoint {
        StorageEndpointArgs::Local { .. } => "/",
        StorageEndpointArgs::Connection { path, .. } => path,
    }
}

async fn endpoint_client(
    endpoint: &StorageEndpointArgs,
    runtime: &RssRuntime,
) -> std::result::Result<Box<dyn CloudDriveClientTrait>, String> {
    match endpoint {
        StorageEndpointArgs::Local { path } => {
            anime_organizer::rss::local::LocalStorageClient::new(path)
                .map(|client| Box::new(client) as Box<dyn CloudDriveClientTrait>)
                .map_err(|error| error.to_string())
        }
        StorageEndpointArgs::Connection { connection_id, .. } => {
            let connection = runtime
                .cloud
                .repository
                .get(*connection_id)
                .map_err(|error| error.to_string())?;
            runtime
                .cloud
                .authenticated_client(&connection)
                .await
                .map_err(|error| error.to_string())
        }
    }
}

async fn cleanup_remote_empty_directories(
    client: &dyn CloudDriveClientTrait,
    root: &str,
    dry_run: bool,
    progress: &ProgressReporter<'_>,
) -> std::result::Result<Vec<String>, String> {
    let root = if root == "/" {
        "/"
    } else {
        root.trim_end_matches('/')
    };
    let mut pending = vec![root.to_string()];
    let mut directories = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in client
            .list_folder_fresh(&directory)
            .await
            .map_err(|error| error.to_string())?
        {
            if entry.is_directory {
                if !safe_remote_component(&entry.name) {
                    return Err(format!(
                        "CloudDrive returned an unsafe directory name under '{directory}'"
                    ));
                }
                let path = if directory == "/" {
                    format!("/{}", entry.name)
                } else {
                    format!("{}/{}", directory.trim_end_matches('/'), entry.name)
                };
                directories.push(path.clone());
                pending.push(path);
            }
        }
    }
    directories.sort_by_key(|path| std::cmp::Reverse(path.matches('/').count()));

    let mut removed = Vec::new();
    for directory in directories {
        let entries = client
            .list_folder_fresh(&directory)
            .await
            .map_err(|error| error.to_string())?;
        let effectively_empty = entries.iter().all(|entry| {
            entry.is_directory
                && removed.iter().any(|removed_path| {
                    let child = if directory == "/" {
                        format!("/{}", entry.name)
                    } else {
                        format!("{}/{}", directory.trim_end_matches('/'), entry.name)
                    };
                    removed_path == &child
                })
        });
        if effectively_empty {
            if !dry_run {
                client
                    .delete_file(&directory)
                    .await
                    .map_err(|error| error.to_string())?;
            }
            progress(
                "info",
                Some(removed.len() + 1),
                None,
                &format!(
                    "{} empty source directory '{}'",
                    if dry_run { "Found" } else { "Removed" },
                    directory
                ),
            );
            removed.push(directory);
        }
    }
    Ok(removed)
}

fn safe_remote_component(value: &str) -> bool {
    !value.is_empty() && value != "." && value != ".." && !value.contains(['/', '\\'])
}

async fn poll_one(
    db: &RssDatabase,
    subscription_id: i64,
    runtime: &RssRuntime,
) -> std::result::Result<usize, String> {
    let subscription = db
        .get_subscription(subscription_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("RSS subscription {subscription_id} was not found"))?;
    if !subscription.enabled {
        return Err(format!("RSS subscription {subscription_id} is disabled"));
    }
    poll_subscription(db, &subscription, runtime).await
}

async fn poll_subscription(
    db: &RssDatabase,
    subscription: &anime_organizer::rss::db::Subscription,
    runtime: &RssRuntime,
) -> std::result::Result<usize, String> {
    let connection_id = subscription.connection_id.ok_or_else(|| {
        format!(
            "RSS subscription {} has no CloudDrive connection",
            subscription.id
        )
    })?;
    let connection = runtime
        .cloud
        .repository
        .get(connection_id)
        .map_err(|error| error.to_string())?;
    let client = runtime
        .cloud
        .authenticated_client(&connection)
        .await
        .map_err(|error| error.to_string())?;

    let filter = subscription
        .filter_regex
        .as_deref()
        .map(RssFilter::new)
        .transpose()
        .map_err(|error| error.to_string())?;
    let http = build_http_client(&ProxyConfig::from_env()).map_err(|error| error.to_string())?;
    let cd_client: Arc<dyn CloudDriveClientTrait> = client.into();
    let processor = RssProcessor::new(Arc::new(HttpClient::new(http)), cd_client);
    let submitted = processor
        .process_subscription(
            db,
            subscription.id,
            &subscription.url,
            &filter,
            &subscription.target_folder,
            false,
        )
        .await
        .map_err(|error| error.to_string())?;
    // Only a fully completed processor call advances the schedule. A failed item
    // is intentionally retried by a later due window.
    db.mark_subscription_checked(subscription.id)
        .map_err(|error| error.to_string())?;
    Ok(submitted)
}

pub(crate) fn start_scheduler(
    queue: QueueRepository,
    rss_db_path: PathBuf,
    wake: mpsc::Sender<()>,
) {
    drop(tokio::spawn(async move {
        let mut interval = tokio::time::interval(SCHEDULER_INTERVAL);
        loop {
            interval.tick().await;
            let Ok(db) = RssDatabase::new(&rss_db_path) else {
                continue;
            };
            let Ok(subscriptions) = db.list_due_subscriptions() else {
                continue;
            };
            for subscription in subscriptions {
                if subscription.connection_id.is_none() {
                    continue;
                }
                let key = format!("rss:{}:{}", subscription.id, due_window(&subscription));
                let request = EnqueueRequest {
                    idempotency_key: Some(key),
                    origin: JobOrigin::Scheduled,
                    confirmed: false,
                    job: JobSpec::RssPoll {
                        subscription_id: subscription.id,
                    },
                };
                // A duplicate due window or a still-active prior window is expected.
                if queue.enqueue(&request).is_ok() {
                    let _ = wake.send(());
                }
            }
            let Ok(subscriptions) = db.list_due_organization_subscriptions() else {
                continue;
            };
            for subscription in subscriptions {
                let request = EnqueueRequest {
                    idempotency_key: Some(format!(
                        "rss-organize:{}:{}",
                        subscription.id,
                        organize_due_window(&subscription)
                    )),
                    origin: JobOrigin::Scheduled,
                    confirmed: false,
                    job: JobSpec::RemoteRssOrganize {
                        subscription_id: subscription.id,
                    },
                };
                if queue.enqueue(&request).is_ok() {
                    let _ = wake.send(());
                }
            }
        }
    }));
}

fn organize_due_window(subscription: &anime_organizer::rss::db::Subscription) -> u64 {
    let interval = u64::try_from(subscription.organize_interval_secs.max(1)).unwrap_or(1);
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / interval
}

fn due_window(subscription: &anime_organizer::rss::db::Subscription) -> u64 {
    let interval = u64::try_from(subscription.interval_secs.max(1)).unwrap_or(1);
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / interval
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daemon::cloud::{CloudConnectionRepository, CloudDriveClientFactory};
    use anime_organizer::error::Result;
    use anime_organizer::rss::client::proto::CloudDriveFile;
    use anime_organizer::rss::http_client::HttpClientTrait;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::tempdir;

    #[derive(Clone)]
    struct MockCloud;

    struct MockHttp;

    #[async_trait]
    impl HttpClientTrait for MockHttp {
        async fn get(&self, _: &str) -> Result<String> {
            Ok("<rss><channel><item><title>Episode 1</title><guid>item-1</guid><link>magnet:?xt=urn:btih:test</link></item></channel></rss>".to_string())
        }
    }

    #[derive(Clone)]
    struct CountingCloud(Arc<AtomicUsize>);

    #[async_trait]
    impl CloudDriveClientTrait for MockCloud {
        async fn login(&mut self, _: &str, _: &str) -> Result<String> {
            Ok("token".to_string())
        }
        async fn add_offline_files(&self, _: Vec<String>, _: &str) -> Result<()> {
            Ok(())
        }
        async fn list_folder(&self, _: &str) -> Result<Vec<CloudDriveFile>> {
            Ok(Vec::new())
        }
    }

    #[async_trait]
    impl CloudDriveClientTrait for CountingCloud {
        async fn login(&mut self, _: &str, _: &str) -> Result<String> {
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

    #[derive(Clone, Default)]
    struct EmptyTreeCloud {
        deleted: Arc<std::sync::Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl CloudDriveClientTrait for EmptyTreeCloud {
        async fn login(&mut self, _: &str, _: &str) -> Result<String> {
            Ok("token".to_string())
        }

        async fn add_offline_files(&self, _: Vec<String>, _: &str) -> Result<()> {
            Ok(())
        }

        async fn list_folder(&self, path: &str) -> Result<Vec<CloudDriveFile>> {
            let deleted = self.deleted.lock().unwrap();
            let child = match path {
                "/source" if !deleted.iter().any(|value| value == "/source/parent") => {
                    Some("parent")
                }
                "/source/parent" if !deleted.iter().any(|value| value == "/source/parent/leaf") => {
                    Some("leaf")
                }
                _ => None,
            };
            Ok(child
                .map(|name| CloudDriveFile {
                    name: name.to_string(),
                    is_directory: true,
                    ..Default::default()
                })
                .into_iter()
                .collect())
        }

        async fn delete_file(&self, path: &str) -> Result<()> {
            self.deleted.lock().unwrap().push(path.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn storage_organize_moves_local_video_and_subtitle_as_one_bundle() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("source");
        let target = directory.path().join("target");
        std::fs::create_dir_all(source.join("empty").join("nested")).unwrap();
        std::fs::create_dir_all(&target).unwrap();
        let video_name = "[ANi] Storage Show - 01 [1080P].mkv";
        let subtitle_name = "[ANi] Storage Show - 01 [1080P].zh-CN.ass";
        std::fs::write(source.join(video_name), b"video bytes").unwrap();
        std::fs::write(source.join(subtitle_name), b"subtitle bytes").unwrap();
        let repository =
            CloudConnectionRepository::new(&directory.path().join("daemon.db")).unwrap();
        let runtime = RssRuntime {
            cloud: CloudDriveState::with_factory(
                repository,
                Arc::new(|_| Ok(Box::new(MockCloud) as Box<dyn CloudDriveClientTrait>)),
            ),
            rss_db_path: directory.path().join("rss.db"),
        };
        let args = StorageOrganizeJobArgs {
            source: StorageEndpointArgs::Local {
                path: source.clone(),
            },
            target: StorageEndpointArgs::Local {
                path: target.clone(),
            },
            mode: StorageOrganizeMode::Move,
            season_mode: false,
            mlip: false,
            remove_empty_dirs: true,
        };

        let result = execute_storage_organize(&args, &runtime, &|_, _, _, _| {})
            .await
            .unwrap();

        assert_eq!(result.data["moved_media"], 1);
        assert!(source.is_dir());
        assert!(!source.join("empty").exists());
        assert!(!source.join(video_name).exists());
        assert!(!source.join(subtitle_name).exists());
        assert_eq!(
            std::fs::read(target.join("Storage Show").join(video_name)).unwrap(),
            b"video bytes"
        );
        assert_eq!(
            std::fs::read(target.join("Storage Show").join(subtitle_name)).unwrap(),
            b"subtitle bytes"
        );
    }

    #[tokio::test]
    async fn storage_organize_rejects_nested_local_endpoints() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("source");
        let target = source.join("target");
        std::fs::create_dir_all(&target).unwrap();
        let repository =
            CloudConnectionRepository::new(&directory.path().join("daemon.db")).unwrap();
        let runtime = RssRuntime {
            cloud: CloudDriveState::with_factory(
                repository,
                Arc::new(|_| Ok(Box::new(MockCloud) as Box<dyn CloudDriveClientTrait>)),
            ),
            rss_db_path: directory.path().join("rss.db"),
        };
        let args = StorageOrganizeJobArgs {
            source: StorageEndpointArgs::Local { path: source },
            target: StorageEndpointArgs::Local { path: target },
            mode: StorageOrganizeMode::Copy,
            season_mode: false,
            mlip: false,
            remove_empty_dirs: false,
        };

        let error = execute_storage_organize(&args, &runtime, &|_, _, _, _| {})
            .await
            .unwrap_err();
        assert!(error.contains("must not be equal or nested"));
    }

    #[tokio::test]
    async fn cleanup_removes_empty_descendants_deepest_first_and_keeps_root() {
        let cloud = EmptyTreeCloud::default();
        let removed = cleanup_remote_empty_directories(&cloud, "/source", false, &|_, _, _, _| {})
            .await
            .unwrap();

        assert_eq!(
            removed,
            vec![
                "/source/parent/leaf".to_string(),
                "/source/parent".to_string()
            ]
        );
        assert_eq!(*cloud.deleted.lock().unwrap(), removed);
    }

    #[tokio::test]
    async fn repeated_finite_polls_submit_one_item_once() {
        let directory = tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = db
            .add_subscription("https://example.test/feed.xml", None, "/anime", 300)
            .unwrap();
        let count = Arc::new(AtomicUsize::new(0));
        let processor =
            RssProcessor::new(Arc::new(MockHttp), Arc::new(CountingCloud(count.clone())));
        let first = processor
            .process_subscription(
                &db,
                id,
                "https://example.test/feed.xml",
                &None,
                "/anime",
                false,
            )
            .await
            .unwrap();
        let second = processor
            .process_subscription(
                &db,
                id,
                "https://example.test/feed.xml",
                &None,
                "/anime",
                false,
            )
            .await
            .unwrap();
        assert_eq!(first, 1);
        assert_eq!(second, 0);
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn poll_all_reports_every_subscription_failure() {
        let directory = tempdir().unwrap();
        let rss_db_path = directory.path().join("rss.db");
        let db = RssDatabase::new(&rss_db_path).unwrap();
        let first = db
            .add_subscription("https://example.test/one.xml", None, "/anime", 300)
            .unwrap();
        let second = db
            .add_subscription("https://example.test/two.xml", None, "/anime", 300)
            .unwrap();
        let repository =
            CloudConnectionRepository::new(&directory.path().join("daemon.db")).unwrap();
        let factory: CloudDriveClientFactory =
            Arc::new(|_| Ok(Box::new(MockCloud) as Box<dyn CloudDriveClientTrait>));
        let runtime = RssRuntime {
            cloud: CloudDriveState::with_factory(repository, factory),
            rss_db_path,
        };

        let error = execute(JobSpec::RssPollAll, &runtime).await.unwrap_err();
        assert!(error.contains(&format!("{first}:")));
        assert!(error.contains(&format!("{second}:")));
        assert!(error.contains("2 subscription(s) failed"));
    }

    #[test]
    fn due_window_is_stable_for_an_interval() {
        let subscription = anime_organizer::rss::db::Subscription {
            id: 7,
            url: "https://example.test/rss".to_string(),
            filter_regex: None,
            target_folder: "/anime".to_string(),
            interval_secs: 300,
            enabled: true,
            last_checked_at: None,
            connection_id: Some(1),
            auto_organize: false,
            organize_target_folder: None,
            organize_target_connection_id: None,
            organize_season_mode: true,
            remove_empty_dirs: false,
            remote_mlip: false,
            organize_mode: "offline".to_string(),
            organize_interval_secs: 300,
            last_organize_checked_at: None,
        };
        assert_eq!(due_window(&subscription), due_window(&subscription));
    }

    #[test]
    fn runtime_can_be_built_with_a_mock_client() {
        let directory = tempdir().unwrap();
        let repository =
            CloudConnectionRepository::new(&directory.path().join("daemon.db")).unwrap();
        let factory: CloudDriveClientFactory =
            Arc::new(|_| Ok(Box::new(MockCloud) as Box<dyn CloudDriveClientTrait>));
        let state = CloudDriveState::with_factory(repository, factory);
        let runtime = RssRuntime {
            cloud: state,
            rss_db_path: directory.path().join("rss.db"),
        };
        assert!(runtime.rss_db_path.ends_with("rss.db"));
    }
}
