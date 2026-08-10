//! CloudDrive2 gRPC 客户端模块
//!
//! 提供与 CloudDrive2 服务通信的客户端实现，包括远程文件操作和离线下载功能。

use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio_stream::StreamExt;
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

    /// 列出目录下的文件和子目录
    async fn list_folder(&self, path: &str) -> Result<Vec<proto::CloudDriveFile>>;

    /// 强制刷新后列出目录下的文件和子目录
    async fn list_folder_fresh(&self, path: &str) -> Result<Vec<proto::CloudDriveFile>> {
        self.list_folder(path).await
    }

    /// 列出指定远程路径的离线下载任务
    async fn list_offline_files_by_path(&self, _path: &str) -> Result<Vec<proto::OfflineFile>> {
        Err(unsupported_operation("list offline files by path"))
    }

    /// 在指定远程父目录创建文件夹
    async fn create_folder(
        &self,
        _parent_path: &str,
        _folder_name: &str,
    ) -> Result<proto::CloudDriveFile> {
        Err(unsupported_operation("create folder"))
    }

    /// 将远程文件移动到目标目录，目标冲突时跳过
    async fn move_files(&self, _paths: Vec<String>, _destination: &str) -> Result<()> {
        Err(unsupported_operation("move files"))
    }

    /// 通过远程父目录和路径查找文件
    async fn find_file_by_path(
        &self,
        _parent_path: &str,
        _path: &str,
    ) -> Result<proto::CloudDriveFile> {
        Err(unsupported_operation("find file by path"))
    }

    /// 删除远程文件或文件夹，不永久删除
    async fn delete_file(&self, _path: &str) -> Result<()> {
        Err(unsupported_operation("delete file"))
    }

    /// 计算远程文件的完整 SHA-256
    async fn sha256_file(&self, _path: &str) -> Result<String> {
        Err(unsupported_operation("SHA-256 file hashing"))
    }
}

fn unsupported_operation(operation: &str) -> AppError {
    AppError::MetadataFetchError(format!("CloudDrive client does not support {operation}"))
}

fn rpc_error(operation: &str, error: impl std::fmt::Display) -> AppError {
    AppError::MetadataFetchError(format!("{operation} failed: {error}"))
}

fn ensure_success(operation: &str, success: bool, error_message: &str) -> Result<()> {
    if success {
        Ok(())
    } else {
        Err(AppError::MetadataFetchError(format!(
            "{operation} failed: {error_message}"
        )))
    }
}

fn operation_result(operation: &str, result: &proto::FileOperationResult) -> Result<()> {
    ensure_success(operation, result.success, &result.error_message)
}

/// CloudDrive2 gRPC 客户端
///
/// 用于与 CloudDrive2 服务通信，支持 Token 认证和远程文件操作。
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
            .map_err(|e| AppError::MetadataFetchError(format!("Invalid endpoint URL: {e}")))?;

        if parsed_url.username() != "" || parsed_url.password().is_some() {
            return Err(AppError::MetadataFetchError(
                "Endpoint URL must not contain embedded credentials".to_string(),
            ));
        }

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

    fn authenticated_request<T>(&self, message: T) -> Result<tonic::Request<T>> {
        let token = self.token.as_deref().ok_or_else(|| {
            AppError::MetadataFetchError("Not authenticated. Call login() first.".to_string())
        })?;
        let header_value: tonic::metadata::MetadataValue<_> = format!("Bearer {token}")
            .parse()
            .map_err(|_| AppError::MetadataFetchError("Invalid authorization token".to_string()))?;
        let mut request = tonic::Request::new(message);
        request.metadata_mut().insert("authorization", header_value);
        Ok(request)
    }

    /// 构建 gRPC 通道
    async fn build_channel(&self) -> Result<Channel> {
        let endpoint = Endpoint::from_shared(self.endpoint.clone())
            .map_err(|e| AppError::MetadataFetchError(format!("Invalid endpoint: {e}")))?;

        let channel = if self.endpoint.starts_with("https") {
            let tls_config = tonic::transport::ClientTlsConfig::new().domain_name(
                url::Url::parse(&self.endpoint)
                    .ok()
                    .and_then(|u| u.host_str().map(|s| s.to_string()))
                    .unwrap_or_default(),
            );
            endpoint
                .tls_config(tls_config)
                .map_err(|e| AppError::MetadataFetchError(format!("TLS config error: {e}")))?
                .timeout(Duration::from_secs(30))
                .connect()
                .await
                .map_err(|e| AppError::MetadataFetchError(format!("Connection failed: {e}")))?
        } else {
            endpoint
                .timeout(Duration::from_secs(30))
                .connect()
                .await
                .map_err(|e| AppError::MetadataFetchError(format!("Connection failed: {e}")))?
        };

        Ok(channel)
    }

    async fn list_folder_with_refresh(
        &self,
        path: &str,
        force_refresh: bool,
    ) -> Result<Vec<proto::CloudDriveFile>> {
        let channel = self.build_channel().await?;
        let mut client = proto::cloud_drive_file_srv_client::CloudDriveFileSrvClient::new(channel);
        let request = self.authenticated_request(proto::ListSubFileRequest {
            path: path.to_string(),
            force_refresh,
            check_expires: None,
        })?;
        let mut stream = client
            .get_sub_files(request)
            .await
            .map_err(|error| rpc_error("GetSubFiles", error))?
            .into_inner();

        let mut files = Vec::new();
        while let Some(reply) = stream
            .next()
            .await
            .transpose()
            .map_err(|error| rpc_error("GetSubFiles stream", error))?
        {
            files.extend(reply.sub_files);
        }
        Ok(files)
    }

    fn download_url(&self, info: &proto::DownloadUrlPathInfo) -> Result<(url::Url, bool)> {
        if let Some(direct_url) = info.direct_url.as_deref().filter(|url| !url.is_empty()) {
            let url = url::Url::parse(direct_url).map_err(|_| {
                AppError::MetadataFetchError("Invalid CloudDrive direct download URL".to_string())
            })?;
            if url.username() != "" || url.password().is_some() {
                return Err(AppError::MetadataFetchError(
                    "CloudDrive direct download URL must not contain embedded credentials"
                        .to_string(),
                ));
            }
            if !matches!(url.scheme(), "http" | "https") {
                return Err(AppError::MetadataFetchError(
                    "Invalid CloudDrive direct download URL scheme".to_string(),
                ));
            }
            return Ok((url, true));
        }

        if !info.download_url_path.starts_with('/') {
            return Err(AppError::MetadataFetchError(
                "Invalid CloudDrive download URL path".to_string(),
            ));
        }

        let endpoint = url::Url::parse(&self.endpoint).map_err(|_| {
            AppError::MetadataFetchError("Invalid CloudDrive endpoint URL".to_string())
        })?;
        let origin = endpoint.origin().ascii_serialization();
        let host = origin
            .strip_prefix(&format!("{}://", endpoint.scheme()))
            .expect("HTTP(S) URL origin has its scheme prefix");
        let path = info
            .download_url_path
            .replace("{SCHEME}", endpoint.scheme())
            .replace("{HOST}", host)
            .replace("{PREVIEW}", "false");
        let url = url::Url::parse(&format!("{origin}{path}")).map_err(|_| {
            AppError::MetadataFetchError("Invalid CloudDrive download URL path".to_string())
        })?;
        Ok((url, false))
    }

    fn download_request(
        &self,
        info: &proto::DownloadUrlPathInfo,
    ) -> Result<(reqwest::Client, reqwest::Request)> {
        let (url, is_direct) = self.download_url(info)?;
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|_| {
                AppError::MetadataFetchError("Failed to create remote download client".to_string())
            })?;
        let mut headers = reqwest::header::HeaderMap::new();
        if is_direct {
            for (name, value) in &info.additional_headers {
                let name =
                    reqwest::header::HeaderName::from_bytes(name.as_bytes()).map_err(|_| {
                        AppError::MetadataFetchError(
                            "Invalid CloudDrive direct download header".to_string(),
                        )
                    })?;
                let value = reqwest::header::HeaderValue::from_str(value).map_err(|_| {
                    AppError::MetadataFetchError(
                        "Invalid CloudDrive direct download header".to_string(),
                    )
                })?;
                headers.insert(name, value);
            }
            if let Some(user_agent) = info.user_agent.as_deref().filter(|value| !value.is_empty()) {
                let value = reqwest::header::HeaderValue::from_str(user_agent).map_err(|_| {
                    AppError::MetadataFetchError(
                        "Invalid CloudDrive direct download user agent".to_string(),
                    )
                })?;
                headers.insert(reqwest::header::USER_AGENT, value);
            }
        }
        let request = client.get(url).headers(headers).build().map_err(|_| {
            AppError::MetadataFetchError("Failed to build remote download request".to_string())
        })?;
        Ok((client, request))
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
            .map_err(|error| rpc_error("GetToken", error))?
            .into_inner();

        ensure_success("Login", response.success, &response.error_message)?;

        self.token = Some(response.token.clone());
        Ok(response.token)
    }

    /// 添加离线下载任务
    async fn add_offline_files(&self, urls: Vec<String>, to_folder: &str) -> Result<()> {
        let channel = self.build_channel().await?;
        let mut client = proto::cloud_drive_file_srv_client::CloudDriveFileSrvClient::new(channel);
        let request = self.authenticated_request(proto::AddOfflineFileRequest {
            urls: urls.join("\n"),
            to_folder: to_folder.to_string(),
            check_folder_after_secs: None,
        })?;
        let response = client
            .add_offline_files(request)
            .await
            .map_err(|error| rpc_error("AddOfflineFiles", error))?;
        operation_result("AddOfflineFiles", &response.into_inner())
    }

    async fn list_folder(&self, path: &str) -> Result<Vec<proto::CloudDriveFile>> {
        self.list_folder_with_refresh(path, false).await
    }

    async fn list_folder_fresh(&self, path: &str) -> Result<Vec<proto::CloudDriveFile>> {
        self.list_folder_with_refresh(path, true).await
    }

    async fn list_offline_files_by_path(&self, path: &str) -> Result<Vec<proto::OfflineFile>> {
        let channel = self.build_channel().await?;
        let mut client = proto::cloud_drive_file_srv_client::CloudDriveFileSrvClient::new(channel);
        let request = self.authenticated_request(proto::FileRequest {
            path: path.to_string(),
            force_refresh: None,
        })?;
        let response = client
            .list_offline_files_by_path(request)
            .await
            .map_err(|error| rpc_error("ListOfflineFilesByPath", error))?;
        Ok(response.into_inner().offline_files)
    }

    async fn create_folder(
        &self,
        parent_path: &str,
        folder_name: &str,
    ) -> Result<proto::CloudDriveFile> {
        let channel = self.build_channel().await?;
        let mut client = proto::cloud_drive_file_srv_client::CloudDriveFileSrvClient::new(channel);
        let request = self.authenticated_request(proto::CreateFolderRequest {
            parent_path: parent_path.to_string(),
            folder_name: folder_name.to_string(),
        })?;
        let response = client
            .create_folder(request)
            .await
            .map_err(|error| rpc_error("CreateFolder", error))?
            .into_inner();
        let result = response.result.as_ref().ok_or_else(|| {
            AppError::MetadataFetchError("CreateFolder returned no operation result".to_string())
        })?;
        operation_result("CreateFolder", result)?;
        response.folder_created.ok_or_else(|| {
            AppError::MetadataFetchError(
                "CreateFolder succeeded without a created folder".to_string(),
            )
        })
    }

    async fn move_files(&self, paths: Vec<String>, destination: &str) -> Result<()> {
        let channel = self.build_channel().await?;
        let mut client = proto::cloud_drive_file_srv_client::CloudDriveFileSrvClient::new(channel);
        let request = self.authenticated_request(proto::MoveFileRequest {
            the_file_paths: paths,
            dest_path: destination.to_string(),
            conflict_policy: Some(proto::move_file_request::ConflictPolicy::Skip as i32),
            move_across_clouds: None,
            handle_conflict_recursively: None,
        })?;
        let response = client
            .move_file(request)
            .await
            .map_err(|error| rpc_error("MoveFile", error))?;
        operation_result("MoveFile", &response.into_inner())
    }

    async fn find_file_by_path(
        &self,
        parent_path: &str,
        path: &str,
    ) -> Result<proto::CloudDriveFile> {
        let channel = self.build_channel().await?;
        let mut client = proto::cloud_drive_file_srv_client::CloudDriveFileSrvClient::new(channel);
        let request = self.authenticated_request(proto::FindFileByPathRequest {
            parent_path: parent_path.to_string(),
            path: path.to_string(),
        })?;
        client
            .find_file_by_path(request)
            .await
            .map_err(|error| rpc_error("FindFileByPath", error))
            .map(tonic::Response::into_inner)
    }

    async fn delete_file(&self, path: &str) -> Result<()> {
        let channel = self.build_channel().await?;
        let mut client = proto::cloud_drive_file_srv_client::CloudDriveFileSrvClient::new(channel);
        let request = self.authenticated_request(proto::FileRequest {
            path: path.to_string(),
            force_refresh: None,
        })?;
        let response = client
            .delete_file(request)
            .await
            .map_err(|error| rpc_error("DeleteFile", error))?;
        operation_result("DeleteFile", &response.into_inner())
    }

    async fn sha256_file(&self, path: &str) -> Result<String> {
        let channel = self.build_channel().await?;
        let mut client = proto::cloud_drive_file_srv_client::CloudDriveFileSrvClient::new(channel);
        let request = self.authenticated_request(proto::GetDownloadUrlPathRequest {
            path: path.to_string(),
            preview: false,
            lazy_read: false,
            get_direct_url: true,
        })?;
        let info = client
            .get_download_url_path(request)
            .await
            .map_err(|error| rpc_error("GetDownloadUrlPath", error))?
            .into_inner();
        let (http, request) = self.download_request(&info)?;
        let mut response = http
            .execute(request)
            .await
            .map_err(|_| AppError::MetadataFetchError("Remote file download failed".to_string()))?;
        if !response.status().is_success() {
            return Err(AppError::MetadataFetchError(format!(
                "Remote file download failed with HTTP {}",
                response.status()
            )));
        }

        const HASH_CHUNK_SIZE: usize = 64 * 1024;
        let mut sha = Sha256::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| {
            AppError::MetadataFetchError("Remote file download stream failed".to_string())
        })? {
            for part in chunk.chunks(HASH_CHUNK_SIZE) {
                sha.update(part);
            }
        }
        Ok(format!("{:x}", sha.finalize()))
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

    #[test]
    fn operation_result_includes_server_error() {
        let error = operation_result(
            "MoveFile",
            &proto::FileOperationResult {
                success: false,
                error_message: "destination is read-only".to_string(),
                result_file_paths: Vec::new(),
            },
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("MoveFile failed: destination is read-only"));
    }

    struct LegacyClient;

    #[async_trait]
    impl CloudDriveClientTrait for LegacyClient {
        async fn login(&mut self, _: &str, _: &str) -> Result<String> {
            Ok(String::new())
        }

        async fn add_offline_files(&self, _: Vec<String>, _: &str) -> Result<()> {
            Ok(())
        }

        async fn list_folder(&self, path: &str) -> Result<Vec<proto::CloudDriveFile>> {
            Ok(vec![proto::CloudDriveFile {
                full_path_name: path.to_string(),
                ..Default::default()
            }])
        }
    }

    #[tokio::test]
    async fn legacy_clients_use_list_folder_for_fresh_listing() {
        let client = LegacyClient;
        let files = client.list_folder_fresh("/remote/anime").await.unwrap();
        assert_eq!(files[0].full_path_name, "/remote/anime");
    }

    #[tokio::test]
    async fn legacy_clients_reject_remote_file_hashing() {
        let error = LegacyClient
            .sha256_file("/remote/anime/episode.mkv")
            .await
            .unwrap_err();
        assert!(error.to_string().contains("SHA-256 file hashing"));
    }

    #[test]
    fn direct_download_request_applies_headers_and_user_agent() {
        let client = CloudDriveClient::new("https://api.example.com:8443", None).unwrap();
        let info = proto::DownloadUrlPathInfo {
            direct_url: Some("https://files.example.com/episode.mkv".to_string()),
            user_agent: Some("CloudDrive test agent".to_string()),
            additional_headers: [(
                "referer".to_string(),
                "https://app.example.com/".to_string(),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };

        let (_, request) = client.download_request(&info).unwrap();
        assert_eq!(
            request.url().as_str(),
            "https://files.example.com/episode.mkv"
        );
        assert_eq!(request.headers()["referer"], "https://app.example.com/");
        assert_eq!(
            request.headers()[reqwest::header::USER_AGENT],
            "CloudDrive test agent"
        );
    }

    #[test]
    fn endpoint_relative_download_request_replaces_placeholders() {
        let client = CloudDriveClient::new("https://api.example.com:8443", None).unwrap();
        let info = proto::DownloadUrlPathInfo {
            download_url_path: "/static/{SCHEME}/{HOST}/{PREVIEW}/episode.mkv?origin={HOST}"
                .to_string(),
            ..Default::default()
        };

        let (_, request) = client.download_request(&info).unwrap();
        assert_eq!(
            request.url().as_str(),
            "https://api.example.com:8443/static/https/api.example.com:8443/false/episode.mkv?origin=api.example.com:8443"
        );
        assert!(request.headers().is_empty());
    }

    /// Integration test for CloudDrive2 offline download
    /// Run with: CLOUDDRIVE_URL=http://... CLOUDDRIVE_TOKEN=... cargo test --features clouddrive clouddrive_offline_integration -- --nocapture
    #[tokio::test]
    #[ignore]
    async fn test_clouddrive_offline_integration() {
        let url = std::env::var("CLOUDDRIVE_URL").expect("CLOUDDRIVE_URL env var required");
        let token = std::env::var("CLOUDDRIVE_TOKEN").expect("CLOUDDRIVE_TOKEN env var required");

        let client =
            CloudDriveClient::new(&url, Some(token)).expect("Failed to create CloudDriveClient");

        let magnet = "magnet:?xt=urn:btih:47A7OI47YGU3GSDZFXHX4E6BBJKF4YAX";
        let target_folder = "/downloads/Ani";

        println!("Submitting magnet to CloudDrive2...");
        println!("  Target: {}", target_folder);

        client
            .add_offline_files(vec![magnet.to_string()], target_folder)
            .await
            .expect("CloudDrive2 offline download failed");
        println!("✅ Successfully submitted offline download!");
    }

    /// Test CloudDrive2 server connectivity
    /// Run with: CLOUDDRIVE_URL=http://... CLOUDDRIVE_TOKEN=... cargo test --features clouddrive clouddrive_connectivity -- --nocapture
    #[tokio::test]
    #[ignore]
    async fn test_clouddrive_connectivity() {
        let url = std::env::var("CLOUDDRIVE_URL").expect("CLOUDDRIVE_URL env var required");
        let token = std::env::var("CLOUDDRIVE_TOKEN").expect("CLOUDDRIVE_TOKEN env var required");

        let client =
            CloudDriveClient::new(&url, Some(token)).expect("Failed to create CloudDriveClient");

        let test_magnet = "magnet:?xt=urn:btih:d41d8cd98f00b204e9800998ecf8427e";

        client
            .add_offline_files(vec![test_magnet.to_string()], "/")
            .await
            .expect("CloudDrive2 connectivity test failed");
        println!("✅ CloudDrive2 server is reachable and authenticated!");
    }
}
