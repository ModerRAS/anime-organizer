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
    /// Whether completed CloudDrive downloads should be organized automatically.
    pub auto_organize: bool,
    /// CloudDrive remote destination for organized files.
    pub organize_target_folder: Option<String>,
    /// Optional organization destination connection. `None` reuses `connection_id`.
    pub organize_target_connection_id: Option<i64>,
    /// Whether automatic organization should create season directories.
    pub organize_season_mode: bool,
    /// Whether empty CloudDrive remote directories should be removed afterward.
    pub remove_empty_dirs: bool,
    /// Whether automatic organization should publish a remote MLIP library.db.
    pub remote_mlip: bool,
    /// Source selection for organization: `offline` correlates CloudDrive tasks;
    /// `original` scans the configured remote source directory directly.
    pub organize_mode: String,
    /// Seconds between CloudDrive offline-status checks for this subscription.
    pub organize_interval_secs: i64,
    /// Last time CloudDrive offline status was successfully reconciled.
    pub last_organize_checked_at: Option<String>,
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
    /// BitTorrent info hash reported by CloudDrive's OfflineFile record.
    pub info_hash: Option<String>,
    /// CloudDrive remote name reported by the corresponding OfflineFile record.
    pub remote_name: Option<String>,
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

fn subscription_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Subscription> {
    Ok(Subscription {
        id: row.get(0)?,
        url: row.get(1)?,
        filter_regex: row.get(2)?,
        target_folder: row.get(3)?,
        interval_secs: row.get(4)?,
        enabled: row.get(5)?,
        last_checked_at: row.get(6)?,
        connection_id: row.get(7)?,
        auto_organize: row.get(8)?,
        organize_target_folder: row.get(9)?,
        organize_season_mode: row.get(10)?,
        remove_empty_dirs: row.get(11)?,
        organize_interval_secs: row.get(12)?,
        last_organize_checked_at: row.get(13)?,
        remote_mlip: row.get(14)?,
        organize_mode: row.get(15)?,
        organize_target_connection_id: row.get(16)?,
    })
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
                    connection_id INTEGER,
                    auto_organize BOOLEAN NOT NULL DEFAULT 0,
                    organize_target_folder TEXT,
                    organize_season_mode BOOLEAN NOT NULL DEFAULT 1,
                    remove_empty_dirs BOOLEAN NOT NULL DEFAULT 1,
                    organize_interval_secs INTEGER NOT NULL DEFAULT 300,
                    last_organize_checked_at TIMESTAMP,
                    remote_mlip BOOLEAN NOT NULL DEFAULT 0,
                    organize_mode TEXT NOT NULL DEFAULT 'offline',
                    organize_target_connection_id INTEGER
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
                    info_hash TEXT,
                    remote_name TEXT,
                    status TEXT DEFAULT 'pending',
                    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    completed_at TIMESTAMP,
                    FOREIGN KEY (subscription_id) REFERENCES subscriptions(id)
                );
                "#,
            )
            .map_err(|e| AppError::MetadataFetchError(format!("创建表失败: {e}")))?;

        // SQLite has no IF NOT EXISTS form for ADD COLUMN, so inspect each
        // table before applying backwards-compatible migrations.
        let subscription_columns = self.table_columns("subscriptions")?;
        self.add_column_if_missing(
            "subscriptions",
            &subscription_columns,
            "last_checked_at",
            "TIMESTAMP",
        )?;
        self.add_column_if_missing(
            "subscriptions",
            &subscription_columns,
            "connection_id",
            "INTEGER",
        )?;
        self.add_column_if_missing(
            "subscriptions",
            &subscription_columns,
            "auto_organize",
            "BOOLEAN NOT NULL DEFAULT 0",
        )?;
        self.add_column_if_missing(
            "subscriptions",
            &subscription_columns,
            "organize_target_folder",
            "TEXT",
        )?;
        self.add_column_if_missing(
            "subscriptions",
            &subscription_columns,
            "organize_season_mode",
            "BOOLEAN NOT NULL DEFAULT 1",
        )?;
        self.add_column_if_missing(
            "subscriptions",
            &subscription_columns,
            "remove_empty_dirs",
            "BOOLEAN NOT NULL DEFAULT 1",
        )?;
        self.add_column_if_missing(
            "subscriptions",
            &subscription_columns,
            "organize_interval_secs",
            "INTEGER NOT NULL DEFAULT 300",
        )?;
        self.add_column_if_missing(
            "subscriptions",
            &subscription_columns,
            "last_organize_checked_at",
            "TIMESTAMP",
        )?;
        self.add_column_if_missing(
            "subscriptions",
            &subscription_columns,
            "remote_mlip",
            "BOOLEAN NOT NULL DEFAULT 0",
        )?;

        self.add_column_if_missing(
            "subscriptions",
            &subscription_columns,
            "organize_mode",
            "TEXT NOT NULL DEFAULT 'offline'",
        )?;
        self.add_column_if_missing(
            "subscriptions",
            &subscription_columns,
            "organize_target_connection_id",
            "INTEGER",
        )?;

        let download_task_columns = self.table_columns("download_tasks")?;
        self.add_column_if_missing(
            "download_tasks",
            &download_task_columns,
            "info_hash",
            "TEXT",
        )?;
        self.add_column_if_missing(
            "download_tasks",
            &download_task_columns,
            "remote_name",
            "TEXT",
        )?;
        self.conn
            .execute_batch(
                "CREATE INDEX IF NOT EXISTS idx_download_tasks_subscription_info_hash ON download_tasks(subscription_id, info_hash)",
            )
            .map_err(|e| AppError::MetadataFetchError(format!("创建下载任务索引失败: {e}")))?;
        self.conn
            .execute_batch("PRAGMA user_version = 7")
            .map_err(|e| AppError::MetadataFetchError(format!("写入 RSS schema 版本失败: {e}")))?;

        Ok(())
    }

    fn table_columns(&self, table_name: &str) -> Result<Vec<String>> {
        let mut statement = self
            .conn
            .prepare(&format!("PRAGMA table_info({table_name})"))
            .map_err(|e| {
                AppError::MetadataFetchError(format!("读取 {table_name} 表结构失败: {e}"))
            })?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| AppError::MetadataFetchError(format!("读取 {table_name} 列失败: {e}")))?;
        rows.map(|row| {
            row.map_err(|e| AppError::MetadataFetchError(format!("读取 {table_name} 列失败: {e}")))
        })
        .collect()
    }

    fn add_column_if_missing(
        &self,
        table_name: &str,
        columns: &[String],
        column_name: &str,
        definition: &str,
    ) -> Result<()> {
        if columns.iter().any(|column| column == column_name) {
            return Ok(());
        }

        self.conn
            .execute_batch(&format!(
                "ALTER TABLE {table_name} ADD COLUMN {column_name} {definition}"
            ))
            .map_err(|e| {
                AppError::MetadataFetchError(format!("迁移 {table_name}.{column_name} 失败: {e}"))
            })
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
                "SELECT id, url, filter_regex, target_folder, interval_secs, enabled, last_checked_at, connection_id, auto_organize, organize_target_folder, organize_season_mode, remove_empty_dirs, organize_interval_secs, last_organize_checked_at, remote_mlip, organize_mode, organize_target_connection_id FROM subscriptions WHERE enabled = 1 ORDER BY id",
            )
        } else {
            self.conn.prepare(
                "SELECT id, url, filter_regex, target_folder, interval_secs, enabled, last_checked_at, connection_id, auto_organize, organize_target_folder, organize_season_mode, remove_empty_dirs, organize_interval_secs, last_organize_checked_at, remote_mlip, organize_mode, organize_target_connection_id FROM subscriptions ORDER BY id",
            )
        }
        .map_err(|e| AppError::MetadataFetchError(format!("查询订阅失败: {e}")))?;
        let rows = stmt
            .query_map([], subscription_from_row)
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
                "SELECT id, url, filter_regex, target_folder, interval_secs, enabled, last_checked_at, connection_id, auto_organize, organize_target_folder, organize_season_mode, remove_empty_dirs, organize_interval_secs, last_organize_checked_at, remote_mlip, organize_mode, organize_target_connection_id FROM subscriptions WHERE enabled = 1 AND (last_checked_at IS NULL OR datetime(last_checked_at, '+' || interval_secs || ' seconds') <= CURRENT_TIMESTAMP) ORDER BY id",
            )
            .map_err(|e| AppError::MetadataFetchError(format!("查询到期订阅失败: {e}")))?;
        let rows = statement
            .query_map([], subscription_from_row)
            .map_err(|e| AppError::MetadataFetchError(format!("遍历到期订阅失败: {e}")))?;
        rows.map(|row| {
            row.map_err(|e| AppError::MetadataFetchError(format!("读取到期订阅失败: {e}")))
        })
        .collect()
    }

    /// Return auto-organize subscriptions that are due and have an unfinished,
    /// hash-correlated CloudDrive task. Legacy rows without a hash are excluded.
    pub fn list_due_organization_subscriptions(&self) -> Result<Vec<Subscription>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, url, filter_regex, target_folder, interval_secs, enabled, last_checked_at, connection_id, auto_organize, organize_target_folder, organize_season_mode, remove_empty_dirs, organize_interval_secs, last_organize_checked_at, remote_mlip, organize_mode, organize_target_connection_id FROM subscriptions WHERE enabled = 1 AND auto_organize = 1 AND connection_id IS NOT NULL AND organize_target_folder IS NOT NULL AND trim(organize_target_folder) != '' AND (last_organize_checked_at IS NULL OR datetime(last_organize_checked_at, '+' || organize_interval_secs || ' seconds') <= CURRENT_TIMESTAMP) AND (organize_mode = 'original' OR EXISTS (SELECT 1 FROM download_tasks WHERE download_tasks.subscription_id = subscriptions.id AND trim(COALESCE(info_hash, '')) != '' AND COALESCE(status, 'pending') != 'completed')) ORDER BY id",
            )
            .map_err(|e| AppError::MetadataFetchError(format!("查询到期整理订阅失败: {e}")))?;
        let rows = statement
            .query_map([], subscription_from_row)
            .map_err(|e| AppError::MetadataFetchError(format!("遍历到期整理订阅失败: {e}")))?;
        rows.map(|row| {
            row.map_err(|e| AppError::MetadataFetchError(format!("读取到期整理订阅失败: {e}")))
        })
        .collect()
    }

    /// Get one subscription by its RSS database ID.
    pub fn get_subscription(&self, id: i64) -> Result<Option<Subscription>> {
        self.conn
            .query_row(
                "SELECT id, url, filter_regex, target_folder, interval_secs, enabled, last_checked_at, connection_id, auto_organize, organize_target_folder, organize_season_mode, remove_empty_dirs, organize_interval_secs, last_organize_checked_at, remote_mlip, organize_mode, organize_target_connection_id FROM subscriptions WHERE id = ?1",
                params![id],
                subscription_from_row,
            )
            .optional()
            .map_err(|e| AppError::MetadataFetchError(format!("查询订阅失败: {e}")))
    }

    /// Return whether this subscription has a nonterminal task that can move
    /// remote data. Callers use this before changing its CloudDrive identity or
    /// destination settings.
    pub fn has_unfinished_correlated_download_tasks(&self, subscription_id: i64) -> Result<bool> {
        self.conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM download_tasks WHERE subscription_id = ?1 AND trim(COALESCE(info_hash, '')) != '' AND COALESCE(status, 'pending') != 'completed')",
                params![subscription_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|value| value != 0)
            .map_err(|error| {
                AppError::MetadataFetchError(format!("检查未完成下载任务失败: {error}"))
            })
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
        let existing = self
            .get_subscription(id)?
            .ok_or_else(|| AppError::MetadataFetchError(format!("订阅不存在: {id}")))?;
        if (existing.target_folder != target_folder || existing.connection_id != connection_id)
            && self.has_unfinished_correlated_download_tasks(id)?
        {
            return Err(AppError::MetadataFetchError(
                "存在未完成的关联下载任务，不能更改 CloudDrive 源或连接".to_string(),
            ));
        }
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

    /// Update the organization options without changing polling or connection state.
    ///
    /// `organize_target_folder` is stored as a CloudDrive remote path and is
    /// deliberately not checked against the local filesystem.
    pub fn update_subscription_organization_settings(
        &self,
        id: i64,
        auto_organize: bool,
        organize_target_folder: Option<&str>,
        organize_season_mode: bool,
        remove_empty_dirs: bool,
    ) -> Result<()> {
        let remote_mlip = self
            .get_subscription(id)?
            .is_some_and(|subscription| subscription.remote_mlip);
        let organize_mode = self.get_subscription(id)?.map_or_else(
            || "offline".to_string(),
            |subscription| subscription.organize_mode,
        );
        self.update_subscription_organization_settings_with_mlip_and_mode(
            id,
            auto_organize,
            organize_target_folder,
            organize_season_mode,
            remove_empty_dirs,
            remote_mlip,
            &organize_mode,
        )
    }

    pub fn update_subscription_organization_settings_with_mlip(
        &self,
        id: i64,
        auto_organize: bool,
        organize_target_folder: Option<&str>,
        organize_season_mode: bool,
        remove_empty_dirs: bool,
        remote_mlip: bool,
    ) -> Result<()> {
        let organize_mode = self.get_subscription(id)?.map_or_else(
            || "offline".to_string(),
            |subscription| subscription.organize_mode,
        );
        self.update_subscription_organization_settings_with_mlip_and_mode(
            id,
            auto_organize,
            organize_target_folder,
            organize_season_mode,
            remove_empty_dirs,
            remote_mlip,
            &organize_mode,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_subscription_organization_settings_with_mlip_and_mode(
        &self,
        id: i64,
        auto_organize: bool,
        organize_target_folder: Option<&str>,
        organize_season_mode: bool,
        remove_empty_dirs: bool,
        remote_mlip: bool,
        organize_mode: &str,
    ) -> Result<()> {
        if !matches!(organize_mode, "offline" | "original") {
            return Err(AppError::MetadataFetchError(format!(
                "无效的 RSS 整理模式: {organize_mode}"
            )));
        }
        let existing = self
            .get_subscription(id)?
            .ok_or_else(|| AppError::MetadataFetchError(format!("订阅不存在: {id}")))?;
        if existing.auto_organize
            && (existing.auto_organize != auto_organize
                || existing.organize_target_folder.as_deref() != organize_target_folder
                || existing.organize_season_mode != organize_season_mode
                || existing.remote_mlip != remote_mlip)
            && self.has_unfinished_correlated_download_tasks(id)?
        {
            return Err(AppError::MetadataFetchError(
                "存在未完成的关联下载任务，不能更改自动整理设置".to_string(),
            ));
        }
        let changed = self
            .conn
            .execute(
                "UPDATE subscriptions SET auto_organize = ?1, organize_target_folder = ?2, organize_season_mode = ?3, remove_empty_dirs = ?4, remote_mlip = ?5, organize_mode = ?6 WHERE id = ?7",
                params![
                    auto_organize,
                    organize_target_folder,
                    organize_season_mode,
                    remove_empty_dirs,
                    remote_mlip,
                    organize_mode,
                    id
                ],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("更新订阅整理设置失败: {e}")))?;
        if changed == 1 {
            Ok(())
        } else {
            Err(AppError::MetadataFetchError(format!("订阅不存在: {id}")))
        }
    }

    pub fn set_subscription_organize_target_connection(
        &self,
        id: i64,
        connection_id: Option<i64>,
    ) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE subscriptions SET organize_target_connection_id = ?1 WHERE id = ?2",
                params![connection_id, id],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("更新整理目标连接失败: {e}")))?;
        if changed == 1 {
            Ok(())
        } else {
            Err(AppError::MetadataFetchError(format!("订阅不存在: {id}")))
        }
    }

    /// Set the organization polling interval without changing feed polling.
    pub fn set_subscription_organize_interval(&self, id: i64, interval_secs: i64) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE subscriptions SET organize_interval_secs = ?1 WHERE id = ?2",
                params![interval_secs, id],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("更新整理间隔失败: {e}")))?;
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
        let existing = self
            .get_subscription(id)?
            .ok_or_else(|| AppError::MetadataFetchError(format!("订阅不存在: {id}")))?;
        if existing.connection_id != connection_id
            && self.has_unfinished_correlated_download_tasks(id)?
        {
            return Err(AppError::MetadataFetchError(
                "存在未完成的关联下载任务，不能更改 CloudDrive 连接".to_string(),
            ));
        }
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

    /// Record a successful CloudDrive offline-status reconciliation.
    pub fn mark_subscription_organize_checked(&self, id: i64) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE subscriptions SET last_organize_checked_at = CURRENT_TIMESTAMP WHERE id = ?1",
                params![id],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("更新整理检查时间失败: {e}")))?;
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

    /// Persist a successfully submitted RSS item and its optional v1 BTIH in
    /// one transaction, so a local failure cannot suppress later correlation
    /// after recording only `processed_items`.
    pub fn record_submitted_item(
        &self,
        subscription_id: i64,
        item_hash: &str,
        title: &str,
        info_hash: Option<&str>,
    ) -> Result<()> {
        let transaction = self
            .conn
            .unchecked_transaction()
            .map_err(|error| AppError::MetadataFetchError(format!("创建提交事务失败: {error}")))?;
        transaction
            .execute(
                "INSERT OR IGNORE INTO processed_items (subscription_id, item_hash, title) VALUES (?1, ?2, ?3)",
                params![subscription_id, item_hash, title],
            )
            .map_err(|error| AppError::MetadataFetchError(format!("标记已处理失败: {error}")))?;
        transaction
            .execute(
                "INSERT INTO download_tasks (subscription_id, item_hash, info_hash) VALUES (?1, ?2, ?3)",
                params![subscription_id, item_hash, info_hash],
            )
            .map_err(|error| AppError::MetadataFetchError(format!("保存下载任务失败: {error}")))?;
        transaction
            .commit()
            .map_err(|error| AppError::MetadataFetchError(format!("提交下载任务失败: {error}")))
    }

    /// Return whether a previously submitted item still lacks the v1 BTIH
    /// needed to correlate it with one CloudDrive offline task.
    pub fn download_task_needs_correlation(
        &self,
        subscription_id: i64,
        item_hash: &str,
    ) -> Result<bool> {
        self.conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM download_tasks WHERE subscription_id = ?1 AND item_hash = ?2 AND trim(COALESCE(info_hash, '')) = '')",
                params![subscription_id, item_hash],
                |row| row.get(0),
            )
            .map_err(|error| {
                AppError::MetadataFetchError(format!("查询下载关联状态失败: {error}"))
            })
    }

    /// Save a missing CloudDrive OfflineFile correlation for an RSS task.
    pub fn save_download_correlation(
        &self,
        subscription_id: i64,
        item_hash: &str,
        info_hash: &str,
        remote_name: Option<&str>,
    ) -> Result<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE download_tasks SET info_hash = ?1, remote_name = ?2 WHERE subscription_id = ?3 AND item_hash = ?4 AND trim(COALESCE(info_hash, '')) = ''",
                params![info_hash, remote_name, subscription_id, item_hash],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("保存下载关联失败: {e}")))?;
        if changed == 0 {
            return Err(AppError::MetadataFetchError(
                "下载任务不存在或已有下载关联".to_string(),
            ));
        }
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
                "SELECT id, subscription_id, item_hash, cloud_name, info_hash, remote_name, status, added_at, completed_at FROM download_tasks WHERE subscription_id = ?1 AND (?2 IS NULL OR status = ?2) ORDER BY id DESC",
            )
            .map_err(|e| AppError::MetadataFetchError(format!("查询下载任务失败: {e}")))?;
        let rows = statement
            .query_map(params![subscription_id, status], |row| {
                Ok(DownloadTask {
                    id: row.get(0)?,
                    subscription_id: row.get(1)?,
                    item_hash: row.get(2)?,
                    cloud_name: row.get(3)?,
                    info_hash: row.get(4)?,
                    remote_name: row.get(5)?,
                    status: row.get(6)?,
                    added_at: row.get(7)?,
                    completed_at: row.get(8)?,
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

    /// Reconcile one task with a matching CloudDrive OfflineFile. Completed
    /// tasks remain terminal until successful organization changes them.
    pub fn reconcile_download_task(
        &self,
        task_id: i64,
        subscription_id: i64,
        info_hash: &str,
        status: &str,
        remote_name: &str,
    ) -> Result<bool> {
        let changed = self
            .conn
            .execute(
                "UPDATE download_tasks SET remote_name = ?1, status = CASE WHEN status = 'completed' THEN status ELSE ?2 END WHERE id = ?3 AND subscription_id = ?4 AND info_hash = ?5",
                params![remote_name, status, task_id, subscription_id, info_hash],
            )
            .map_err(|e| AppError::MetadataFetchError(format!("同步下载任务失败: {e}")))?;
        Ok(changed != 0)
    }

    /// Mark one correlated task as completed after its remote organization has
    /// succeeded. The task primary key prevents same-subscription duplicate
    /// hashes from changing each other's state.
    pub fn complete_download_task(
        &self,
        task_id: i64,
        subscription_id: i64,
        info_hash: &str,
    ) -> Result<()> {
        let changed = self.conn.execute(
            "UPDATE download_tasks SET status = 'completed', completed_at = CURRENT_TIMESTAMP WHERE id = ?1 AND subscription_id = ?2 AND info_hash = ?3 AND status != 'completed'",
            params![task_id, subscription_id, info_hash],
        ).map_err(|e| AppError::MetadataFetchError(format!("完成下载任务失败: {e}")))?;
        if changed == 0 {
            return Err(AppError::MetadataFetchError(
                "下载任务不存在或已完成".to_string(),
            ));
        }
        Ok(())
    }

    /// Count unfinished historical rows that cannot be safely correlated to an
    /// OfflineFile because they have no torrent info hash.
    pub fn count_uncorrelated_download_tasks(&self, subscription_id: i64) -> Result<i64> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM download_tasks WHERE subscription_id = ?1 AND trim(COALESCE(info_hash, '')) = '' AND COALESCE(status, 'pending') != 'completed'",
            params![subscription_id],
            |row| row.get(0),
        ).map_err(|e| AppError::MetadataFetchError(format!("统计未关联下载任务失败: {e}")))
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
        assert_eq!(user_version, 7);

        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(subscriptions)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .map(|row| row.unwrap())
            .collect();
        assert!(columns.iter().any(|column| column == "last_checked_at"));
        assert!(columns.iter().any(|column| column == "connection_id"));
        assert!(columns.iter().any(|column| column == "auto_organize"));
        assert!(columns
            .iter()
            .any(|column| column == "organize_target_folder"));
        assert!(columns
            .iter()
            .any(|column| column == "organize_season_mode"));
        assert!(columns.iter().any(|column| column == "remove_empty_dirs"));
        assert!(columns
            .iter()
            .any(|column| column == "organize_interval_secs"));
        assert!(columns
            .iter()
            .any(|column| column == "last_organize_checked_at"));
        assert!(columns.iter().any(|column| column == "remote_mlip"));
        assert!(columns.iter().any(|column| column == "organize_mode"));
        assert!(columns
            .iter()
            .any(|column| column == "organize_target_connection_id"));

        let download_task_columns: Vec<String> = conn
            .prepare("PRAGMA table_info(download_tasks)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .map(|row| row.unwrap())
            .collect();
        assert!(download_task_columns
            .iter()
            .any(|column| column == "info_hash"));
        assert!(download_task_columns
            .iter()
            .any(|column| column == "remote_name"));

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
        assert!(!subs[0].auto_organize);
        assert!(subs[0].organize_target_folder.is_none());
        assert!(subs[0].organize_target_connection_id.is_none());
        assert!(subs[0].organize_season_mode);
        assert!(subs[0].remove_empty_dirs);
        assert!(!subs[0].remote_mlip);
        assert_eq!(subs[0].organize_mode, "offline");
        assert_eq!(subs[0].organize_interval_secs, 300);
        assert!(subs[0].last_organize_checked_at.is_none());
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
                "CREATE TABLE subscriptions (id INTEGER PRIMARY KEY, url TEXT NOT NULL UNIQUE, filter_regex TEXT, target_folder TEXT NOT NULL, interval_secs INTEGER DEFAULT 300, enabled BOOLEAN DEFAULT 1, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP); CREATE TABLE download_tasks (id INTEGER PRIMARY KEY, subscription_id INTEGER NOT NULL, item_hash TEXT NOT NULL, cloud_name TEXT DEFAULT '115', status TEXT DEFAULT 'pending', added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, completed_at TIMESTAMP); INSERT INTO subscriptions (url, target_folder) VALUES ('https://legacy.example/rss.xml', '/legacy'); INSERT INTO download_tasks (subscription_id, item_hash) VALUES (1, 'legacy-item'); PRAGMA user_version = 1;",
            )
            .unwrap();
        }

        let db = RssDatabase::new(&db_path).unwrap();
        let migrated = db.get_subscription(1).unwrap().unwrap();
        assert_eq!(migrated.url, "https://legacy.example/rss.xml");
        assert!(migrated.last_checked_at.is_none());
        assert!(migrated.connection_id.is_none());
        assert!(!migrated.auto_organize);
        assert!(migrated.organize_target_folder.is_none());
        assert!(migrated.organize_target_connection_id.is_none());
        assert!(migrated.organize_season_mode);
        assert!(migrated.remove_empty_dirs);
        assert!(!migrated.remote_mlip);
        assert_eq!(migrated.organize_mode, "offline");
        assert_eq!(migrated.organize_interval_secs, 300);
        assert!(migrated.last_organize_checked_at.is_none());
        let tasks = db.list_download_tasks(1, None).unwrap();
        assert_eq!(tasks.len(), 1);
        assert!(tasks[0].info_hash.is_none());
        assert!(tasks[0].remote_name.is_none());
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
    fn test_v2_schema_migrates_organization_and_download_correlation_columns() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("v2.db");
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "CREATE TABLE subscriptions (id INTEGER PRIMARY KEY, url TEXT NOT NULL UNIQUE, filter_regex TEXT, target_folder TEXT NOT NULL, interval_secs INTEGER DEFAULT 300, enabled BOOLEAN DEFAULT 1, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, last_checked_at TIMESTAMP, connection_id INTEGER); CREATE TABLE processed_items (id INTEGER PRIMARY KEY, subscription_id INTEGER NOT NULL, item_hash TEXT NOT NULL, title TEXT, processed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, UNIQUE(subscription_id, item_hash)); CREATE TABLE download_tasks (id INTEGER PRIMARY KEY, subscription_id INTEGER NOT NULL, item_hash TEXT NOT NULL, cloud_name TEXT DEFAULT '115', status TEXT DEFAULT 'pending', added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, completed_at TIMESTAMP); INSERT INTO subscriptions (url, target_folder, connection_id) VALUES ('https://v2.example/rss.xml', '/v2', 42); INSERT INTO download_tasks (subscription_id, item_hash) VALUES (1, 'v2-item'); PRAGMA user_version = 2;",
            )
            .unwrap();
        }

        let db = RssDatabase::new(&db_path).unwrap();
        let subscription = db.get_subscription(1).unwrap().unwrap();
        assert_eq!(subscription.connection_id, Some(42));
        assert!(!subscription.auto_organize);
        assert!(subscription.organize_target_folder.is_none());
        assert!(subscription.organize_target_connection_id.is_none());
        assert!(subscription.organize_season_mode);
        assert!(subscription.remove_empty_dirs);
        assert!(!subscription.remote_mlip);
        assert_eq!(subscription.organize_mode, "offline");
        assert_eq!(subscription.organize_interval_secs, 300);
        assert!(subscription.last_organize_checked_at.is_none());
        let task = db.list_download_tasks(1, None).unwrap().pop().unwrap();
        assert_eq!(task.item_hash, "v2-item");
        assert!(task.info_hash.is_none());
        assert!(task.remote_name.is_none());
        let user_version: i32 = db
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(user_version, 7);
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
        assert!(!subscription.auto_organize);
        assert!(subscription.organize_target_folder.is_none());
        assert!(subscription.organize_season_mode);
        assert!(subscription.remove_empty_dirs);
        assert!(!subscription.remote_mlip);

        db.update_subscription_organization_settings_with_mlip(
            id,
            true,
            Some("/CloudDrive/Organized Anime"),
            false,
            true,
            true,
        )
        .unwrap();
        let subscription = db.get_subscription(id).unwrap().unwrap();
        assert!(subscription.auto_organize);
        assert_eq!(
            subscription.organize_target_folder.as_deref(),
            Some("/CloudDrive/Organized Anime")
        );
        assert!(!subscription.organize_season_mode);
        assert!(subscription.remove_empty_dirs);
        assert!(subscription.remote_mlip);

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
        assert!(subscription.auto_organize);
        assert_eq!(
            subscription.organize_target_folder.as_deref(),
            Some("/CloudDrive/Organized Anime")
        );
        assert!(!subscription.organize_season_mode);
        assert!(subscription.remove_empty_dirs);

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
        let task = db.list_download_tasks(sub_id, None).unwrap().pop().unwrap();
        assert!(task.info_hash.is_none());
        assert!(task.remote_name.is_none());

        db.save_download_correlation(
            sub_id,
            "hash456",
            "f2b7e10f3dcfd3d4f24f9f3ce9eddd27d7782e8b",
            Some("[Group] Episode 01.mkv"),
        )
        .unwrap();
        let task = db.list_download_tasks(sub_id, None).unwrap().pop().unwrap();
        db.reconcile_download_task(
            task.id,
            sub_id,
            "f2b7e10f3dcfd3d4f24f9f3ce9eddd27d7782e8b",
            "finished",
            "[Group] Episode 01 (renamed).mkv",
        )
        .unwrap();
        db.complete_download_task(task.id, sub_id, "f2b7e10f3dcfd3d4f24f9f3ce9eddd27d7782e8b")
            .unwrap();

        let task = db.list_download_tasks(sub_id, None).unwrap().pop().unwrap();
        assert_eq!(
            task.info_hash.as_deref(),
            Some("f2b7e10f3dcfd3d4f24f9f3ce9eddd27d7782e8b")
        );
        assert_eq!(
            task.remote_name.as_deref(),
            Some("[Group] Episode 01 (renamed).mkv")
        );
        assert_eq!(task.status.as_deref(), Some("completed"));
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn due_organization_requires_a_correlated_nonterminal_task_and_interval() {
        let temp_dir = tempdir().unwrap();
        let db = RssDatabase::new(&temp_dir.path().join("test.db")).unwrap();
        let id = db
            .add_subscription_with_connection(
                "https://example.com/rss.xml",
                None,
                "/downloads",
                300,
                Some(1),
            )
            .unwrap();
        db.update_subscription_organization_settings(id, true, Some("/library"), true, false)
            .unwrap();
        db.set_subscription_organize_interval(id, 60).unwrap();
        assert_eq!(
            db.get_subscription(id)
                .unwrap()
                .unwrap()
                .organize_interval_secs,
            60
        );
        db.save_download_task(id, "legacy").unwrap();
        assert!(db.list_due_organization_subscriptions().unwrap().is_empty());

        db.save_download_task(id, "correlated").unwrap();
        db.save_download_correlation(
            id,
            "correlated",
            "abcdef1234567890abcdef1234567890abcdef12",
            None,
        )
        .unwrap();
        assert_eq!(db.list_due_organization_subscriptions().unwrap().len(), 1);
        db.mark_subscription_organize_checked(id).unwrap();
        assert!(db.list_due_organization_subscriptions().unwrap().is_empty());
        assert_eq!(db.count_uncorrelated_download_tasks(id).unwrap(), 1);
    }

    #[test]
    fn original_mode_is_due_without_an_offline_task() {
        let temp_dir = tempdir().unwrap();
        let db = RssDatabase::new(&temp_dir.path().join("test.db")).unwrap();
        let id = db
            .add_subscription_with_connection(
                "https://example.com/original.xml",
                None,
                "/downloads",
                300,
                Some(1),
            )
            .unwrap();
        db.update_subscription_organization_settings_with_mlip_and_mode(
            id,
            true,
            Some("/library"),
            true,
            false,
            true,
            "original",
        )
        .unwrap();

        let due = db.list_due_organization_subscriptions().unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].organize_mode, "original");
        db.mark_subscription_organize_checked(id).unwrap();
        assert!(db.list_due_organization_subscriptions().unwrap().is_empty());
    }

    #[test]
    fn unfinished_correlated_tasks_can_receive_their_initial_organization_settings() {
        let temp_dir = tempdir().unwrap();
        let db = RssDatabase::new(&temp_dir.path().join("test.db")).unwrap();
        let id = db
            .add_subscription_with_connection(
                "https://example.com/rss.xml",
                None,
                "/downloads",
                300,
                Some(1),
            )
            .unwrap();
        db.save_download_task(id, "correlated").unwrap();
        db.save_download_correlation(
            id,
            "correlated",
            "abcdef1234567890abcdef1234567890abcdef12",
            None,
        )
        .unwrap();

        db.update_subscription_organization_settings_with_mlip(
            id,
            true,
            Some("/library"),
            true,
            false,
            true,
        )
        .unwrap();
        let subscription = db.get_subscription(id).unwrap().unwrap();
        assert!(subscription.auto_organize);
        assert_eq!(
            subscription.organize_target_folder.as_deref(),
            Some("/library")
        );
        assert!(subscription.remote_mlip);
    }

    #[test]
    fn unfinished_correlated_tasks_lock_remote_identity_and_destination() {
        let temp_dir = tempdir().unwrap();
        let db = RssDatabase::new(&temp_dir.path().join("test.db")).unwrap();
        let id = db
            .add_subscription_with_connection(
                "https://example.com/rss.xml",
                None,
                "/downloads",
                300,
                Some(1),
            )
            .unwrap();
        db.update_subscription_organization_settings(id, true, Some("/library"), true, false)
            .unwrap();
        db.save_download_task(id, "correlated").unwrap();
        db.save_download_correlation(
            id,
            "correlated",
            "abcdef1234567890abcdef1234567890abcdef12",
            None,
        )
        .unwrap();

        assert!(db
            .update_subscription(
                id,
                "https://example.com/rss.xml",
                None,
                "/another-source",
                300,
                Some(1),
            )
            .is_err());
        assert!(db
            .update_subscription_organization_settings(
                id,
                true,
                Some("/another-library"),
                true,
                false
            )
            .is_err());
        db.update_subscription_organization_settings(id, true, Some("/library"), true, true)
            .unwrap();
        assert!(db.get_subscription(id).unwrap().unwrap().remove_empty_dirs);
        assert!(db.set_subscription_connection(id, Some(2)).is_err());
        db.update_subscription_organization_settings_with_mlip_and_mode(
            id,
            true,
            Some("/library"),
            true,
            false,
            false,
            "original",
        )
        .unwrap();
        assert_eq!(
            db.get_subscription(id).unwrap().unwrap().organize_mode,
            "original"
        );
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
