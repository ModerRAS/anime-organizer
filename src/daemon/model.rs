use crate::cli::OrganizeArgs;
use anime_organizer::OperationMode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum JobState {
    Queued,
    Running,
    Succeeded,
    Failed,
    Canceled,
}

impl JobState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Canceled => "canceled",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "queued" => Some(Self::Queued),
            "running" => Some(Self::Running),
            "succeeded" => Some(Self::Succeeded),
            "failed" => Some(Self::Failed),
            "canceled" => Some(Self::Canceled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum JobOrigin {
    Qbittorrent,
    Manual,
    Scheduled,
}

impl JobOrigin {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Qbittorrent => "qbittorrent",
            Self::Manual => "manual",
            Self::Scheduled => "scheduled",
        }
    }

    pub(crate) fn priority(self) -> i64 {
        match self {
            Self::Qbittorrent => 300,
            Self::Manual => 200,
            Self::Scheduled => 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "args", rename_all = "snake_case")]
pub(crate) enum JobSpec {
    Organize(OrganizeArgs),
    #[cfg(feature = "clouddrive")]
    RssPoll {
        subscription_id: i64,
    },
    #[cfg(feature = "clouddrive")]
    RssPollAll,
    #[cfg(feature = "clouddrive")]
    CloudAddOffline(CloudAddOfflineJobArgs),
    #[cfg(feature = "scraper")]
    Scrape(crate::cli::ScrapeArgs),
    #[cfg(feature = "scraper")]
    MatchAliases(crate::cli::MatchArgs),
    #[cfg(feature = "scraper")]
    BuildBangumiDb(crate::cli::BuildDbArgs),
    #[cfg(feature = "scraper")]
    ExtractAliases(crate::cli::ExtractAliasesArgs),
    #[cfg(feature = "scraper")]
    MergeAliases(crate::cli::MergeAliasesArgs),
    #[cfg(feature = "scraper")]
    ApplyMatches(crate::cli::ApplyMatchesArgs),
    #[cfg(feature = "scraper")]
    CreateAliasIssues(crate::cli::CreateAliasIssuesArgs),
    #[cfg(feature = "torrent-scraper")]
    TorrentScrape(crate::cli::TorrentScrapeArgs),
}

#[cfg(feature = "clouddrive")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CloudAddOfflineJobArgs {
    pub(crate) connection_id: i64,
    pub(crate) url: String,
    pub(crate) target: String,
}

impl JobSpec {
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::Organize(_) => "organize",
            #[cfg(feature = "clouddrive")]
            Self::RssPoll { .. } => "rss_poll",
            #[cfg(feature = "clouddrive")]
            Self::RssPollAll => "rss_poll_all",
            #[cfg(feature = "clouddrive")]
            Self::CloudAddOffline(_) => "cloud_add_offline",
            #[cfg(feature = "scraper")]
            Self::Scrape(_) => "scrape",
            #[cfg(feature = "scraper")]
            Self::MatchAliases(_) => "match_aliases",
            #[cfg(feature = "scraper")]
            Self::BuildBangumiDb(_) => "build_bangumi_db",
            #[cfg(feature = "scraper")]
            Self::ExtractAliases(_) => "extract_aliases",
            #[cfg(feature = "scraper")]
            Self::MergeAliases(_) => "merge_aliases",
            #[cfg(feature = "scraper")]
            Self::ApplyMatches(_) => "apply_matches",
            #[cfg(feature = "scraper")]
            Self::CreateAliasIssues(_) => "create_alias_issues",
            #[cfg(feature = "torrent-scraper")]
            Self::TorrentScrape(_) => "torrent_scrape",
        }
    }

    pub(crate) fn resource_key(&self) -> Option<String> {
        #[cfg(feature = "clouddrive")]
        match self {
            Self::RssPoll { subscription_id } => Some(format!("rss:{subscription_id}")),
            Self::RssPollAll => Some("rss:all".to_string()),
            _ => None,
        }
        #[cfg(not(feature = "clouddrive"))]
        {
            None
        }
    }

    pub(crate) fn is_registered(&self) -> bool {
        match self {
            Self::Organize(_) => true,
            #[cfg(feature = "clouddrive")]
            Self::RssPoll { .. } | Self::RssPollAll | Self::CloudAddOffline(_) => true,
            #[cfg(feature = "scraper")]
            Self::Scrape(_) | Self::MatchAliases(_) => true,
            #[cfg(feature = "scraper")]
            Self::BuildBangumiDb(_)
            | Self::ExtractAliases(_)
            | Self::MergeAliases(_)
            | Self::ApplyMatches(_) => true,
            #[cfg(feature = "scraper")]
            Self::CreateAliasIssues(_) => crate::commands::gh_available(),
            #[cfg(feature = "torrent-scraper")]
            Self::TorrentScrape(_) => true,
        }
    }

    pub(crate) fn validate(
        &self,
        confirmed: bool,
        origin: JobOrigin,
        idempotency_key: Option<&str>,
    ) -> Result<(), String> {
        #[allow(unreachable_patterns)]
        match self {
            Self::Organize(args) => {
                if args.source.is_none() {
                    return Err("source is required".to_string());
                }
                if args.target.is_none() {
                    return Err("target is required".to_string());
                }
                if args.rebuild_library_index && !args.writes_library_index() {
                    return Err("rebuild_library_index requires library_index or mlip".to_string());
                }
                #[cfg(not(feature = "anifilebert"))]
                if args.filename_parser == crate::cli::FilenameParserMode::Anifilebert {
                    return Err(
                        "filename_parser anifilebert requires the anifilebert feature".to_string(),
                    );
                }
                if (args.mode == OperationMode::Move || args.rebuild_library_index) && !confirmed {
                    return Err("this organize operation requires confirmed=true".to_string());
                }
                if origin == JobOrigin::Qbittorrent
                    && idempotency_key.is_none_or(|key| !key.starts_with("qbittorrent:"))
                {
                    return Err(
                        "qBittorrent jobs require a qbittorrent:<info-hash> idempotency key"
                            .to_string(),
                    );
                }
                Ok(())
            }
            #[cfg(feature = "clouddrive")]
            Self::CloudAddOffline(args) => {
                if args.connection_id <= 0 {
                    return Err("connection_id must be positive".to_string());
                }
                let url = args.url.trim();
                if url.is_empty() || url.len() > 16 * 1024 {
                    return Err("url must contain 1-16384 characters".to_string());
                }
                if url != args.url {
                    return Err("url must not contain surrounding whitespace".to_string());
                }
                let target = args.target.trim();
                if target != args.target {
                    return Err("target must not contain surrounding whitespace".to_string());
                }
                if !target.starts_with('/') || target.len() > 4096 {
                    return Err(
                        "target must be an absolute CloudDrive path up to 4096 bytes".to_string(),
                    );
                }
                Ok(())
            }
            #[cfg(feature = "scraper")]
            Self::MergeAliases(_) | Self::ApplyMatches(_) if !confirmed => {
                Err("this alias mutation requires confirmed=true".to_string())
            }
            #[cfg(feature = "scraper")]
            Self::CreateAliasIssues(_) if !confirmed => {
                Err("issue creation requires confirmed=true".to_string())
            }
            _ if !self.is_registered() => {
                Err(format!("job type '{}' is not available", self.kind()))
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EnqueueRequest {
    #[serde(default)]
    pub(crate) idempotency_key: Option<String>,
    #[serde(default = "default_origin")]
    pub(crate) origin: JobOrigin,
    #[serde(default)]
    pub(crate) confirmed: bool,
    pub(crate) job: JobSpec,
}

fn default_origin() -> JobOrigin {
    JobOrigin::Manual
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JobResult {
    pub(crate) summary: String,
    #[serde(default)]
    pub(crate) data: Value,
    #[serde(default)]
    pub(crate) artifacts: Vec<Value>,
}

#[derive(Debug, Clone)]
pub(crate) struct StoredJob {
    pub(crate) id: i64,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) origin: JobOrigin,
    pub(crate) kind: String,
    pub(crate) resource_key: Option<String>,
    pub(crate) request_json: String,
    pub(crate) state: JobState,
    pub(crate) priority: i64,
    pub(crate) attempts: i64,
    pub(crate) progress_current: Option<i64>,
    pub(crate) progress_total: Option<i64>,
    pub(crate) progress_message: Option<String>,
    pub(crate) result_json: Option<String>,
    pub(crate) error: Option<String>,
    pub(crate) created_at: String,
    pub(crate) started_at: Option<String>,
    pub(crate) finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct JobView {
    pub(crate) id: i64,
    pub(crate) idempotency_key: Option<String>,
    pub(crate) origin: JobOrigin,
    pub(crate) kind: String,
    pub(crate) resource_key: Option<String>,
    pub(crate) request: Value,
    pub(crate) state: JobState,
    pub(crate) priority: i64,
    pub(crate) attempts: i64,
    pub(crate) progress_current: Option<i64>,
    pub(crate) progress_total: Option<i64>,
    pub(crate) progress_message: Option<String>,
    pub(crate) result: Option<Value>,
    pub(crate) error: Option<String>,
    pub(crate) created_at: String,
    pub(crate) started_at: Option<String>,
    pub(crate) finished_at: Option<String>,
}

impl From<StoredJob> for JobView {
    fn from(job: StoredJob) -> Self {
        let mut request = serde_json::from_str(&job.request_json).unwrap_or(Value::Null);
        if job.kind == "cloud_add_offline" {
            if let Some(url) = request.pointer_mut("/args/url") {
                *url = Value::String("[redacted]".to_string());
            }
        }
        redact_secrets(&mut request);
        let result = job
            .result_json
            .as_deref()
            .and_then(|json| serde_json::from_str(json).ok())
            .map(|mut value| {
                redact_secrets(&mut value);
                value
            });
        Self {
            id: job.id,
            idempotency_key: job.idempotency_key,
            origin: job.origin,
            kind: job.kind,
            resource_key: job.resource_key,
            request,
            state: job.state,
            priority: job.priority,
            attempts: job.attempts,
            progress_current: job.progress_current,
            progress_total: job.progress_total,
            progress_message: job.progress_message,
            result,
            error: job.error,
            created_at: format_job_timestamp(job.created_at),
            started_at: job.started_at.map(format_job_timestamp),
            finished_at: job.finished_at.map(format_job_timestamp),
        }
    }
}

fn format_job_timestamp(value: String) -> String {
    value
        .parse::<i64>()
        .ok()
        .and_then(|timestamp| OffsetDateTime::from_unix_timestamp(timestamp).ok())
        .and_then(|timestamp| timestamp.format(&Rfc3339).ok())
        .unwrap_or(value)
}

fn redact_secrets(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for (key, value) in object.iter_mut() {
                let key = key.to_ascii_lowercase();
                if key == "url" {
                    *value = match value.as_str() {
                        Some(url) => Value::String(redact_url(url)),
                        None => Value::String("[redacted]".to_string()),
                    };
                } else if key.contains("token")
                    || key.contains("password")
                    || key.ends_with("_pass")
                    || key.ends_with("_key")
                {
                    *value = Value::String("[redacted]".to_string());
                } else {
                    redact_secrets(value);
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(redact_secrets),
        _ => {}
    }
}

fn redact_url(value: &str) -> String {
    let Ok(mut url) = url::Url::parse(value) else {
        return "[redacted]".to_string();
    };
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_fragment(None);
    if url.query().is_some() {
        let retained = url
            .query_pairs()
            .filter(|(key, _)| !is_sensitive_url_parameter(key))
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect::<Vec<_>>();
        url.query_pairs_mut().clear().extend_pairs(retained);
    }
    url.to_string()
}

fn is_sensitive_url_parameter(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("token")
        || key.contains("apikey")
        || key.contains("api-key")
        || key.contains("password")
        || key.contains("passkey")
        || key.contains("secret")
        || key.contains("signature")
        || key.contains("credential")
        || key == "auth"
        || key.ends_with("_auth")
        || key == "sig"
        || key.ends_with("_sig")
        || key == "key"
        || key.ends_with("_key")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{FilenameParserMode, OrganizeArgs};
    use anime_organizer::OperationMode;
    use std::path::PathBuf;

    fn organize() -> OrganizeArgs {
        OrganizeArgs {
            source: Some(PathBuf::from("source")),
            target: Some(PathBuf::from("target")),
            mode: OperationMode::Copy,
            fallback_on_link_failure: None,
            dry_run: false,
            include_ext: None,
            verbose: true,
            scrape_metadata: false,
            tmdb_api_key: Some("secret".to_string()),
            alias_file: None,
            no_images: false,
            no_episode_metadata: false,
            force_overwrite: false,
            bangumi_cache: None,
            metadata_source: None,
            season_mode: false,
            library_index: false,
            mlip: false,
            rebuild_library_index: false,
            probe_runtime: false,
            filename_parser: FilenameParserMode::Rules,
        }
    }

    #[test]
    fn organize_round_trips_and_redacts_secrets() {
        let request = EnqueueRequest {
            idempotency_key: Some("qbittorrent:ABC".to_string()),
            origin: JobOrigin::Qbittorrent,
            confirmed: false,
            job: JobSpec::Organize(organize()),
        };
        let json = serde_json::to_string(&request).unwrap();
        let decoded: EnqueueRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.job.kind(), "organize");
        let stored = StoredJob {
            id: 1,
            idempotency_key: decoded.idempotency_key,
            origin: decoded.origin,
            kind: decoded.job.kind().to_string(),
            resource_key: None,
            request_json: serde_json::to_string(&decoded.job).unwrap(),
            state: JobState::Queued,
            priority: 300,
            attempts: 0,
            progress_current: None,
            progress_total: None,
            progress_message: None,
            result_json: None,
            error: None,
            created_at: "0".to_string(),
            started_at: None,
            finished_at: None,
        };
        let response = serde_json::to_string(&JobView::from(stored)).unwrap();
        assert!(!response.contains("secret"));
        assert!(response.contains("[redacted]"));
        assert!(response.contains("1970-01-01T00:00:00Z"));
    }

    #[cfg(feature = "clouddrive")]
    #[test]
    fn cloud_offline_url_is_redacted_from_job_views() {
        let spec = JobSpec::CloudAddOffline(CloudAddOfflineJobArgs {
            connection_id: 1,
            url: "https://user:password@example.test/file?passkey=secret".to_string(),
            target: "/anime".to_string(),
        });
        let stored = StoredJob {
            id: 1,
            idempotency_key: None,
            origin: JobOrigin::Manual,
            kind: spec.kind().to_string(),
            resource_key: None,
            request_json: serde_json::to_string(&spec).unwrap(),
            state: JobState::Queued,
            priority: 200,
            attempts: 0,
            progress_current: None,
            progress_total: None,
            progress_message: None,
            result_json: None,
            error: None,
            created_at: "0".to_string(),
            started_at: None,
            finished_at: None,
        };

        let response = serde_json::to_string(&JobView::from(stored)).unwrap();
        assert!(!response.contains("passkey"));
        assert!(!response.contains("password"));
        assert!(!response.contains("secret"));
        assert!(!response.contains("example.test/file"));
        assert!(response.contains("[redacted]"));
    }

    #[test]
    fn public_urls_remain_visible_in_job_views() {
        let mut value = serde_json::json!({
            "url": "https://example.test/public?id=42&apikey=secret",
            "password": "secret"
        });
        redact_secrets(&mut value);
        assert_eq!(value["url"], "https://example.test/public?id=42");
        assert_eq!(value["password"], "[redacted]");
    }

    #[test]
    fn qbittorrent_fixture_is_a_complete_organize_request() {
        let request: EnqueueRequest = serde_json::from_str(include_str!(
            "../../tests/fixtures/qbittorrent-job-request.json"
        ))
        .unwrap();

        assert_eq!(
            request.idempotency_key.as_deref(),
            Some("qbittorrent:0123456789abcdef")
        );
        assert_eq!(request.origin, JobOrigin::Qbittorrent);
        assert!(!request.confirmed);
        assert!(request
            .job
            .validate(
                request.confirmed,
                request.origin,
                request.idempotency_key.as_deref()
            )
            .is_ok());

        #[allow(irrefutable_let_patterns)]
        let JobSpec::Organize(args) = request.job
        else {
            panic!("expected organize job");
        };
        assert_eq!(
            args.source.unwrap().to_string_lossy(),
            r"C:\Downloads\Ani\[ANi] Example - 01.mkv"
        );
        assert_eq!(args.target.unwrap().to_string_lossy(), "S:\\动漫");
        assert_eq!(args.mode, OperationMode::Copy);
        assert!(args.mlip);
        assert!(args.verbose);
    }

    #[cfg(feature = "scraper")]
    #[test]
    fn alias_mutations_require_confirmation_without_touching_database() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("original.db");
        rusqlite::Connection::open(&database)
            .unwrap()
            .execute_batch("CREATE TABLE aliases (subject_id INTEGER, alias TEXT);")
            .unwrap();
        let request = EnqueueRequest {
            idempotency_key: None,
            origin: JobOrigin::Manual,
            confirmed: false,
            job: JobSpec::MergeAliases(crate::cli::MergeAliasesArgs {
                input: directory.path().join("aliases.json"),
                target: Some(database.clone()),
            }),
        };
        assert!(request
            .job
            .validate(request.confirmed, request.origin, None)
            .is_err());
        let count: i64 = rusqlite::Connection::open(database)
            .unwrap()
            .query_row("SELECT COUNT(*) FROM aliases", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[cfg(feature = "scraper")]
    #[test]
    fn scraper_fixture_jobs_are_typed_and_registered() {
        let scrape = EnqueueRequest {
            idempotency_key: None,
            origin: JobOrigin::Manual,
            confirmed: false,
            job: JobSpec::Scrape(crate::cli::ScrapeArgs {
                days: 7,
                format: crate::cli::ScrapeOutputFormat::Json,
                tmdb_api_key: None,
            }),
        };
        let match_job = EnqueueRequest {
            idempotency_key: None,
            origin: JobOrigin::Manual,
            confirmed: false,
            job: JobSpec::MatchAliases(crate::cli::MatchArgs {
                input: std::path::PathBuf::from("tests/fixtures/scraped-anime.json"),
                format: crate::cli::MatchOutputFormat::Github,
            }),
        };
        assert!(scrape.job.is_registered());
        assert!(match_job.job.is_registered());
        assert!(JobSpec::BuildBangumiDb(crate::cli::BuildDbArgs::default()).is_registered());
        assert!(JobSpec::ExtractAliases(crate::cli::ExtractAliasesArgs::default()).is_registered());
        assert!(JobSpec::MergeAliases(crate::cli::MergeAliasesArgs::default()).is_registered());
        assert!(JobSpec::ApplyMatches(crate::cli::ApplyMatchesArgs::default()).is_registered());
        assert_eq!(scrape.job.kind(), "scrape");
        assert_eq!(match_job.job.kind(), "match_aliases");
    }

    #[test]
    fn omitted_origin_defaults_to_manual() {
        let value = serde_json::json!({
            "job": {
                "type": "organize",
                "args": {"source": "source", "target": "target"}
            }
        });
        let request: EnqueueRequest = serde_json::from_value(value).unwrap();
        assert_eq!(request.origin, JobOrigin::Manual);
        #[allow(irrefutable_let_patterns)]
        let JobSpec::Organize(args) = request.job
        else {
            panic!("expected organize job");
        };
        assert_eq!(args.mode, OperationMode::Link);
        assert_eq!(args.filename_parser, FilenameParserMode::Rules);
    }

    fn assert_round_trip(spec: JobSpec) {
        let json = serde_json::to_vec(&spec).unwrap();
        let decoded: JobSpec = serde_json::from_slice(&json).unwrap();
        assert_eq!(decoded.kind(), spec.kind());
    }

    #[test]
    fn every_compiled_job_variant_round_trips() {
        assert_round_trip(JobSpec::Organize(organize()));
        #[cfg(feature = "clouddrive")]
        {
            assert_round_trip(JobSpec::RssPoll { subscription_id: 1 });
            assert_round_trip(JobSpec::RssPollAll);
            let cloud_job = JobSpec::CloudAddOffline(CloudAddOfflineJobArgs {
                connection_id: 1,
                url: "magnet:?xt=test".to_string(),
                target: "/anime".to_string(),
            });
            assert!(cloud_job.is_registered());
            assert!(cloud_job.validate(false, JobOrigin::Manual, None).is_ok());
            assert_round_trip(cloud_job);
            let invalid = JobSpec::CloudAddOffline(CloudAddOfflineJobArgs {
                connection_id: 1,
                url: " magnet:?xt=secret ".to_string(),
                target: " /anime ".to_string(),
            });
            assert!(invalid.validate(false, JobOrigin::Manual, None).is_err());
        }
        #[cfg(feature = "scraper")]
        {
            assert_round_trip(JobSpec::Scrape(crate::cli::ScrapeArgs::default()));
            assert_round_trip(JobSpec::MatchAliases(crate::cli::MatchArgs::default()));
            assert_round_trip(JobSpec::BuildBangumiDb(crate::cli::BuildDbArgs::default()));
            assert_round_trip(JobSpec::ExtractAliases(
                crate::cli::ExtractAliasesArgs::default(),
            ));
            assert_round_trip(JobSpec::MergeAliases(
                crate::cli::MergeAliasesArgs::default(),
            ));
            assert_round_trip(JobSpec::ApplyMatches(
                crate::cli::ApplyMatchesArgs::default(),
            ));
            assert_round_trip(JobSpec::CreateAliasIssues(
                crate::cli::CreateAliasIssuesArgs::default(),
            ));
        }
        #[cfg(feature = "torrent-scraper")]
        assert_round_trip(JobSpec::TorrentScrape(
            crate::cli::TorrentScrapeArgs::default(),
        ));
    }
}
