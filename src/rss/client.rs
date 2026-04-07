//! CloudDrive2 gRPC 客户端模块
//!
//! 提供与 CloudDrive2 服务通信的客户端实现，包括 Token 获取和离线文件添加功能。

use async_trait::async_trait;
use std::time::Duration;
use tonic::transport::{Channel, Endpoint};

use crate::error::{AppError, Result};

/// CloudDrive2 proto 生成模块
///
/// 通过 tonic-build 从 `proto/clouddrive.proto` 生成的 Rust 代码。
/// 包含所有 CloudDrive2 gRPC 服务的请求/响应类型和客户端 stub。
pub mod proto {
    #![allow(clippy::large_enum_variant)]
    tonic::include_proto!("clouddrive");
}

/// CloudDrive2 客户端 trait，用于测试时 mock
///
/// 定义与 CloudDrive2 服务交互的核心操作。
#[async_trait]
pub trait CloudDriveClientTrait: Send + Sync {
    /// 登录获取 JWT 令牌
    async fn login(&mut self, username: &str, password: &str) -> Result<String>;

    /// 添加离线下载任务
    async fn add_offline_files(&self, urls: Vec<String>, to_folder: &str) -> Result<()>;
}

/// CloudDrive2 gRPC 客户端
///
/// 用于与 CloudDrive2 服务通信，支持 Token 认证和离线文件提交。
#[derive(Debug, Clone)]
pub struct CloudDriveClient {
    /// CloudDrive2 gRPC 服务端点 URL
    endpoint: String,
    /// JWT 认证令牌
    token: Option<String>,
}

impl CloudDriveClient {
    /// 创建新的 CloudDriveClient 实例
    pub fn new(url: &str, token: Option<String>) -> Result<Self> {
        let parsed_url = url::Url::parse(url)
            .map_err(|e| AppError::MetadataFetchError(format!("Invalid endpoint URL: {}", e)))?;

        if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
            return Err(AppError::MetadataFetchError(format!(
                "Invalid URL scheme: {} (expected http or https)",
                parsed_url.scheme()
            )));
        }

        Ok(Self {
            endpoint: url.to_string(),
            token,
        })
    }

    /// 设置 JWT 认证令牌
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    /// 获取当前 JWT 令牌
    pub fn get_token_value(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// 构建 gRPC 通道
    async fn build_channel(&self) -> Result<Channel> {
        let endpoint = Endpoint::from_shared(self.endpoint.clone())
            .map_err(|e| AppError::MetadataFetchError(format!("Invalid endpoint: {}", e)))?;

        let channel = if self.endpoint.starts_with("https") {
            let tls_config = tonic::transport::ClientTlsConfig::new().domain_name(
                url::Url::parse(&self.endpoint)
                    .ok()
                    .and_then(|u| u.host_str().map(|s| s.to_string()))
                    .unwrap_or_default(),
            );
            endpoint
                .tls_config(tls_config)
                .map_err(|e| AppError::MetadataFetchError(format!("TLS config error: {}", e)))?
                .timeout(Duration::from_secs(30))
                .connect()
                .await
                .map_err(|e| AppError::MetadataFetchError(format!("Connection failed: {}", e)))?
        } else {
            endpoint
                .timeout(Duration::from_secs(30))
                .connect()
                .await
                .map_err(|e| AppError::MetadataFetchError(format!("Connection failed: {}", e)))?
        };

        Ok(channel)
    }
}

#[async_trait]
impl CloudDriveClientTrait for CloudDriveClient {
    /// 登录获取 JWT 令牌
    async fn login(&mut self, username: &str, password: &str) -> Result<String> {
        let channel = self.build_channel().await?;

        let mut client = proto::cloud_drive_file_srv_client::CloudDriveFileSrvClient::new(channel);

        let request = tonic::Request::new(proto::GetTokenRequest {
            user_name: username.to_string(),
            password: password.to_string(),
            totp_code: None,
        });

        let response = client
            .get_token(request)
            .await
            .map_err(|e| AppError::MetadataFetchError(format!("GetToken failed: {}", e)))?;

        let token_response = response.into_inner();

        if !token_response.success {
            return Err(AppError::MetadataFetchError(format!(
                "Login failed: {}",
                token_response.error_message
            )));
        }

        let token = token_response.token;
        self.token = Some(token.clone());

        Ok(token)
    }

    /// 添加离线下载任务
    async fn add_offline_files(&self, urls: Vec<String>, to_folder: &str) -> Result<()> {
        let token = self.token.clone().ok_or_else(|| {
            AppError::MetadataFetchError("Not authenticated. Call login() first.".to_string())
        })?;

        let channel = self.build_channel().await?;

        let mut client =
            proto::cloud_drive_file_srv_client::CloudDriveFileSrvClient::with_interceptor(
                channel,
                move |mut req: tonic::Request<()>| {
                    let header_value = format!("Bearer {}", token);
                    let metadata_value: tonic::metadata::MetadataValue<_> =
                        header_value.parse().map_err(|_| {
                            tonic::Status::invalid_argument("Invalid authorization token")
                        })?;
                    req.metadata_mut().insert("authorization", metadata_value);
                    Ok(req)
                },
            );

        let urls_str = urls.join("\n");

        let request = tonic::Request::new(proto::AddOfflineFileRequest {
            urls: urls_str,
            to_folder: to_folder.to_string(),
            check_folder_after_secs: None,
        });

        let response = client
            .add_offline_files(request)
            .await
            .map_err(|e| AppError::MetadataFetchError(format!("AddOfflineFiles failed: {}", e)))?;

        let result = response.into_inner();

        if !result.success {
            return Err(AppError::MetadataFetchError(format!(
                "AddOfflineFiles failed: {}",
                result.error_message
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_new_valid_http_url() {
        let client = CloudDriveClient::new("http://localhost:8080", None);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_new_valid_https_url() {
        let client = CloudDriveClient::new("https://localhost:443", None);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_new_with_token() {
        let client = CloudDriveClient::new("http://localhost:8080", Some("test_token".to_string()));
        assert!(client.is_ok());
        assert_eq!(client.unwrap().get_token_value(), Some("test_token"));
    }

    #[test]
    fn test_client_new_invalid_url() {
        let client = CloudDriveClient::new("invalid_url", None);
        assert!(client.is_err());
    }

    #[test]
    fn test_client_new_invalid_scheme() {
        let client = CloudDriveClient::new("ftp://localhost:8080", None);
        assert!(client.is_err());
    }

    #[test]
    fn test_client_set_token() {
        let mut client = CloudDriveClient::new("http://localhost:8080", None).unwrap();
        assert_eq!(client.get_token_value(), None);

        client.set_token("new_token".to_string());
        assert_eq!(client.get_token_value(), Some("new_token"));
    }
}
