use anime_organizer::rss::client::{proto, CloudDriveClientTrait};
use anime_organizer::rss::db::{RssDatabase, Subscription};
use anime_organizer::{organize_directory_components, FileOrganizer, FilenameParser};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Serialize)]
pub(crate) struct RemoteOrganizeSummary {
    pub(crate) finished_directories: usize,
    pub(crate) moved_media: usize,
    pub(crate) moved_files: usize,
    pub(crate) skipped_conflicts: usize,
    pub(crate) removed_empty_directories: usize,
    pub(crate) uncorrelated_legacy_tasks: i64,
}

pub(crate) async fn organize_subscription(
    db: &RssDatabase,
    subscription: &Subscription,
    client: &dyn CloudDriveClientTrait,
) -> Result<RemoteOrganizeSummary, String> {
    let target = subscription
        .organize_target_folder
        .as_deref()
        .filter(|target| !target.trim().is_empty())
        .ok_or_else(|| "remote organize target is missing".to_string())?;
    let source_root = canonical_remote_path(&subscription.target_folder)
        .ok_or_else(|| "RSS source must be an absolute remote path".to_string())?;
    let target_root = canonical_remote_path(target)
        .ok_or_else(|| "remote organize target must be an absolute remote path".to_string())?;
    validate_remote_organize_paths(&source_root, &target_root)?;

    let tasks = db
        .list_download_tasks(subscription.id, None)
        .map_err(|error| error.to_string())?;
    let mut tasks_by_hash: HashMap<String, Vec<_>> = HashMap::new();
    for task in &tasks {
        if let Some(hash) = task.info_hash.as_deref().and_then(normalize_info_hash) {
            tasks_by_hash.entry(hash).or_default().push(task);
        }
    }
    let uncorrelated_legacy_tasks = db
        .count_uncorrelated_download_tasks(subscription.id)
        .map_err(|error| error.to_string())?;

    // This is the only offline-task query for a job. It is a status lookup,
    // not a filesystem listing of the RSS source root.
    let offline_files = client
        .list_offline_files_by_path(&source_root)
        .await
        .map_err(|error| error.to_string())?;
    let mut offline_hash_counts = HashMap::new();
    for offline in &offline_files {
        if let Some(hash) = normalize_info_hash(&offline.info_hash) {
            *offline_hash_counts.entry(hash).or_insert(0usize) += 1;
        }
    }
    let mut finished_roots = Vec::new();
    for offline in offline_files {
        let Some(info_hash) = normalize_info_hash(&offline.info_hash) else {
            continue;
        };
        if offline_hash_counts.get(&info_hash) != Some(&1) {
            continue;
        }
        // CloudDrive exposes no immutable per-submission identifier in the
        // persisted task. A duplicate v1 BTIH in one subscription is therefore
        // ambiguous and must remain untouched rather than moving either root.
        let Some([task]) = tasks_by_hash.get(&info_hash).map(Vec::as_slice) else {
            continue;
        };
        let was_completed = task.status.as_deref() == Some("completed");
        let status = offline_status_name(offline.status);
        if !db
            .reconcile_download_task(task.id, subscription.id, &info_hash, status, &offline.name)
            .map_err(|error| error.to_string())?
        {
            continue;
        }
        if offline.status == proto::OfflineFileStatus::OfflineFinished as i32
            && !was_completed
            && safe_component(&offline.name)
        {
            finished_roots.push((offline.name, task.id, info_hash));
        }
    }
    // A successful reconciliation advances only the organization schedule.
    db.mark_subscription_organize_checked(subscription.id)
        .map_err(|error| error.to_string())?;

    let mut summary = RemoteOrganizeSummary {
        finished_directories: finished_roots.len(),
        uncorrelated_legacy_tasks,
        ..RemoteOrganizeSummary::default()
    };
    for (offline_name, task_id, info_hash) in finished_roots {
        let root = join_remote_path(&source_root, &offline_name);
        let root_entry = client
            .find_file_by_path(&source_root, &offline_name)
            .await
            .map_err(|error| error.to_string())?;
        let tree = if root_entry.is_directory {
            list_tree(client, &root).await?
        } else {
            RemoteTree {
                files: vec![RemoteEntry {
                    path: root.clone(),
                    size: root_entry.size,
                }],
            }
        };
        let subtitle_candidates = tree
            .files
            .iter()
            .filter(|file| is_external_subtitle(&file.path))
            .map(|file| PathBuf::from(&file.path))
            .collect::<Vec<_>>();
        let sizes_by_path = tree
            .files
            .iter()
            .map(|file| (file.path.as_str(), file.size))
            .collect::<HashMap<_, _>>();
        let mut claimed_subtitles = HashSet::new();
        let mut groups = Vec::new();
        let mut invalid_root = false;

        // Every file in a completed torrent root must belong to exactly one
        // media bundle. Otherwise a retry preserves the entire root.
        for media in tree
            .files
            .iter()
            .filter(|file| !is_external_subtitle(&file.path))
        {
            let Some(anime) = FilenameParser::parse(Path::new(&media.path)) else {
                invalid_root = true;
                break;
            };
            let components =
                organize_directory_components(&anime, subscription.organize_season_mode);
            if components
                .iter()
                .any(|component| !safe_component(component))
            {
                invalid_root = true;
                break;
            }
            let subtitles = FileOrganizer::find_external_subtitles_from(
                Path::new(&media.path),
                &subtitle_candidates,
            );
            if subtitles
                .iter()
                .any(|path| !claimed_subtitles.insert(path.clone()))
            {
                invalid_root = true;
                break;
            }
            let mut files = subtitles
                .iter()
                .filter_map(|path| {
                    path.to_str().map(|path| RemoteEntry {
                        path: path.to_string(),
                        size: *sizes_by_path.get(path).unwrap_or(&-1),
                    })
                })
                .collect::<Vec<_>>();
            files.push(media.clone());
            groups.push((destination_path(&target_root, &components), files));
        }
        if claimed_subtitles.len() != subtitle_candidates.len() {
            invalid_root = true;
        }
        if invalid_root || groups.is_empty() {
            continue;
        }

        // Resolve every bundle before issuing a mutation. A hash or transport
        // failure therefore leaves this root untouched.
        let mut planned = Vec::new();
        let mut conflicted = false;
        for (destination, files) in groups {
            let existing = inspect_destination(client, &destination).await?;
            let names = files
                .iter()
                .map(|file| {
                    remote_file_name(&file.path)
                        .map(str::to_string)
                        .ok_or_else(|| "remote source path has no filename".to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            let conflict_count = names
                .iter()
                .filter(|name| existing.contains_key(*name))
                .count();
            let action = if conflict_count == 0 {
                GroupAction::Move
            } else if conflict_count == files.len()
                && files.iter().zip(&names).all(|(file, name)| {
                    file.size >= 0 && existing.get(name).is_some_and(|size| *size == file.size)
                })
            {
                let mut hashes_match = true;
                for (file, name) in files.iter().zip(&names) {
                    // The only remote reads: one streaming SHA-256 for each
                    // side of a same-name, same-size conflict.
                    let source_hash = client
                        .sha256_file(&file.path)
                        .await
                        .map_err(|error| error.to_string())?;
                    let target_hash = client
                        .sha256_file(&join_remote_path(&destination, name))
                        .await
                        .map_err(|error| error.to_string())?;
                    if source_hash != target_hash {
                        hashes_match = false;
                        break;
                    }
                }
                if hashes_match {
                    GroupAction::DeleteDuplicates
                } else {
                    summary.skipped_conflicts += 1;
                    conflicted = true;
                    continue;
                }
            } else {
                // Mixed bundles and unequal-size collisions preserve both
                // sides without opening a remote download stream.
                summary.skipped_conflicts += 1;
                conflicted = true;
                continue;
            };
            planned.push(PlannedGroup {
                destination,
                files,
                names,
                action,
            });
        }
        if conflicted || planned.is_empty() {
            continue;
        }
        let mut planned_destination_names = HashSet::new();
        if planned.iter().any(|group| {
            group.names.iter().any(|name| {
                !planned_destination_names.insert((group.destination.as_str(), name.as_str()))
            })
        }) {
            summary.skipped_conflicts += 1;
            continue;
        }

        // Re-list each destination immediately before the first mutation so a
        // stale preflight cannot overwrite a newly-created destination file.
        for group in planned
            .iter()
            .filter(|group| group.action == GroupAction::Move)
        {
            ensure_directory(client, &group.destination).await?;
            let existing = inspect_destination(client, &group.destination).await?;
            if group.names.iter().any(|name| existing.contains_key(name)) {
                return Err("remote destination changed during organization".to_string());
            }
        }

        for group in planned
            .iter()
            .filter(|group| group.action == GroupAction::Move)
        {
            client
                .move_files(
                    group.files.iter().map(|file| file.path.clone()).collect(),
                    &group.destination,
                )
                .await
                .map_err(|error| error.to_string())?;
            if !group_was_moved(client, group).await? {
                return Err("CloudDrive did not move every planned bundle member".to_string());
            }
            summary.moved_media += 1;
            summary.moved_files += group.files.len();
        }
        for group in planned
            .iter()
            .filter(|group| group.action == GroupAction::DeleteDuplicates)
        {
            for file in &group.files {
                // DeleteFile is CloudDrive's non-permanent delete. It is only
                // used after both full hashes proved the target is identical.
                client
                    .delete_file(&file.path)
                    .await
                    .map_err(|error| error.to_string())?;
            }
            summary.moved_media += 1;
            summary.moved_files += group.files.len();
        }

        if subscription.remove_empty_dirs && root_entry.is_directory {
            // Re-list the whole source tree after every move. Delete only the
            // completed torrent root when no file remains anywhere below it.
            if list_tree(client, &root).await?.files.is_empty() {
                client
                    .delete_file(&root)
                    .await
                    .map_err(|error| error.to_string())?;
                summary.removed_empty_directories += 1;
            }
        }
        db.complete_download_task(task_id, subscription.id, &info_hash)
            .map_err(|error| error.to_string())?;
    }
    Ok(summary)
}

struct RemoteTree {
    files: Vec<RemoteEntry>,
}

#[derive(Clone)]
struct RemoteEntry {
    path: String,
    size: i64,
}

struct PlannedGroup {
    destination: String,
    files: Vec<RemoteEntry>,
    names: Vec<String>,
    action: GroupAction,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GroupAction {
    Move,
    DeleteDuplicates,
}

async fn list_tree(client: &dyn CloudDriveClientTrait, root: &str) -> Result<RemoteTree, String> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_string()];
    while let Some(directory) = pending.pop() {
        for entry in client
            .list_folder_fresh(&directory)
            .await
            .map_err(|error| error.to_string())?
        {
            if !safe_component(&entry.name) {
                return Err(format!(
                    "CloudDrive returned an unsafe entry name under {directory}"
                ));
            }
            let path = join_remote_path(&directory, &entry.name);
            if entry.is_directory {
                pending.push(path);
            } else {
                files.push(RemoteEntry {
                    path,
                    size: entry.size,
                });
            }
        }
    }
    Ok(RemoteTree { files })
}

async fn ensure_directory(
    client: &dyn CloudDriveClientTrait,
    destination: &str,
) -> Result<(), String> {
    let mut path = "/".to_string();
    for component in destination
        .trim_matches('/')
        .split('/')
        .filter(|component| !component.is_empty())
    {
        let children = client
            .list_folder_fresh(&path)
            .await
            .map_err(|error| error.to_string())?;
        match children.iter().find(|child| child.name == component) {
            Some(child) if child.is_directory && safe_component(&child.name) => {
                path = join_remote_path(&path, &child.name)
            }
            Some(child) if child.is_directory => {
                return Err(format!(
                    "CloudDrive returned an unsafe directory name: {}",
                    child.name
                ))
            }
            Some(_) => return Err(format!("remote destination is a file: {path}/{component}")),
            None => {
                client
                    .create_folder(&path, component)
                    .await
                    .map_err(|error| error.to_string())?;
                path = join_remote_path(&path, component);
            }
        }
    }
    Ok(())
}

async fn inspect_destination(
    client: &dyn CloudDriveClientTrait,
    destination: &str,
) -> Result<HashMap<String, i64>, String> {
    let mut path = "/".to_string();
    for component in destination
        .trim_matches('/')
        .split('/')
        .filter(|component| !component.is_empty())
    {
        let children = client
            .list_folder_fresh(&path)
            .await
            .map_err(|error| error.to_string())?;
        match children.iter().find(|child| child.name == component) {
            Some(child) if child.is_directory && safe_component(&child.name) => {
                path = join_remote_path(&path, &child.name)
            }
            Some(child) if child.is_directory => {
                return Err(format!(
                    "CloudDrive returned an unsafe directory name: {}",
                    child.name
                ))
            }
            Some(_) => return Err(format!("remote destination is a file: {path}/{component}")),
            None => return Ok(HashMap::new()),
        }
    }
    Ok(client
        .list_folder_fresh(destination)
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|file| (file.name, file.size))
        .collect())
}

async fn group_was_moved(
    client: &dyn CloudDriveClientTrait,
    group: &PlannedGroup,
) -> Result<bool, String> {
    let destination = inspect_destination(client, &group.destination).await?;
    if group
        .names
        .iter()
        .any(|name| !destination.contains_key(name))
    {
        return Ok(false);
    }
    for file in &group.files {
        let Some(parent) = remote_parent(&file.path) else {
            return Ok(false);
        };
        let Some(name) = remote_file_name(&file.path) else {
            return Ok(false);
        };
        if client
            .list_folder_fresh(parent)
            .await
            .map_err(|error| error.to_string())?
            .iter()
            .any(|entry| entry.name == name)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn destination_path(root: &str, components: &[String]) -> String {
    components.iter().fold(root.to_string(), |path, component| {
        join_remote_path(&path, component)
    })
}

fn normalize_info_hash(value: &str) -> Option<String> {
    (value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| value.to_ascii_lowercase())
}

fn is_external_subtitle(path: &str) -> bool {
    matches!(
        Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("srt" | "ass" | "ssa" | "vtt")
    )
}

fn safe_component(component: &str) -> bool {
    !component.is_empty()
        && !matches!(component, "." | "..")
        && !component.contains(['/', '\\', '\0'])
}

fn offline_status_name(status: i32) -> &'static str {
    match status {
        value if value == proto::OfflineFileStatus::OfflineInit as i32 => "init",
        value if value == proto::OfflineFileStatus::OfflineDownloading as i32 => "downloading",
        value if value == proto::OfflineFileStatus::OfflineFinished as i32 => "finished",
        value if value == proto::OfflineFileStatus::OfflineError as i32 => "error",
        _ => "unknown",
    }
}

pub(crate) fn canonical_remote_path(path: &str) -> Option<String> {
    let path = path.trim();
    if !path.starts_with('/') {
        return None;
    }
    let mut components = Vec::new();
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop()?;
            }
            component if safe_component(component) => components.push(component),
            _ => return None,
        }
    }
    Some(if components.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", components.join("/"))
    })
}

pub(crate) fn validate_remote_organize_paths(source: &str, target: &str) -> Result<(), String> {
    let source = canonical_remote_path(source)
        .ok_or_else(|| "RSS source must be an absolute remote path".to_string())?;
    let target = canonical_remote_path(target)
        .ok_or_else(|| "remote organize target must be an absolute remote path".to_string())?;
    if source == target
        || source.starts_with(&(target.clone() + "/"))
        || target.starts_with(&(source + "/"))
    {
        return Err("remote organize source and target must not be equal or nested".to_string());
    }
    Ok(())
}

fn join_remote_path(parent: &str, name: &str) -> String {
    if parent == "/" {
        format!("/{name}")
    } else {
        format!("{}/{}", parent.trim_end_matches('/'), name)
    }
}

fn remote_parent(path: &str) -> Option<&str> {
    let index = path.rfind('/')?;
    (index == 0).then_some("/").or_else(|| path.get(..index))
}

fn remote_file_name(path: &str) -> Option<&str> {
    path.rsplit('/').find(|component| !component.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anime_organizer::error::Result;
    use async_trait::async_trait;
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct MockCloud {
        folders: Arc<Mutex<BTreeMap<String, Vec<proto::CloudDriveFile>>>>,
        moves: Arc<Mutex<Vec<(Vec<String>, String)>>>,
        deletes: Arc<Mutex<Vec<String>>>,
        listed_paths: Arc<Mutex<Vec<String>>>,
        offline_calls: Arc<AtomicUsize>,
        hashes: Arc<Mutex<HashMap<String, String>>>,
        hash_calls: Arc<Mutex<Vec<String>>>,
        fail_hashes: Arc<AtomicUsize>,
        fail_moves: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl CloudDriveClientTrait for MockCloud {
        async fn login(&mut self, _: &str, _: &str) -> Result<String> {
            Ok("token".to_string())
        }
        async fn add_offline_files(&self, _: Vec<String>, _: &str) -> Result<()> {
            Ok(())
        }
        async fn list_folder(&self, path: &str) -> Result<Vec<proto::CloudDriveFile>> {
            self.listed_paths.lock().unwrap().push(path.to_string());
            let folders = self.folders.lock().unwrap();
            let mut entries = folders.get(path).cloned().unwrap_or_default();
            for directory in folders.keys() {
                if remote_parent(directory) == Some(path) {
                    let name = remote_file_name(directory).expect("directory has a name");
                    if !entries.iter().any(|entry| entry.name == name) {
                        entries.push(file(directory, true));
                    }
                }
            }
            Ok(entries)
        }
        async fn list_offline_files_by_path(&self, _: &str) -> Result<Vec<proto::OfflineFile>> {
            self.offline_calls.fetch_add(1, Ordering::SeqCst);
            Ok(vec![proto::OfflineFile {
                name: "torrent".to_string(),
                status: proto::OfflineFileStatus::OfflineFinished as i32,
                info_hash: "ABCDEF1234567890ABCDEF1234567890ABCDEF12".to_string(),
                ..Default::default()
            }])
        }
        async fn find_file_by_path(
            &self,
            parent_path: &str,
            path: &str,
        ) -> Result<proto::CloudDriveFile> {
            let full_path = join_remote_path(parent_path, path);
            if self.folders.lock().unwrap().contains_key(&full_path) {
                return Ok(file(&full_path, true));
            }
            self.folders
                .lock()
                .unwrap()
                .get(parent_path)
                .and_then(|entries| entries.iter().find(|entry| entry.name == path).cloned())
                .ok_or_else(|| {
                    anime_organizer::error::AppError::MetadataFetchError(
                        "mock file was not found".to_string(),
                    )
                })
        }
        async fn create_folder(&self, parent: &str, name: &str) -> Result<proto::CloudDriveFile> {
            let path = join_remote_path(parent, name);
            self.folders
                .lock()
                .unwrap()
                .entry(path.clone())
                .or_default();
            Ok(proto::CloudDriveFile {
                name: name.to_string(),
                full_path_name: path,
                is_directory: true,
                ..Default::default()
            })
        }
        async fn move_files(&self, paths: Vec<String>, destination: &str) -> Result<()> {
            if self
                .fail_moves
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
                    if remaining > 0 {
                        Some(remaining - 1)
                    } else {
                        None
                    }
                })
                .is_ok()
            {
                return Err(anime_organizer::error::AppError::MetadataFetchError(
                    "transient move failure".to_string(),
                ));
            }
            let mut folders = self.folders.lock().unwrap();
            let mut moved = Vec::new();
            for path in &paths {
                let parent = remote_parent(path).expect("mock source has a parent");
                let name = remote_file_name(path).expect("mock source has a name");
                let source = folders
                    .get_mut(parent)
                    .and_then(|entries| {
                        entries
                            .iter()
                            .position(|entry| entry.name == name)
                            .map(|index| entries.remove(index))
                    })
                    .expect("mock source exists");
                moved.push(source);
            }
            let destination_entries = folders.entry(destination.to_string()).or_default();
            for mut entry in moved {
                entry.name = remote_file_name(&entry.full_path_name)
                    .expect("mock source has a name")
                    .to_string();
                entry.full_path_name = join_remote_path(destination, &entry.name);
                destination_entries.push(entry);
            }
            self.moves
                .lock()
                .unwrap()
                .push((paths, destination.to_string()));
            Ok(())
        }
        async fn delete_file(&self, path: &str) -> Result<()> {
            let parent = remote_parent(path).expect("mock source has a parent");
            let name = remote_file_name(path).expect("mock source has a name");
            let mut folders = self.folders.lock().unwrap();
            if let Some(entries) = folders.get_mut(parent) {
                entries.retain(|entry| entry.name != name);
            }
            folders.remove(path);
            self.deletes.lock().unwrap().push(path.to_string());
            Ok(())
        }
        async fn sha256_file(&self, path: &str) -> Result<String> {
            if self
                .fail_hashes
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
                    if remaining > 0 {
                        Some(remaining - 1)
                    } else {
                        None
                    }
                })
                .is_ok()
            {
                return Err(anime_organizer::error::AppError::MetadataFetchError(
                    "transient hash failure".to_string(),
                ));
            }
            self.hash_calls.lock().unwrap().push(path.to_string());
            Ok(self
                .hashes
                .lock()
                .unwrap()
                .get(path)
                .cloned()
                .unwrap_or_else(|| path.to_string()))
        }
    }

    fn file(path: &str, directory: bool) -> proto::CloudDriveFile {
        file_with_size(path, directory, 0)
    }

    fn file_with_size(path: &str, directory: bool, size: i64) -> proto::CloudDriveFile {
        proto::CloudDriveFile {
            name: remote_file_name(path).unwrap().to_string(),
            full_path_name: path.to_string(),
            is_directory: directory,
            size,
            ..Default::default()
        }
    }

    fn configured_subscription(db: &RssDatabase) -> i64 {
        let id = db
            .add_subscription("https://example.test/rss", None, "/source", 300)
            .unwrap();
        db.update_subscription_organization_settings(id, true, Some("/library"), false, false)
            .unwrap();
        db.save_download_task(id, "rss-item").unwrap();
        db.save_download_correlation(
            id,
            "rss-item",
            "abcdef1234567890abcdef1234567890abcdef12",
            None,
        )
        .unwrap();
        id
    }

    #[tokio::test]
    async fn equal_same_size_collision_hashes_both_sides_then_deletes_only_the_source() {
        let client = MockCloud::default();
        let source = "/source/torrent/[ANi] Test Show - 01 [1080P].mkv";
        let target = "/library/Test Show/[ANi] Test Show - 01 [1080P].mkv";
        client.folders.lock().unwrap().extend([
            ("/source".to_string(), vec![file("/source/torrent", true)]),
            (
                "/source/torrent".to_string(),
                vec![file_with_size(source, false, 42)],
            ),
            (
                "/library".to_string(),
                vec![file("/library/Test Show", true)],
            ),
            (
                "/library/Test Show".to_string(),
                vec![file_with_size(target, false, 42)],
            ),
        ]);
        client.hashes.lock().unwrap().extend([
            (source.to_string(), "same".to_string()),
            (target.to_string(), "same".to_string()),
        ]);
        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = configured_subscription(&db);

        organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
            .await
            .unwrap();

        assert_eq!(
            client.hash_calls.lock().unwrap().as_slice(),
            [source, target]
        );
        assert!(client.moves.lock().unwrap().is_empty());
        assert_eq!(client.deletes.lock().unwrap().as_slice(), [source]);
        assert_eq!(
            db.list_download_tasks(id, None).unwrap()[0]
                .status
                .as_deref(),
            Some("completed")
        );
    }

    #[tokio::test]
    async fn size_mismatch_does_not_hash_or_mutate_either_side() {
        let client = MockCloud::default();
        let source = "/source/torrent/[ANi] Test Show - 01 [1080P].mkv";
        let target = "/library/Test Show/[ANi] Test Show - 01 [1080P].mkv";
        client.folders.lock().unwrap().extend([
            ("/source".to_string(), vec![file("/source/torrent", true)]),
            (
                "/source/torrent".to_string(),
                vec![file_with_size(source, false, 41)],
            ),
            (
                "/library".to_string(),
                vec![file("/library/Test Show", true)],
            ),
            (
                "/library/Test Show".to_string(),
                vec![file_with_size(target, false, 42)],
            ),
        ]);
        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = configured_subscription(&db);

        let summary =
            organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
                .await
                .unwrap();

        assert_eq!(summary.skipped_conflicts, 1);
        assert!(client.hash_calls.lock().unwrap().is_empty());
        assert!(client.moves.lock().unwrap().is_empty());
        assert!(client.deletes.lock().unwrap().is_empty());
        assert_eq!(
            db.list_download_tasks(id, None).unwrap()[0]
                .status
                .as_deref(),
            Some("finished")
        );
    }

    #[tokio::test]
    async fn failed_or_unequal_hash_preserves_the_source() {
        let client = MockCloud::default();
        let source = "/source/torrent/[ANi] Test Show - 01 [1080P].mkv";
        let target = "/library/Test Show/[ANi] Test Show - 01 [1080P].mkv";
        client.folders.lock().unwrap().extend([
            ("/source".to_string(), vec![file("/source/torrent", true)]),
            (
                "/source/torrent".to_string(),
                vec![file_with_size(source, false, 42)],
            ),
            (
                "/library".to_string(),
                vec![file("/library/Test Show", true)],
            ),
            (
                "/library/Test Show".to_string(),
                vec![file_with_size(target, false, 42)],
            ),
        ]);
        client.hashes.lock().unwrap().extend([
            (source.to_string(), "source".to_string()),
            (target.to_string(), "target".to_string()),
        ]);
        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = configured_subscription(&db);

        let summary =
            organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
                .await
                .unwrap();
        assert_eq!(summary.skipped_conflicts, 1);
        assert_eq!(client.hash_calls.lock().unwrap().len(), 2);
        assert!(client.moves.lock().unwrap().is_empty());
        assert!(client.deletes.lock().unwrap().is_empty());

        client.fail_hashes.store(1, Ordering::SeqCst);
        assert!(
            organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
                .await
                .is_err()
        );
        assert!(client.moves.lock().unwrap().is_empty());
        assert!(client.deletes.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn moves_parsed_media_and_matching_subtitles_from_finished_directory() {
        let client = MockCloud::default();
        client.folders.lock().unwrap().extend([
            ("/source".to_string(), vec![file("/source/torrent", true)]),
            (
                "/source/torrent".to_string(),
                vec![
                    file("/source/torrent/[ANi] Test Show - 01 [1080P].mkv", false),
                    file("/source/torrent/[ANi] Test Show - 01 [1080P].zh.ass", false),
                ],
            ),
            ("/library".to_string(), Vec::new()),
        ]);
        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = db
            .add_subscription("https://example.test/rss", None, "/source", 300)
            .unwrap();
        db.update_subscription_organization_settings(id, true, Some("/library"), false, true)
            .unwrap();
        db.save_download_task(id, "rss-item").unwrap();
        db.save_download_correlation(
            id,
            "rss-item",
            "abcdef1234567890abcdef1234567890abcdef12",
            None,
        )
        .unwrap();
        let other_id = db
            .add_subscription("https://example.test/other", None, "/other", 300)
            .unwrap();
        db.save_download_task(other_id, "other-rss-item").unwrap();
        db.save_download_correlation(
            other_id,
            "other-rss-item",
            "abcdef1234567890abcdef1234567890abcdef12",
            None,
        )
        .unwrap();
        let summary =
            organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
                .await
                .unwrap();
        assert_eq!(summary.moved_media, 1);
        assert_eq!(summary.moved_files, 2);
        assert_eq!(summary.removed_empty_directories, 1);
        assert_eq!(client.moves.lock().unwrap()[0].0.len(), 2);
        assert_eq!(
            client.deletes.lock().unwrap().as_slice(),
            ["/source/torrent"]
        );
        assert_eq!(client.offline_calls.load(Ordering::SeqCst), 1);
        assert!(!client
            .listed_paths
            .lock()
            .unwrap()
            .iter()
            .any(|path| path == "/source"));
        let task = db.list_download_tasks(id, None).unwrap().pop().unwrap();
        assert_eq!(task.status.as_deref(), Some("completed"));
        assert_eq!(task.remote_name.as_deref(), Some("torrent"));
        assert_eq!(
            db.list_download_tasks(other_id, None).unwrap()[0]
                .status
                .as_deref(),
            Some("pending")
        );
    }

    #[tokio::test]
    async fn conflicts_leave_the_finished_task_retryable() {
        let client = MockCloud::default();
        client.folders.lock().unwrap().extend([
            ("/source".to_string(), vec![file("/source/torrent", true)]),
            (
                "/source/torrent".to_string(),
                vec![file(
                    "/source/torrent/[ANi] Test Show - 01 [1080P].mkv",
                    false,
                )],
            ),
            (
                "/library".to_string(),
                vec![file("/library/Test Show", true)],
            ),
            (
                "/library/Test Show".to_string(),
                vec![file(
                    "/library/Test Show/[ANi] Test Show - 01 [1080P].mkv",
                    false,
                )],
            ),
        ]);
        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = db
            .add_subscription("https://example.test/rss", None, "/source", 300)
            .unwrap();
        db.update_subscription_organization_settings(id, true, Some("/library"), false, false)
            .unwrap();
        db.save_download_task(id, "rss-item").unwrap();
        db.save_download_correlation(
            id,
            "rss-item",
            "abcdef1234567890abcdef1234567890abcdef12",
            None,
        )
        .unwrap();
        let summary =
            organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
                .await
                .unwrap();
        assert_eq!(summary.skipped_conflicts, 1);
        assert!(client.moves.lock().unwrap().is_empty());
        assert_eq!(
            db.list_download_tasks(id, None).unwrap()[0]
                .status
                .as_deref(),
            Some("finished")
        );
    }

    #[tokio::test]
    async fn keeps_empty_source_directory_when_cleanup_is_disabled() {
        let client = MockCloud::default();
        client.folders.lock().unwrap().extend([
            ("/source".to_string(), vec![file("/source/torrent", true)]),
            (
                "/source/torrent".to_string(),
                vec![file(
                    "/source/torrent/[ANi] Test Show - 01 [1080P].mkv",
                    false,
                )],
            ),
            ("/library".to_string(), Vec::new()),
        ]);
        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = configured_subscription(&db);

        let summary =
            organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
                .await
                .unwrap();

        assert_eq!(summary.moved_media, 1);
        assert_eq!(summary.removed_empty_directories, 0);
        assert!(client.deletes.lock().unwrap().is_empty());
        assert!(client
            .folders
            .lock()
            .unwrap()
            .contains_key("/source/torrent"));
    }

    #[tokio::test]
    async fn does_not_delete_source_tree_when_an_unmoved_file_remains() {
        let client = MockCloud::default();
        client.folders.lock().unwrap().extend([
            ("/source".to_string(), vec![file("/source/torrent", true)]),
            (
                "/source/torrent".to_string(),
                vec![
                    file("/source/torrent/[ANi] Test Show - 01 [1080P].mkv", false),
                    file("/source/torrent/unrecognized.txt", false),
                ],
            ),
            ("/library".to_string(), Vec::new()),
        ]);
        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = db
            .add_subscription("https://example.test/rss", None, "/source", 300)
            .unwrap();
        db.update_subscription_organization_settings(id, true, Some("/library"), false, true)
            .unwrap();
        db.save_download_task(id, "rss-item").unwrap();
        db.save_download_correlation(
            id,
            "rss-item",
            "abcdef1234567890abcdef1234567890abcdef12",
            None,
        )
        .unwrap();

        let summary =
            organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
                .await
                .unwrap();

        assert_eq!(summary.moved_media, 0);
        assert_eq!(summary.removed_empty_directories, 0);
        assert!(client.moves.lock().unwrap().is_empty());
        assert!(client.deletes.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn leaves_empty_finished_roots_retryable_without_deleting_them() {
        let client = MockCloud::default();
        client.folders.lock().unwrap().extend([
            ("/source".to_string(), vec![file("/source/torrent", true)]),
            ("/source/torrent".to_string(), Vec::new()),
            ("/library".to_string(), Vec::new()),
        ]);
        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = db
            .add_subscription("https://example.test/rss", None, "/source", 300)
            .unwrap();
        db.update_subscription_organization_settings(id, true, Some("/library"), false, true)
            .unwrap();
        db.save_download_task(id, "rss-item").unwrap();
        db.save_download_correlation(
            id,
            "rss-item",
            "abcdef1234567890abcdef1234567890abcdef12",
            None,
        )
        .unwrap();
        organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
            .await
            .unwrap();
        assert!(client.deletes.lock().unwrap().is_empty());
        assert_eq!(
            db.list_download_tasks(id, None).unwrap()[0]
                .status
                .as_deref(),
            Some("finished")
        );
    }

    #[tokio::test]
    async fn failed_organization_stays_retryable_until_a_move_succeeds() {
        let client = MockCloud::default();
        client.folders.lock().unwrap().extend([
            (
                "/source/torrent".to_string(),
                vec![file(
                    "/source/torrent/[ANi] Test Show - 01 [1080P].mkv",
                    false,
                )],
            ),
            ("/library".to_string(), Vec::new()),
        ]);
        client.fail_moves.store(1, Ordering::SeqCst);
        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = db
            .add_subscription("https://example.test/rss", None, "/source", 300)
            .unwrap();
        db.update_subscription_organization_settings(id, true, Some("/library"), false, false)
            .unwrap();
        db.save_download_task(id, "rss-item").unwrap();
        db.save_download_correlation(
            id,
            "rss-item",
            "abcdef1234567890abcdef1234567890abcdef12",
            None,
        )
        .unwrap();

        assert!(
            organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
                .await
                .is_err()
        );
        let task = db.list_download_tasks(id, None).unwrap().pop().unwrap();
        assert_eq!(task.status.as_deref(), Some("finished"));
        assert!(task.completed_at.is_none());
        assert!(db
            .get_subscription(id)
            .unwrap()
            .unwrap()
            .last_organize_checked_at
            .is_some());

        organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
            .await
            .unwrap();
        assert_eq!(
            db.list_download_tasks(id, None).unwrap()[0]
                .status
                .as_deref(),
            Some("completed")
        );
    }

    #[tokio::test]
    async fn legacy_tasks_without_info_hash_are_reported_and_not_adopted() {
        let client = MockCloud::default();
        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = db
            .add_subscription("https://example.test/rss", None, "/source", 300)
            .unwrap();
        db.update_subscription_organization_settings(id, true, Some("/library"), false, false)
            .unwrap();
        db.save_download_task(id, "legacy-rss-item").unwrap();

        let summary =
            organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
                .await
                .unwrap();
        assert_eq!(summary.uncorrelated_legacy_tasks, 1);
        assert!(client.moves.lock().unwrap().is_empty());
        assert_eq!(
            db.list_download_tasks(id, None).unwrap()[0]
                .status
                .as_deref(),
            Some("pending")
        );
    }

    #[test]
    fn remote_paths_are_canonical_and_never_nested() {
        assert_eq!(
            canonical_remote_path("/source/./child//"),
            Some("/source/child".to_string())
        );
        assert_eq!(
            canonical_remote_path("/source/child/../next"),
            Some("/source/next".to_string())
        );
        assert!(canonical_remote_path("/../../source").is_none());
        assert!(canonical_remote_path("/source\\child").is_none());
        assert!(validate_remote_organize_paths("/source", "/source/library").is_err());
        assert!(validate_remote_organize_paths("/source/library", "/source").is_err());
    }
}
