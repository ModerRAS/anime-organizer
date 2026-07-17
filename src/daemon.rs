use anime_organizer::error::AppError;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Instant;

pub(crate) mod api;
#[cfg(feature = "clouddrive")]
pub(crate) mod cloud;
pub(crate) mod model;
pub(crate) mod queue;
#[cfg(feature = "clouddrive")]
pub(crate) mod rss_schedule;
pub(crate) mod web;
pub(crate) mod worker;

use queue::QueueRepository;
use worker::WorkerSnapshot;

pub(crate) struct DaemonState {
    pub(crate) queue: QueueRepository,
    #[cfg(feature = "clouddrive")]
    pub(crate) cloud: cloud::CloudDriveState,
    #[cfg(feature = "clouddrive")]
    pub(crate) rss_db_path: PathBuf,
    pub(crate) started_at: Instant,
    pub(crate) wake: mpsc::Sender<()>,
    pub(crate) worker: Arc<Mutex<WorkerSnapshot>>,
}

pub(crate) fn run() -> Result<(), AppError> {
    let queue = QueueRepository::new(&daemon_db_path())
        .map_err(|error| AppError::MetadataFetchError(error.to_string()))?;
    queue
        .recover_running()
        .map_err(|error| AppError::MetadataFetchError(error.to_string()))?;
    #[cfg(feature = "clouddrive")]
    let cloud = cloud::CloudDriveState::new(&daemon_db_path())
        .map_err(|error| AppError::MetadataFetchError(error.to_string()))?;

    let (wake_tx, wake_rx) = mpsc::channel();
    let worker = Arc::new(Mutex::new(WorkerSnapshot::default()));
    let shutting_down = Arc::new(AtomicBool::new(false));
    #[cfg(feature = "clouddrive")]
    let rss_db_path = anime_organizer::rss::db::default_db_path();
    let state = Arc::new(DaemonState {
        queue: queue.clone(),
        #[cfg(feature = "clouddrive")]
        cloud,
        #[cfg(feature = "clouddrive")]
        rss_db_path: rss_db_path.clone(),
        started_at: Instant::now(),
        wake: wake_tx.clone(),
        worker: worker.clone(),
    });
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| AppError::MetadataFetchError(format!("创建异步运行时失败: {error}")))?;
    let worker_handle = worker::start(
        queue,
        wake_rx,
        worker,
        shutting_down.clone(),
        #[cfg(feature = "clouddrive")]
        rss_schedule::RssRuntime {
            cloud: state.cloud.clone(),
            rss_db_path: rss_db_path.clone(),
        },
    );
    let result = runtime.block_on(run_http(state));
    shutting_down.store(true, Ordering::Release);
    let _ = wake_tx.send(());
    worker_handle.join();
    result
}

async fn run_http(state: Arc<DaemonState>) -> Result<(), AppError> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:32145")
        .await
        .map_err(|error| AppError::MetadataFetchError(format!("绑定 daemon 地址失败: {error}")))?;
    println!("anime-organizer daemon listening at http://127.0.0.1:32145/");

    #[cfg(feature = "clouddrive")]
    rss_schedule::start_scheduler(
        state.queue.clone(),
        state.rss_db_path.clone(),
        state.wake.clone(),
    );

    let server = axum::serve(listener, api::router(state));
    server
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|error| AppError::MetadataFetchError(format!("daemon HTTP 服务失败: {error}")))
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

pub(crate) fn daemon_db_path() -> PathBuf {
    #[cfg(windows)]
    {
        let root = std::env::var_os("LOCALAPPDATA")
            .or_else(|| std::env::var_os("APPDATA"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        root.join("anime-organizer").join("daemon.db")
    }

    #[cfg(not(windows))]
    {
        let root = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .map(|home| home.join(".local").join("share"))
            })
            .unwrap_or_else(|| PathBuf::from("."));
        root.join("anime-organizer").join("daemon.db")
    }
}
