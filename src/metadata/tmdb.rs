//! TMDB API 客户端
//!
//! 使用 The Movie Database (TMDB) API 获取动画海报和剧照等图片资源。
//!
//! ## 图片回退链
//!
//! 1. TMDB（主要来源）
//! 2. AniDB（仅当 alias 中有 anidb_id 时）
//! 3. 跳过（不报错）

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// TMDB API 基础 URL
const TMDB_API_BASE: &str = "https://api.themoviedb.org/3";

/// TMDB 图片基础 URL
const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p";

/// AniDB 图片 CDN
const ANIDB_IMAGE_CDN: &str = "https://cdn.anidb.net/images/main";

/// TMDB 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbSearchResult {
    /// 搜索结果列表
    pub results: Vec<TmdbTvShow>,
}

/// TMDB 电视剧信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbTvShow {
    /// TMDB ID
    pub id: u32,
    /// 名称
    pub name: String,
    /// 原始名称
    pub original_name: Option<String>,
    /// 海报路径
    pub poster_path: Option<String>,
    /// 背景图路径
    pub backdrop_path: Option<String>,
    /// 概要
    pub overview: Option<String>,
    /// 首播日期
    pub first_air_date: Option<String>,
}

/// TMDB 图片信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbImages {
    /// TMDB ID
    pub id: Option<u32>,
    /// 海报列表
    #[serde(default)]
    pub posters: Vec<TmdbImage>,
    /// 背景图列表
    #[serde(default)]
    pub backdrops: Vec<TmdbImage>,
}

/// 单张 TMDB 图片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbImage {
    /// 文件路径
    pub file_path: String,
    /// 宽度
    pub width: Option<u32>,
    /// 高度
    pub height: Option<u32>,
    /// 投票平均分
    pub vote_average: Option<f32>,
    /// 语言代码
    pub iso_639_1: Option<String>,
}

/// TMDB 图片尺寸
#[derive(Debug, Clone, Copy)]
pub enum ImageSize {
    /// 原始尺寸
    Original,
    /// 500px 宽度（海报）
    W500,
    /// 780px 宽度（背景图）
    W780,
    /// 1280px 宽度（大背景图）
    W1280,
}

impl ImageSize {
    fn as_str(&self) -> &str {
        match self {
            Self::Original => "original",
            Self::W500 => "w500",
            Self::W780 => "w780",
            Self::W1280 => "w1280",
        }
    }
}

/// TMDB API 客户端
///
/// 提供 TMDB API 调用功能，包括搜索、详情和图片下载。
///
/// # 示例
///
/// ```no_run
/// # async fn example() -> anime_organizer::error::Result<()> {
/// use anime_organizer::metadata::TmdbClient;
///
/// let client = TmdbClient::new("your-api-key".to_string());
/// let results = client.search_tv("Bocchi the Rock", None).await?;
/// if let Some(show) = results.first() {
///     if let Some(url) = client.poster_url(show) {
///         println!("海报 URL: {url}");
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct TmdbClient {
    api_key: String,
    #[cfg(feature = "metadata")]
    http: reqwest::Client,
}

impl TmdbClient {
    /// 创建新的 TMDB 客户端
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            #[cfg(feature = "metadata")]
            http: reqwest::Client::builder()
                .user_agent("anime-organizer/0.1")
                .build()
                .expect("创建 HTTP 客户端失败"),
        }
    }

    /// 搜索 TMDB 电视剧
    #[cfg(feature = "metadata")]
    pub async fn search_tv(&self, query: &str, year: Option<i32>) -> Result<Vec<TmdbTvShow>> {
        let url = format!("{TMDB_API_BASE}/search/tv");
        let mut params = vec![
            ("api_key", self.api_key.as_str()),
            ("query", query),
            ("language", "zh-CN"),
        ];
        let year_str = year.map(|y| y.to_string());
        if let Some(ref y) = year_str {
            params.push(("first_air_date_year", y));
        }

        let resp = self
            .http
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| AppError::TmdbApiError(format!("TMDB 搜索请求失败: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::TmdbApiError(format!(
                "TMDB API 返回错误 (HTTP {})",
                resp.status()
            )));
        }

        let search_result: TmdbSearchResult = resp
            .json()
            .await
            .map_err(|e| AppError::TmdbApiError(format!("解析 TMDB 搜索结果失败: {e}")))?;

        Ok(search_result.results)
    }

    /// 按 TMDB ID 获取电视剧详情
    #[cfg(feature = "metadata")]
    pub async fn find_by_tmdb_id(&self, tmdb_id: u32) -> Result<TmdbTvShow> {
        let url = format!("{TMDB_API_BASE}/tv/{tmdb_id}");
        let resp = self
            .http
            .get(&url)
            .query(&[("api_key", self.api_key.as_str()), ("language", "zh-CN")])
            .send()
            .await
            .map_err(|e| AppError::TmdbApiError(format!("TMDB 详情请求失败: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::TmdbApiError(format!(
                "TMDB TV {tmdb_id} 未找到 (HTTP {})",
                resp.status()
            )));
        }

        let show: TmdbTvShow = resp
            .json()
            .await
            .map_err(|e| AppError::TmdbApiError(format!("解析 TMDB 详情失败: {e}")))?;

        Ok(show)
    }

    /// 获取 TMDB 电视剧图片列表
    #[cfg(feature = "metadata")]
    pub async fn get_images(&self, tmdb_id: u32) -> Result<TmdbImages> {
        let url = format!("{TMDB_API_BASE}/tv/{tmdb_id}/images");
        let resp = self
            .http
            .get(&url)
            .query(&[("api_key", self.api_key.as_str())])
            .send()
            .await
            .map_err(|e| AppError::TmdbApiError(format!("TMDB 图片请求失败: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::TmdbApiError(format!(
                "获取 TMDB 图片失败 (HTTP {})",
                resp.status()
            )));
        }

        let images: TmdbImages = resp
            .json()
            .await
            .map_err(|e| AppError::TmdbApiError(format!("解析 TMDB 图片数据失败: {e}")))?;

        Ok(images)
    }

    /// 生成海报图片 URL
    pub fn poster_url(&self, show: &TmdbTvShow) -> Option<String> {
        show.poster_path
            .as_ref()
            .map(|path| format!("{}/{}{}", TMDB_IMAGE_BASE, ImageSize::W500.as_str(), path))
    }

    /// 生成背景图片 URL
    pub fn backdrop_url(&self, show: &TmdbTvShow) -> Option<String> {
        show.backdrop_path
            .as_ref()
            .map(|path| format!("{}/{}{}", TMDB_IMAGE_BASE, ImageSize::W1280.as_str(), path))
    }

    /// 生成指定尺寸的图片 URL
    pub fn image_url(path: &str, size: ImageSize) -> String {
        format!("{}/{}{}", TMDB_IMAGE_BASE, size.as_str(), path)
    }

    /// 生成 AniDB 图片 URL（回退用）
    pub fn anidb_image_url(filename: &str) -> String {
        format!("{ANIDB_IMAGE_CDN}/{filename}")
    }

    /// 下载图片并保存到文件
    #[cfg(feature = "metadata")]
    pub async fn download_image(&self, url: &str, save_path: &Path) -> Result<()> {
        let resp = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::ImageDownloadError(format!("图片下载请求失败: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::ImageDownloadError(format!(
                "图片下载失败 (HTTP {})",
                resp.status()
            )));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| AppError::ImageDownloadError(format!("读取图片数据失败: {e}")))?;

        if let Some(parent) = save_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::ImageDownloadError(format!("创建目录失败: {e}")))?;
        }

        std::fs::write(save_path, &bytes)
            .map_err(|e| AppError::ImageDownloadError(format!("保存图片失败: {e}")))?;

        Ok(())
    }

    /// 下载图片内容为字节
    #[cfg(feature = "metadata")]
    pub async fn download_image_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let resp = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::ImageDownloadError(format!("图片下载请求失败: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::ImageDownloadError(format!(
                "图片下载失败 (HTTP {})",
                resp.status()
            )));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| AppError::ImageDownloadError(format!("读取图片数据失败: {e}")))?;

        Ok(bytes.to_vec())
    }
}
