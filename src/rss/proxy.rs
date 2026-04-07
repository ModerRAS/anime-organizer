//! 代理配置模块
//!
//! 从环境变量读取 HTTP 代理配置，供网络请求使用。

use crate::error::AppError;

/// 代理配置结构体
///
/// 从环境变量 `http_proxy` / `HTTP_PROXY` 和 `https_proxy` / `HTTPS_PROXY` 读取代理设置。
#[derive(Debug, Clone, Default)]
pub struct ProxyConfig {
    /// HTTP 代理 URL
    pub http_proxy: Option<String>,
    /// HTTPS 代理 URL
    pub https_proxy: Option<String>,
}

impl ProxyConfig {
    /// 从环境变量创建代理配置
    ///
    /// 读取优先级：
    /// - `http_proxy` → `HTTP_PROXY`（小写优先）
    /// - `https_proxy` → `HTTPS_PROXY`（小写优先）
    ///
    /// 如果均未设置，返回 `None`。
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let http_proxy = std::env::var("http_proxy")
            .or_else(|_| std::env::var("HTTP_PROXY"))
            .ok();
        let https_proxy = std::env::var("https_proxy")
            .or_else(|_| std::env::var("HTTPS_PROXY"))
            .ok();

        if http_proxy.is_none() && https_proxy.is_none() {
            None
        } else {
            Some(Self {
                http_proxy,
                https_proxy,
            })
        }
    }
}

/// 使用代理配置构建 HTTP 客户端
///
/// # Errors
///
/// 如果代理 URL 格式无效或客户端构建失败，返回 `AppError::MetadataFetchError`。
pub fn build_http_client(proxy_config: &Option<ProxyConfig>) -> Result<reqwest::Client, AppError> {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(60));

    if let Some(proxy) = proxy_config {
        if let Some(http_proxy) = &proxy.http_proxy {
            let proxy = reqwest::Proxy::http(http_proxy)
                .map_err(|e| AppError::MetadataFetchError(format!("HTTP 代理配置失败: {e}")))?;
            builder = builder.proxy(proxy);
        }
        if let Some(https_proxy) = &proxy.https_proxy {
            let proxy = reqwest::Proxy::https(https_proxy)
                .map_err(|e| AppError::MetadataFetchError(format!("HTTPS 代理配置失败: {e}")))?;
            builder = builder.proxy(proxy);
        }
    }

    builder
        .build()
        .map_err(|e| AppError::MetadataFetchError(format!("HTTP 客户端创建失败: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_proxy_config_uppercase_fallback() {
        // 清理所有可能的环境变量
        env::remove_var("http_proxy");
        env::remove_var("HTTP_PROXY");
        env::remove_var("https_proxy");
        env::remove_var("HTTPS_PROXY");

        // 仅设置大写版本
        env::set_var("HTTP_PROXY", "http://10.0.0.1:8080");
        env::set_var("HTTPS_PROXY", "http://10.0.0.1:8443");

        let config = ProxyConfig::from_env();
        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!(config.http_proxy, Some("http://10.0.0.1:8080".to_string()));
        assert_eq!(config.https_proxy, Some("http://10.0.0.1:8443".to_string()));

        // 清理
        env::remove_var("http_proxy");
        env::remove_var("HTTP_PROXY");
        env::remove_var("https_proxy");
        env::remove_var("HTTPS_PROXY");
    }

    #[test]
    fn test_proxy_config_from_env() {
        // 清理所有可能的环境变量
        env::remove_var("http_proxy");
        env::remove_var("HTTP_PROXY");
        env::remove_var("https_proxy");
        env::remove_var("HTTPS_PROXY");

        // 设置环境变量
        env::set_var("http_proxy", "http://127.0.0.1:7890");
        env::set_var("https_proxy", "http://127.0.0.1:7891");

        let config = ProxyConfig::from_env();
        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!(config.http_proxy, Some("http://127.0.0.1:7890".to_string()));
        assert_eq!(
            config.https_proxy,
            Some("http://127.0.0.1:7891".to_string())
        );

        // 清理
        env::remove_var("http_proxy");
        env::remove_var("HTTP_PROXY");
        env::remove_var("https_proxy");
        env::remove_var("HTTPS_PROXY");
    }

    #[test]
    fn test_proxy_config_lowercase_priority() {
        // 清理所有可能的环境变量
        env::remove_var("http_proxy");
        env::remove_var("HTTP_PROXY");
        env::remove_var("https_proxy");
        env::remove_var("HTTPS_PROXY");

        // 仅设置小写版本
        env::set_var("http_proxy", "http://127.0.0.1:8080");

        let config = ProxyConfig::from_env().unwrap();
        assert_eq!(config.http_proxy, Some("http://127.0.0.1:8080".to_string()));

        // 清理
        env::remove_var("http_proxy");
        env::remove_var("HTTP_PROXY");
        env::remove_var("https_proxy");
        env::remove_var("HTTPS_PROXY");
    }

    #[test]
    fn test_proxy_config_none_when_unset() {
        // 确保清理所有可能的环境变量
        env::remove_var("http_proxy");
        env::remove_var("HTTP_PROXY");
        env::remove_var("https_proxy");
        env::remove_var("HTTPS_PROXY");

        let config = ProxyConfig::from_env();
        assert!(config.is_none());
    }

    #[test]
    fn test_build_http_client_no_proxy() {
        let client = build_http_client(&None);
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_http_client_with_proxy() {
        // 清理所有可能的环境变量
        env::remove_var("http_proxy");
        env::remove_var("HTTP_PROXY");
        env::remove_var("https_proxy");
        env::remove_var("HTTPS_PROXY");

        env::set_var("http_proxy", "http://127.0.0.1:7890");
        env::set_var("https_proxy", "http://127.0.0.1:7891");

        let config = ProxyConfig::from_env();
        let client = build_http_client(&config);
        assert!(client.is_ok());

        // 清理
        env::remove_var("http_proxy");
        env::remove_var("HTTP_PROXY");
        env::remove_var("https_proxy");
        env::remove_var("HTTPS_PROXY");
    }
}
