//! HTTP 下载模块
//!
//! 原生 HTTP 下载器，用于替代 aria2c 子进程调用。
//!
//! 支持多线程分块下载、代理配置、断点续传等功能。

use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::AsyncWriteExt;

use crate::error::{AppError, Result};

#[cfg(feature = "clouddrive")]
use crate::rss::proxy::{build_http_client, ProxyConfig};

/// 默认分块大小：10MB
const DEFAULT_CHUNK_SIZE: u64 = 10 * 1024 * 1024;

/// 默认重试次数
const DEFAULT_MAX_RETRIES: usize = 5;

/// 默认重试等待时间（秒）
const DEFAULT_RETRY_WAIT_SECS: u64 = 10;

/// HTTP 下载器
///
/// 基于 `reqwest` 实现的多线程分块下载器，用于替代 aria2c。
#[derive(Debug)]
pub struct HttpDownloader {
    /// 下载 URL
    pub url: String,
    /// 输出路径
    pub output_path: PathBuf,
    /// 临时目录（用于存放分块文件）
    pub temp_dir: PathBuf,
    /// HTTP 客户端
    pub client: reqwest::Client,
    /// 分块数量（默认 16）
    pub chunk_count: usize,
    /// 是否显示详细日志
    pub verbose: bool,
}

/// 下载分块
///
/// 用于跟踪分块下载进度。
#[derive(Debug, Clone)]
pub struct Chunk {
    /// 分块索引
    pub index: usize,
    /// 分块起始位置
    pub start: u64,
    /// 分块结束位置（inclusive）
    pub end: u64,
    /// 临时文件路径
    pub temp_path: PathBuf,
    /// 已下载字节数
    pub downloaded: u64,
    /// 是否完成
    pub completed: bool,
}

impl Chunk {
    /// 创建新的分块
    fn new(index: usize, start: u64, end: u64, temp_dir: &Path) -> Self {
        Self {
            index,
            start,
            end,
            temp_path: temp_dir.join(format!("chunk_{}.tmp", index)),
            downloaded: 0,
            completed: false,
        }
    }

    /// 获取分块大小
    fn size(&self) -> u64 {
        self.end - self.start + 1
    }
}

impl HttpDownloader {
    /// 创建新的 HTTP 下载器
    ///
    /// # Errors
    ///
    /// 如果 HTTP 客户端创建失败，返回错误。
    #[cfg(feature = "clouddrive")]
    pub fn new(url: String, temp_dir: PathBuf, verbose: bool) -> Result<Self> {
        let proxy_config = ProxyConfig::from_env();
        let client = build_http_client(&proxy_config)
            .map_err(|e| AppError::MetadataFetchError(format!("创建 HTTP 客户端失败: {e}")))?;

        Ok(Self {
            url,
            output_path: PathBuf::new(),
            temp_dir,
            client,
            chunk_count: 16,
            verbose,
        })
    }

    /// 使用指定的 HTTP 客户端创建下载器
    ///
    /// 当不需要代理配置时使用此方法。
    pub fn with_client(
        url: String,
        temp_dir: PathBuf,
        client: reqwest::Client,
        verbose: bool,
    ) -> Self {
        Self {
            url,
            output_path: PathBuf::new(),
            temp_dir,
            client,
            chunk_count: 16,
            verbose,
        }
    }

    /// 设置输出路径
    #[must_use]
    pub fn with_output_path(mut self, path: PathBuf) -> Self {
        self.output_path = path;
        self
    }

    /// 设置分块数量
    #[must_use]
    pub fn with_chunk_count(mut self, count: usize) -> Self {
        self.chunk_count = count;
        self
    }

    /// 获取文件大小
    ///
    /// 发送 HEAD 请求获取 `Content-Length` 头。
    ///
    /// # Errors
    ///
    /// 如果请求失败或无法解析 Content-Length，返回错误。
    pub async fn fetch_size(&self) -> Result<u64> {
        if self.verbose {
            eprintln!("[DOWNLOAD] 获取文件大小: {}", self.url);
        }

        let resp = self.client.head(&self.url).send().await.map_err(|e| {
            AppError::MetadataFetchError(format!("HEAD 请求失败 ({}): {e}", self.url))
        })?;

        if !resp.status().is_success() && resp.status().as_u16() != 206 {
            return Err(AppError::MetadataFetchError(format!(
                "HEAD 请求失败 (HTTP {}): {}",
                resp.status(),
                self.url
            )));
        }

        let content_length = resp
            .headers()
            .get("Content-Length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or_else(|| {
                AppError::MetadataFetchError(format!("无法解析 Content-Length: {}", self.url))
            })?;

        if self.verbose {
            eprintln!(
                "[DOWNLOAD] 文件大小: {} bytes ({} MB)",
                content_length,
                content_length / (1024 * 1024)
            );
        }

        Ok(content_length)
    }

    /// 将文件分割为分块
    ///
    /// 分块策略：
    /// - 如果文件小于分块大小，返回单个分块
    /// - 否则分割为最多 `chunk_count` 个分块，每个约 10MB
    fn split_chunks(&self, total_size: u64) -> Vec<Chunk> {
        // 小文件使用单个分块
        if total_size <= DEFAULT_CHUNK_SIZE {
            if self.verbose {
                eprintln!("[DOWNLOAD] 文件较小，使用单个分块");
            }
            return vec![Chunk::new(0, 0, total_size - 1, &self.temp_dir)];
        }

        // 计算使用 DEFAULT_CHUNK_SIZE 需要多少个分块
        let chunks_by_size = total_size.div_ceil(DEFAULT_CHUNK_SIZE) as usize;
        // 使用两者中较小的分块数
        let actual_chunk_count = chunks_by_size.min(self.chunk_count);

        // 计算每个分块的大小
        let chunk_size = total_size.div_ceil(actual_chunk_count as u64);

        if self.verbose {
            eprintln!(
                "[DOWNLOAD] 分割为 {} 个分块，每块约 {} MB",
                actual_chunk_count,
                chunk_size / (1024 * 1024)
            );
        }

        let mut chunks = Vec::with_capacity(actual_chunk_count);
        let mut start: u64 = 0;

        for i in 0..actual_chunk_count {
            let end = if i == actual_chunk_count - 1 {
                total_size - 1 // 最后一个分块到文件末尾
            } else {
                (start + chunk_size).min(total_size) - 1
            };
            chunks.push(Chunk::new(i, start, end, &self.temp_dir));
            start = end + 1;
        }

        chunks
    }

    /// 下载单个分块（带重试）
    ///
    /// # Errors
    ///
    /// 如果所有重试次数耗尽仍失败，返回错误。
    async fn download_chunk(&self, chunk: &Chunk, retries: usize) -> Result<()> {
        let range_header = format!("bytes={}-{}", chunk.start, chunk.end);
        let mut last_error = None;

        for attempt in 0..=retries {
            if attempt > 0 {
                // 重试等待（带 jitter）
                let jitter = rand_jitter(3);
                let wait_secs = DEFAULT_RETRY_WAIT_SECS + jitter;
                if self.verbose {
                    eprintln!(
                        "[DOWNLOAD] 分块 {} 重试 {}/{}，等待 {} 秒...",
                        chunk.index, attempt, retries, wait_secs
                    );
                }
                tokio::time::sleep(Duration::from_secs(wait_secs)).await;
            }

            match self.do_download_chunk(chunk, &range_header).await {
                Ok(_) => {
                    if self.verbose {
                        eprintln!("[DOWNLOAD] 分块 {} 下载完成", chunk.index);
                    }
                    return Ok(());
                }
                Err(e) => {
                    if self.verbose {
                        eprintln!(
                            "[DOWNLOAD] 分块 {} 下载失败 (尝试 {}/{}): {}",
                            chunk.index,
                            attempt + 1,
                            retries,
                            e
                        );
                    }
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            AppError::MetadataFetchError(format!(
                "分块 {} 下载失败（已重试 {} 次）",
                chunk.index, retries
            ))
        }))
    }

    /// 执行分块下载
    async fn do_download_chunk(&self, chunk: &Chunk, range_header: &str) -> Result<()> {
        let resp = self
            .client
            .get(&self.url)
            .header("Range", range_header)
            .send()
            .await
            .map_err(|e| {
                AppError::MetadataFetchError(format!("分块 {} 请求失败: {e}", chunk.index))
            })?;

        let status = resp.status();
        // 接受 200 (服务器不支持 Range) 或 206 (Partial Content)
        if !status.is_success() && status.as_u16() != 206 {
            return Err(AppError::MetadataFetchError(format!(
                "分块 {} 下载失败 (HTTP {}): {}",
                chunk.index, status, self.url
            )));
        }

        // 写入文件
        let bytes = resp.bytes().await.map_err(|e| {
            AppError::MetadataFetchError(format!("读取分块 {} 响应失败: {e}", chunk.index))
        })?;

        tokio::fs::write(&chunk.temp_path, &bytes)
            .await
            .map_err(|e| {
                AppError::MetadataFetchError(format!("写入分块 {} 文件失败: {e}", chunk.index))
            })?;

        // 验证大小
        let written_size = bytes.len() as u64;
        let expected_size = chunk.size();

        // 如果服务器不支持 Range，会返回整个文件（第一个分块）
        if written_size != expected_size && chunk.start > 0 {
            return Err(AppError::MetadataFetchError(format!(
                "分块 {} 大小不匹配: 期望 {} bytes, 实际 {} bytes",
                chunk.index, expected_size, written_size
            )));
        }

        Ok(())
    }

    /// 并发下载所有分块
    ///
    /// # Errors
    ///
    /// 如果任何分块下载失败，返回错误。
    pub async fn download_chunks(&self, chunks: Vec<Chunk>) -> Result<()> {
        if chunks.is_empty() {
            return Ok(());
        }

        let total_chunks = chunks.len();

        if self.verbose {
            eprintln!("[DOWNLOAD] 开始并发下载 {} 个分块...", total_chunks);
        }

        // 使用 tokio::task::JoinSet 进行并发下载
        let mut join_set = tokio::task::JoinSet::new();

        for chunk in chunks.iter() {
            let downloader = Self {
                url: self.url.clone(),
                output_path: self.output_path.clone(),
                temp_dir: self.temp_dir.clone(),
                client: self.client.clone(),
                chunk_count: self.chunk_count,
                verbose: self.verbose,
            };
            let chunk_clone = chunk.clone();

            join_set.spawn(async move {
                downloader
                    .download_chunk(&chunk_clone, DEFAULT_MAX_RETRIES)
                    .await
            });
        }

        // 收集结果
        let mut completed = 0;
        while let Some(result) = join_set.join_next().await {
            completed += 1;

            match result {
                Ok(Ok(())) => {
                    if self.verbose {
                        eprintln!("[DOWNLOAD] 进度: {}/{} 分块完成", completed, total_chunks);
                    }
                }
                Ok(Err(e)) => {
                    // 一个分块失败，取消所有其他任务
                    join_set.abort_all();
                    return Err(e);
                }
                Err(e) => {
                    join_set.abort_all();
                    return Err(AppError::MetadataFetchError(format!("分块任务被取消: {e}")));
                }
            }
        }

        if self.verbose {
            eprintln!("[DOWNLOAD] 所有 {} 个分块下载完成", total_chunks);
        }

        Ok(())
    }

    /// 合并分块到最终文件
    ///
    /// # Errors
    ///
    /// 如果合并失败或文件大小不匹配，返回错误。
    pub async fn merge_chunks(&self, chunks: &[Chunk], expected_size: u64) -> Result<PathBuf> {
        if self.verbose {
            eprintln!("[DOWNLOAD] 开始合并分块到: {:?}", self.output_path);
        }

        // 创建输出文件
        let file = tokio::fs::File::create(&self.output_path)
            .await
            .map_err(|e| AppError::MetadataFetchError(format!("创建输出文件失败: {e}")))?;

        // 使用 BufWriter 提高写入效率
        use tokio::io::BufWriter;
        let mut writer = BufWriter::new(file);

        // 按顺序写入每个分块
        for chunk in chunks.iter() {
            if self.verbose {
                eprintln!("[DOWNLOAD] 合并分块 {}...", chunk.index);
            }

            let data = tokio::fs::read(&chunk.temp_path).await.map_err(|e| {
                AppError::MetadataFetchError(format!("读取分块 {} 失败: {e}", chunk.index))
            })?;

            tokio::io::AsyncWriteExt::write_all(&mut writer, &data)
                .await
                .map_err(|e| {
                    AppError::MetadataFetchError(format!("写入分块 {} 失败: {e}", chunk.index))
                })?;

            // 删除临时分块文件
            if let Err(e) = tokio::fs::remove_file(&chunk.temp_path).await {
                // 只记录警告，不中断流程
                eprintln!(
                    "[DOWNLOAD] 警告: 删除分块文件 {:?} 失败: {}",
                    chunk.temp_path, e
                );
            }
        }

        // 刷新缓冲区
        writer
            .flush()
            .await
            .map_err(|e| AppError::MetadataFetchError(format!("刷新文件缓冲区失败: {e}")))?;

        // 确保数据写入磁盘
        writer
            .shutdown()
            .await
            .map_err(|e| AppError::MetadataFetchError(format!("关闭文件失败: {e}")))?;

        // 验证文件大小
        let actual_size = tokio::fs::metadata(&self.output_path)
            .await
            .map_err(|e| AppError::MetadataFetchError(format!("获取输出文件大小失败: {e}")))?
            .len();

        if actual_size != expected_size {
            return Err(AppError::MetadataFetchError(format!(
                "文件大小不匹配: 期望 {} bytes, 实际 {} bytes",
                expected_size, actual_size
            )));
        }

        if self.verbose {
            eprintln!(
                "[DOWNLOAD] 合并完成: {} bytes ({} MB)",
                actual_size,
                actual_size / (1024 * 1024)
            );
        }

        Ok(self.output_path.clone())
    }

    /// 下载文件（带进度显示）
    ///
    /// 这是下载器的主入口方法，执行完整下载流程：
    /// 1. 获取文件大小
    /// 2. 分割分块
    /// 3. 并发下载所有分块
    /// 4. 合并分块
    /// 5. 验证最终文件
    ///
    /// # Errors
    ///
    /// 如果任何步骤失败，返回错误。
    pub async fn download_with_progress(&self) -> Result<PathBuf> {
        if self.output_path.as_os_str().is_empty() {
            return Err(AppError::MetadataFetchError("未设置输出路径".to_string()));
        }

        let start_time = std::time::Instant::now();

        // Step 1: 获取文件大小
        if self.verbose {
            eprintln!("[DOWNLOAD] ===== 开始下载 =====");
            eprintln!("[DOWNLOAD] URL: {}", self.url);
            eprintln!("[DOWNLOAD] 输出: {:?}", self.output_path);
        }

        let file_size = self.fetch_size().await?;

        // Step 2: 分割分块
        let chunks = self.split_chunks(file_size);

        // Step 3: 并发下载所有分块
        self.download_chunks(chunks.clone()).await?;

        // Step 4: 合并分块
        let output_path = self.merge_chunks(&chunks, file_size).await?;

        let elapsed = start_time.elapsed();
        let speed_mbps = (file_size as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64();

        if self.verbose {
            eprintln!("[DOWNLOAD] ===== 下载完成 =====");
            eprintln!(
                "[DOWNLOAD] 耗时: {:.2}s, 速度: {:.2} MB/s",
                elapsed.as_secs_f64(),
                speed_mbps
            );
        }

        Ok(output_path)
    }
}

/// 生成随机 jitter（0-3 秒）
fn rand_jitter(max: u64) -> u64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    nanos % (max + 1)
}

impl Clone for HttpDownloader {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            output_path: self.output_path.clone(),
            temp_dir: self.temp_dir.clone(),
            client: self.client.clone(),
            chunk_count: self.chunk_count,
            verbose: self.verbose,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_size() {
        let temp_dir = PathBuf::from("/tmp");
        let chunk = Chunk::new(0, 0, 9, &temp_dir);
        assert_eq!(chunk.size(), 10);
    }

    #[test]
    fn test_split_chunks_small_file() {
        let temp_dir = PathBuf::from("/tmp");
        let downloader = HttpDownloader {
            url: "http://example.com/file.zip".to_string(),
            output_path: PathBuf::from("/tmp/output.zip"),
            temp_dir: temp_dir.clone(),
            client: reqwest::Client::new(),
            chunk_count: 16,
            verbose: false,
        };

        // 小于分块大小的文件
        let chunks = downloader.split_chunks(1024); // 1KB
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].start, 0);
        assert_eq!(chunks[0].end, 1023);
    }

    #[test]
    fn test_split_chunks_large_file() {
        let temp_dir = PathBuf::from("/tmp");
        let downloader = HttpDownloader {
            url: "http://example.com/file.zip".to_string(),
            output_path: PathBuf::from("/tmp/output.zip"),
            temp_dir: temp_dir.clone(),
            client: reqwest::Client::new(),
            chunk_count: 4, // 4 chunks
            verbose: false,
        };

        // 100MB 文件
        let file_size = 100 * 1024 * 1024;
        let chunks = downloader.split_chunks(file_size);

        // 应该分成约 4 个分块
        assert!(chunks.len() <= 4);

        // 验证分块连续性
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i);
            if i > 0 {
                assert_eq!(chunk.start, chunks[i - 1].end + 1);
            }
        }

        // 最后一个分块应该到文件末尾
        let last = chunks.last().unwrap();
        assert_eq!(last.end, file_size - 1);
    }

    #[tokio::test]
    async fn test_http_downloader_creation() {
        let temp_dir = tempfile::tempdir().unwrap().path().to_path_buf();
        let _downloader = HttpDownloader::with_client(
            "http://example.com/test.zip".to_string(),
            temp_dir,
            reqwest::Client::new(),
            false,
        );
    }
}
