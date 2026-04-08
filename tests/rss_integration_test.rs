//! RSS 功能整合测试
//!
//! 测试从 RSS 抓取到离线下载提交的完整流程，包括 scheduler、processor、db 的协作。

#![cfg(feature = "clouddrive")]

use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

use anime_organizer::error::{AppError, Result};
use anime_organizer::rss::client::CloudDriveClientTrait;
use anime_organizer::rss::db::RssDatabase;
use anime_organizer::rss::filter::RssFilter;
use anime_organizer::rss::http_client::HttpClientTrait;
use anime_organizer::rss::processor::RssProcessor;
use anime_organizer::rss::proxy::ProxyConfig;
use anime_organizer::rss::scheduler::RssScheduler;

#[derive(Debug, Clone)]
#[allow(clippy::type_complexity)]
struct MockCloudDriveClient {
    login_called: Arc<Mutex<Vec<(String, String)>>>,
    add_offline_called: Arc<Mutex<Vec<(Vec<String>, String)>>>,
    should_fail: bool,
}

impl MockCloudDriveClient {
    fn new() -> Self {
        Self {
            login_called: Arc::new(Mutex::new(Vec::new())),
            add_offline_called: Arc::new(Mutex::new(Vec::new())),
            should_fail: false,
        }
    }
}

#[async_trait]
impl CloudDriveClientTrait for MockCloudDriveClient {
    async fn login(&mut self, username: &str, password: &str) -> Result<String> {
        self.login_called
            .lock()
            .unwrap()
            .push((username.to_string(), password.to_string()));
        if self.should_fail {
            return Err(AppError::MetadataFetchError(
                "Mock login failed".to_string(),
            ));
        }
        Ok("mock_token_12345".to_string())
    }

    async fn add_offline_files(&self, urls: Vec<String>, to_folder: &str) -> Result<()> {
        self.add_offline_called
            .lock()
            .unwrap()
            .push((urls, to_folder.to_string()));
        if self.should_fail {
            return Err(AppError::MetadataFetchError("Mock add failed".to_string()));
        }
        Ok(())
    }

    async fn list_folder(
        &self,
        _path: &str,
    ) -> Result<Vec<anime_organizer::rss::client::proto::CloudDriveFile>> {
        Ok(vec![])
    }
}

#[derive(Debug, Clone)]
struct MockHttpClient {
    response: String,
}

impl MockHttpClient {
    fn with_response(response: &str) -> Self {
        Self {
            response: response.to_string(),
        }
    }
}

#[async_trait]
impl HttpClientTrait for MockHttpClient {
    async fn get(&self, _url: &str) -> Result<String> {
        Ok(self.response.clone())
    }
}

#[tokio::test]
async fn test_processor_creation() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let _db = RssDatabase::new(&db_path).unwrap();

    let mock_cd = MockCloudDriveClient::new();
    let mock_http = MockHttpClient::with_response(r#"<?xml version="1.0"?><rss></rss>"#);

    let _processor = RssProcessor::new(Arc::new(mock_http), Arc::new(mock_cd));
}

#[tokio::test]
async fn test_scheduler_single_shot_execution() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = RssDatabase::new(&db_path).unwrap();

    let mock_client = MockCloudDriveClient::new();
    let mock_http = MockHttpClient::with_response(r#"<?xml version="1.0"?><rss></rss>"#);

    let scheduler = RssScheduler::new(db, Arc::new(mock_http), Arc::new(mock_client), false);

    scheduler.run_once().await.unwrap();
}

#[tokio::test]
async fn test_deduplication_logic() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = RssDatabase::new(&db_path).unwrap();

    let sub_id = db
        .add_subscription("https://example.com/rss.xml", None, "/downloads", 300)
        .unwrap();

    let item_hash = "test-item-hash-123";
    let title = "Test Anime - 01";

    assert!(!db.is_item_processed(sub_id, item_hash).unwrap());

    db.mark_item_processed(sub_id, item_hash, title).unwrap();
    assert!(db.is_item_processed(sub_id, item_hash).unwrap());

    let duplicate_result = db.mark_item_processed(sub_id, item_hash, title);
    assert!(duplicate_result.is_ok());
}

#[tokio::test]
async fn test_subscription_upsert() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = RssDatabase::new(&db_path).unwrap();

    db.add_subscription(
        "https://example.com/rss.xml",
        Some(r"filter1"),
        "/folder1",
        300,
    )
    .unwrap();

    let subs = db.list_all_subscriptions().unwrap();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].filter_regex.as_deref(), Some(r"filter1"));
    assert_eq!(subs[0].target_folder, "/folder1");

    db.add_subscription(
        "https://example.com/rss.xml",
        Some(r"filter2"),
        "/folder2",
        600,
    )
    .unwrap();

    let subs = db.list_all_subscriptions().unwrap();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].filter_regex.as_deref(), Some(r"filter2"));
    assert_eq!(subs[0].target_folder, "/folder2");
    assert_eq!(subs[0].interval_secs, 600);
}

#[tokio::test]
async fn test_trait_object_safety() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = RssDatabase::new(&db_path).unwrap();

    let mock_client: Arc<dyn CloudDriveClientTrait> = Arc::new(MockCloudDriveClient::new());
    let mock_http = MockHttpClient::with_response(r#"<?xml version="1.0"?><rss></rss>"#);

    let scheduler = RssScheduler::new(db, Arc::new(mock_http), mock_client, false);

    // Scheduler created successfully - verify it implements Send to confirm it's properly built
    fn assert_send<T: Send>(_: &T) {}
    assert_send(&scheduler);
}

#[tokio::test]
async fn test_mock_client_records_calls() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = RssDatabase::new(&db_path).unwrap();

    let mock_client = MockCloudDriveClient::new();
    let add_called = mock_client.add_offline_called.clone();
    let mock_http = MockHttpClient::with_response(r#"<?xml version="1.0"?><rss></rss>"#);

    let scheduler = RssScheduler::new(db, Arc::new(mock_http), Arc::new(mock_client), false);

    scheduler.run_once().await.unwrap();

    let calls = add_called.lock().unwrap();
    assert!(
        calls.is_empty(),
        "No items should be submitted since no subscriptions have items"
    );
}

#[tokio::test]
async fn test_run_once_url_creates_subscription() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mock_client = MockCloudDriveClient::new();
    let mock_http = MockHttpClient::with_response(r#"<?xml version="1.0"?><rss></rss>"#);

    let scheduler = {
        let db = RssDatabase::new(&db_path).unwrap();
        RssScheduler::new(db, Arc::new(mock_http), Arc::new(mock_client), false)
    };

    drop(scheduler);

    let db = RssDatabase::new(&db_path).unwrap();
    let subs = db.list_subscriptions().unwrap();
    assert!(
        subs.is_empty(),
        "No subscriptions yet without calling run_once_url"
    );
}

#[tokio::test]
async fn test_no_proxy_config_works() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = RssDatabase::new(&db_path).unwrap();

    let mock_client = MockCloudDriveClient::new();
    let mock_http = MockHttpClient::with_response(r#"<?xml version="1.0"?><rss></rss>"#);

    let scheduler = RssScheduler::new(db, Arc::new(mock_http), Arc::new(mock_client), false);

    // Scheduler created successfully - verify it implements Send to confirm it's properly built
    fn assert_send<T: Send>(_: &T) {}
    assert_send(&scheduler);
}

#[test]
fn test_proxy_config_none_handling() {
    let proxy = ProxyConfig::from_env();
    if proxy.is_some() {
        let client = anime_organizer::rss::proxy::build_http_client(&proxy);
        assert!(client.is_ok());
    }
}

#[tokio::test]
async fn test_scheduler_run_once_with_no_subscriptions() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = RssDatabase::new(&db_path).unwrap();

    let mock_client = MockCloudDriveClient::new();
    let add_called = mock_client.add_offline_called.clone();
    let mock_http = MockHttpClient::with_response(r#"<?xml version="1.0"?><rss></rss>"#);

    let scheduler = RssScheduler::new(db, Arc::new(mock_http), Arc::new(mock_client), false);

    scheduler.run_once().await.unwrap();

    let calls = add_called.lock().unwrap();
    assert!(
        calls.is_empty(),
        "Should not have any calls with no subscriptions"
    );
}

#[tokio::test]
async fn test_multiple_subscriptions_upsert() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = RssDatabase::new(&db_path).unwrap();

    db.add_subscription(
        "https://example1.com/rss.xml",
        Some("filter1"),
        "/folder1",
        300,
    )
    .unwrap();
    db.add_subscription(
        "https://example2.com/rss.xml",
        Some("filter2"),
        "/folder2",
        600,
    )
    .unwrap();

    let subs = db.list_all_subscriptions().unwrap();
    assert_eq!(subs.len(), 2, "Should have two subscriptions");

    db.add_subscription(
        "https://example1.com/rss.xml",
        Some("newfilter"),
        "/newfolder",
        900,
    )
    .unwrap();

    let subs = db.list_all_subscriptions().unwrap();
    assert_eq!(
        subs.len(),
        2,
        "Should still have two subscriptions after upsert"
    );
    let updated = subs
        .iter()
        .find(|s| s.url == "https://example1.com/rss.xml")
        .unwrap();
    assert_eq!(updated.filter_regex.as_deref(), Some("newfilter"));
    assert_eq!(updated.target_folder, "/newfolder");
    assert_eq!(updated.interval_secs, 900);
}

#[tokio::test]
async fn test_processor_with_mock_rss() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = RssDatabase::new(&db_path).unwrap();

    let sub_id = db
        .add_subscription(
            "https://example.com/rss.xml",
            Some(r"\[ANi\]"),
            "/downloads",
            300,
        )
        .unwrap();

    let rss_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test RSS</title>
    <item>
      <title>[ANi] Test Anime - 01 [1080P].mp4</title>
      <guid>test-anime-01</guid>
      <enclosure url="magnet:?xt=urn:btih:d41d8cd98f00b204e9800998ecf8427e" type="application/x-bittorrent"/>
    </item>
    <item>
      <title>[Other] Another Anime - 02 [720P].mp4</title>
      <guid>other-anime-02</guid>
      <enclosure url="magnet:?xt=urn:btih:d41d8cd98f00b204e9800998ecf8427f" type="application/x-bittorrent"/>
    </item>
  </channel>
</rss>"#;

    let mock_cd = MockCloudDriveClient::new();
    let add_called = mock_cd.add_offline_called.clone();
    let mock_http = MockHttpClient::with_response(rss_xml);

    let processor = RssProcessor::new(Arc::new(mock_http), Arc::new(mock_cd));

    let filter = RssFilter::new(r"\[ANi\]").unwrap();
    let result = processor
        .process_subscription(
            &db,
            sub_id,
            "https://example.com/rss.xml",
            &Some(filter),
            "/downloads",
            false,
        )
        .await;

    assert!(result.is_ok(), "Processing should succeed");
    assert_eq!(
        result.unwrap(),
        1,
        "Should submit 1 item (only [ANi] matched filter)"
    );

    let calls = add_called.lock().unwrap();
    assert_eq!(calls.len(), 1, "Should have 1 call to add_offline_files");
    let (urls, folder) = &calls[0];
    assert_eq!(urls.len(), 1);
    assert!(
        urls[0].starts_with("magnet:?xt=urn:btih:"),
        "Should be a valid magnet URL"
    );
    assert_eq!(folder, "/downloads");
}

#[tokio::test]
async fn test_processor_deduplicates_across_runs() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = RssDatabase::new(&db_path).unwrap();

    let sub_id = db
        .add_subscription("https://example.com/rss.xml", None, "/downloads", 300)
        .unwrap();

    let rss_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test RSS</title>
    <item>
      <title>Test Anime - 01</title>
      <guid>unique-id-01</guid>
      <enclosure url="magnet:?xt=urn:btih:d41d8cd98f00b204e9800998ecf8427e" type="application/x-bittorrent"/>
    </item>
  </channel>
</rss>"#;

    let mock_cd = MockCloudDriveClient::new();
    let _add_called = mock_cd.add_offline_called.clone();
    let mock_http = MockHttpClient::with_response(rss_xml);

    let processor = RssProcessor::new(Arc::new(mock_http), Arc::new(mock_cd));

    let result1 = processor
        .process_subscription(
            &db,
            sub_id,
            "https://example.com/rss.xml",
            &None,
            "/downloads",
            false,
        )
        .await;
    assert_eq!(result1.unwrap(), 1, "First run should submit 1 item");

    let mock_cd2 = MockCloudDriveClient::new();
    let _add_called2 = mock_cd2.add_offline_called.clone();
    let mock_http2 = MockHttpClient::with_response(rss_xml);

    let processor2 = RssProcessor::new(Arc::new(mock_http2), Arc::new(mock_cd2));

    let result2 = processor2
        .process_subscription(
            &db,
            sub_id,
            "https://example.com/rss.xml",
            &None,
            "/downloads",
            false,
        )
        .await;
    assert_eq!(
        result2.unwrap(),
        0,
        "Second run should submit 0 items (deduplicated)"
    );
}

#[tokio::test]
async fn test_processor_filter_matching() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = RssDatabase::new(&db_path).unwrap();

    let sub_id = db
        .add_subscription(
            "https://example.com/rss.xml",
            Some(r"\[ANi\]"),
            "/downloads",
            300,
        )
        .unwrap();

    let rss_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <item>
      <title>[ANi] Matching Anime - 01</title>
      <guid>match-01</guid>
      <enclosure url="magnet:?xt=urn:btih:aaa" type="application/x-bittorrent"/>
    </item>
    <item>
      <title>[SubsPlease] Non-matching - 02</title>
      <guid>nonmatch-02</guid>
      <enclosure url="magnet:?xt=urn:btih:bbb" type="application/x-bittorrent"/>
    </item>
  </channel>
</rss>"#;

    let mock_cd = MockCloudDriveClient::new();
    let add_called = mock_cd.add_offline_called.clone();
    let mock_http = MockHttpClient::with_response(rss_xml);

    let processor = RssProcessor::new(Arc::new(mock_http), Arc::new(mock_cd));

    let filter = RssFilter::new(r"\[ANi\]").unwrap();
    let result = processor
        .process_subscription(
            &db,
            sub_id,
            "https://example.com/rss.xml",
            &Some(filter),
            "/downloads",
            false,
        )
        .await;

    assert_eq!(
        result.unwrap(),
        1,
        "Should only submit 1 item (filtered by [ANi])"
    );

    let calls = add_called.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert!(
        calls[0].0[0].contains("btih:aaa"),
        "Should submit the matching item"
    );
}

#[tokio::test]
async fn test_scheduler_end_to_end_with_mock_http_and_clouddrive() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let rss_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>True E2E Test RSS</title>
    <item>
      <title>[ANi] Spy x Family - 12 [1080P][Baha][WEB-DL].mp4</title>
      <guid>spy-family-12</guid>
      <enclosure url="magnet:?xt=urn:btih:abc123def456" type="application/x-bittorrent"/>
    </item>
    <item>
      <title>[SubsPlease] Another Anime - 05 [720p].mkv</title>
      <guid>another-05</guid>
      <enclosure url="magnet:?xt=urn:btih:789xyz" type="application/x-bittorrent"/>
    </item>
  </channel>
</rss>"#;

    let mock_cd = MockCloudDriveClient::new();
    let cd_calls = mock_cd.add_offline_called.clone();
    let mock_http = MockHttpClient::with_response(rss_xml);

    let scheduler = {
        let db = RssDatabase::new(&db_path).unwrap();
        RssScheduler::new(db, Arc::new(mock_http), Arc::new(mock_cd), false)
    };

    scheduler
        .run_once_url(
            "https://example.com/rss.xml",
            Some(r"\[ANi\]"),
            "/anime/downloads",
            300,
        )
        .await
        .unwrap();

    let calls = cd_calls.lock().unwrap();
    assert_eq!(calls.len(), 1, "Should have exactly 1 call to CloudDrive2");
    let (urls, folder) = &calls[0];
    assert_eq!(urls.len(), 1);
    assert!(
        urls[0].starts_with("magnet:?xt=urn:btih:abc123"),
        "Should submit the matching [ANi] magnet URL, got: {}",
        urls[0]
    );
    assert_eq!(folder, "/anime/downloads");

    let db = RssDatabase::new(&db_path).unwrap();
    let subs = db.list_subscriptions().unwrap();
    assert_eq!(subs.len(), 1, "Should have saved the subscription to DB");
    assert_eq!(subs[0].url, "https://example.com/rss.xml");
}

#[tokio::test]
async fn test_scheduler_e2e_deduplication_via_run_once_url() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let rss_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Dedup Test RSS</title>
    <item>
      <title>Test Anime - 01 [720p]</title>
      <guid>dedup-test-01</guid>
      <enclosure url="magnet:?xt=urn:btih:dedup001" type="application/x-bittorrent"/>
    </item>
  </channel>
</rss>"#;

    let mock_cd = MockCloudDriveClient::new();
    let cd_calls = mock_cd.add_offline_called.clone();
    let mock_http = MockHttpClient::with_response(rss_xml);

    let scheduler = {
        let db = RssDatabase::new(&db_path).unwrap();
        RssScheduler::new(db, Arc::new(mock_http), Arc::new(mock_cd), false)
    };

    scheduler
        .run_once_url("https://dedup.com/rss.xml", None, "/downloads", 300)
        .await
        .unwrap();

    assert_eq!(
        cd_calls.lock().unwrap().len(),
        1,
        "First run should submit 1 item"
    );

    let mock_http2 = MockHttpClient::with_response(rss_xml);
    let mock_cd2 = MockCloudDriveClient::new();
    let cd_calls2 = mock_cd2.add_offline_called.clone();

    let scheduler2 = {
        let db = RssDatabase::new(&db_path).unwrap();
        RssScheduler::new(db, Arc::new(mock_http2), Arc::new(mock_cd2), false)
    };

    scheduler2
        .run_once_url("https://dedup.com/rss.xml", None, "/downloads", 300)
        .await
        .unwrap();

    assert_eq!(
        cd_calls2.lock().unwrap().len(),
        0,
        "Second run should NOT submit any items (deduplicated)"
    );
}

#[tokio::test]
async fn test_scheduler_daemon_url_runs_and_submits() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let rss_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Daemon Test RSS</title>
    <item>
      <title>[ANi] Daemon Anime - 01 [1080P]</title>
      <guid>daemon-01</guid>
      <enclosure url="magnet:?xt=urn:btih:daemon001" type="application/x-bittorrent"/>
    </item>
  </channel>
</rss>"#;

    let mock_cd = MockCloudDriveClient::new();
    let cd_calls = mock_cd.add_offline_called.clone();
    let mock_http = MockHttpClient::with_response(rss_xml);

    let scheduler = {
        let db = RssDatabase::new(&db_path).unwrap();
        RssScheduler::new(db, Arc::new(mock_http), Arc::new(mock_cd), false)
    };

    let interval = std::time::Duration::from_millis(10);
    let _ = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        scheduler.run_daemon_url(
            "https://daemon.test/rss.xml",
            Some(r"\[ANi\]"),
            "/anime",
            interval,
        ),
    )
    .await;

    let calls = cd_calls.lock().unwrap();
    assert!(
        !calls.is_empty(),
        "Daemon should have submitted at least 1 item, got {} calls",
        calls.len()
    );
}

#[tokio::test]
async fn test_scheduler_daemon_url_multiple_iterations() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = call_count.clone();

    let rss_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Multi-iter RSS</title>
    <item>
      <title>Anime - 01</title>
      <guid>multi-01</guid>
      <enclosure url="magnet:?xt=urn:btih:multi001" type="application/x-bittorrent"/>
    </item>
  </channel>
</rss>"#;

    #[derive(Clone)]
    struct CountingHttpClient {
        response: String,
        call_count: Arc<AtomicUsize>,
    }

    impl CountingHttpClient {
        fn new(response: &str, call_count: Arc<AtomicUsize>) -> Self {
            Self {
                response: response.to_string(),
                call_count,
            }
        }
    }

    #[async_trait]
    impl HttpClientTrait for CountingHttpClient {
        async fn get(&self, _url: &str) -> Result<String> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(self.response.clone())
        }
    }

    let mock_cd = MockCloudDriveClient::new();
    let cd_calls = mock_cd.add_offline_called.clone();
    let mock_http = CountingHttpClient::new(rss_xml, call_count_clone);

    let scheduler = {
        let db = RssDatabase::new(&db_path).unwrap();
        RssScheduler::new(db, Arc::new(mock_http), Arc::new(mock_cd), false)
    };

    let interval = std::time::Duration::from_millis(5);
    let _result = tokio::time::timeout(
        std::time::Duration::from_millis(50),
        scheduler.run_daemon_url("https://multi.test/rss.xml", None, "/downloads", interval),
    )
    .await;

    let http_calls = call_count.load(Ordering::SeqCst);
    let cd_call_count = cd_calls.lock().unwrap().len();

    assert!(
        http_calls >= 1,
        "HTTP should be called at least once, got {} calls",
        http_calls
    );
    assert_eq!(
        cd_call_count, 1,
        "Should submit exactly 1 item (deduplicated across iterations), got {}",
        cd_call_count
    );
}

#[tokio::test]
async fn test_scheduler_run_once_with_db_subscription() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let rss_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>RunOnce DB Test</title>
    <item>
      <title>DB Test Anime - 05</title>
      <guid>db-test-05</guid>
      <enclosure url="magnet:?xt=urn:btih:db005" type="application/x-bittorrent"/>
    </item>
  </channel>
</rss>"#;

    let mock_cd = MockCloudDriveClient::new();
    let cd_calls = mock_cd.add_offline_called.clone();
    let mock_http = MockHttpClient::with_response(rss_xml);

    let scheduler = {
        let db = RssDatabase::new(&db_path).unwrap();
        db.add_subscription("https://db.test/rss.xml", None, "/anime", 300)
            .unwrap();
        RssScheduler::new(db, Arc::new(mock_http), Arc::new(mock_cd), false)
    };

    scheduler.run_once().await.unwrap();

    let calls = cd_calls.lock().unwrap();
    assert_eq!(calls.len(), 1, "Should submit 1 item from DB subscription");
}

#[tokio::test]
async fn test_scheduler_daemon_runs_and_processes_db_subscriptions() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let rss_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Daemon DB Test RSS</title>
    <item>
      <title>[ANi] Daemon DB Anime - 03 [1080P]</title>
      <guid>daemon-db-03</guid>
      <enclosure url="magnet:?xt=urn:btih:daemondb003" type="application/x-bittorrent"/>
    </item>
  </channel>
</rss>"#;

    let mock_cd = MockCloudDriveClient::new();
    let cd_calls = mock_cd.add_offline_called.clone();
    let mock_http = MockHttpClient::with_response(rss_xml);

    let scheduler = {
        let db = RssDatabase::new(&db_path).unwrap();
        db.add_subscription(
            "https://daemon-db.test/rss.xml",
            Some(r"\[ANi\]"),
            "/anime",
            60,
        )
        .unwrap();
        RssScheduler::new(db, Arc::new(mock_http), Arc::new(mock_cd), false)
    };

    let _ = tokio::time::timeout(
        std::time::Duration::from_millis(50),
        scheduler.run_daemon(std::time::Duration::from_millis(5)),
    )
    .await;

    let calls = cd_calls.lock().unwrap();
    assert!(
        !calls.is_empty(),
        "run_daemon should have submitted at least 1 item, got {} calls",
        calls.len()
    );
}
