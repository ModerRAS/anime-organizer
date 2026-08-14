use super::client::{proto, unsupported_operation, CloudDriveClientTrait};
use crate::error::{AppError, Result};
use async_trait::async_trait;
use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest::{Method, StatusCode};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

#[derive(Debug, Clone)]
pub struct WebDavClient {
    base_url: url::Url,
    username: Option<String>,
    password: Option<String>,
    client: reqwest::Client,
}

impl WebDavClient {
    pub fn new(url: &str, username: Option<String>, password: Option<String>) -> Result<Self> {
        let mut base_url = url::Url::parse(url).map_err(|error| {
            AppError::MetadataFetchError(format!("Invalid WebDAV URL: {error}"))
        })?;
        if !matches!(base_url.scheme(), "http" | "https")
            || base_url.host_str().is_none()
            || !base_url.username().is_empty()
            || base_url.password().is_some()
            || base_url.query().is_some()
            || base_url.fragment().is_some()
        {
            return Err(AppError::MetadataFetchError(
                "WebDAV URL must be an HTTP(S) endpoint without embedded credentials, query, or fragment"
                    .to_string(),
            ));
        }
        if username.is_some() != password.is_some() {
            return Err(AppError::MetadataFetchError(
                "WebDAV username and password must be provided together".to_string(),
            ));
        }
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(300))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| {
                AppError::MetadataFetchError(format!("WebDAV client failed: {error}"))
            })?;
        Ok(Self {
            base_url,
            username,
            password,
            client,
        })
    }

    fn url(&self, path: &str) -> Result<url::Url> {
        if !path.starts_with('/') || path.contains('\0') {
            return Err(AppError::MetadataFetchError(
                "WebDAV path must be absolute and contain no NUL".to_string(),
            ));
        }
        let components = path
            .split('/')
            .filter(|component| !component.is_empty())
            .collect::<Vec<_>>();
        let mut url = self.base_url.clone();
        if !components.is_empty() {
            let mut segments = url.path_segments_mut().map_err(|_| {
                AppError::MetadataFetchError("WebDAV base URL cannot be a base".to_string())
            })?;
            segments.pop_if_empty();
            for component in components {
                if component == "." || component == ".." || component.contains(['/', '\\']) {
                    return Err(AppError::MetadataFetchError(
                        "WebDAV path contains an unsafe component".to_string(),
                    ));
                }
                segments.push(component);
            }
        }
        Ok(url)
    }

    fn request(&self, method: Method, url: url::Url) -> reqwest::RequestBuilder {
        let request = self.client.request(method, url);
        match (&self.username, &self.password) {
            (Some(username), Some(password)) => request.basic_auth(username, Some(password)),
            _ => request,
        }
    }

    async fn ensure_status(
        &self,
        operation: &str,
        response: reqwest::Response,
        accepted: &[StatusCode],
    ) -> Result<reqwest::Response> {
        if accepted.contains(&response.status()) {
            return Ok(response);
        }
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(AppError::MetadataFetchError(format!(
            "WebDAV {operation} failed with {status}: {}",
            body.chars().take(300).collect::<String>()
        )))
    }

    async fn propfind(&self, path: &str, depth: &str) -> Result<Vec<WebDavEntry>> {
        let method = Method::from_bytes(b"PROPFIND").expect("valid WebDAV method");
        let response = self
            .request(method, self.url(path)?)
            .header("Depth", depth)
            .header("Content-Type", "application/xml; charset=utf-8")
            .body("<?xml version=\"1.0\" encoding=\"utf-8\"?><d:propfind xmlns:d=\"DAV:\"><d:prop><d:resourcetype/><d:getcontentlength/><d:getlastmodified/></d:prop></d:propfind>")
            .send()
            .await
            .map_err(|error| AppError::MetadataFetchError(format!("WebDAV PROPFIND failed: {error}")))?;
        let response = self
            .ensure_status("PROPFIND", response, &[StatusCode::MULTI_STATUS])
            .await?;
        parse_multistatus(&response.bytes().await.map_err(|error| {
            AppError::MetadataFetchError(format!("WebDAV PROPFIND body failed: {error}"))
        })?)
    }

    async fn copy_or_move(
        &self,
        method: Method,
        paths: Vec<String>,
        destination: &str,
    ) -> Result<()> {
        for path in paths {
            let name = path
                .rsplit('/')
                .find(|part| !part.is_empty())
                .ok_or_else(|| {
                    AppError::MetadataFetchError("WebDAV source path has no file name".to_string())
                })?;
            let destination_path = format!("{}/{}", destination.trim_end_matches('/'), name);
            let destination_url = self.url(&destination_path)?;
            let response = self
                .request(method.clone(), self.url(&path)?)
                .header("Destination", destination_url.as_str())
                .header("Overwrite", "F")
                .send()
                .await
                .map_err(|error| {
                    AppError::MetadataFetchError(format!("WebDAV transfer failed: {error}"))
                })?;
            self.ensure_status(
                "transfer",
                response,
                &[StatusCode::CREATED, StatusCode::NO_CONTENT],
            )
            .await?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct WebDavEntry {
    href: String,
    size: i64,
    is_directory: bool,
}

fn parse_multistatus(xml: &[u8]) -> Result<Vec<WebDavEntry>> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut entries = Vec::new();
    let mut in_response = false;
    let mut current_tag = Vec::new();
    let mut href = None;
    let mut size = 0_i64;
    let mut is_directory = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => {
                let name = event.local_name().as_ref().to_vec();
                if name == b"response" {
                    in_response = true;
                    href = None;
                    size = 0;
                    is_directory = false;
                } else if in_response {
                    if name == b"collection" {
                        is_directory = true;
                    }
                    current_tag = name;
                }
            }
            Ok(Event::Empty(event)) if in_response => {
                if event.local_name().as_ref() == b"collection" {
                    is_directory = true;
                }
            }
            Ok(Event::Text(text)) if in_response => {
                let value = text.unescape().map_err(|error| {
                    AppError::MetadataFetchError(format!("Invalid WebDAV XML text: {error}"))
                })?;
                match current_tag.as_slice() {
                    b"href" => href = Some(value.into_owned()),
                    b"getcontentlength" => size = value.parse().unwrap_or(0),
                    _ => {}
                }
            }
            Ok(Event::End(event)) => {
                if event.local_name().as_ref() == b"response" {
                    if let Some(href) = href.take() {
                        entries.push(WebDavEntry {
                            href,
                            size,
                            is_directory,
                        });
                    }
                    in_response = false;
                }
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(error) => {
                return Err(AppError::MetadataFetchError(format!(
                    "Invalid WebDAV multistatus XML: {error}"
                )))
            }
            _ => {}
        }
    }
    Ok(entries)
}

fn href_path(href: &str) -> Result<String> {
    let path = if let Ok(url) = url::Url::parse(href) {
        url.path().to_string()
    } else {
        href.split('?').next().unwrap_or(href).to_string()
    };
    percent_decode_path(&path)
}

fn percent_decode_path(path: &str) -> Result<String> {
    urlencoding::decode(path)
        .map(|value| value.into_owned())
        .map_err(|_| AppError::MetadataFetchError("Invalid UTF-8 WebDAV href".to_string()))
}

#[async_trait]
impl CloudDriveClientTrait for WebDavClient {
    async fn login(&mut self, _username: &str, _password: &str) -> Result<String> {
        Err(unsupported_operation("WebDAV login token exchange"))
    }

    async fn add_offline_files(&self, _urls: Vec<String>, _to_folder: &str) -> Result<()> {
        Err(unsupported_operation("WebDAV offline downloads"))
    }

    async fn list_folder(&self, path: &str) -> Result<Vec<proto::CloudDriveFile>> {
        let requested_url_path = href_path(self.url(path)?.path())?
            .trim_end_matches('/')
            .to_string();
        let mut files = Vec::new();
        for entry in self.propfind(path, "1").await? {
            let full_path = href_path(&entry.href)?;
            let entry_url_path = full_path.trim_end_matches('/');
            if entry_url_path == requested_url_path {
                continue;
            }
            let expected_prefix = format!("{requested_url_path}/");
            let relative = entry_url_path
                .strip_prefix(&expected_prefix)
                .ok_or_else(|| {
                    AppError::MetadataFetchError(format!(
                        "WebDAV returned an entry outside the requested directory: {full_path}"
                    ))
                })?;
            if relative.is_empty() || relative.contains('/') {
                return Err(AppError::MetadataFetchError(format!(
                    "WebDAV returned a non-child entry for the requested directory: {full_path}"
                )));
            }
            let name = relative.to_string();
            let logical_path = format!("{}/{}", path.trim_end_matches('/'), name);
            files.push(proto::CloudDriveFile {
                id: logical_path.clone(),
                name,
                full_path_name: logical_path,
                size: entry.size,
                file_type: if entry.is_directory { 0 } else { 1 },
                is_directory: entry.is_directory,
                ..Default::default()
            });
        }
        Ok(files)
    }

    async fn create_folder(
        &self,
        parent_path: &str,
        folder_name: &str,
    ) -> Result<proto::CloudDriveFile> {
        let path = format!("{}/{}", parent_path.trim_end_matches('/'), folder_name);
        let method = Method::from_bytes(b"MKCOL").expect("valid WebDAV method");
        let response = self
            .request(method, self.url(&path)?)
            .send()
            .await
            .map_err(|error| {
                AppError::MetadataFetchError(format!("WebDAV MKCOL failed: {error}"))
            })?;
        self.ensure_status("MKCOL", response, &[StatusCode::CREATED])
            .await?;
        Ok(proto::CloudDriveFile {
            id: path.clone(),
            name: folder_name.to_string(),
            full_path_name: path,
            is_directory: true,
            ..Default::default()
        })
    }

    async fn move_files(&self, paths: Vec<String>, destination: &str) -> Result<()> {
        self.copy_or_move(
            Method::from_bytes(b"MOVE").expect("valid WebDAV method"),
            paths,
            destination,
        )
        .await
    }

    async fn copy_files(&self, paths: Vec<String>, destination: &str) -> Result<()> {
        self.copy_or_move(
            Method::from_bytes(b"COPY").expect("valid WebDAV method"),
            paths,
            destination,
        )
        .await
    }

    async fn delete_file(&self, path: &str) -> Result<()> {
        let response = self
            .request(Method::DELETE, self.url(path)?)
            .send()
            .await
            .map_err(|error| {
                AppError::MetadataFetchError(format!("WebDAV DELETE failed: {error}"))
            })?;
        self.ensure_status(
            "DELETE",
            response,
            &[StatusCode::NO_CONTENT, StatusCode::OK],
        )
        .await?;
        Ok(())
    }

    async fn download_file(&self, path: &str, destination: &Path) -> Result<()> {
        let response = self
            .request(Method::GET, self.url(path)?)
            .send()
            .await
            .map_err(|error| AppError::MetadataFetchError(format!("WebDAV GET failed: {error}")))?;
        let mut response = self
            .ensure_status("GET", response, &[StatusCode::OK])
            .await?;
        let mut file = tokio::fs::File::create(destination)
            .await
            .map_err(|error| {
                AppError::MetadataFetchError(format!(
                    "Create WebDAV download destination failed: {error}"
                ))
            })?;
        while let Some(chunk) = response.chunk().await.map_err(|error| {
            AppError::MetadataFetchError(format!("WebDAV GET stream failed: {error}"))
        })? {
            file.write_all(&chunk).await.map_err(|error| {
                AppError::MetadataFetchError(format!("Write WebDAV download failed: {error}"))
            })?;
        }
        file.flush().await.map_err(|error| {
            AppError::MetadataFetchError(format!("Flush WebDAV download failed: {error}"))
        })
    }

    async fn upload_file(&self, parent: &str, name: &str, source: &Path) -> Result<()> {
        let file = tokio::fs::File::open(source).await.map_err(|error| {
            AppError::MetadataFetchError(format!("Open WebDAV upload source failed: {error}"))
        })?;
        let size = file
            .metadata()
            .await
            .map_err(|error| {
                AppError::MetadataFetchError(format!("Read WebDAV upload metadata failed: {error}"))
            })?
            .len();
        let path = format!("{}/{}", parent.trim_end_matches('/'), name);
        let response = self
            .request(Method::PUT, self.url(&path)?)
            .header(reqwest::header::CONTENT_LENGTH, size)
            .body(reqwest::Body::wrap_stream(ReaderStream::new(file)))
            .send()
            .await
            .map_err(|error| AppError::MetadataFetchError(format!("WebDAV PUT failed: {error}")))?;
        self.ensure_status(
            "PUT",
            response,
            &[StatusCode::CREATED, StatusCode::NO_CONTENT, StatusCode::OK],
        )
        .await?;
        Ok(())
    }

    async fn rename_file(&self, path: &str, new_name: &str) -> Result<()> {
        let parent = path.rsplit_once('/').map_or("/", |(parent, _)| parent);
        let current_name = path.rsplit('/').next().unwrap_or_default();
        if current_name == new_name {
            return Ok(());
        }
        let destination = format!("{}/{}", parent.trim_end_matches('/'), new_name);
        let response = self
            .request(
                Method::from_bytes(b"MOVE").expect("valid WebDAV method"),
                self.url(path)?,
            )
            .header("Destination", self.url(&destination)?.as_str())
            .header("Overwrite", "F")
            .send()
            .await
            .map_err(|error| {
                AppError::MetadataFetchError(format!("WebDAV rename failed: {error}"))
            })?;
        self.ensure_status(
            "rename",
            response,
            &[StatusCode::CREATED, StatusCode::NO_CONTENT],
        )
        .await?;
        Ok(())
    }

    async fn sha256_file(&self, path: &str) -> Result<String> {
        let response = self
            .request(Method::GET, self.url(path)?)
            .send()
            .await
            .map_err(|error| {
                AppError::MetadataFetchError(format!("WebDAV hash GET failed: {error}"))
            })?;
        let mut response = self
            .ensure_status("hash GET", response, &[StatusCode::OK])
            .await?;
        let mut sha = Sha256::new();
        while let Some(chunk) = response.chunk().await.map_err(|error| {
            AppError::MetadataFetchError(format!("WebDAV hash stream failed: {error}"))
        })? {
            sha.update(&chunk);
        }
        Ok(format!("{:x}", sha.finalize()))
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_multistatus, percent_decode_path, WebDavClient};
    use crate::rss::client::CloudDriveClientTrait;
    use axum::body::Bytes;
    use axum::extract::State;
    use axum::http::{HeaderMap, Method, StatusCode, Uri};
    use axum::response::Response;
    use axum::routing::any;
    use axum::Router;
    use sha2::{Digest, Sha256};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    type Files = Arc<Mutex<HashMap<String, Vec<u8>>>>;

    async fn webdav_handler(
        State(files): State<Files>,
        method: Method,
        uri: Uri,
        headers: HeaderMap,
        body: Bytes,
    ) -> Response {
        let path = percent_decode_path(uri.path()).unwrap();
        match method.as_str() {
            "PROPFIND" => {
                let entries = files
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|(entry, _)| entry.rsplit_once('/').map(|(parent, _)| parent) == Some(path.trim_end_matches('/')))
                    .map(|(entry, bytes)| format!("<d:response><d:href>{}</d:href><d:propstat><d:prop><d:resourcetype/><d:getcontentlength>{}</d:getcontentlength></d:prop></d:propstat></d:response>", entry.replace(' ', "%20"), bytes.len()))
                    .collect::<String>();
                let body = format!("<?xml version=\"1.0\"?><d:multistatus xmlns:d=\"DAV:\"><d:response><d:href>{}/</d:href><d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop></d:propstat></d:response>{entries}</d:multistatus>", path.trim_end_matches('/'));
                Response::builder()
                    .status(StatusCode::MULTI_STATUS)
                    .body(body.into())
                    .unwrap()
            }
            "PUT" => {
                files.lock().unwrap().insert(path, body.to_vec());
                Response::builder()
                    .status(StatusCode::CREATED)
                    .body("".into())
                    .unwrap()
            }
            "GET" => match files.lock().unwrap().get(&path).cloned() {
                Some(bytes) => Response::builder()
                    .status(StatusCode::OK)
                    .body(bytes.into())
                    .unwrap(),
                None => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body("".into())
                    .unwrap(),
            },
            "MOVE" => {
                let destination = headers
                    .get("destination")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| url::Url::parse(value).ok())
                    .and_then(|url| percent_decode_path(url.path()).ok())
                    .unwrap();
                let bytes = files.lock().unwrap().remove(&path).unwrap();
                files.lock().unwrap().insert(destination, bytes);
                Response::builder()
                    .status(StatusCode::CREATED)
                    .body("".into())
                    .unwrap()
            }
            "DELETE" => {
                files.lock().unwrap().remove(&path);
                Response::builder()
                    .status(StatusCode::NO_CONTENT)
                    .body("".into())
                    .unwrap()
            }
            _ => Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body("".into())
                .unwrap(),
        }
    }

    #[tokio::test]
    async fn webdav_round_trip_uses_encoded_paths_and_standard_methods() {
        let files = Files::default();
        files
            .lock()
            .unwrap()
            .insert("/dav/anime/Existing File.mkv".to_string(), b"old".to_vec());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = Router::new()
            .fallback(any(webdav_handler))
            .with_state(files.clone());
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let client = WebDavClient::new(&format!("http://{address}/dav"), None, None).unwrap();

        let listed = client.list_folder("/anime").await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "Existing File.mkv");
        let local = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(local.path(), b"new bytes").unwrap();
        client
            .upload_file("/anime", "New File.mkv", local.path())
            .await
            .unwrap();
        assert_eq!(
            client.sha256_file("/anime/New File.mkv").await.unwrap(),
            format!("{:x}", Sha256::digest(b"new bytes"))
        );
        client
            .rename_file("/anime/New File.mkv", "Final File.mkv")
            .await
            .unwrap();
        let downloaded = tempfile::NamedTempFile::new().unwrap();
        client
            .download_file("/anime/Final File.mkv", downloaded.path())
            .await
            .unwrap();
        assert_eq!(std::fs::read(downloaded.path()).unwrap(), b"new bytes");
        client.delete_file("/anime/Final File.mkv").await.unwrap();
        assert!(!files
            .lock()
            .unwrap()
            .contains_key("/dav/anime/Final File.mkv"));
        server.abort();
    }

    #[test]
    fn parses_namespaced_multistatus() {
        let xml = br#"<?xml version="1.0"?><d:multistatus xmlns:d="DAV:"><d:response><d:href>/dav/anime/</d:href><d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop></d:propstat></d:response><d:response><d:href>/dav/anime/Episode%2001%20&amp;%2002.mkv</d:href><d:propstat><d:prop><d:resourcetype/><d:getcontentlength>42</d:getcontentlength></d:prop></d:propstat></d:response></d:multistatus>"#;
        let entries = parse_multistatus(xml).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries[0].is_directory);
        assert_eq!(entries[1].size, 42);
        assert_eq!(
            percent_decode_path(&entries[1].href).unwrap(),
            "/dav/anime/Episode 01 & 02.mkv"
        );
    }
}
