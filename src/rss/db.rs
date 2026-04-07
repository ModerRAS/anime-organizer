//! 数据库操作模块
//!
//! 管理 RSS 订阅记录和已处理项的 SQLite 数据库操作。

use crate::error::{AppError, Result};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

/// 订阅记录
#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: i64,
    pub url: String,
    pub filter_regex: Option<String>,
    pub target_folder: String,
    pub interval_secs: i64,
    pub enabled: bool,
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
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
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
        self.conn
            .execute(
                r#"INSERT INTO subscriptions (url, filter_regex, target_folder, interval_secs)
                   VALUES (?1, ?2, ?3, ?4)
                   ON CONFLICT(url) DO UPDATE SET
                       filter_regex = excluded.filter_regex,
                       target_folder = excluded.target_folder,
                       interval_secs = excluded.interval_secs"#,
                params![url, filter_regex, target_folder, interval_secs],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("添加订阅失败: {e}")))?;

        let id = self.conn.last_insert_rowid();
        // ON CONFLICT UPDATE 时 last_insert_rowid 为 0，需要手动查询
        if id == 0 {
            let actual_id: i64 = self
                .conn
                .query_row(
                    "SELECT id FROM subscriptions WHERE url = ?1",
                    params![url],
                    |row| row.get(0),
                )
                .map_err(|e| AppError::MetadataFetchError(format!("查询订阅ID失败: {e}")))?;
            return Ok(actual_id);
        }
        Ok(id)
    }

    /// 列出所有已启用的订阅
    pub fn list_subscriptions(&self) -> Result<Vec<Subscription>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, url, filter_regex, target_folder, interval_secs, enabled FROM subscriptions WHERE enabled = 1",
            )
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
                })
            })
            .map_err(|e| AppError::MetadataFetchError(format!("遍历订阅失败: {e}")))?;

        let mut subs = Vec::new();
        for row in rows {
            subs.push(
                row.map_err(|e| AppError::MetadataFetchError(format!("读取订阅行失败: {e}")))?,
            );
        }
        Ok(subs)
    }

    /// 列出所有订阅（包括禁用的）
    pub fn list_all_subscriptions(&self) -> Result<Vec<Subscription>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, url, filter_regex, target_folder, interval_secs, enabled FROM subscriptions",
            )
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
                })
            })
            .map_err(|e| AppError::MetadataFetchError(format!("遍历订阅失败: {e}")))?;

        let mut subs = Vec::new();
        for row in rows {
            subs.push(
                row.map_err(|e| AppError::MetadataFetchError(format!("读取订阅行失败: {e}")))?,
            );
        }
        Ok(subs)
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

    /// 更新下载任务状态
    pub fn update_download_status(
        &self,
        subscription_id: i64,
        item_hash: &str,
        status: &str,
    ) -> Result<()> {
        let completed_clause = if status == "completed" {
            ", completed_at = CURRENT_TIMESTAMP"
        } else {
            ""
        };
        let sql = format!(
            "UPDATE download_tasks SET status = ?1{completed_clause} WHERE subscription_id = ?2 AND item_hash = ?3"
        );
        self.conn
            .execute(&sql, params![status, subscription_id, item_hash])
            .map_err(|e| AppError::MetadataFetchError(format!("更新下载状态失败: {e}")))?;
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
