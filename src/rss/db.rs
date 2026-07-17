//! 数据库操作模块
//!
//! 管理 RSS 订阅记录和已处理项的 SQLite 数据库操作。

use crate::error::{AppError, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// 订阅记录
#[derive(Debug, Clone, Serialize)]
pub struct Subscription {
    pub id: i64,
    pub url: String,
    pub filter_regex: Option<String>,
    pub target_folder: String,
    pub interval_secs: i64,
    pub enabled: bool,
    pub last_checked_at: Option<String>,
    /// Application-validated reference to a connection in daemon.db.
    pub connection_id: Option<i64>,
}

/// A previously processed RSS item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProcessedItem {
    pub id: i64,
    pub subscription_id: i64,
    pub item_hash: String,
    pub title: Option<String>,
    pub processed_at: Option<String>,
}

/// A CloudDrive download submitted for an RSS item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DownloadTask {
    pub id: i64,
    pub subscription_id: i64,
    pub item_hash: String,
    pub cloud_name: Option<String>,
    pub status: Option<String>,
    pub added_at: Option<String>,
    pub completed_at: Option<String>,
}

/// RSS 数据库结构体
///
/// 封装 RSS 订阅状态存储的 SQLite 连接。
#[derive(Debug)]
pub struct RssDatabase {
    conn: Connection,
}

impl RssDatabase {
    /// 创建或打开 RSS 数据库
    ///
    /// # Arguments
    /// * `db_path` - 数据库文件路径
    ///
    /// # Returns
    /// * `Result<Self>` - 成功返回 RssDatabase 实例
    pub fn new(db_path: &Path) -> Result<Self> {
        // Create parent directories if they don't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::MetadataFetchError(format!("创建数据库目录失败: {e}")))?;
        }

        // Open or create the database
        let conn = Connection::open(db_path)
            .map_err(|e| AppError::MetadataFetchError(format!("打开数据库失败: {e}")))?;

        let db = Self { conn };
        db.init_schema()?;

        Ok(db)
    }

    /// 初始化数据库 schema
    ///
    /// 创建所有必要的表：subscriptions, processed_items, download_tasks
    fn init_schema(&self) -> Result<()> {
        self.conn
            .execute_batch(
                r#"
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS subscriptions (
                    id INTEGER PRIMARY KEY,
                    url TEXT NOT NULL UNIQUE,
                    filter_regex TEXT,
                    target_folder TEXT NOT NULL,
                    interval_secs INTEGER DEFAULT 300,
                    enabled BOOLEAN DEFAULT 1,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    last_checked_at TIMESTAMP,
                    connection_id INTEGER
                );

                CREATE TABLE IF NOT EXISTS processed_items (
                    id INTEGER PRIMARY KEY,
                    subscription_id INTEGER NOT NULL,
                    item_hash TEXT NOT NULL,
                    title TEXT,
                    processed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (subscription_id) REFERENCES subscriptions(id),
                    UNIQUE(subscription_id, item_hash)
                );

                CREATE TABLE IF NOT EXISTS download_tasks (
                    id INTEGER PRIMARY KEY,
                    subscription_id INTEGER NOT NULL,
                    item_hash TEXT NOT NULL,
                    cloud_name TEXT DEFAULT '115',
                    status TEXT DEFAULT 'pending',
                    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    completed_at TIMESTAMP,
                    FOREIGN KEY (subscription_id) REFERENCES subscriptions(id)
                );
                "#,
            )
            .map_err(|e| AppError::MetadataFetchError(format!("创建表失败: {e}")))?;

        // Databases created before Task 10 do not have these columns. SQLite
        // has no IF NOT EXISTS form for ADD COLUMN, so inspect the table first.
        let mut columns = Vec::new();
        let mut statement = self
            .conn
            .prepare("PRAGMA table_info(subscriptions)")
            .map_err(|e| AppError::MetadataFetchError(format!("读取订阅表结构失败: {e}")))?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| AppError::MetadataFetchError(format!("读取订阅列失败: {e}")))?;
        for row in rows {
            columns.push(
                row.map_err(|e| AppError::MetadataFetchError(format!("读取订阅列失败: {e}")))?,
            );
        }
        drop(statement);
        if !columns.iter().any(|column| column == "last_checked_at") {
            self.conn
                .execute_batch("ALTER TABLE subscriptions ADD COLUMN last_checked_at TIMESTAMP")
                .map_err(|e| {
                    AppError::MetadataFetchError(format!("迁移 last_checked_at 失败: {e}"))
                })?;
        }
        if !columns.iter().any(|column| column == "connection_id") {
            self.conn
                .execute_batch("ALTER TABLE subscriptions ADD COLUMN connection_id INTEGER")
                .map_err(|e| {
                    AppError::MetadataFetchError(format!("迁移 connection_id 失败: {e}"))
                })?;
        }
        self.conn
            .execute_batch("PRAGMA user_version = 2")
            .map_err(|e| AppError::MetadataFetchError(format!("写入 RSS schema 版本失败: {e}")))?;

        Ok(())
    }

    /// 添加一条订阅记录
    ///
    /// 如果 URL 已存在，则更新 filter_regex / target_folder / interval_secs。
    pub fn add_subscription(
        &self,
        url: &str,
        filter_regex: Option<&str>,
        target_folder: &str,
        interval_secs: i64,
    ) -> Result<i64> {
        self.add_subscription_with_connection(url, filter_regex, target_folder, interval_secs, None)
    }

    /// Add or update a subscription and associate it with a daemon connection.
    ///
    /// The connection ID is intentionally not a SQLite foreign key: the target
    /// row lives in daemon.db and is validated by the application layer.
    pub fn add_subscription_with_connection(
        &self,
        url: &str,
        filter_regex: Option<&str>,
        target_folder: &str,
        interval_secs: i64,
        connection_id: Option<i64>,
    ) -> Result<i64> {
        self.conn
            .execute(
                r#"INSERT INTO subscriptions
                       (url, filter_regex, target_folder, interval_secs, connection_id)
                   VALUES (?1, ?2, ?3, ?4, ?5)
                   ON CONFLICT(url) DO UPDATE SET
                       filter_regex = excluded.filter_regex,
                       target_folder = excluded.target_folder,
                       interval_secs = excluded.interval_secs,
                       connection_id = COALESCE(excluded.connection_id, subscriptions.connection_id)"#,
                params![url, filter_regex, target_folder, interval_secs, connection_id],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("添加订阅失败: {e}")))?;

        self.conn
            .query_row(
                "SELECT id FROM subscriptions WHERE url = ?1",
                params![url],
                |row| row.get(0),
            )
            .map_err(|e| AppError::MetadataFetchError(format!("查询订阅ID失败: {e}")))
    }

    /// 列出所有已启用的订阅
    pub fn list_subscriptions(&self) -> Result<Vec<Subscription>> {
        self.list_subscriptions_where(true)
    }

    /// 列出所有订阅（包括禁用的）
    pub fn list_all_subscriptions(&self) -> Result<Vec<Subscription>> {
        self.list_subscriptions_where(false)
    }

    fn list_subscriptions_where(&self, enabled_only: bool) -> Result<Vec<Subscription>> {
        let mut stmt = if enabled_only {
            self.conn.prepare(
                "SELECT id, url, filter_regex, target_folder, interval_secs, enabled, last_checked_at, connection_id FROM subscriptions WHERE enabled = 1 ORDER BY id",
            )
        } else {
            self.conn.prepare(
                "SELECT id, url, filter_regex, target_folder, interval_secs, enabled, last_checked_at, connection_id FROM subscriptions ORDER BY id",
            )
        }
        .map_err(|e| AppError::MetadataFetchError(format!("查询订阅失败: {e}")))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Subscription {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    filter_regex: row.get(2)?,
                    target_folder: row.get(3)?,
                    interval_secs: row.get(4)?,
                    enabled: row.get(5)?,
                    last_checked_at: row.get(6)?,
                    connection_id: row.get(7)?,
                })
            })
            .map_err(|e| AppError::MetadataFetchError(format!("遍历订阅失败: {e}")))?;
        rows.map(|row| {
            row.map_err(|e| AppError::MetadataFetchError(format!("读取订阅行失败: {e}")))
        })
        .collect()
    }

    /// Return enabled subscriptions whose interval has elapsed.
    pub fn list_due_subscriptions(&self) -> Result<Vec<Subscription>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, url, filter_regex, target_folder, interval_secs, enabled, last_checked_at, connection_id FROM subscriptions WHERE enabled = 1 AND (last_checked_at IS NULL OR datetime(last_checked_at, '+' || interval_secs || ' seconds') <= CURRENT_TIMESTAMP) ORDER BY id",
            )
            .map_err(|e| AppError::MetadataFetchError(format!("查询到期订阅失败: {e}")))?;
        let rows = statement
            .query_map([], |row| {
                Ok(Subscription {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    filter_regex: row.get(2)?,
                    target_folder: row.get(3)?,
                    interval_secs: row.get(4)?,
                    enabled: row.get(5)?,
                    last_checked_at: row.get(6)?,
                    connection_id: row.get(7)?,
                })
            })
            .map_err(|e| AppError::MetadataFetchError(format!("遍历到期订阅失败: {e}")))?;
        rows.map(|row| {
            row.map_err(|e| AppError::MetadataFetchError(format!("读取到期订阅失败: {e}")))
        })
        .collect()
    }

    /// Get one subscription by its RSS database ID.
    pub fn get_subscription(&self, id: i64) -> Result<Option<Subscription>> {
        self.conn
            .query_row(
                "SELECT id, url, filter_regex, target_folder, interval_secs, enabled, last_checked_at, connection_id FROM subscriptions WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Subscription {
                        id: row.get(0)?,
                        url: row.get(1)?,
                        filter_regex: row.get(2)?,
                        target_folder: row.get(3)?,
                        interval_secs: row.get(4)?,
                        enabled: row.get(5)?,
                        last_checked_at: row.get(6)?,
                        connection_id: row.get(7)?,
                    })
                },
            )
            .optional()
            .map_err(|e| AppError::MetadataFetchError(format!("查询订阅失败: {e}")))
    }

    /// Update the editable fields of a subscription without resetting its state.
    pub fn update_subscription(
        &self,
        id: i64,
        url: &str,
        filter_regex: Option<&str>,
        target_folder: &str,
        interval_secs: i64,
        connection_id: Option<i64>,
    ) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE subscriptions SET url = ?1, filter_regex = ?2, target_folder = ?3, interval_secs = ?4, connection_id = ?5 WHERE id = ?6",
                params![url, filter_regex, target_folder, interval_secs, connection_id, id],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("更新订阅失败: {e}")))?;
        if changed == 1 {
            Ok(())
        } else {
            Err(AppError::MetadataFetchError(format!("订阅不存在: {id}")))
        }
    }

    /// Delete a subscription and its RSS-local history.
    pub fn delete_subscription(&self, id: i64) -> Result<()> {
        let transaction = self
            .conn
            .unchecked_transaction()
            .map_err(|e| AppError::MetadataFetchError(format!("删除订阅失败: {e}")))?;
        transaction
            .execute(
                "DELETE FROM processed_items WHERE subscription_id = ?1",
                params![id],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("删除已处理项失败: {e}")))?;
        transaction
            .execute(
                "DELETE FROM download_tasks WHERE subscription_id = ?1",
                params![id],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("删除下载任务失败: {e}")))?;
        let changed = transaction
            .execute("DELETE FROM subscriptions WHERE id = ?1", params![id])
            .map_err(|e| AppError::MetadataFetchError(format!("删除订阅失败: {e}")))?;
        if changed != 1 {
            return Err(AppError::MetadataFetchError(format!("订阅不存在: {id}")));
        }
        transaction
            .commit()
            .map_err(|e| AppError::MetadataFetchError(format!("提交删除订阅失败: {e}")))
    }

    /// Set the enabled state of a subscription.
    pub fn set_subscription_enabled(&self, id: i64, enabled: bool) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE subscriptions SET enabled = ?1 WHERE id = ?2",
                params![enabled, id],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("更新订阅状态失败: {e}")))?;
        if changed == 1 {
            Ok(())
        } else {
            Err(AppError::MetadataFetchError(format!("订阅不存在: {id}")))
        }
    }

    pub fn enable_subscription(&self, id: i64) -> Result<()> {
        self.set_subscription_enabled(id, true)
    }

    pub fn disable_subscription(&self, id: i64) -> Result<()> {
        self.set_subscription_enabled(id, false)
    }

    /// Set or clear the application-validated daemon connection reference.
    pub fn set_subscription_connection(&self, id: i64, connection_id: Option<i64>) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE subscriptions SET connection_id = ?1 WHERE id = ?2",
                params![connection_id, id],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("更新订阅连接失败: {e}")))?;
        if changed == 1 {
            Ok(())
        } else {
            Err(AppError::MetadataFetchError(format!("订阅不存在: {id}")))
        }
    }

    /// Set the last successful poll time. A NULL value resets the schedule.
    pub fn set_last_checked_at(&self, id: i64, last_checked_at: Option<&str>) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE subscriptions SET last_checked_at = ?1 WHERE id = ?2",
                params![last_checked_at, id],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("更新 RSS 检查时间失败: {e}")))?;
        if changed == 1 {
            Ok(())
        } else {
            Err(AppError::MetadataFetchError(format!("订阅不存在: {id}")))
        }
    }

    /// Record the current SQLite UTC timestamp after a successful poll.
    pub fn mark_subscription_checked(&self, id: i64) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE subscriptions SET last_checked_at = CURRENT_TIMESTAMP WHERE id = ?1",
                params![id],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("更新 RSS 检查时间失败: {e}")))?;
        if changed == 1 {
            Ok(())
        } else {
            Err(AppError::MetadataFetchError(format!("订阅不存在: {id}")))
        }
    }

    /// Alias used by schedulers that explicitly describe this as an update.
    pub fn update_last_checked_at(&self, id: i64, last_checked_at: Option<&str>) -> Result<()> {
        self.set_last_checked_at(id, last_checked_at)
    }

    /// 检查某项是否已处理过
    pub fn is_item_processed(&self, subscription_id: i64, item_hash: &str) -> Result<bool> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM processed_items WHERE subscription_id = ?1 AND item_hash = ?2",
                params![subscription_id, item_hash],
                |row| row.get(0),
            )
            .map_err(|e| AppError::MetadataFetchError(format!("查询已处理项失败: {e}")))?;
        Ok(count > 0)
    }

    /// 标记某项为已处理
    pub fn mark_item_processed(
        &self,
        subscription_id: i64,
        item_hash: &str,
        title: &str,
    ) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO processed_items (subscription_id, item_hash, title) VALUES (?1, ?2, ?3)",
                params![subscription_id, item_hash, title],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("标记已处理失败: {e}")))?;
        Ok(())
    }

    /// List processed RSS history for one subscription.
    pub fn list_processed_items(&self, subscription_id: i64) -> Result<Vec<ProcessedItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, subscription_id, item_hash, title, processed_at FROM processed_items WHERE subscription_id = ?1 ORDER BY id DESC",
            )
            .map_err(|e| AppError::MetadataFetchError(format!("查询已处理项失败: {e}")))?;
        let rows = statement
            .query_map(params![subscription_id], |row| {
                Ok(ProcessedItem {
                    id: row.get(0)?,
                    subscription_id: row.get(1)?,
                    item_hash: row.get(2)?,
                    title: row.get(3)?,
                    processed_at: row.get(4)?,
                })
            })
            .map_err(|e| AppError::MetadataFetchError(format!("遍历已处理项失败: {e}")))?;
        rows.map(|row| {
            row.map_err(|e| AppError::MetadataFetchError(format!("读取已处理项失败: {e}")))
        })
        .collect()
    }

    /// 保存下载任务记录
    pub fn save_download_task(&self, subscription_id: i64, item_hash: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO download_tasks (subscription_id, item_hash) VALUES (?1, ?2)",
                params![subscription_id, item_hash],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("保存下载任务失败: {e}")))?;
        Ok(())
    }

    /// List RSS download history, optionally filtered by status.
    pub fn list_download_tasks(
        &self,
        subscription_id: i64,
        status: Option<&str>,
    ) -> Result<Vec<DownloadTask>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, subscription_id, item_hash, cloud_name, status, added_at, completed_at FROM download_tasks WHERE subscription_id = ?1 AND (?2 IS NULL OR status = ?2) ORDER BY id DESC",
            )
            .map_err(|e| AppError::MetadataFetchError(format!("查询下载任务失败: {e}")))?;
        let rows = statement
            .query_map(params![subscription_id, status], |row| {
                Ok(DownloadTask {
                    id: row.get(0)?,
                    subscription_id: row.get(1)?,
                    item_hash: row.get(2)?,
                    cloud_name: row.get(3)?,
                    status: row.get(4)?,
                    added_at: row.get(5)?,
                    completed_at: row.get(6)?,
                })
            })
            .map_err(|e| AppError::MetadataFetchError(format!("遍历下载任务失败: {e}")))?;
        rows.map(|row| {
            row.map_err(|e| AppError::MetadataFetchError(format!("读取下载任务失败: {e}")))
        })
        .collect()
    }

    /// 更新下载任务状态
    pub fn update_download_status(
        &self,
        subscription_id: i64,
        item_hash: &str,
        status: &str,
    ) -> Result<()> {
        let changed = if status == "completed" {
            self.conn
                .execute(
                    "UPDATE download_tasks SET status = ?1, completed_at = CURRENT_TIMESTAMP WHERE subscription_id = ?2 AND item_hash = ?3",
                    params![status, subscription_id, item_hash],
                )
                .map_err(|e| AppError::MetadataFetchError(format!("更新下载状态失败: {e}")))?
        } else {
            self.conn
                .execute(
                    "UPDATE download_tasks SET status = ?1 WHERE subscription_id = ?2 AND item_hash = ?3",
                    params![status, subscription_id, item_hash],
                )
                .map_err(|e| AppError::MetadataFetchError(format!("更新下载状态失败: {e}")))?
        };
        if changed == 0 {
            return Err(AppError::MetadataFetchError("下载任务不存在".to_string()));
        }
        Ok(())
    }
}

/// 计算 RSS item 的 hash（用作去重标识）
///
/// 优先使用 guid，否则使用 title 的 SHA1 值。
pub fn compute_item_hash(guid: Option<&str>, title: &str) -> String {
    if let Some(guid) = guid {
        if !guid.is_empty() {
            return guid.to_string();
        }
    }
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(title.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{b:02x}")).collect()
}

/// 返回默认的 RSS 数据库路径
///
/// # Returns
/// * Windows: `%LOCALAPPDATA%\anime-organizer\rss.db`
/// * Other: `~/.local/share/anime-organizer/rss.db`
#[cfg(windows)]
pub fn default_db_path() -> PathBuf {
    let local_app_data = std::env::var("LOCALAPPDATA")
        .unwrap_or_else(|_| std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string()));
    PathBuf::from(local_app_data)
        .join("anime-organizer")
        .join("rss.db")
}

#[cfg(not(windows))]
pub fn default_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("anime-organizer")
        .join("rss.db")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_rss_db_init() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test_rss.db");

        let db = RssDatabase::new(&db_path).expect("Failed to create database");
        let conn = &db.conn;

        let user_version: i32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(user_version, 2);

        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(subscriptions)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .map(|row| row.unwrap())
            .collect();
        assert!(columns.iter().any(|column| column == "last_checked_at"));
        assert!(columns.iter().any(|column| column == "connection_id"));

        let subs_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='subscriptions'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(subs_count, 1);

        let proc_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='processed_items'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(proc_count, 1);

        let dl_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='download_tasks'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(dl_count, 1);
    }

    #[test]
    fn test_add_and_list_subscriptions() {
        let temp_dir = tempdir().unwrap();
        let db = RssDatabase::new(&temp_dir.path().join("test.db")).unwrap();

        let id = db
            .add_subscription(
                "https://example.com/rss.xml",
                Some(r"\[ANi\]"),
                "/downloads",
                300,
            )
            .unwrap();
        assert!(id > 0);

        let subs = db.list_subscriptions().unwrap();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].url, "https://example.com/rss.xml");
        assert_eq!(subs[0].filter_regex, Some(r"\[ANi\]".to_string()));
        assert_eq!(subs[0].target_folder, "/downloads");
    }

    #[test]
    fn test_add_subscription_upsert() {
        let temp_dir = tempdir().unwrap();
        let db = RssDatabase::new(&temp_dir.path().join("test.db")).unwrap();

        db.add_subscription("https://example.com/rss.xml", None, "/old", 300)
            .unwrap();
        db.add_subscription("https://example.com/rss.xml", Some("filter"), "/new", 600)
            .unwrap();

        let subs = db.list_all_subscriptions().unwrap();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].target_folder, "/new");
        assert_eq!(subs[0].interval_secs, 600);
    }

    #[test]
    fn test_legacy_schema_migrates_without_a_cross_database_foreign_key() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("legacy.db");
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "CREATE TABLE subscriptions (id INTEGER PRIMARY KEY, url TEXT NOT NULL UNIQUE, filter_regex TEXT, target_folder TEXT NOT NULL, interval_secs INTEGER DEFAULT 300, enabled BOOLEAN DEFAULT 1, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP); INSERT INTO subscriptions (url, target_folder) VALUES ('https://legacy.example/rss.xml', '/legacy');",
            )
            .unwrap();
        }

        let db = RssDatabase::new(&db_path).unwrap();
        let migrated = db.get_subscription(1).unwrap().unwrap();
        assert_eq!(migrated.url, "https://legacy.example/rss.xml");
        assert!(migrated.last_checked_at.is_none());
        assert!(migrated.connection_id.is_none());
        let foreign_keys = db
            .conn
            .prepare("PRAGMA foreign_key_list(subscriptions)")
            .unwrap()
            .query_map([], |_| Ok(()))
            .unwrap()
            .count();
        assert_eq!(foreign_keys, 0);
    }

    #[test]
    fn test_subscription_crud_and_state() {
        let temp_dir = tempdir().unwrap();
        let db = RssDatabase::new(&temp_dir.path().join("test.db")).unwrap();

        let id = db
            .add_subscription_with_connection(
                "https://example.com/rss.xml",
                Some("episode"),
                "/anime",
                600,
                Some(17),
            )
            .unwrap();
        let subscription = db.get_subscription(id).unwrap().unwrap();
        assert_eq!(subscription.connection_id, Some(17));
        assert!(subscription.enabled);
        assert!(subscription.last_checked_at.is_none());

        db.update_subscription(
            id,
            "https://example.com/updated.xml",
            None,
            "/new",
            900,
            Some(18),
        )
        .unwrap();
        db.disable_subscription(id).unwrap();
        assert!(db.list_subscriptions().unwrap().is_empty());
        db.enable_subscription(id).unwrap();
        db.set_subscription_connection(id, None).unwrap();
        db.set_last_checked_at(id, Some("2026-01-01 00:00:00"))
            .unwrap();
        let subscription = db.get_subscription(id).unwrap().unwrap();
        assert_eq!(subscription.url, "https://example.com/updated.xml");
        assert_eq!(
            subscription.last_checked_at.as_deref(),
            Some("2026-01-01 00:00:00")
        );
        assert_eq!(subscription.connection_id, None);

        db.mark_subscription_checked(id).unwrap();
        assert!(db
            .get_subscription(id)
            .unwrap()
            .unwrap()
            .last_checked_at
            .is_some());
    }

    #[test]
    fn test_history_queries_and_delete() {
        let temp_dir = tempdir().unwrap();
        let db = RssDatabase::new(&temp_dir.path().join("test.db")).unwrap();
        let id = db
            .add_subscription("https://example.com/rss.xml", None, "/dl", 300)
            .unwrap();

        db.mark_item_processed(id, "hash123", "Episode 1").unwrap();
        db.mark_item_processed(id, "hash123", "Episode 1").unwrap();
        let processed = db.list_processed_items(id).unwrap();
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].item_hash, "hash123");
        assert_eq!(processed[0].title.as_deref(), Some("Episode 1"));

        db.save_download_task(id, "hash123").unwrap();
        db.update_download_status(id, "hash123", "completed")
            .unwrap();
        let tasks = db.list_download_tasks(id, None).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status.as_deref(), Some("completed"));
        assert!(tasks[0].completed_at.is_some());
        assert_eq!(
            db.list_download_tasks(id, Some("pending")).unwrap().len(),
            0
        );

        db.delete_subscription(id).unwrap();
        assert!(db.get_subscription(id).unwrap().is_none());
        assert!(db.list_processed_items(id).unwrap().is_empty());
        assert!(db.list_download_tasks(id, None).unwrap().is_empty());
    }

    #[test]
    fn test_processed_items() {
        let temp_dir = tempdir().unwrap();
        let db = RssDatabase::new(&temp_dir.path().join("test.db")).unwrap();

        let sub_id = db
            .add_subscription("https://example.com/rss.xml", None, "/dl", 300)
            .unwrap();

        assert!(!db.is_item_processed(sub_id, "hash123").unwrap());

        db.mark_item_processed(sub_id, "hash123", "Test Title")
            .unwrap();
        assert!(db.is_item_processed(sub_id, "hash123").unwrap());

        // Duplicate insert should be ignored
        db.mark_item_processed(sub_id, "hash123", "Test Title")
            .unwrap();
    }

    #[test]
    fn test_download_tasks() {
        let temp_dir = tempdir().unwrap();
        let db = RssDatabase::new(&temp_dir.path().join("test.db")).unwrap();

        let sub_id = db
            .add_subscription("https://example.com/rss.xml", None, "/dl", 300)
            .unwrap();

        db.save_download_task(sub_id, "hash456").unwrap();
        db.update_download_status(sub_id, "hash456", "completed")
            .unwrap();
    }

    #[test]
    fn test_compute_item_hash_with_guid() {
        assert_eq!(compute_item_hash(Some("guid-123"), "title"), "guid-123");
    }

    #[test]
    fn test_compute_item_hash_without_guid() {
        let hash = compute_item_hash(None, "test title");
        assert_eq!(hash.len(), 40); // SHA1 hex
    }

    #[test]
    fn test_default_db_path() {
        let path = default_db_path();
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("anime-organizer"));
        assert!(path_str.contains("rss.db"));
    }
}
