use super::client::{proto, unsupported_operation, CloudDriveClientTrait};
use crate::error::{AppError, Result};
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone)]
pub struct LocalStorageClient {
    root: PathBuf,
}

impl LocalStorageClient {
    pub fn new(root: &Path) -> Result<Self> {
        if !root.is_absolute() {
            return Err(AppError::MetadataFetchError(
                "Local storage root must be absolute".to_string(),
            ));
        }
        let root = root.canonicalize().map_err(|error| {
            AppError::MetadataFetchError(format!("Open local storage root failed: {error}"))
        })?;
        if !root.is_dir() {
            return Err(AppError::MetadataFetchError(
                "Local storage root must be a directory".to_string(),
            ));
        }
        Ok(Self { root })
    }

    fn path(&self, logical: &str) -> Result<PathBuf> {
        if !logical.starts_with('/') || logical.contains('\0') {
            return Err(AppError::MetadataFetchError(
                "Local storage path must be absolute and contain no NUL".to_string(),
            ));
        }
        let mut path = self.root.clone();
        for component in logical.split('/').filter(|component| !component.is_empty()) {
            if component == "."
                || component == ".."
                || component.contains(['/', '\\'])
                || component.contains([':', '\0'])
            {
                return Err(AppError::MetadataFetchError(
                    "Local storage path contains an unsafe component".to_string(),
                ));
            }
            path.push(component);
        }
        Ok(path)
    }

    fn existing_path(&self, logical: &str) -> Result<PathBuf> {
        let path = self.path(logical)?;
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
            AppError::MetadataFetchError(format!("Read local storage path failed: {error}"))
        })?;
        if metadata.file_type().is_symlink() {
            return Err(AppError::MetadataFetchError(
                "Local storage symlinks are not supported".to_string(),
            ));
        }
        let canonical = path.canonicalize().map_err(|error| {
            AppError::MetadataFetchError(format!("Resolve local storage path failed: {error}"))
        })?;
        if !canonical.starts_with(&self.root) {
            return Err(AppError::MetadataFetchError(
                "Local storage path escapes its configured root".to_string(),
            ));
        }
        Ok(canonical)
    }

    fn destination_path(&self, parent: &str, name: &str) -> Result<PathBuf> {
        if !safe_component(name) {
            return Err(AppError::MetadataFetchError(
                "Local destination name is unsafe".to_string(),
            ));
        }
        let parent = self.existing_path(parent)?;
        if !parent.is_dir() {
            return Err(AppError::MetadataFetchError(
                "Local destination parent is not a directory".to_string(),
            ));
        }
        Ok(parent.join(name))
    }
}

fn safe_component(value: &str) -> bool {
    !value.is_empty() && value != "." && value != ".." && !value.contains(['/', '\\', ':', '\0'])
}

fn join_logical(parent: &str, name: &str) -> String {
    if parent == "/" {
        format!("/{name}")
    } else {
        format!("{}/{name}", parent.trim_end_matches('/'))
    }
}

#[async_trait]
impl CloudDriveClientTrait for LocalStorageClient {
    async fn login(&mut self, _username: &str, _password: &str) -> Result<String> {
        Err(unsupported_operation("local login"))
    }

    async fn add_offline_files(&self, _urls: Vec<String>, _to_folder: &str) -> Result<()> {
        Err(unsupported_operation("local offline downloads"))
    }

    async fn list_folder(&self, path: &str) -> Result<Vec<proto::CloudDriveFile>> {
        let directory = self.existing_path(path)?;
        if !directory.is_dir() {
            return Err(AppError::MetadataFetchError(
                "Local storage path is not a directory".to_string(),
            ));
        }
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(directory).map_err(|error| {
            AppError::MetadataFetchError(format!("List local storage directory failed: {error}"))
        })? {
            let entry = entry.map_err(|error| {
                AppError::MetadataFetchError(format!("Read local storage entry failed: {error}"))
            })?;
            let name = entry.file_name().into_string().map_err(|_| {
                AppError::MetadataFetchError("Local storage filename is not UTF-8".to_string())
            })?;
            if !safe_component(&name) {
                return Err(AppError::MetadataFetchError(format!(
                    "Local storage returned an unsafe entry name: {name}"
                )));
            }
            let metadata = std::fs::symlink_metadata(entry.path()).map_err(|error| {
                AppError::MetadataFetchError(format!("Read local storage metadata failed: {error}"))
            })?;
            if metadata.file_type().is_symlink() {
                return Err(AppError::MetadataFetchError(format!(
                    "Local storage symlink is not supported: {name}"
                )));
            }
            let logical = join_logical(path, &name);
            entries.push(proto::CloudDriveFile {
                id: logical.clone(),
                name,
                full_path_name: logical,
                size: i64::try_from(metadata.len()).unwrap_or(i64::MAX),
                file_type: if metadata.is_dir() { 0 } else { 1 },
                is_directory: metadata.is_dir(),
                ..Default::default()
            });
        }
        Ok(entries)
    }

    async fn create_folder(
        &self,
        parent_path: &str,
        folder_name: &str,
    ) -> Result<proto::CloudDriveFile> {
        let path = self.destination_path(parent_path, folder_name)?;
        std::fs::create_dir(&path).map_err(|error| {
            AppError::MetadataFetchError(format!("Create local storage directory failed: {error}"))
        })?;
        let logical = join_logical(parent_path, folder_name);
        Ok(proto::CloudDriveFile {
            id: logical.clone(),
            name: folder_name.to_string(),
            full_path_name: logical,
            is_directory: true,
            ..Default::default()
        })
    }

    async fn move_files(&self, paths: Vec<String>, destination: &str) -> Result<()> {
        for path in paths {
            let name = path
                .rsplit('/')
                .find(|part| !part.is_empty())
                .ok_or_else(|| {
                    AppError::MetadataFetchError("Local source path has no filename".to_string())
                })?;
            let source = self.existing_path(&path)?;
            let target = self.destination_path(destination, name)?;
            if target.exists() {
                return Err(AppError::MetadataFetchError(format!(
                    "Local destination already exists: {}",
                    target.display()
                )));
            }
            std::fs::rename(source, target).map_err(|error| {
                AppError::MetadataFetchError(format!("Move local storage file failed: {error}"))
            })?;
        }
        Ok(())
    }

    async fn copy_files(&self, paths: Vec<String>, destination: &str) -> Result<()> {
        for path in paths {
            let name = path
                .rsplit('/')
                .find(|part| !part.is_empty())
                .ok_or_else(|| {
                    AppError::MetadataFetchError("Local source path has no filename".to_string())
                })?;
            let source = self.existing_path(&path)?;
            let target = self.destination_path(destination, name)?;
            let mut input = tokio::fs::File::open(source).await.map_err(|error| {
                AppError::MetadataFetchError(format!("Open local copy source failed: {error}"))
            })?;
            let mut output = tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(target)
                .await
                .map_err(|error| {
                    AppError::MetadataFetchError(format!(
                        "Create local copy target failed: {error}"
                    ))
                })?;
            tokio::io::copy(&mut input, &mut output)
                .await
                .map_err(|error| {
                    AppError::MetadataFetchError(format!("Copy local storage file failed: {error}"))
                })?;
            output.flush().await.map_err(|error| {
                AppError::MetadataFetchError(format!("Flush local copy target failed: {error}"))
            })?;
        }
        Ok(())
    }

    async fn delete_file(&self, path: &str) -> Result<()> {
        let path = self.existing_path(path)?;
        if path.is_dir() {
            std::fs::remove_dir(path)
        } else {
            std::fs::remove_file(path)
        }
        .map_err(|error| {
            AppError::MetadataFetchError(format!("Delete local storage path failed: {error}"))
        })
    }

    async fn download_file(&self, path: &str, destination: &Path) -> Result<()> {
        let source = self.existing_path(path)?;
        tokio::fs::copy(source, destination)
            .await
            .map_err(|error| {
                AppError::MetadataFetchError(format!(
                    "Copy local source to staging failed: {error}"
                ))
            })?;
        Ok(())
    }

    async fn upload_file(&self, parent: &str, name: &str, source: &Path) -> Result<()> {
        let target = self.destination_path(parent, name)?;
        let mut input = tokio::fs::File::open(source).await.map_err(|error| {
            AppError::MetadataFetchError(format!("Open local upload source failed: {error}"))
        })?;
        let mut output = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target)
            .await
            .map_err(|error| {
                AppError::MetadataFetchError(format!("Create local upload target failed: {error}"))
            })?;
        tokio::io::copy(&mut input, &mut output)
            .await
            .map_err(|error| {
                AppError::MetadataFetchError(format!("Upload to local storage failed: {error}"))
            })?;
        output.flush().await.map_err(|error| {
            AppError::MetadataFetchError(format!("Flush local upload target failed: {error}"))
        })
    }

    async fn rename_file(&self, path: &str, new_name: &str) -> Result<()> {
        let source = self.existing_path(path)?;
        let parent =
            path.rsplit_once('/').map_or(
                "/",
                |(parent, _)| if parent.is_empty() { "/" } else { parent },
            );
        let target = self.destination_path(parent, new_name)?;
        if target.exists() {
            return Err(AppError::MetadataFetchError(format!(
                "Local rename target already exists: {}",
                target.display()
            )));
        }
        std::fs::rename(source, target).map_err(|error| {
            AppError::MetadataFetchError(format!("Rename local storage path failed: {error}"))
        })
    }

    async fn sha256_file(&self, path: &str) -> Result<String> {
        let path = self.existing_path(path)?;
        let mut file = tokio::fs::File::open(path).await.map_err(|error| {
            AppError::MetadataFetchError(format!("Open local storage hash source failed: {error}"))
        })?;
        let mut sha = Sha256::new();
        let mut buffer = vec![0_u8; 64 * 1024];
        loop {
            let length = file.read(&mut buffer).await.map_err(|error| {
                AppError::MetadataFetchError(format!(
                    "Read local storage hash source failed: {error}"
                ))
            })?;
            if length == 0 {
                break;
            }
            sha.update(&buffer[..length]);
        }
        Ok(format!("{:x}", sha.finalize()))
    }
}

#[cfg(test)]
mod tests {
    use super::LocalStorageClient;
    use crate::rss::client::CloudDriveClientTrait;

    #[tokio::test]
    async fn local_storage_round_trip_rejects_overwrite() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.bin");
        std::fs::write(&source, b"bytes").unwrap();
        let client = LocalStorageClient::new(directory.path()).unwrap();
        client
            .upload_file("/", "target.bin", &source)
            .await
            .unwrap();
        assert!(client
            .upload_file("/", "target.bin", &source)
            .await
            .is_err());
        client
            .rename_file("/target.bin", "final.bin")
            .await
            .unwrap();
        assert_eq!(
            std::fs::read(directory.path().join("final.bin")).unwrap(),
            b"bytes"
        );
        client.delete_file("/final.bin").await.unwrap();
    }
}
