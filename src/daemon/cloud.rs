use anime_organizer::error::Result;
use anime_organizer::rss::client::{CloudDriveClient, CloudDriveClientTrait};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

pub(crate) const MAX_FOLDER_PATH_BYTES: usize = 4096;
pub(crate) const MAX_FOLDER_ENTRIES: usize = 1000;
pub(crate) const CLOUD_OPERATION_TIMEOUT_SECS: u64 = 30;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS cloud_connections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    token TEXT,
    username TEXT,
    password TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'clouddrive'
);
"#;

#[derive(Debug, Error)]
pub(crate) enum CloudError {
    #[error("cloud database error: {0}")]
    Database(String),
    #[error("cloud connection {0} was not found")]
    NotFound(i64),
    #[error("invalid cloud connection: {0}")]
    Invalid(String),
    #[error("cloud operation failed")]
    Operation,
}

pub(crate) type CloudResult<T> = std::result::Result<T, CloudError>;

#[derive(Debug, Clone)]
pub(crate) struct StoredCloudConnection {
    pub(crate) id: i64,
    pub(crate) kind: String,
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) token: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CloudConnectionRequest {
    #[serde(default = "default_connection_kind")]
    pub(crate) kind: String,
    pub(crate) name: String,
    pub(crate) url: String,
    #[serde(default)]
    pub(crate) token: Option<String>,
    #[serde(default)]
    pub(crate) username: Option<String>,
    #[serde(default)]
    pub(crate) password: Option<String>,
}

impl CloudConnectionRequest {
    pub(crate) fn normalize(self) -> CloudResult<Self> {
        let kind = self.kind.trim().to_ascii_lowercase();
        if !matches!(kind.as_str(), "clouddrive" | "webdav") {
            return Err(CloudError::Invalid(
                "kind must be clouddrive or webdav".to_string(),
            ));
        }
        let name = self.name.trim().to_string();
        if name.is_empty() || name.len() > 200 {
            return Err(CloudError::Invalid(
                "name must contain 1-200 characters".to_string(),
            ));
        }

        let url = self.url.trim().to_string();
        let parsed = url::Url::parse(&url)
            .map_err(|_| CloudError::Invalid("url must be a valid HTTP(S) URL".to_string()))?;
        if !matches!(parsed.scheme(), "http" | "https")
            || parsed.host_str().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
        {
            return Err(CloudError::Invalid(
                "url must be an HTTP(S) endpoint without embedded credentials".to_string(),
            ));
        }
        if url.len() > 2048 {
            return Err(CloudError::Invalid("url is too long".to_string()));
        }

        let token = non_empty(self.token);
        let username = non_empty(self.username);
        let password = non_empty(self.password);
        if username.is_some() != password.is_some() {
            return Err(CloudError::Invalid(
                "username and password must be provided together".to_string(),
            ));
        }
        if token.is_some() && username.is_some() {
            return Err(CloudError::Invalid(
                "choose either token or username/password authentication".to_string(),
            ));
        }
        if token.as_ref().is_some_and(|value| value.len() > 16 * 1024)
            || username.as_ref().is_some_and(|value| value.len() > 1024)
            || password
                .as_ref()
                .is_some_and(|value| value.len() > 16 * 1024)
        {
            return Err(CloudError::Invalid(
                "cloud credentials are too long".to_string(),
            ));
        }

        if kind == "webdav" && token.is_some() {
            return Err(CloudError::Invalid(
                "WebDAV connections use optional username/password, not token".to_string(),
            ));
        }

        Ok(Self {
            kind,
            name,
            url,
            token,
            username,
            password,
        })
    }
}

fn default_connection_kind() -> String {
    "clouddrive".to_string()
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| (!value.is_empty()).then_some(value))
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CloudConnectionView {
    pub(crate) id: i64,
    pub(crate) kind: String,
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) has_token: bool,
    pub(crate) has_username: bool,
    pub(crate) has_password: bool,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

impl From<&StoredCloudConnection> for CloudConnectionView {
    fn from(connection: &StoredCloudConnection) -> Self {
        Self {
            id: connection.id,
            kind: connection.kind.clone(),
            name: connection.name.clone(),
            url: connection.url.clone(),
            has_token: connection.token.is_some(),
            has_username: connection.username.is_some(),
            has_password: connection.password.is_some(),
            created_at: connection.created_at.clone(),
            updated_at: connection.updated_at.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CloudFolderEntry {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) size: i64,
    pub(crate) is_directory: bool,
}

impl From<anime_organizer::rss::client::proto::CloudDriveFile> for CloudFolderEntry {
    fn from(file: anime_organizer::rss::client::proto::CloudDriveFile) -> Self {
        Self {
            id: file.id,
            name: file.name,
            path: file.full_path_name,
            size: file.size,
            is_directory: file.is_directory,
        }
    }
}

pub(crate) type CloudDriveClientFactory =
    Arc<dyn Fn(&StoredCloudConnection) -> Result<Box<dyn CloudDriveClientTrait>> + Send + Sync>;

#[derive(Clone)]
pub(crate) struct CloudDriveState {
    pub(crate) repository: CloudConnectionRepository,
    pub(crate) client_factory: CloudDriveClientFactory,
}

impl CloudDriveState {
    pub(crate) fn new(path: &Path) -> CloudResult<Self> {
        let factory: CloudDriveClientFactory = Arc::new(|connection| {
            let client: Box<dyn CloudDriveClientTrait> = match connection.kind.as_str() {
                "clouddrive" => Box::new(CloudDriveClient::new(
                    &connection.url,
                    connection.token.clone(),
                )?),
                "webdav" => Box::new(anime_organizer::rss::webdav::WebDavClient::new(
                    &connection.url,
                    connection.username.clone(),
                    connection.password.clone(),
                )?),
                kind => {
                    return Err(anime_organizer::error::AppError::MetadataFetchError(
                        format!("Unsupported storage connection kind: {kind}"),
                    ))
                }
            };
            Ok(client)
        });
        Ok(Self {
            repository: CloudConnectionRepository::new(path)?,
            client_factory: factory,
        })
    }

    #[cfg(test)]
    pub(crate) fn with_factory(
        repository: CloudConnectionRepository,
        client_factory: CloudDriveClientFactory,
    ) -> Self {
        Self {
            repository,
            client_factory,
        }
    }

    pub(crate) async fn authenticated_client(
        &self,
        connection: &StoredCloudConnection,
    ) -> CloudResult<Box<dyn CloudDriveClientTrait>> {
        let mut client = (self.client_factory)(connection).map_err(|_| CloudError::Operation)?;
        if connection.kind == "webdav" {
            return Ok(client);
        }
        if let Some(username) = connection.username.as_deref() {
            let password = connection
                .password
                .as_deref()
                .ok_or_else(|| CloudError::Invalid("username requires a password".to_string()))?;
            let token = tokio::time::timeout(
                std::time::Duration::from_secs(CLOUD_OPERATION_TIMEOUT_SECS),
                client.login(username, password),
            )
            .await
            .map_err(|_| CloudError::Operation)?
            .map_err(|_| CloudError::Operation)?;
            self.repository.set_token(connection.id, &token)?;
        } else if connection.token.is_none() {
            return Err(CloudError::Invalid(
                "connection requires a token or username/password".to_string(),
            ));
        }
        Ok(client)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CloudConnectionRepository {
    path: PathBuf,
}

impl CloudConnectionRepository {
    pub(crate) fn new(path: &Path) -> CloudResult<Self> {
        let repository = Self {
            path: path.to_path_buf(),
        };
        repository.with_connection(|connection| {
            connection
                .execute_batch(SCHEMA)
                .map_err(|error| CloudError::Database(error.to_string()))?;
            let columns = connection
                .prepare("PRAGMA table_info(cloud_connections)")
                .and_then(|mut statement| {
                    statement
                        .query_map([], |row| row.get::<_, String>(1))?
                        .collect::<rusqlite::Result<Vec<_>>>()
                })
                .map_err(|error| CloudError::Database(error.to_string()))?;
            if !columns.iter().any(|column| column == "kind") {
                connection
                    .execute(
                        "ALTER TABLE cloud_connections ADD COLUMN kind TEXT NOT NULL DEFAULT 'clouddrive'",
                        [],
                    )
                    .map_err(|error| CloudError::Database(error.to_string()))?;
            }
            Ok(())
        })?;
        Ok(repository)
    }

    fn with_connection<T>(
        &self,
        operation: impl FnOnce(&Connection) -> CloudResult<T>,
    ) -> CloudResult<T> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| CloudError::Database(error.to_string()))?;
        }
        let connection = Connection::open(&self.path)
            .map_err(|error| CloudError::Database(error.to_string()))?;
        connection
            .execute_batch("PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;")
            .map_err(|error| CloudError::Database(error.to_string()))?;
        operation(&connection)
    }

    pub(crate) fn list(&self) -> CloudResult<Vec<StoredCloudConnection>> {
        self.with_connection(|connection| {
            let mut statement = connection
                .prepare(SELECT_COLUMNS)
                .map_err(|error| CloudError::Database(error.to_string()))?;
            let rows = statement
                .query_map([], row_to_connection)
                .map_err(|error| CloudError::Database(error.to_string()))?;
            rows.map(|row| row.map_err(|error| CloudError::Database(error.to_string())))
                .collect()
        })
    }

    pub(crate) fn get(&self, id: i64) -> CloudResult<StoredCloudConnection> {
        self.with_connection(|connection| {
            connection
                .query_row(
                    &format!("{SELECT_COLUMNS} WHERE id = ?1"),
                    params![id],
                    row_to_connection,
                )
                .optional()
                .map_err(|error| CloudError::Database(error.to_string()))?
                .ok_or(CloudError::NotFound(id))
        })
    }

    pub(crate) fn create(
        &self,
        request: &CloudConnectionRequest,
    ) -> CloudResult<StoredCloudConnection> {
        if request.kind == "clouddrive" && request.token.is_none() && request.username.is_none() {
            return Err(CloudError::Invalid(
                "a token or username/password login is required".to_string(),
            ));
        }
        let now = now_string();
        self.with_connection(|connection| {
            connection
                .execute(
                    "INSERT INTO cloud_connections (kind, name, url, token, username, password, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
                    params![request.kind, request.name, request.url, request.token, request.username, request.password, now],
                )
                .map_err(|error| CloudError::Database(error.to_string()))?;
            let id = connection.last_insert_rowid();
            connection
                .query_row(
                    &format!("{SELECT_COLUMNS} WHERE id = ?1"),
                    params![id],
                    row_to_connection,
                )
                .map_err(|error| CloudError::Database(error.to_string()))
        })
    }

    pub(crate) fn update(
        &self,
        id: i64,
        request: &CloudConnectionRequest,
    ) -> CloudResult<StoredCloudConnection> {
        let existing = self.get(id)?;
        if request.kind == "clouddrive"
            && request.token.is_none()
            && request.username.is_none()
            && existing.token.is_none()
            && existing.username.is_none()
        {
            return Err(CloudError::Invalid(
                "a token or username/password login is required".to_string(),
            ));
        }
        let changed = self.with_connection(|connection| {
            connection
                .execute(
                    "UPDATE cloud_connections SET kind = ?1, name = ?2, url = ?3, token = CASE WHEN ?4 IS NOT NULL THEN ?4 WHEN ?5 IS NOT NULL OR ?1 = 'webdav' THEN NULL ELSE token END, username = CASE WHEN ?4 IS NOT NULL THEN NULL WHEN ?5 IS NOT NULL THEN ?5 ELSE username END, password = CASE WHEN ?4 IS NOT NULL THEN NULL WHEN ?5 IS NOT NULL THEN ?6 ELSE password END, updated_at = ?7 WHERE id = ?8",
                    params![request.kind, request.name, request.url, request.token, request.username, request.password, now_string(), id],
                )
                .map_err(|error| CloudError::Database(error.to_string()))
        })?;
        if changed != 1 {
            return Err(CloudError::NotFound(id));
        }
        self.get(id)
    }

    pub(crate) fn set_token(&self, id: i64, token: &str) -> CloudResult<()> {
        let changed = self.with_connection(|connection| {
            connection
                .execute(
                    "UPDATE cloud_connections SET token = ?1, updated_at = ?2 WHERE id = ?3",
                    params![token, now_string(), id],
                )
                .map_err(|error| CloudError::Database(error.to_string()))
        })?;
        if changed == 1 {
            Ok(())
        } else {
            Err(CloudError::NotFound(id))
        }
    }

    pub(crate) fn delete(&self, id: i64) -> CloudResult<()> {
        let changed = self.with_connection(|connection| {
            connection
                .execute("DELETE FROM cloud_connections WHERE id = ?1", params![id])
                .map_err(|error| CloudError::Database(error.to_string()))
        })?;
        if changed == 1 {
            Ok(())
        } else {
            Err(CloudError::NotFound(id))
        }
    }
}

fn row_to_connection(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredCloudConnection> {
    Ok(StoredCloudConnection {
        id: row.get(0)?,
        name: row.get(1)?,
        url: row.get(2)?,
        token: row.get(3)?,
        username: row.get(4)?,
        password: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        kind: row.get(8)?,
    })
}

const SELECT_COLUMNS: &str = "SELECT id, name, url, token, username, password, created_at, updated_at, kind FROM cloud_connections";

fn now_string() -> String {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(value) => value.as_secs().to_string(),
        Err(_) => "0".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn request() -> CloudConnectionRequest {
        CloudConnectionRequest {
            kind: default_connection_kind(),
            name: "primary".to_string(),
            url: "http://localhost:19798".to_string(),
            token: Some("secret-token".to_string()),
            username: None,
            password: None,
        }
    }

    #[test]
    fn connection_crud_and_secret_redaction() {
        let directory = tempdir().unwrap();
        let repository =
            CloudConnectionRepository::new(&directory.path().join("daemon.db")).unwrap();
        let created = repository.create(&request()).unwrap();
        let view = CloudConnectionView::from(&created);
        let json = serde_json::to_string(&view).unwrap();
        assert!(!json.contains("secret-token"));
        assert!(view.has_token);
        assert!(!view.has_username && !view.has_password);

        let updated = repository
            .update(
                created.id,
                &CloudConnectionRequest {
                    kind: default_connection_kind(),
                    name: "renamed".to_string(),
                    url: "https://localhost:19798".to_string(),
                    token: None,
                    username: None,
                    password: None,
                },
            )
            .unwrap();
        assert_eq!(updated.name, "renamed");
        assert_eq!(updated.token.as_deref(), Some("secret-token"));
        assert!(repository.list().unwrap().len() == 1);
        repository.delete(created.id).unwrap();
        assert!(matches!(
            repository.get(created.id),
            Err(CloudError::NotFound(_))
        ));
    }

    #[test]
    fn credential_updates_replace_the_previous_mode() {
        let directory = tempdir().unwrap();
        let repository =
            CloudConnectionRepository::new(&directory.path().join("daemon.db")).unwrap();
        let created = repository.create(&request()).unwrap();
        let login = repository
            .update(
                created.id,
                &CloudConnectionRequest {
                    kind: default_connection_kind(),
                    name: "primary".to_string(),
                    url: "https://localhost".to_string(),
                    token: None,
                    username: Some("user".to_string()),
                    password: Some("password".to_string()),
                }
                .normalize()
                .unwrap(),
            )
            .unwrap();
        assert!(login.token.is_none());
        assert_eq!(login.username.as_deref(), Some("user"));

        let token = repository
            .update(
                created.id,
                &CloudConnectionRequest {
                    kind: default_connection_kind(),
                    name: "primary".to_string(),
                    url: "https://localhost".to_string(),
                    token: Some("replacement".to_string()),
                    username: None,
                    password: None,
                }
                .normalize()
                .unwrap(),
            )
            .unwrap();
        assert_eq!(token.token.as_deref(), Some("replacement"));
        assert!(token.username.is_none() && token.password.is_none());
    }

    #[test]
    fn request_normalization_rejects_unbounded_or_non_http_values() {
        let invalid = CloudConnectionRequest {
            kind: default_connection_kind(),
            name: " ".to_string(),
            url: "ftp://localhost".to_string(),
            token: None,
            username: None,
            password: None,
        };
        assert!(matches!(invalid.normalize(), Err(CloudError::Invalid(_))));

        let incomplete_login = CloudConnectionRequest {
            kind: default_connection_kind(),
            name: "primary".to_string(),
            url: "https://localhost".to_string(),
            token: None,
            username: Some("user".to_string()),
            password: None,
        };
        assert!(matches!(
            incomplete_login.normalize(),
            Err(CloudError::Invalid(_))
        ));
    }

    #[test]
    fn legacy_schema_migrates_to_clouddrive_kind() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("daemon.db");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE cloud_connections (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    url TEXT NOT NULL,
                    token TEXT,
                    username TEXT,
                    password TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );
                INSERT INTO cloud_connections
                    (name, url, token, created_at, updated_at)
                VALUES ('legacy', 'https://localhost', 'token', '1', '1');",
            )
            .unwrap();
        drop(connection);

        let repository = CloudConnectionRepository::new(&path).unwrap();
        assert_eq!(repository.get(1).unwrap().kind, "clouddrive");
    }

    #[test]
    fn anonymous_webdav_connection_is_allowed_and_redacted() {
        let directory = tempdir().unwrap();
        let repository =
            CloudConnectionRepository::new(&directory.path().join("daemon.db")).unwrap();
        let connection = repository
            .create(
                &CloudConnectionRequest {
                    kind: "webdav".to_string(),
                    name: "dav".to_string(),
                    url: "https://localhost/dav".to_string(),
                    token: None,
                    username: None,
                    password: None,
                }
                .normalize()
                .unwrap(),
            )
            .unwrap();
        let view = CloudConnectionView::from(&connection);
        assert_eq!(view.kind, "webdav");
        assert!(!view.has_token && !view.has_username && !view.has_password);
    }

    #[test]
    fn connection_creation_requires_credentials() {
        let directory = tempdir().unwrap();
        let repository =
            CloudConnectionRepository::new(&directory.path().join("daemon.db")).unwrap();
        let request = CloudConnectionRequest {
            kind: default_connection_kind(),
            name: "primary".to_string(),
            url: "https://localhost".to_string(),
            token: None,
            username: None,
            password: None,
        }
        .normalize()
        .unwrap();
        assert!(matches!(
            repository.create(&request),
            Err(CloudError::Invalid(_))
        ));
    }
}
