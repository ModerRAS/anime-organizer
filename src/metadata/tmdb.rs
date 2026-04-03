//! TMDB API 客户端模块
//!
//! 提供 TMDB (The Movie Database) 的搜索和图片下载功能。

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// TMDB API 基础 URL
const TMDB_API_BASE: &str = "https://api.themoviedb.org/3";

/// TMDB 图片基础 URL
const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p/";

/// TMDB 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbSearchResult {
    /// TMDB TV ID
    pub id: u32,
    /// 名称
    pub name: String,
    /// 原始名称
    pub original_name: Option<String>,
    /// 首播日期
    pub first_air_date: Option<String>,
    /// 海报路径
    pub poster_path: Option<String>,
    /// 背景图路径
    pub backdrop_path: Option<String>,
    /// 简介
    pub overview: Option<String>,
}

/// TMDB 搜索响应
#[derive(Debug, Clone, Deserialize)]
struct TmdbSearchResponse {
    results: Vec<TmdbSearchResult>,
}

/// TMDB 图片信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbImages {
    /// 海报 URL
    pub poster_url: Option<String>,
    /// 背景图 URL
    pub fanart_url: Option<String>,
    /// Banner URL
    pub banner_url: Option<String>,
}

/// TMDB 客户端
///
/// 使用 TMDB API v3 进行搜索和图片下载。
pub struct TmdbClient {
    /// API Key
    api_key: String,
}

impl TmdbClient {
    /// 创建新的 TMDB 客户端
    ///
    /// # 参数
    ///
    /// * `api_key` - TMDB API Key
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }

    /// 构建搜索 URL
    pub fn build_search_url(&self, title: &str, year: Option<u32>) -> String {
        let encoded_title =
            percent_encode(title.as_bytes(), &QUERY_ENCODE_SET).unwrap_or_else(|| title.to_string());
        let mut url = format!(
            "{}/search/tv?api_key={}&query={}",
            TMDB_API_BASE, self.api_key, encoded_title
        );
        if let Some(y) = year {
            url.push_str(&format!("&first_air_date_year={y}"));
        }
        url
    }

    /// 构建图片 URL
    ///
    /// # 参数
    ///
    /// * `path` - TMDB 图片路径（如 `/aiy3e4...jpg`）
    /// * `size` - 图片尺寸（如 `w500`, `w1280`, `original`）
    pub fn build_image_url(path: &str, size: &str) -> String {
        format!("{}{}{}", TMDB_IMAGE_BASE, size, path)
    }

    /// 从 TMDB 搜索结果构建图片信息
    pub fn build_images_from_result(result: &TmdbSearchResult) -> TmdbImages {
        TmdbImages {
            poster_url: result
                .poster_path
                .as_ref()
                .map(|p| Self::build_image_url(p, "w500")),
            fanart_url: result
                .backdrop_path
                .as_ref()
                .map(|p| Self::build_image_url(p, "w1280")),
            banner_url: None, // TMDB doesn't have a dedicated banner endpoint for TV
        }
    }

    /// 获取 API Key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

/// 简单 URL 编码
fn percent_encode(input: &[u8], _set: &()) -> Option<String> {
    let mut result = String::new();
    for &byte in input {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    Some(result)
}

/// 空的编码集（占位）
static QUERY_ENCODE_SET: () = ();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_search_url() {
        let client = TmdbClient::new("test_key");
        let url = client.build_search_url("Attack on Titan", Some(2013));
        assert!(url.contains("api_key=test_key"));
        assert!(url.contains("query=Attack%20on%20Titan"));
        assert!(url.contains("first_air_date_year=2013"));
    }

    #[test]
    fn test_build_search_url_without_year() {
        let client = TmdbClient::new("test_key");
        let url = client.build_search_url("進撃の巨人", None);
        assert!(url.contains("api_key=test_key"));
        assert!(!url.contains("first_air_date_year"));
    }

    #[test]
    fn test_build_image_url() {
        let url = TmdbClient::build_image_url("/abc123.jpg", "w500");
        assert_eq!(url, "https://image.tmdb.org/t/p/w500/abc123.jpg");
    }

    #[test]
    fn test_build_images_from_result() {
        let result = TmdbSearchResult {
            id: 1429,
            name: "Attack on Titan".to_string(),
            original_name: Some("進撃の巨人".to_string()),
            first_air_date: Some("2013-04-07".to_string()),
            poster_path: Some("/poster.jpg".to_string()),
            backdrop_path: Some("/backdrop.jpg".to_string()),
            overview: None,
        };

        let images = TmdbClient::build_images_from_result(&result);
        assert_eq!(
            images.poster_url.as_deref(),
            Some("https://image.tmdb.org/t/p/w500/poster.jpg")
        );
        assert_eq!(
            images.fanart_url.as_deref(),
            Some("https://image.tmdb.org/t/p/w1280/backdrop.jpg")
        );
    }

    #[test]
    fn test_build_images_from_result_missing() {
        let result = TmdbSearchResult {
            id: 1,
            name: "Test".to_string(),
            original_name: None,
            first_air_date: None,
            poster_path: None,
            backdrop_path: None,
            overview: None,
        };

        let images = TmdbClient::build_images_from_result(&result);
        assert!(images.poster_url.is_none());
        assert!(images.fanart_url.is_none());
    }
}
