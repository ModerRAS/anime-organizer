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
    updated_at TEXT NOT NULL
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

        Ok(Self {
            name,
            url,
            token,
            username,
            password,
        })
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| (!value.is_empty()).then_some(value))
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CloudConnectionView {
    pub(crate) id: i64,
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
            let client = CloudDriveClient::new(&connection.url, connection.token.clone())?;
            Ok(Box::new(client) as Box<dyn CloudDriveClientTrait>)
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
                .map_err(|error| CloudError::Database(error.to_string()))
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
        let now = now_string();
        self.with_connection(|connection| {
            connection
                .execute(
                    "INSERT INTO cloud_connections (name, url, token, username, password, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                    params![request.name, request.url, request.token, request.username, request.password, now],
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
        let changed = self.with_connection(|connection| {
            connection
                .execute(
                    "UPDATE cloud_connections SET name = ?1, url = ?2, token = COALESCE(?3, token), username = COALESCE(?4, username), password = COALESCE(?5, password), updated_at = ?6 WHERE id = ?7",
                    params![request.name, request.url, request.token, request.username, request.password, now_string(), id],
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
    })
}

const SELECT_COLUMNS: &str = "SELECT id, name, url, token, username, password, created_at, updated_at FROM cloud_connections";

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
            name: "primary".to_string(),
            url: "http://localhost:19798".to_string(),
            token: Some("secret-token".to_string()),
            username: Some("user".to_string()),
            password: Some("secret-password".to_string()),
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
        assert!(!json.contains("secret-password"));
        assert!(view.has_token && view.has_username && view.has_password);

        let updated = repository
            .update(
                created.id,
                &CloudConnectionRequest {
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
    fn request_normalization_rejects_unbounded_or_non_http_values() {
        let invalid = CloudConnectionRequest {
            name: " ".to_string(),
            url: "ftp://localhost".to_string(),
            token: None,
            username: None,
            password: None,
        };
        assert!(matches!(invalid.normalize(), Err(CloudError::Invalid(_))));
    }
}
