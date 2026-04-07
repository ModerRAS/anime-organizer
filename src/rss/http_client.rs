//! HTTP 客户端 trait
//!
//! 用于测试时 mock HTTP 请求。

use crate::error::{AppError, Result};
use async_trait::async_trait;

#[async_trait]
pub trait HttpClientTrait: Send + Sync {
    async fn get(&self, url: &str) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
}

impl HttpClient {
    pub fn new(inner: reqwest::Client) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl HttpClientTrait for HttpClient {
    async fn get(&self, url: &str) -> Result<String> {
        let resp = self
            .inner
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::MetadataFetchError(format!("抓取 RSS 失败: {e}")))?;
        resp.text()
            .await
            .map_err(|e| AppError::MetadataFetchError(format!("读取 RSS 内容失败: {e}")))
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new(reqwest::Client::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_client_get() {
        let client = reqwest::Client::builder().build().unwrap();
        let _http = HttpClient::new(client);
    }
}
