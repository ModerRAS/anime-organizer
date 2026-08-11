use anime_organizer::rss::client::{proto, CloudDriveClientTrait};
use anime_organizer::rss::db::{DownloadTask, RssDatabase, Subscription};
use anime_organizer::rss::proxy::{build_http_client, ProxyConfig};
use anime_organizer::rss::torrent::download_torrent_to_magnet;
use anime_organizer::{
    organize_directory_components, FileOrganizer, FilenameParser, LibraryIndex, LibraryIndexRecord,
};
use regex::Regex;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::LazyLock;
use std::time::UNIX_EPOCH;

static REMOTE_MLIP_COUNTER: AtomicU64 = AtomicU64::new(0);
static TORRENT_HREF_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)href\s*=\s*[\"']([^\"']+\.torrent(?:\?[^\"']*)?)[\"']"#)
        .expect("valid DMHY torrent href regex")
});

type ProgressReporter<'a> = dyn Fn(&str, Option<usize>, Option<usize>, &str) + 'a;

#[derive(Debug, Default, Serialize)]
pub(crate) struct RemoteOrganizeSummary {
    pub(crate) finished_directories: usize,
    pub(crate) moved_media: usize,
    pub(crate) moved_files: usize,
    pub(crate) copied_media: usize,
    pub(crate) copied_files: usize,
    pub(crate) skipped_conflicts: usize,
    pub(crate) removed_empty_directories: usize,
    pub(crate) uncorrelated_legacy_tasks: i64,
}

#[cfg(test)]
pub(crate) async fn organize_subscription(
    db: &RssDatabase,
    subscription: &Subscription,
    client: &dyn CloudDriveClientTrait,
) -> Result<RemoteOrganizeSummary, String> {
    organize_subscription_with_progress(db, subscription, client, &|_, _, _, _| {}).await
}

pub(crate) async fn organize_subscription_with_progress(
    db: &RssDatabase,
    subscription: &Subscription,
    client: &dyn CloudDriveClientTrait,
    progress: &ProgressReporter<'_>,
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
    progress(
        "info",
        None,
        None,
        &format!(
            "Starting remote organization: subscription={}, source='{source_root}', target='{target_root}', mode={}, season_mode={}, remote_mlip={}, remove_empty_dirs={}",
            subscription.id,
            subscription.organize_mode,
            subscription.organize_season_mode,
            subscription.remote_mlip,
            subscription.remove_empty_dirs
        ),
    );

    let original_mode = subscription.organize_mode == "original";
    let mut tasks = if original_mode {
        Vec::new()
    } else {
        db.list_download_tasks(subscription.id, None)
            .map_err(|error| error.to_string())?
    };
    let initial_uncorrelated_legacy_tasks = if original_mode {
        0
    } else {
        db.count_uncorrelated_download_tasks(subscription.id)
            .map_err(|error| error.to_string())?
    };

    let source_entries = client
        .list_folder_fresh(&source_root)
        .await
        .map_err(|error| error.to_string())?;
    let source_entry_count = source_entries.len();
    let mut source_entries_by_name = HashMap::new();
    for entry in source_entries {
        source_entries_by_name
            .entry(entry.name.clone())
            .and_modify(|existing| *existing = None)
            .or_insert(Some(entry));
    }

    let mut finished_roots: Vec<(String, Option<(i64, String)>)> = Vec::new();
    let uncorrelated_legacy_tasks;
    if original_mode {
        finished_roots.extend(source_entries_by_name.iter().filter_map(|(name, entry)| {
            entry
                .as_ref()
                .filter(|_| safe_component(name))
                .map(|_| (name.clone(), None))
        }));
        uncorrelated_legacy_tasks = 0;
        progress(
            "info",
            Some(0),
            Some(finished_roots.len()),
            &format!(
                "Original mode selected {} unique source root(s) directly; CloudDrive offline tasks were not queried",
                finished_roots.len()
            ),
        );
    } else {
        // This is the only offline-task query for an offline-mode job. It is a
        // status lookup, not a filesystem listing of the RSS source root.
        let offline_files = client
            .list_offline_files_by_path(&source_root)
            .await
            .map_err(|error| error.to_string())?;
        let offline_file_count = offline_files.len();
        progress(
            "info",
            None,
            None,
            &format!(
                "Loaded {} RSS task(s), {source_entry_count} source root entry/entries, {offline_file_count} CloudDrive offline task(s), {initial_uncorrelated_legacy_tasks} uncorrelated legacy task(s)",
                tasks.len()
            ),
        );
        let mut offline_hash_counts = HashMap::new();
        for offline in &offline_files {
            if let Some(hash) = normalize_info_hash(&offline.info_hash) {
                *offline_hash_counts.entry(hash).or_insert(0usize) += 1;
            }
        }
        let recovered_legacy_tasks = backfill_legacy_download_correlations(
            db,
            subscription.id,
            &mut tasks,
            &offline_files,
            &offline_hash_counts,
            progress,
        )
        .await;
        uncorrelated_legacy_tasks = db
            .count_uncorrelated_download_tasks(subscription.id)
            .map_err(|error| error.to_string())?;
        if initial_uncorrelated_legacy_tasks > 0 {
            progress(
                "info",
                Some(recovered_legacy_tasks),
                usize::try_from(initial_uncorrelated_legacy_tasks).ok(),
                &format!(
                    "Legacy correlation fallback recovered {recovered_legacy_tasks} task(s); {uncorrelated_legacy_tasks} remain uncorrelated"
                ),
            );
        }

        let mut tasks_by_hash: HashMap<String, Vec<_>> = HashMap::new();
        for task in &tasks {
            if let Some(hash) = task.info_hash.as_deref().and_then(normalize_info_hash) {
                tasks_by_hash.entry(hash).or_default().push(task);
            }
        }

        for offline in offline_files {
            let Some(info_hash) = normalize_info_hash(&offline.info_hash) else {
                continue;
            };
            if offline_hash_counts.get(&info_hash) != Some(&1) {
                continue;
            }
            let Some([task]) = tasks_by_hash.get(&info_hash).map(Vec::as_slice) else {
                continue;
            };
            let was_completed = task.status.as_deref() == Some("completed");
            let status = offline_status_name(offline.status);
            if !db
                .reconcile_download_task(
                    task.id,
                    subscription.id,
                    &info_hash,
                    status,
                    &offline.name,
                )
                .map_err(|error| error.to_string())?
            {
                continue;
            }
            if offline.status == proto::OfflineFileStatus::OfflineFinished as i32
                && !was_completed
                && safe_component(&offline.name)
            {
                finished_roots.push((offline.name, Some((task.id, info_hash))));
            }
        }
        progress(
            "info",
            Some(0),
            Some(finished_roots.len()),
            &format!(
                "Reconciled CloudDrive status: {} completed root(s) are eligible for planning",
                finished_roots.len()
            ),
        );
    }
    db.mark_subscription_organize_checked(subscription.id)
        .map_err(|error| error.to_string())?;

    let mut summary = RemoteOrganizeSummary {
        finished_directories: finished_roots.len(),
        uncorrelated_legacy_tasks,
        ..RemoteOrganizeSummary::default()
    };
    let mut planned_roots = Vec::new();
    let mut planned_destination_names = HashSet::new();
    let mut ensured_destinations = HashSet::new();
    let finished_root_count = finished_roots.len();
    for (root_index, (source_name, task)) in finished_roots.into_iter().enumerate() {
        let task_context = task.as_ref().map_or_else(
            || "mode=original".to_string(),
            |(task_id, info_hash)| format!("task_id={task_id}, info_hash={info_hash}"),
        );
        progress(
            "info",
            Some(root_index + 1),
            Some(finished_root_count),
            &format!(
                "Planning root {}/{}: '{}' ({task_context})",
                root_index + 1,
                finished_root_count,
                source_name
            ),
        );
        let Some(root_entry) = source_entries_by_name
            .get(&source_name)
            .and_then(Option::as_ref)
        else {
            progress(
                "warning",
                Some(root_index + 1),
                Some(finished_root_count),
                &format!(
                    "Skipped root '{}': no unique matching entry exists under '{}'",
                    source_name, source_root
                ),
            );
            continue;
        };
        let offline_name = source_name;
        let (task_id, info_hash) = task.map_or((None, None), |(id, hash)| (Some(id), Some(hash)));
        let root = join_remote_path(&source_root, &offline_name);
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
        progress(
            "info",
            Some(root_index + 1),
            Some(finished_root_count),
            &format!(
                "Scanned root '{}': {} file(s), {} media candidate(s), {} subtitle candidate(s)",
                root,
                tree.files.len(),
                tree.files.len().saturating_sub(subtitle_candidates.len()),
                subtitle_candidates.len()
            ),
        );
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
                progress(
                    "warning",
                    Some(root_index + 1),
                    Some(finished_root_count),
                    &format!(
                        "Skipped root '{offline_name}': filename parser rejected '{}'",
                        media.path
                    ),
                );
                invalid_root = true;
                break;
            };
            let episode = match anime.episode.parse::<f64>() {
                Ok(episode) if episode.is_finite() => episode,
                _ => {
                    progress(
                        "warning",
                        Some(root_index + 1),
                        Some(finished_root_count),
                        &format!(
                            "Skipped root '{offline_name}': invalid episode '{}' parsed from '{}'",
                            anime.episode, media.path
                        ),
                    );
                    invalid_root = true;
                    break;
                }
            };
            let series_title = anime.series_name();
            let season = i64::from(anime.season_number().unwrap_or(1));
            let components =
                organize_directory_components(&anime, subscription.organize_season_mode);
            if components
                .iter()
                .any(|component| !safe_component(component))
            {
                progress(
                    "warning",
                    Some(root_index + 1),
                    Some(finished_root_count),
                    &format!(
                        "Skipped root '{offline_name}': parsed destination contains an unsafe component: {:?}",
                        components
                    ),
                );
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
                progress(
                    "warning",
                    Some(root_index + 1),
                    Some(finished_root_count),
                    &format!(
                        "Skipped root '{offline_name}': a subtitle matched more than one media file"
                    ),
                );
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
            let destination = destination_path(&target_root, &components);
            progress(
                "info",
                Some(root_index + 1),
                Some(finished_root_count),
                &format!(
                    "Parsed '{}': series='{}', season={}, episode={}, destination='{}', matched_subtitles={}",
                    media.path,
                    series_title,
                    season,
                    episode,
                    destination,
                    subtitles.len()
                ),
            );
            for subtitle in &subtitles {
                progress(
                    "info",
                    Some(root_index + 1),
                    Some(finished_root_count),
                    &format!(
                        "Subtitle mapping: '{}' -> '{}'",
                        subtitle.display(),
                        join_remote_path(
                            &destination,
                            subtitle
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or("<invalid-name>")
                        )
                    ),
                );
            }
            groups.push((destination, files, series_title, season, episode));
        }
        if claimed_subtitles.len() != subtitle_candidates.len() {
            progress(
                "warning",
                Some(root_index + 1),
                Some(finished_root_count),
                &format!(
                    "Skipped root '{offline_name}': {} subtitle file(s) were not uniquely matched",
                    subtitle_candidates
                        .len()
                        .saturating_sub(claimed_subtitles.len())
                ),
            );
            invalid_root = true;
        }
        if invalid_root || groups.is_empty() {
            if groups.is_empty() && !invalid_root {
                progress(
                    "warning",
                    Some(root_index + 1),
                    Some(finished_root_count),
                    &format!("Skipped root '{offline_name}': no media bundle was parsed"),
                );
            }
            continue;
        }

        // Resolve every bundle before issuing a mutation. A hash or transport
        // failure therefore leaves this root untouched.
        let mut planned = Vec::new();
        let mut conflicted = false;
        for (destination, files, series_title, season, episode) in groups {
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
                if subscription.remote_mlip || !original_mode {
                    GroupAction::Move
                } else {
                    GroupAction::Copy
                }
            } else if conflict_count == files.len()
                && files.iter().zip(&names).all(|(file, name)| {
                    file.size >= 0 && existing.get(name).is_some_and(|size| *size == file.size)
                })
            {
                if original_mode && !subscription.remote_mlip {
                    progress(
                        "info",
                        Some(root_index + 1),
                        Some(finished_root_count),
                        &format!(
                            "Copy target already contains every source name with the same size; retaining original bundle without downloading it: source={:?}, destination='{}'",
                            files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>(),
                            destination
                        ),
                    );
                    GroupAction::AlreadyCopied
                } else {
                    let mut hashes_match = true;
                    for (file, name) in files.iter().zip(&names) {
                        let target_path = join_remote_path(&destination, name);
                        progress(
                            "info",
                            Some(root_index + 1),
                            Some(finished_root_count),
                            &format!(
                                "Verifying same-size conflict by SHA-256: source='{}', destination='{}'",
                                file.path, target_path
                            ),
                        );
                        let source_hash = client
                            .sha256_file(&file.path)
                            .await
                            .map_err(|error| error.to_string())?;
                        let target_hash = client
                            .sha256_file(&target_path)
                            .await
                            .map_err(|error| error.to_string())?;
                        if source_hash != target_hash {
                            hashes_match = false;
                            break;
                        }
                    }
                    if hashes_match {
                        progress(
                            "info",
                            Some(root_index + 1),
                            Some(finished_root_count),
                            &format!(
                                "Duplicate bundle verified by SHA-256: source file(s) {:?}; existing destination='{}'",
                                files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>(),
                                destination
                            ),
                        );
                        GroupAction::DeleteDuplicates
                    } else {
                        summary.skipped_conflicts += 1;
                        progress(
                            "warning",
                            Some(root_index + 1),
                            Some(finished_root_count),
                            &format!(
                                "Skipped root '{offline_name}': same-size destination conflict failed SHA-256 verification at '{}'",
                                destination
                            ),
                        );
                        conflicted = true;
                        continue;
                    }
                }
            } else {
                // Mixed bundles and unequal-size collisions preserve both
                // sides without opening a remote download stream.
                summary.skipped_conflicts += 1;
                progress(
                    "warning",
                    Some(root_index + 1),
                    Some(finished_root_count),
                    &format!(
                        "Skipped root '{offline_name}': destination conflict at '{}' ({} of {} names already exist or sizes differ)",
                        destination,
                        conflict_count,
                        files.len()
                    ),
                );
                conflicted = true;
                continue;
            };
            planned.push(PlannedGroup {
                destination,
                files,
                names,
                action,
                series_title,
                season,
                episode,
            });
        }
        if conflicted || planned.is_empty() {
            continue;
        }
        let root_destination_names = planned
            .iter()
            .flat_map(|group| {
                group
                    .names
                    .iter()
                    .map(|name| (group.destination.clone(), name.clone()))
            })
            .collect::<HashSet<_>>();
        let planned_name_count = planned.iter().map(|group| group.names.len()).sum::<usize>();
        if root_destination_names.len() != planned_name_count
            || root_destination_names
                .iter()
                .any(|name| planned_destination_names.contains(name))
        {
            summary.skipped_conflicts += 1;
            progress(
                "warning",
                Some(root_index + 1),
                Some(finished_root_count),
                &format!(
                    "Skipped root '{offline_name}': two planned bundles target the same remote filename"
                ),
            );
            continue;
        }
        planned_destination_names.extend(root_destination_names);

        // Create each distinct target directory once. A final fresh listing is
        // performed after MLIP publication immediately before each API action.
        for group in planned
            .iter()
            .filter(|group| matches!(group.action, GroupAction::Move | GroupAction::Copy))
        {
            if ensured_destinations.insert(group.destination.clone()) {
                ensure_directory(client, &group.destination).await?;
            }
        }

        progress(
            "info",
            Some(root_index + 1),
            Some(finished_root_count),
            &format!(
                "Planned root '{offline_name}': {} bundle(s), {} file(s)",
                planned.len(),
                planned.iter().map(|group| group.files.len()).sum::<usize>()
            ),
        );
        planned_roots.push(PlannedRoot {
            root,
            root_is_directory: root_entry.is_directory,
            task_id,
            info_hash,
            groups: planned,
        });
    }

    let planned_group_count = planned_roots
        .iter()
        .map(|root| root.groups.len())
        .sum::<usize>();
    progress(
        "info",
        Some(planned_roots.len()),
        Some(finished_root_count),
        &format!(
            "Planning finished: {} root(s), {planned_group_count} media bundle(s), {} conflict(s) skipped",
            planned_roots.len(),
            summary.skipped_conflicts
        ),
    );

    if subscription.remote_mlip && !planned_roots.is_empty() {
        let groups = planned_roots
            .iter()
            .flat_map(|root| root.groups.iter().cloned())
            .collect::<Vec<_>>();
        publish_remote_mlip(client, &target_root, &groups, progress).await?;
    } else if !subscription.remote_mlip {
        progress(
            "info",
            None,
            None,
            "Remote MLIP publication is disabled for this subscription",
        );
    }

    let mut completed_actions = 0usize;
    for planned_root in planned_roots {
        for group in planned_root
            .groups
            .iter()
            .filter(|group| matches!(group.action, GroupAction::Move | GroupAction::Copy))
        {
            completed_actions += 1;
            let copying = group.action == GroupAction::Copy;
            let verb = if copying { "Copying" } else { "Moving" };
            let mappings = group
                .files
                .iter()
                .zip(&group.names)
                .map(|(file, name)| {
                    format!(
                        "'{}' -> '{}'",
                        file.path,
                        join_remote_path(&group.destination, name)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            progress(
                "info",
                Some(completed_actions),
                Some(planned_group_count),
                &format!(
                    "{verb} bundle {completed_actions}/{planned_group_count}: series='{}', season={}, episode={}; {mappings}",
                    group.series_title, group.season, group.episode
                ),
            );
            let existing = inspect_destination(client, &group.destination).await?;
            if group.names.iter().any(|name| existing.contains_key(name)) {
                return Err(format!(
                    "remote destination changed before API action: destination='{}', planned_names={:?}",
                    group.destination, group.names
                ));
            }
            let paths = group.files.iter().map(|file| file.path.clone()).collect();
            if copying {
                client.copy_files(paths, &group.destination).await
            } else {
                client.move_files(paths, &group.destination).await
            }
            .map_err(|error| error.to_string())?;
            let verified = if copying {
                group_was_copied(client, group).await?
            } else {
                group_was_moved(client, group).await?
            };
            if !verified {
                return Err(format!(
                    "CloudDrive did not {} every planned bundle member to '{}': {:?}",
                    if copying { "copy" } else { "move" },
                    group.destination,
                    group.names
                ));
            }
            if copying {
                summary.copied_media += 1;
                summary.copied_files += group.files.len();
            } else {
                summary.moved_media += 1;
                summary.moved_files += group.files.len();
            }
            progress(
                "info",
                Some(completed_actions),
                Some(planned_group_count),
                &format!(
                    "{} verified for series='{}', season={}, episode={} in '{}'",
                    if copying { "Copy" } else { "Move" },
                    group.series_title,
                    group.season,
                    group.episode,
                    group.destination
                ),
            );
        }
        for group in planned_root
            .groups
            .iter()
            .filter(|group| group.action == GroupAction::DeleteDuplicates)
        {
            completed_actions += 1;
            progress(
                "info",
                Some(completed_actions),
                Some(planned_group_count),
                &format!(
                    "Removing verified duplicate source bundle {completed_actions}/{planned_group_count}: {:?}; keeping destination='{}'",
                    group.files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>(),
                    group.destination
                ),
            );
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
            progress(
                "info",
                Some(completed_actions),
                Some(planned_group_count),
                &format!(
                    "Verified duplicate source bundle removed; destination retained at '{}'",
                    group.destination
                ),
            );
        }

        for group in planned_root
            .groups
            .iter()
            .filter(|group| group.action == GroupAction::AlreadyCopied)
        {
            completed_actions += 1;
            progress(
                "info",
                Some(completed_actions),
                Some(planned_group_count),
                &format!(
                    "Original bundle already copied; retained source without download or API mutation: {:?}; destination='{}'",
                    group.files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>(),
                    group.destination
                ),
            );
        }

        if subscription.remove_empty_dirs && planned_root.root_is_directory {
            // Re-list the whole source tree after every move. Delete only the
            // completed torrent root when no file remains anywhere below it.
            if list_tree(client, &planned_root.root)
                .await?
                .files
                .is_empty()
            {
                client
                    .delete_file(&planned_root.root)
                    .await
                    .map_err(|error| error.to_string())?;
                summary.removed_empty_directories += 1;
                progress(
                    "info",
                    Some(completed_actions),
                    Some(planned_group_count),
                    &format!(
                        "Removed freshly confirmed empty source root '{}'",
                        planned_root.root
                    ),
                );
            }
        }
        if let (Some(task_id), Some(info_hash)) =
            (planned_root.task_id, planned_root.info_hash.as_deref())
        {
            db.complete_download_task(task_id, subscription.id, info_hash)
                .map_err(|error| error.to_string())?;
            progress(
                "info",
                Some(completed_actions),
                Some(planned_group_count),
                &format!(
                    "Completed RSS task {task_id} for source root '{}' (info_hash={info_hash})",
                    planned_root.root
                ),
            );
        } else {
            progress(
                "info",
                Some(completed_actions),
                Some(planned_group_count),
                &format!(
                    "Completed original-mode source root '{}' without offline-task reconciliation",
                    planned_root.root
                ),
            );
        }
    }
    progress(
        "info",
        Some(completed_actions),
        Some(planned_group_count),
        &format!(
            "Remote organization finished: moved_media={}, moved_files={}, copied_media={}, copied_files={}, skipped_conflicts={}, removed_empty_directories={}, uncorrelated_legacy_tasks={}",
            summary.moved_media,
            summary.moved_files,
            summary.copied_media,
            summary.copied_files,
            summary.skipped_conflicts,
            summary.removed_empty_directories,
            summary.uncorrelated_legacy_tasks
        ),
    );
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

struct PlannedRoot {
    root: String,
    root_is_directory: bool,
    task_id: Option<i64>,
    info_hash: Option<String>,
    groups: Vec<PlannedGroup>,
}

#[derive(Clone)]
struct PlannedGroup {
    destination: String,
    files: Vec<RemoteEntry>,
    names: Vec<String>,
    action: GroupAction,
    series_title: String,
    season: i64,
    episode: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GroupAction {
    Move,
    Copy,
    AlreadyCopied,
    DeleteDuplicates,
}

struct LocalTempDatabase(PathBuf);

impl LocalTempDatabase {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("aniorg-rss-mlip-{}.db", unique_suffix())))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for LocalTempDatabase {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

struct LocalTempMedia(PathBuf);

impl LocalTempMedia {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("aniorg-rss-media-{}", unique_suffix())))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for LocalTempMedia {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn publish_remote_mlip(
    client: &dyn CloudDriveClientTrait,
    target_root: &str,
    groups: &[PlannedGroup],
    progress: &ProgressReporter<'_>,
) -> Result<(), String> {
    progress(
        "info",
        Some(0),
        Some(groups.len()),
        &format!(
            "Preparing remote MLIP for {} media bundle(s) under '{target_root}'",
            groups.len()
        ),
    );
    let contents = inspect_destination(client, target_root).await?;
    let had_database = contents.contains_key(anime_organizer::library_index::DATABASE_FILENAME);
    let database_path = join_remote_path(
        target_root,
        anime_organizer::library_index::DATABASE_FILENAME,
    );
    let local = LocalTempDatabase::new();
    if had_database {
        progress(
            "info",
            None,
            None,
            &format!("Downloading existing remote MLIP '{database_path}'"),
        );
        client
            .download_file(&database_path, local.path())
            .await
            .map_err(|error| error.to_string())?;
        progress(
            "info",
            None,
            None,
            &format!(
                "Downloaded existing remote MLIP to temporary workspace ({} bytes)",
                std::fs::metadata(local.path())
                    .map(|metadata| metadata.len())
                    .unwrap_or(0)
            ),
        );
    } else {
        progress(
            "info",
            None,
            None,
            "No remote library.db exists; a new MLIP database will be created",
        );
    }

    let cached_hashes = if had_database {
        let cache_path = local.path().to_path_buf();
        tokio::task::spawn_blocking(move || LibraryIndex::cached_remote_media_hashes(&cache_path))
            .await
            .map_err(|error| format!("remote MLIP hash cache task failed: {error}"))?
            .map_err(|error| error.to_string())?
    } else {
        HashMap::new()
    };
    progress(
        "info",
        None,
        None,
        &format!(
            "Loaded {} reusable media hash record(s) from the existing MLIP",
            cached_hashes.len()
        ),
    );
    let records =
        build_remote_index_records(client, target_root, groups, &cached_hashes, progress).await?;
    progress(
        "info",
        Some(records.len()),
        Some(groups.len()),
        &format!(
            "Writing {} media record(s) to temporary MLIP",
            records.len()
        ),
    );
    let record_count = records.len();
    let local_path = local.path().to_path_buf();
    let remote_root = target_root.to_string();
    tokio::task::spawn_blocking(move || {
        LibraryIndex::update_remote_database(&local_path, &remote_root, &records)
    })
    .await
    .map_err(|error| format!("remote MLIP update task failed: {error}"))?
    .map_err(|error| error.to_string())?;
    let expected_hash = sha256_local_file(local.path())?;
    let local_size = std::fs::metadata(local.path())
        .map_err(|error| format!("read temporary MLIP metadata failed: {error}"))?
        .len();
    progress(
        "info",
        Some(record_count),
        Some(groups.len()),
        &format!(
            "Temporary MLIP validated: {local_size} bytes, sha256={expected_hash}; uploading staged database"
        ),
    );

    let suffix = unique_suffix();
    let temporary_name = format!(".library.db.{suffix}.tmp");
    let backup_name = format!(".library.db.{suffix}.bak");
    let temporary_path = join_remote_path(target_root, &temporary_name);
    let backup_path = join_remote_path(target_root, &backup_name);
    client
        .upload_file(target_root, &temporary_name, local.path())
        .await
        .map_err(|error| error.to_string())?;
    progress(
        "info",
        None,
        None,
        &format!("Uploaded staged MLIP '{temporary_path}'; verifying remote SHA-256"),
    );
    let uploaded_hash = match client.sha256_file(&temporary_path).await {
        Ok(hash) => hash,
        Err(error) => {
            let cleanup = client.delete_file(&temporary_path).await.err();
            return Err(match cleanup {
                Some(cleanup) => format!(
                    "verify uploaded remote MLIP failed: {error}; temporary cleanup failed: {cleanup}"
                ),
                None => format!("verify uploaded remote MLIP failed: {error}"),
            });
        }
    };
    if uploaded_hash != expected_hash {
        let cleanup = client.delete_file(&temporary_path).await.err();
        return Err(match cleanup {
            Some(error) => {
                format!("uploaded remote MLIP hash mismatch; temporary cleanup failed: {error}")
            }
            None => "uploaded remote MLIP hash mismatch".to_string(),
        });
    }
    progress(
        "info",
        None,
        None,
        &format!("Staged MLIP SHA-256 verified: {uploaded_hash}"),
    );

    if had_database {
        if let Err(error) = client
            .rename_file(
                &database_path,
                backup_path
                    .rsplit_once('/')
                    .map_or(backup_name.as_str(), |(_, name)| name),
            )
            .await
        {
            let cleanup = client.delete_file(&temporary_path).await.err();
            return Err(match cleanup {
                Some(cleanup) => {
                    format!("{error}; temporary MLIP cleanup failed: {cleanup}")
                }
                None => error.to_string(),
            });
        }
        progress(
            "info",
            None,
            None,
            &format!("Renamed existing MLIP to rollback backup '{backup_path}'"),
        );
    }
    if let Err(error) = client
        .rename_file(
            &temporary_path,
            database_path
                .rsplit_once('/')
                .map_or("library.db", |(_, name)| name),
        )
        .await
    {
        let restore = if had_database {
            client.rename_file(&backup_path, "library.db").await.err()
        } else {
            None
        };
        let cleanup = client.delete_file(&temporary_path).await.err();
        let mut message = error.to_string();
        if let Some(restore) = restore {
            message.push_str(&format!("; restore previous remote MLIP failed: {restore}"));
        }
        if let Some(cleanup) = cleanup {
            message.push_str(&format!("; temporary MLIP cleanup failed: {cleanup}"));
        }
        return Err(message);
    }
    progress(
        "info",
        None,
        None,
        &format!("Promoted staged MLIP to '{database_path}'; verifying installed SHA-256"),
    );

    let installed_hash = client
        .sha256_file(&database_path)
        .await
        .map_err(|error| error.to_string());
    if installed_hash.as_deref() != Ok(expected_hash.as_str()) {
        let mut message = match installed_hash {
            Ok(_) => "installed remote MLIP hash mismatch".to_string(),
            Err(error) => format!("verify installed remote MLIP failed: {error}"),
        };
        if let Err(error) = client.delete_file(&database_path).await {
            message.push_str(&format!("; invalid remote MLIP cleanup failed: {error}"));
        }
        if had_database {
            if let Err(error) = client.rename_file(&backup_path, "library.db").await {
                message.push_str(&format!("; restore previous remote MLIP failed: {error}"));
            }
        }
        return Err(message);
    }
    if had_database {
        client
            .delete_file(&backup_path)
            .await
            .map_err(|error| error.to_string())?;
        progress(
            "info",
            None,
            None,
            &format!("Installed MLIP verified; removed rollback backup '{backup_path}'"),
        );
    } else {
        progress("info", None, None, "Installed MLIP verified");
    }
    Ok(())
}

async fn build_remote_index_records(
    client: &dyn CloudDriveClientTrait,
    target_root: &str,
    groups: &[PlannedGroup],
    cached_hashes: &HashMap<(String, Option<i64>), String>,
    progress: &ProgressReporter<'_>,
) -> Result<Vec<LibraryIndexRecord>, String> {
    let mut records = Vec::with_capacity(groups.len());
    for (group_index, group) in groups.iter().enumerate() {
        let media_index = group
            .files
            .iter()
            .position(|file| !is_external_subtitle(&file.path))
            .ok_or_else(|| "remote media bundle has no video".to_string())?;
        let media = &group.files[media_index];
        let media_target = join_remote_path(&group.destination, &group.names[media_index]);
        let hash_path = if group.action == GroupAction::Move {
            media.path.as_str()
        } else {
            media_target.as_str()
        };
        let relative_path = remote_relative_path(target_root, &media_target)?;
        let media_size = (media.size >= 0).then_some(media.size);
        progress(
            "info",
            Some(group_index + 1),
            Some(groups.len()),
            &format!(
                "Indexing media {}/{}: source='{}', target='{}', relative_path='{}', size={:?}",
                group_index + 1,
                groups.len(),
                hash_path,
                media_target,
                relative_path,
                media_size
            ),
        );
        let (sha256_full, hash_source) = if let Some(hash) =
            cached_hashes.get(&(relative_path.clone(), media_size))
        {
            (hash.clone(), "reused from existing MLIP")
        } else {
            progress(
                    "info",
                    Some(group_index + 1),
                    Some(groups.len()),
                    &format!(
                        "Downloading original media {}/{} to a daemon temporary file for local SHA-256: '{}'",
                        group_index + 1,
                        groups.len(),
                        hash_path
                    ),
                );
            let local_media = LocalTempMedia::new();
            client
                .download_file(hash_path, local_media.path())
                .await
                .map_err(|error| error.to_string())?;
            if let Some(expected_size) = media_size {
                let downloaded_size = std::fs::metadata(local_media.path())
                    .map_err(|error| format!("read downloaded media metadata failed: {error}"))?
                    .len();
                if downloaded_size != expected_size as u64 {
                    return Err(format!(
                            "downloaded media size mismatch for '{hash_path}': expected {expected_size}, got {downloaded_size}"
                        ));
                }
            }
            (
                sha256_local_file(local_media.path())?,
                "computed from downloaded original media",
            )
        };
        if sha256_full.len() != 64 || !sha256_full.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("CloudDrive returned an invalid media SHA-256".to_string());
        }
        let subtitle_paths = group
            .files
            .iter()
            .zip(&group.names)
            .filter(|(file, _)| is_external_subtitle(&file.path))
            .map(|(_, name)| {
                remote_relative_path(target_root, &join_remote_path(&group.destination, name))
            })
            .collect::<Result<Vec<_>, _>>()?;
        progress(
            "info",
            Some(group_index + 1),
            Some(groups.len()),
            &format!(
                "MLIP record {}/{}: series='{}', season={}, episode={}, path='{}', subtitles={:?}, sha256={} ({hash_source})",
                group_index + 1,
                groups.len(),
                group.series_title,
                group.season,
                group.episode,
                relative_path,
                subtitle_paths,
                sha256_full.to_ascii_lowercase()
            ),
        );
        records.push(LibraryIndexRecord {
            series_title: group.series_title.clone(),
            original_title: None,
            sort_title: None,
            summary: None,
            year: None,
            air_date: None,
            series_type: 1,
            season: group.season,
            episode: group.episode,
            sort_order: group.episode,
            episode_title: None,
            episode_summary: None,
            runtime: None,
            relative_path,
            size: media_size,
            modified_time: None,
            sha256_full: Some(sha256_full.to_ascii_lowercase()),
            subtitle_paths,
            genres: Vec::new(),
            external_ids: Vec::new(),
            series_artwork: Vec::new(),
            episode_artwork: Vec::new(),
        });
    }
    Ok(records)
}

fn remote_relative_path(root: &str, path: &str) -> Result<String, String> {
    let prefix = format!("{}/", root.trim_end_matches('/'));
    path.strip_prefix(&prefix)
        .filter(|relative| !relative.is_empty() && !relative.starts_with('/'))
        .map(str::to_string)
        .ok_or_else(|| format!("remote MLIP path is outside target root: {path}"))
}

fn sha256_local_file(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path)
        .map_err(|error| format!("open local file for hashing failed: {error}"))?;
    let mut sha = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let length = file
            .read(&mut buffer)
            .map_err(|error| format!("read local file for hashing failed: {error}"))?;
        if length == 0 {
            break;
        }
        sha.update(&buffer[..length]);
    }
    Ok(format!("{:x}", sha.finalize()))
}

fn unique_suffix() -> String {
    format!(
        "{}-{}-{}",
        std::process::id(),
        REMOTE_MLIP_COUNTER.fetch_add(1, Ordering::Relaxed),
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
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

async fn group_was_copied(
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
        if !client
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

async fn backfill_legacy_download_correlations(
    db: &RssDatabase,
    subscription_id: i64,
    tasks: &mut [DownloadTask],
    offline_files: &[proto::OfflineFile],
    offline_hash_counts: &HashMap<String, usize>,
    progress: &ProgressReporter<'_>,
) -> usize {
    let candidate_indices = tasks
        .iter()
        .enumerate()
        .filter(|(_, task)| {
            task.info_hash.as_deref().is_none_or(str::is_empty)
                && task.status.as_deref() != Some("completed")
                && dmhy_topic_url(&task.item_hash).is_some()
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if candidate_indices.is_empty() {
        return 0;
    }
    let http = match build_http_client(&ProxyConfig::from_env()) {
        Ok(client) => client,
        Err(error) => {
            progress(
                "warning",
                Some(0),
                Some(candidate_indices.len()),
                &format!("Legacy correlation fallback could not create HTTP client: {error}"),
            );
            return 0;
        }
    };

    let mut recovered = 0usize;
    for (position, task_index) in candidate_indices.iter().copied().enumerate() {
        let item_hash = tasks[task_index].item_hash.clone();
        progress(
            "info",
            Some(position + 1),
            Some(candidate_indices.len()),
            &format!(
                "Legacy correlation fallback {}/{}: downloading DMHY torrent for task {} from '{}'",
                position + 1,
                candidate_indices.len(),
                tasks[task_index].id,
                item_hash
            ),
        );
        let info_hash = match recover_dmhy_topic_info_hash(&http, &item_hash).await {
            Ok(info_hash) => info_hash,
            Err(error) => {
                progress(
                    "warning",
                    Some(position + 1),
                    Some(candidate_indices.len()),
                    &format!(
                        "Legacy correlation fallback skipped task {}: {error}",
                        tasks[task_index].id
                    ),
                );
                continue;
            }
        };
        if offline_hash_counts.get(&info_hash) != Some(&1) {
            progress(
                "warning",
                Some(position + 1),
                Some(candidate_indices.len()),
                &format!(
                    "Legacy correlation fallback skipped task {}: computed BTIH {info_hash} does not identify exactly one CloudDrive offline task",
                    tasks[task_index].id
                ),
            );
            continue;
        }
        if tasks.iter().any(|task| {
            task.info_hash
                .as_deref()
                .and_then(normalize_info_hash)
                .as_deref()
                == Some(info_hash.as_str())
        }) {
            progress(
                "warning",
                Some(position + 1),
                Some(candidate_indices.len()),
                &format!(
                    "Legacy correlation fallback skipped task {}: computed BTIH {info_hash} is already assigned to another RSS task",
                    tasks[task_index].id
                ),
            );
            continue;
        }
        let matching_offline = offline_files.iter().find(|offline| {
            normalize_info_hash(&offline.info_hash).as_deref() == Some(info_hash.as_str())
        });
        let Some(matching_offline) = matching_offline else {
            continue;
        };
        if !safe_component(&matching_offline.name) {
            progress(
                "warning",
                Some(position + 1),
                Some(candidate_indices.len()),
                &format!(
                    "Legacy correlation fallback skipped task {}: CloudDrive returned an unsafe remote name",
                    tasks[task_index].id
                ),
            );
            continue;
        }
        if let Err(error) = db.save_download_correlation(
            subscription_id,
            &item_hash,
            &info_hash,
            Some(&matching_offline.name),
        ) {
            progress(
                "warning",
                Some(position + 1),
                Some(candidate_indices.len()),
                &format!(
                    "Legacy correlation fallback could not persist task {}: {error}",
                    tasks[task_index].id
                ),
            );
            continue;
        }
        tasks[task_index].info_hash = Some(info_hash.clone());
        tasks[task_index].remote_name = Some(matching_offline.name.clone());
        recovered += 1;
        progress(
            "info",
            Some(position + 1),
            Some(candidate_indices.len()),
            &format!(
                "Legacy correlation recovered task {}: BTIH={info_hash}, remote_name='{}'",
                tasks[task_index].id, matching_offline.name
            ),
        );
    }
    recovered
}

async fn recover_dmhy_topic_info_hash(
    client: &reqwest::Client,
    topic: &str,
) -> Result<String, String> {
    let topic_url = dmhy_topic_url(topic)
        .ok_or_else(|| "stored item is not an allowed DMHY topic URL".to_string())?;
    let response = client
        .get(topic_url.clone())
        .header(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (compatible; anime-organizer/1.0)",
        )
        .send()
        .await
        .map_err(|error| format!("download DMHY topic failed: {error}"))?
        .error_for_status()
        .map_err(|error| format!("download DMHY topic failed: {error}"))?;
    let html = response
        .text()
        .await
        .map_err(|error| format!("read DMHY topic failed: {error}"))?;
    let torrent_url = extract_dmhy_torrent_url(&topic_url, &html)
        .ok_or_else(|| "DMHY topic contains no allowed .torrent link".to_string())?;
    let magnet = download_torrent_to_magnet(client, torrent_url.as_str())
        .await
        .map_err(|error| format!("download or parse DMHY torrent failed: {error}"))?;
    magnet_info_hash(&magnet)
        .ok_or_else(|| "downloaded DMHY torrent produced no valid v1 BTIH".to_string())
}

fn dmhy_topic_url(value: &str) -> Option<url::Url> {
    let url = url::Url::parse(value).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    ((url.scheme() == "http" || url.scheme() == "https")
        && allowed_dmhy_host(&host)
        && url.path().starts_with("/topics/view/")
        && url.username().is_empty()
        && url.password().is_none())
    .then_some(url)
}

fn extract_dmhy_torrent_url(topic_url: &url::Url, html: &str) -> Option<url::Url> {
    TORRENT_HREF_REGEX.captures_iter(html).find_map(|captures| {
        let href = captures.get(1)?.as_str().replace("&amp;", "&");
        let url = topic_url.join(&href).ok()?;
        let host = url.host_str()?.to_ascii_lowercase();
        ((url.scheme() == "http" || url.scheme() == "https")
            && allowed_dmhy_host(&host)
            && url.username().is_empty()
            && url.password().is_none())
        .then_some(url)
    })
}

fn allowed_dmhy_host(host: &str) -> bool {
    host == "dmhy.org"
        || host.ends_with(".dmhy.org")
        || (cfg!(test) && matches!(host, "127.0.0.1" | "localhost"))
}

fn magnet_info_hash(magnet: &str) -> Option<String> {
    let url = url::Url::parse(magnet).ok()?;
    url.query_pairs().find_map(|(key, value)| {
        key.eq_ignore_ascii_case("xt")
            .then(|| value.strip_prefix("urn:btih:"))
            .flatten()
            .and_then(normalize_info_hash)
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

    type MoveCall = (Vec<String>, String);
    type CopyCall = (Vec<String>, String);

    #[derive(Clone, Default)]
    struct MockCloud {
        folders: Arc<Mutex<BTreeMap<String, Vec<proto::CloudDriveFile>>>>,
        moves: Arc<Mutex<Vec<MoveCall>>>,
        copies: Arc<Mutex<Vec<CopyCall>>>,
        deletes: Arc<Mutex<Vec<String>>>,
        downloads: Arc<Mutex<Vec<String>>>,
        listed_paths: Arc<Mutex<Vec<String>>>,
        offline_calls: Arc<AtomicUsize>,
        find_calls: Arc<AtomicUsize>,
        offline_files: Arc<Mutex<Vec<proto::OfflineFile>>>,
        hashes: Arc<Mutex<HashMap<String, String>>>,
        remote_bytes: Arc<Mutex<HashMap<String, Vec<u8>>>>,
        hash_calls: Arc<Mutex<Vec<String>>>,
        fail_hashes: Arc<AtomicUsize>,
        fail_moves: Arc<AtomicUsize>,
        fail_uploads: Arc<AtomicUsize>,
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
            let configured = self.offline_files.lock().unwrap().clone();
            if !configured.is_empty() {
                return Ok(configured);
            }
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
            self.find_calls.fetch_add(1, Ordering::SeqCst);
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
        async fn copy_files(&self, paths: Vec<String>, destination: &str) -> Result<()> {
            let mut folders = self.folders.lock().unwrap();
            let copied = paths
                .iter()
                .map(|path| {
                    let parent = remote_parent(path).expect("mock source has a parent");
                    let name = remote_file_name(path).expect("mock source has a name");
                    folders
                        .get(parent)
                        .and_then(|entries| entries.iter().find(|entry| entry.name == name))
                        .cloned()
                        .expect("mock source exists")
                })
                .collect::<Vec<_>>();
            let destination_entries = folders.entry(destination.to_string()).or_default();
            for mut entry in copied {
                entry.name = remote_file_name(&entry.full_path_name)
                    .expect("mock source has a name")
                    .to_string();
                entry.full_path_name = join_remote_path(destination, &entry.name);
                destination_entries.push(entry);
            }
            self.copies
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
            self.remote_bytes.lock().unwrap().remove(path);
            self.deletes.lock().unwrap().push(path.to_string());
            Ok(())
        }
        async fn download_file(&self, path: &str, destination: &Path) -> Result<()> {
            self.downloads.lock().unwrap().push(path.to_string());
            let bytes = self
                .remote_bytes
                .lock()
                .unwrap()
                .get(path)
                .cloned()
                .ok_or_else(|| {
                    anime_organizer::error::AppError::MetadataFetchError(
                        "mock remote bytes were not found".to_string(),
                    )
                })?;
            std::fs::write(destination, bytes).unwrap();
            Ok(())
        }
        async fn upload_file(&self, parent: &str, name: &str, source: &Path) -> Result<()> {
            if self
                .fail_uploads
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
                    (remaining > 0).then_some(remaining.saturating_sub(1))
                })
                .is_ok()
            {
                return Err(anime_organizer::error::AppError::MetadataFetchError(
                    "transient upload failure".to_string(),
                ));
            }
            let bytes = std::fs::read(source).unwrap();
            let path = join_remote_path(parent, name);
            self.remote_bytes
                .lock()
                .unwrap()
                .insert(path.clone(), bytes.clone());
            self.folders
                .lock()
                .unwrap()
                .entry(parent.to_string())
                .or_default()
                .push(file_with_size(&path, false, bytes.len() as i64));
            Ok(())
        }
        async fn rename_file(&self, path: &str, new_name: &str) -> Result<()> {
            let parent = remote_parent(path).expect("mock rename source has a parent");
            let new_path = join_remote_path(parent, new_name);
            let bytes = { self.remote_bytes.lock().unwrap().remove(path) };
            if let Some(bytes) = bytes {
                self.remote_bytes
                    .lock()
                    .unwrap()
                    .insert(new_path.clone(), bytes);
            }
            let mut folders = self.folders.lock().unwrap();
            let entry = folders
                .get_mut(parent)
                .and_then(|entries| {
                    entries
                        .iter_mut()
                        .find(|entry| entry.full_path_name == path)
                })
                .ok_or_else(|| {
                    anime_organizer::error::AppError::MetadataFetchError(
                        "mock rename source was not found".to_string(),
                    )
                })?;
            entry.name = new_name.to_string();
            entry.full_path_name = new_path;
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
            if let Some(bytes) = self.remote_bytes.lock().unwrap().get(path) {
                return Ok(format!("{:x}", Sha256::digest(bytes)));
            }
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

    fn finished_offline(name: &str, info_hash: &str) -> proto::OfflineFile {
        proto::OfflineFile {
            name: name.to_string(),
            status: proto::OfflineFileStatus::OfflineFinished as i32,
            info_hash: info_hash.to_string(),
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

    fn configured_mlip_subscription(db: &RssDatabase, remove_empty_dirs: bool) -> i64 {
        let id = db
            .add_subscription("https://example.test/rss", None, "/source", 300)
            .unwrap();
        db.update_subscription_organization_settings_with_mlip(
            id,
            true,
            Some("/library"),
            false,
            remove_empty_dirs,
            true,
        )
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

    fn configured_original_subscription(db: &RssDatabase, remote_mlip: bool) -> i64 {
        let id = db
            .add_subscription("https://example.test/original.xml", None, "/source", 300)
            .unwrap();
        db.update_subscription_organization_settings_with_mlip_and_mode(
            id,
            true,
            Some("/library"),
            false,
            false,
            remote_mlip,
            "original",
        )
        .unwrap();
        id
    }

    #[tokio::test]
    async fn original_mode_with_mlip_downloads_media_once_hashes_locally_then_moves() {
        let client = MockCloud::default();
        let video = "/source/[ANi] Original Show - 01 [1080P].mkv";
        let video_bytes = vec![7; 42];
        let expected_hash = format!("{:x}", Sha256::digest(&video_bytes));
        client.folders.lock().unwrap().extend([
            (
                "/source".to_string(),
                vec![file_with_size(video, false, video_bytes.len() as i64)],
            ),
            ("/library".to_string(), Vec::new()),
        ]);
        client
            .remote_bytes
            .lock()
            .unwrap()
            .insert(video.to_string(), video_bytes);
        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = configured_original_subscription(&db, true);

        let summary =
            organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
                .await
                .unwrap();

        assert_eq!(summary.moved_media, 1);
        assert_eq!(summary.copied_media, 0);
        assert_eq!(client.offline_calls.load(Ordering::SeqCst), 0);
        assert_eq!(client.downloads.lock().unwrap().as_slice(), [video]);
        assert_eq!(client.moves.lock().unwrap().len(), 1);
        assert!(client.copies.lock().unwrap().is_empty());
        let published = client
            .remote_bytes
            .lock()
            .unwrap()
            .get("/library/library.db")
            .cloned()
            .unwrap();
        let local = directory.path().join("original.db");
        std::fs::write(&local, published).unwrap();
        let conn = rusqlite::Connection::open(local).unwrap();
        assert_eq!(
            conn.query_row("SELECT sha256_full FROM media_file", [], |row| {
                row.get::<_, String>(0)
            })
            .unwrap(),
            expected_hash
        );
    }

    #[tokio::test]
    async fn original_mode_without_mlip_copies_without_download_and_is_idempotent() {
        let client = MockCloud::default();
        let video = "/source/[ANi] Copy Show - 01 [1080P].mkv";
        client.folders.lock().unwrap().extend([
            (
                "/source".to_string(),
                vec![file_with_size(video, false, 42)],
            ),
            ("/library".to_string(), Vec::new()),
        ]);
        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = configured_original_subscription(&db, false);
        let subscription = db.get_subscription(id).unwrap().unwrap();

        let first = organize_subscription(&db, &subscription, &client)
            .await
            .unwrap();
        let second = organize_subscription(&db, &subscription, &client)
            .await
            .unwrap();

        assert_eq!(first.copied_media, 1);
        assert_eq!(first.moved_media, 0);
        assert_eq!(second.copied_media, 0);
        assert_eq!(client.copies.lock().unwrap().len(), 1);
        assert!(client.moves.lock().unwrap().is_empty());
        assert!(client.downloads.lock().unwrap().is_empty());
        assert!(client.hash_calls.lock().unwrap().is_empty());
        assert_eq!(client.offline_calls.load(Ordering::SeqCst), 0);
        assert!(client
            .folders
            .lock()
            .unwrap()
            .get("/source")
            .is_some_and(|files| files
                .iter()
                .any(|file| file.name == remote_file_name(video).unwrap())));
    }

    #[tokio::test]
    async fn batches_multiple_finished_roots_into_one_remote_mlip_publication() {
        let client = MockCloud::default();
        let first_hash = "1111111111111111111111111111111111111111";
        let second_hash = "2222222222222222222222222222222222222222";
        let first_video = "/source/torrent-a/[ANi] First Show - 01 [1080P].mkv";
        let second_video = "/source/torrent-b/[ANi] Second Show - 02 [1080P].mkv";
        client.folders.lock().unwrap().extend([
            (
                "/source".to_string(),
                vec![
                    file("/source/torrent-a", true),
                    file("/source/torrent-b", true),
                ],
            ),
            (
                "/source/torrent-a".to_string(),
                vec![file_with_size(first_video, false, 42)],
            ),
            (
                "/source/torrent-b".to_string(),
                vec![file_with_size(second_video, false, 84)],
            ),
            ("/library".to_string(), Vec::new()),
        ]);
        client.offline_files.lock().unwrap().extend([
            finished_offline("torrent-a", first_hash),
            finished_offline("torrent-b", second_hash),
        ]);
        client.remote_bytes.lock().unwrap().extend([
            (first_video.to_string(), vec![1; 42]),
            (second_video.to_string(), vec![2; 84]),
        ]);
        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = db
            .add_subscription("https://example.test/rss", None, "/source", 300)
            .unwrap();
        db.update_subscription_organization_settings_with_mlip(
            id,
            true,
            Some("/library"),
            false,
            false,
            true,
        )
        .unwrap();
        for (item, hash) in [("first", first_hash), ("second", second_hash)] {
            db.save_download_task(id, item).unwrap();
            db.save_download_correlation(id, item, hash, None).unwrap();
        }

        let events = Arc::new(Mutex::new(Vec::new()));
        let captured_events = Arc::clone(&events);
        organize_subscription_with_progress(
            &db,
            &db.get_subscription(id).unwrap().unwrap(),
            &client,
            &move |level, current, total, message| {
                captured_events.lock().unwrap().push((
                    level.to_string(),
                    current,
                    total,
                    message.to_string(),
                ));
            },
        )
        .await
        .unwrap();

        let events = events.lock().unwrap();
        let log = events
            .iter()
            .map(|(_, _, _, message)| message.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(log.contains(&format!("Parsed '{first_video}'")));
        assert!(log.contains("series='First Show', season=1, episode=1"));
        assert!(log.contains(&format!("source='{first_video}'")));
        assert!(log.contains("target='/library/First Show/[ANi] First Show - 01 [1080P].mkv'"));
        assert!(log.contains("Downloading original media 1/2"));
        assert!(log.contains("MLIP record 1/2"));
        assert!(log.contains("Moving bundle 1/2"));
        assert!(log.contains("Move verified for series='First Show'"));
        assert!(log.contains("Remote organization finished"));
        assert!(events
            .iter()
            .any(|(_, current, total, _)| *current == Some(1) && *total == Some(2)));
        drop(events);

        let temporary_hashes = client
            .hash_calls
            .lock()
            .unwrap()
            .iter()
            .filter(|path| path.starts_with("/library/.library.db.") && path.ends_with(".tmp"))
            .count();
        assert_eq!(temporary_hashes, 1);
        assert_eq!(client.offline_calls.load(Ordering::SeqCst), 1);
        assert_eq!(client.find_calls.load(Ordering::SeqCst), 0);
        let source_root_lists = client
            .listed_paths
            .lock()
            .unwrap()
            .iter()
            .filter(|path| path.as_str() == "/source")
            .count();
        assert_eq!(source_root_lists, 1);
        let bytes = client
            .remote_bytes
            .lock()
            .unwrap()
            .get("/library/library.db")
            .cloned()
            .unwrap();
        let local_db = directory.path().join("batched.db");
        std::fs::write(&local_db, bytes).unwrap();
        let conn = rusqlite::Connection::open(local_db).unwrap();
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM media_file", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert!(db
            .list_download_tasks(id, None)
            .unwrap()
            .iter()
            .all(|task| task.status.as_deref() == Some("completed")));
    }

    #[tokio::test]
    async fn publishes_remote_mlip_with_video_hash_and_subtitle_before_cleanup() {
        let client = MockCloud::default();
        let video = "/source/torrent/[ANi] Test Show - 01 [1080P].mkv";
        let subtitle = "/source/torrent/[ANi] Test Show - 01 [1080P].zh-CN.ass";
        client.folders.lock().unwrap().extend([
            ("/source".to_string(), vec![file("/source/torrent", true)]),
            (
                "/source/torrent".to_string(),
                vec![
                    file_with_size(video, false, 42),
                    file_with_size(subtitle, false, 12),
                ],
            ),
            ("/library".to_string(), Vec::new()),
        ]);
        let video_bytes = vec![3; 42];
        let expected_video_hash = format!("{:x}", Sha256::digest(&video_bytes));
        client
            .remote_bytes
            .lock()
            .unwrap()
            .insert(video.to_string(), video_bytes);
        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = configured_mlip_subscription(&db, true);

        organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
            .await
            .unwrap();

        let bytes = client
            .remote_bytes
            .lock()
            .unwrap()
            .get("/library/library.db")
            .cloned()
            .expect("published library.db");
        let local_db = directory.path().join("published.db");
        std::fs::write(&local_db, bytes).unwrap();
        let conn = rusqlite::Connection::open(local_db).unwrap();
        let media: (String, String) = conn
            .query_row("SELECT path, sha256_full FROM media_file", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap();
        assert_eq!(
            media,
            (
                "Test Show/[ANi] Test Show - 01 [1080P].mkv".to_string(),
                expected_video_hash
            )
        );
        let subtitle_path: String = conn
            .query_row("SELECT path FROM media_subtitle", [], |row| row.get(0))
            .unwrap();
        assert_eq!(
            subtitle_path,
            "Test Show/[ANi] Test Show - 01 [1080P].zh-CN.ass"
        );
        assert_eq!(
            db.list_download_tasks(id, None).unwrap()[0]
                .status
                .as_deref(),
            Some("completed")
        );
        assert!(client
            .deletes
            .lock()
            .unwrap()
            .iter()
            .any(|path| path == "/source/torrent"));
    }

    #[tokio::test]
    async fn existing_remote_mlip_is_downloaded_updated_and_replaced() {
        let client = MockCloud::default();
        let video = "/source/torrent/[ANi] Test Show - 02 [1080P].mkv";
        client.folders.lock().unwrap().extend([
            ("/source".to_string(), vec![file("/source/torrent", true)]),
            (
                "/source/torrent".to_string(),
                vec![file_with_size(video, false, 84)],
            ),
        ]);
        client
            .hashes
            .lock()
            .unwrap()
            .insert(video.to_string(), "c".repeat(64));
        let directory = tempfile::tempdir().unwrap();
        let initial_path = directory.path().join("initial.db");
        let mut initial = LibraryIndexRecord::new(
            "Existing Show".to_string(),
            1,
            1.0,
            "Existing Show/01.mkv".to_string(),
            &directory.path().join("not-mounted.mkv"),
        );
        initial.sha256_full = Some("d".repeat(64));
        let mut cached = LibraryIndexRecord::new(
            "Test Show".to_string(),
            1,
            2.0,
            "Test Show/[ANi] Test Show - 02 [1080P].mkv".to_string(),
            &directory.path().join("not-mounted-cached.mkv"),
        );
        cached.size = Some(84);
        cached.sha256_full = Some("c".repeat(64));
        LibraryIndex::update_remote_database(&initial_path, "/library", &[initial, cached])
            .unwrap();
        let conn = rusqlite::Connection::open(&initial_path).unwrap();
        conn.execute(
            "UPDATE series SET summary = 'keep me' WHERE title = 'Existing Show'",
            [],
        )
        .unwrap();
        drop(conn);
        let initial_bytes = std::fs::read(&initial_path).unwrap();
        client
            .remote_bytes
            .lock()
            .unwrap()
            .insert("/library/library.db".to_string(), initial_bytes.clone());
        client.folders.lock().unwrap().insert(
            "/library".to_string(),
            vec![file_with_size(
                "/library/library.db",
                false,
                initial_bytes.len() as i64,
            )],
        );
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = configured_mlip_subscription(&db, false);

        organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
            .await
            .unwrap();

        let published = client
            .remote_bytes
            .lock()
            .unwrap()
            .get("/library/library.db")
            .cloned()
            .unwrap();
        let published_path = directory.path().join("published-existing.db");
        std::fs::write(&published_path, published).unwrap();
        let conn = rusqlite::Connection::open(published_path).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT summary FROM series WHERE title = 'Existing Show'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
            "keep me"
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM media_file", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert!(!client
            .hash_calls
            .lock()
            .unwrap()
            .contains(&video.to_string()));
        assert!(!client
            .remote_bytes
            .lock()
            .unwrap()
            .keys()
            .any(|path| path.ends_with(".bak") || path.ends_with(".tmp")));
    }

    #[tokio::test]
    async fn remote_mlip_upload_failure_preserves_source_and_retryable_task() {
        let client = MockCloud::default();
        let video = "/source/torrent/[ANi] Test Show - 01 [1080P].mkv";
        client.folders.lock().unwrap().extend([
            ("/source".to_string(), vec![file("/source/torrent", true)]),
            (
                "/source/torrent".to_string(),
                vec![file_with_size(video, false, 42)],
            ),
            ("/library".to_string(), Vec::new()),
        ]);
        client
            .remote_bytes
            .lock()
            .unwrap()
            .insert(video.to_string(), vec![4; 42]);
        client.fail_uploads.store(1, Ordering::SeqCst);
        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let id = configured_mlip_subscription(&db, true);

        let error = organize_subscription(&db, &db.get_subscription(id).unwrap().unwrap(), &client)
            .await
            .unwrap_err();

        assert!(error.contains("transient upload failure"));
        assert!(client.moves.lock().unwrap().is_empty());
        assert!(client.deletes.lock().unwrap().is_empty());
        assert!(client
            .folders
            .lock()
            .unwrap()
            .get("/source/torrent")
            .is_some_and(|files| files.iter().any(|file| file.full_path_name == video)));
        assert_ne!(
            db.list_download_tasks(id, None).unwrap()[0]
                .status
                .as_deref(),
            Some("completed")
        );
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
        assert_eq!(
            client
                .listed_paths
                .lock()
                .unwrap()
                .iter()
                .filter(|path| path.as_str() == "/source")
                .count(),
            1
        );
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

    #[tokio::test]
    async fn dmhy_legacy_fallback_downloads_torrent_computes_btih_and_persists_match() {
        use axum::response::Html;
        use axum::routing::get;
        use axum::Router;

        let info = b"d4:name4:test12:piece lengthi262144e6:pieces20:00000000000000000000e";
        let torrent = [b"d4:info".as_slice(), info.as_slice(), b"e"].concat();
        let expected_hash = format!("{:x}", sha1::Sha1::digest(info));
        let torrent_bytes = torrent.clone();
        let app = Router::new()
            .route(
                "/topics/view/legacy.html",
                get(|| async { Html(r#"<a href="/legacy.torrent">download</a>"#) }),
            )
            .route(
                "/legacy.torrent",
                get(move || {
                    let torrent_bytes = torrent_bytes.clone();
                    async move { torrent_bytes }
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let topic = format!("http://{address}/topics/view/legacy.html");

        let directory = tempfile::tempdir().unwrap();
        let db = RssDatabase::new(&directory.path().join("rss.db")).unwrap();
        let subscription_id = db
            .add_subscription("https://example.test/rss", None, "/source", 300)
            .unwrap();
        db.save_download_task(subscription_id, &topic).unwrap();
        let mut tasks = db.list_download_tasks(subscription_id, None).unwrap();
        let offline_files = vec![finished_offline("legacy-root", &expected_hash)];
        let offline_hash_counts = HashMap::from([(expected_hash.clone(), 1usize)]);

        let recovered = backfill_legacy_download_correlations(
            &db,
            subscription_id,
            &mut tasks,
            &offline_files,
            &offline_hash_counts,
            &|_, _, _, _| {},
        )
        .await;
        server.abort();

        assert_eq!(recovered, 1);
        let task = db
            .list_download_tasks(subscription_id, None)
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(task.info_hash.as_deref(), Some(expected_hash.as_str()));
        assert_eq!(task.remote_name.as_deref(), Some("legacy-root"));
    }

    #[test]
    fn dmhy_legacy_fallback_extracts_only_allowed_torrent_links() {
        let topic =
            dmhy_topic_url("http://share.dmhy.org/topics/view/700008_legacy_item.html").unwrap();
        let html = r#"
            <a href="//dl.dmhy.org/2025/07/example.torrent">torrent</a>
            <a href="https://evil.example/payload.torrent">evil</a>
        "#;
        assert_eq!(
            extract_dmhy_torrent_url(&topic, html).unwrap().as_str(),
            "http://dl.dmhy.org/2025/07/example.torrent"
        );
        assert!(dmhy_topic_url("https://evil.example/topics/view/1.html").is_none());
        assert!(extract_dmhy_torrent_url(
            &topic,
            r#"<a href="https://evil.example/payload.torrent">evil</a>"#
        )
        .is_none());
    }

    #[test]
    fn dmhy_legacy_fallback_reads_hex_btih_from_computed_magnet() {
        assert_eq!(
            magnet_info_hash(
                "magnet:?xt=urn:btih:1432A848087810103FDBC2555B0087AD8F3395A4&dn=test"
            )
            .as_deref(),
            Some("1432a848087810103fdbc2555b0087ad8f3395a4")
        );
        assert!(magnet_info_hash("magnet:?xt=urn:btih:not-a-hash").is_none());
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
