use super::cloud::CloudDriveState;
use super::model::{EnqueueRequest, JobOrigin, JobResult, JobSpec};
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

#[derive(Clone)]
pub(crate) struct RssRuntime {
    pub(crate) cloud: CloudDriveState,
    pub(crate) rss_db_path: PathBuf,
}

pub(crate) async fn execute(
    spec: JobSpec,
    runtime: &RssRuntime,
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
        _ => return Err("not an RSS job".to_string()),
    };
    Ok(JobResult {
        summary: format!("RSS poll submitted {submitted} new item(s)"),
        data: serde_json::json!({ "submitted": submitted }),
        artifacts: Vec::new(),
    })
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
        }
    }));
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
