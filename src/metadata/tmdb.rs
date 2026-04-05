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
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::LazyLock;

/// TMDB API 基础 URL
const TMDB_API_BASE: &str = "https://api.themoviedb.org/3";

/// TMDB 图片基础 URL
const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p";

/// AniDB 图片 CDN
const ANIDB_IMAGE_CDN: &str = "https://cdn.anidb.net/images/main";

static ANIDB_IMAGE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"https://cdn\.anidb\.net/images/main/[^"']+"#)
        .expect("AniDB 图片正则表达式编译失败")
});

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

    /// 按标题搜索并返回最佳匹配结果。
    #[cfg(feature = "metadata")]
    pub async fn find_by_title(
        &self,
        title: &str,
        year: Option<i32>,
    ) -> Result<Option<TmdbTvShow>> {
        let results = self.search_tv(title, year).await?;
        let normalized_title = normalize_title(title);

        if let Some(exact) = results.iter().find(|show| {
            normalize_title(&show.name) == normalized_title
                || show
                    .original_name
                    .as_ref()
                    .is_some_and(|original| normalize_title(original) == normalized_title)
        }) {
            return Ok(Some(exact.clone()));
        }

        Ok(results.into_iter().next())
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

    /// 为给定条目获取合适的海报 URL。
    #[cfg(feature = "metadata")]
    pub async fn best_poster_url(&self, show: &TmdbTvShow) -> Result<Option<String>> {
        if let Some(url) = self.poster_url(show) {
            return Ok(Some(url));
        }

        let images = self.get_images(show.id).await?;
        Ok(images
            .posters
            .first()
            .map(|image| Self::image_url(&image.file_path, ImageSize::W500)))
    }

    /// 为给定条目获取合适的背景图 URL。
    #[cfg(feature = "metadata")]
    pub async fn best_backdrop_url(&self, show: &TmdbTvShow) -> Result<Option<String>> {
        if let Some(url) = self.backdrop_url(show) {
            return Ok(Some(url));
        }

        let images = self.get_images(show.id).await?;
        Ok(images
            .backdrops
            .first()
            .map(|image| Self::image_url(&image.file_path, ImageSize::W1280)))
    }

    /// 生成指定尺寸的图片 URL
    pub fn image_url(path: &str, size: ImageSize) -> String {
        format!("{}/{}{}", TMDB_IMAGE_BASE, size.as_str(), path)
    }

    /// 生成 AniDB 图片 URL（回退用）
    pub fn anidb_image_url(filename: &str) -> String {
        format!("{ANIDB_IMAGE_CDN}/{filename}")
    }

    /// 根据 AniDB 条目页提取海报地址并下载。
    #[cfg(feature = "metadata")]
    pub async fn download_anidb_poster(&self, anidb_id: u32, save_path: &Path) -> Result<()> {
        let page_url = format!("https://anidb.net/anime/{anidb_id}");
        let resp = self
            .http
            .get(&page_url)
            .send()
            .await
            .map_err(|e| AppError::ImageDownloadError(format!("AniDB 页面请求失败: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::ImageDownloadError(format!(
                "AniDB 页面请求失败 (HTTP {})",
                resp.status()
            )));
        }

        let body = resp
            .text()
            .await
            .map_err(|e| AppError::ImageDownloadError(format!("读取 AniDB 页面失败: {e}")))?;

        let Some(image_match) = ANIDB_IMAGE_REGEX.find(&body) else {
            return Err(AppError::ImageDownloadError(format!(
                "AniDB 页面中未找到图片地址: {anidb_id}"
            )));
        };

        self.download_image(image_match.as_str(), save_path).await
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

fn normalize_title(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_alphanumeric() || ch.is_alphabetic())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_show() -> TmdbTvShow {
        TmdbTvShow {
            id: 1429,
            name: "Attack on Titan".to_string(),
            original_name: Some("進撃の巨人".to_string()),
            poster_path: Some("/hTP1DtLGFhgk4o1L0Tj0VkUmAIR.jpg".to_string()),
            backdrop_path: Some("/rqbCbjB19amtOtFQbb3K2lgm2zv.jpg".to_string()),
            overview: Some("Several hundred years ago...".to_string()),
            first_air_date: Some("2013-04-07".to_string()),
        }
    }

    #[test]
    fn test_poster_url() {
        let client = TmdbClient::new("test-key".to_string());
        let show = sample_show();
        let url = client.poster_url(&show).unwrap();
        assert_eq!(
            url,
            "https://image.tmdb.org/t/p/w500/hTP1DtLGFhgk4o1L0Tj0VkUmAIR.jpg"
        );
    }

    #[test]
    fn test_poster_url_none() {
        let client = TmdbClient::new("test-key".to_string());
        let mut show = sample_show();
        show.poster_path = None;
        assert!(client.poster_url(&show).is_none());
    }

    #[test]
    fn test_backdrop_url() {
        let client = TmdbClient::new("test-key".to_string());
        let show = sample_show();
        let url = client.backdrop_url(&show).unwrap();
        assert_eq!(
            url,
            "https://image.tmdb.org/t/p/w1280/rqbCbjB19amtOtFQbb3K2lgm2zv.jpg"
        );
    }

    #[test]
    fn test_backdrop_url_none() {
        let client = TmdbClient::new("test-key".to_string());
        let mut show = sample_show();
        show.backdrop_path = None;
        assert!(client.backdrop_url(&show).is_none());
    }

    #[test]
    fn test_image_url_sizes() {
        let path = "/test.jpg";
        assert_eq!(
            TmdbClient::image_url(path, ImageSize::Original),
            "https://image.tmdb.org/t/p/original/test.jpg"
        );
        assert_eq!(
            TmdbClient::image_url(path, ImageSize::W500),
            "https://image.tmdb.org/t/p/w500/test.jpg"
        );
        assert_eq!(
            TmdbClient::image_url(path, ImageSize::W780),
            "https://image.tmdb.org/t/p/w780/test.jpg"
        );
        assert_eq!(
            TmdbClient::image_url(path, ImageSize::W1280),
            "https://image.tmdb.org/t/p/w1280/test.jpg"
        );
    }

    #[test]
    fn test_anidb_image_url() {
        let url = TmdbClient::anidb_image_url("12345.jpg");
        assert_eq!(url, "https://cdn.anidb.net/images/main/12345.jpg");
    }

    #[test]
    fn test_normalize_title() {
        assert_eq!(normalize_title("Attack on Titan"), "attackontitan");
        assert_eq!(normalize_title("進撃の巨人"), "進撃の巨人");
        assert_eq!(normalize_title("  Spy x Family  "), "spyxfamily");
    }

    #[test]
    fn test_tmdb_search_result_deserialization() {
        let json = r#"{
            "results": [
                {
                    "id": 1429,
                    "name": "Attack on Titan",
                    "original_name": "進撃の巨人",
                    "poster_path": "/test.jpg",
                    "backdrop_path": null,
                    "overview": "A story",
                    "first_air_date": "2013-04-07"
                }
            ]
        }"#;

        let result: TmdbSearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.results[0].id, 1429);
        assert_eq!(result.results[0].name, "Attack on Titan");
        assert!(result.results[0].backdrop_path.is_none());
    }

    #[test]
    fn test_tmdb_images_deserialization() {
        let json = r#"{
            "id": 1429,
            "posters": [
                {"file_path": "/poster1.jpg", "width": 500, "height": 750, "vote_average": 5.5, "iso_639_1": "en"}
            ],
            "backdrops": []
        }"#;

        let images: TmdbImages = serde_json::from_str(json).unwrap();
        assert_eq!(images.id, Some(1429));
        assert_eq!(images.posters.len(), 1);
        assert_eq!(images.posters[0].file_path, "/poster1.jpg");
        assert!(images.backdrops.is_empty());
    }
}
