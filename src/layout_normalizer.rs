//! Auditable library layout normalization planner and applier.

use crate::artwork_pack::{sha256_file, sha256_file_prefix};
use crate::error::{AppError, Result};
use crate::library_index::{update_staged_database, DATABASE_FILENAME};
use crate::parser::{split_series_and_season, FilenameParser};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::UNIX_EPOCH;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use walkdir::WalkDir;

const PLAN_VERSION: u32 = 1;
const PREFIX_BYTES: u64 = 63 * 1024 * 1024;
const VIDEO_EXTENSIONS: &[&str] = &[
    "mkv", "mp4", "avi", "mov", "wmv", "flv", "webm", "m4v", "ts", "rmvb",
];
const SIDECAR_EXTENSIONS: &[&str] = &[
    "srt", "ass", "ssa", "sub", "vtt", "nfo", "jpg", "jpeg", "png", "webp",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileIdentity {
    pub size: u64,
    pub modified_time: Option<i64>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogicalIdentity {
    pub bangumi_id: Option<String>,
    pub series: String,
    pub season: i64,
    pub episode: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LayoutKind {
    FlatGenerated,
    SeasonGenerated,
    SeasonOriginal,
    FlatOriginal,
    Other,
}

impl LayoutKind {
    fn priority(self) -> u8 {
        match self {
            Self::SeasonOriginal => 4,
            Self::FlatOriginal => 3,
            Self::SeasonGenerated => 2,
            Self::FlatGenerated => 1,
            Self::Other => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LayoutActionKind {
    Keep,
    Move,
    Deduplicate,
    Conflict,
    Unresolved,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SidecarActionKind {
    Keep,
    Move,
    Deduplicate,
    Conflict,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SidecarAction {
    pub kind: SidecarActionKind,
    pub source: String,
    pub target: String,
    pub size: u64,
    pub modified_time: Option<i64>,
    pub sha256_full: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutAction {
    pub kind: LayoutActionKind,
    pub source: String,
    pub target: Option<String>,
    pub keeper: Option<String>,
    pub layout: LayoutKind,
    pub identity: Option<LogicalIdentity>,
    pub size: u64,
    pub modified_time: Option<i64>,
    pub sha256_prefix_63m: Option<String>,
    pub sha256_full: Option<String>,
    pub sidecars: Vec<SidecarAction>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutPlanSummary {
    pub keep: usize,
    pub move_files: usize,
    pub deduplicate: usize,
    pub conflict: usize,
    pub unresolved: usize,
    pub bytes_to_move: u64,
    pub bytes_to_release: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutPlan {
    pub version: u32,
    pub created_at: String,
    pub library_root: String,
    pub database: FileIdentity,
    pub summary: LayoutPlanSummary,
    pub actions: Vec<LayoutAction>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutApplySummary {
    pub moved: usize,
    pub deduplicated: usize,
    pub sidecars_moved: usize,
    pub sidecars_deduplicated: usize,
}

#[derive(Debug, Clone)]
struct CachedIdentity {
    logical: LogicalIdentity,
    size: Option<i64>,
    modified_time: Option<i64>,
    prefix: Option<String>,
    full: Option<String>,
}

#[derive(Debug, Clone)]
struct InventoryItem {
    action: LayoutAction,
    target: Option<String>,
}

pub fn build_layout_plan(
    target_root: &Path,
    output: &Path,
    force_rehash: bool,
    log: &dyn Fn(&str),
) -> Result<LayoutPlan> {
    let root = canonical_root(target_root)?;
    let db_path = root.join(DATABASE_FILENAME);
    let database = file_identity(&db_path, true)?;
    let cache = load_cache(&db_path)?;
    let mut items = Vec::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if !is_video(path) {
            continue;
        }
        let relative = relative_path(&root, path)?;
        let metadata = file_identity(path, false)?;
        let components = relative.split('/').collect::<Vec<_>>();
        let cached = cache.get(&relative).filter(|cached| {
            cached.size == i64::try_from(metadata.size).ok()
                && cached.modified_time == metadata.modified_time
        });
        let parsed = classify_path(path, &relative, &components, cached);
        let Some((layout, logical, target)) = parsed else {
            items.push(InventoryItem {
                target: None,
                action: LayoutAction {
                    kind: LayoutActionKind::Unresolved,
                    source: relative,
                    target: None,
                    keeper: None,
                    layout: LayoutKind::Other,
                    identity: None,
                    size: metadata.size,
                    modified_time: metadata.modified_time,
                    sha256_prefix_63m: None,
                    sha256_full: None,
                    sidecars: Vec::new(),
                    reason: Some("无法唯一识别作品、季或集数，或路径属于其他嵌套布局".to_string()),
                },
            });
            continue;
        };
        let prefix = if !force_rehash {
            cached.and_then(|cached| cached.prefix.clone())
        } else {
            None
        }
        .map_or_else(|| sha256_file_prefix(path, PREFIX_BYTES), Ok)?;
        let full = if force_rehash {
            None
        } else {
            cached.and_then(|cached| cached.full.clone())
        };
        let kind = if relative == target {
            LayoutActionKind::Keep
        } else {
            LayoutActionKind::Move
        };
        items.push(InventoryItem {
            target: Some(target.clone()),
            action: LayoutAction {
                kind,
                source: relative,
                target: Some(target),
                keeper: None,
                layout,
                identity: Some(logical),
                size: metadata.size,
                modified_time: metadata.modified_time,
                sha256_prefix_63m: Some(prefix),
                sha256_full: full,
                sidecars: Vec::new(),
                reason: None,
            },
        });
    }
    log(&format!("Scanned {} video files", items.len()));

    identify_duplicates(&root, &mut items, force_rehash)?;
    identify_target_conflicts(&root, &mut items);
    attach_sidecars(&root, &mut items)?;
    resolve_sidecar_conflicts(&mut items);

    let mut actions = items
        .into_iter()
        .map(|item| item.action)
        .collect::<Vec<_>>();
    actions.sort_by(|left, right| left.source.cmp(&right.source));
    let summary = summarize(&actions);
    let plan = LayoutPlan {
        version: PLAN_VERSION,
        created_at: OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .map_err(|error| plan_error(format!("格式化 plan 时间失败: {error}")))?,
        library_root: root.to_string_lossy().to_string(),
        database,
        summary,
        actions,
    };
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|error| plan_error(format!("创建 plan 目录失败: {error}")))?;
    }
    let bytes = serde_json::to_vec_pretty(&plan)
        .map_err(|error| plan_error(format!("序列化 layout plan 失败: {error}")))?;
    fs::write(output, bytes)
        .map_err(|error| plan_error(format!("写入 layout plan 失败: {error}")))?;
    log(&format!("Layout plan written to {}", output.display()));
    Ok(plan)
}

pub fn apply_layout_plan(
    target_root: &Path,
    input: &Path,
    confirmed: bool,
    log: &dyn Fn(&str),
) -> Result<LayoutApplySummary> {
    if !confirmed {
        return Err(plan_error("layout apply requires confirmed=true"));
    }
    let bytes =
        fs::read(input).map_err(|error| plan_error(format!("读取 layout plan 失败: {error}")))?;
    let plan: LayoutPlan = serde_json::from_slice(&bytes)
        .map_err(|error| plan_error(format!("解析 layout plan 失败: {error}")))?;
    if plan.version != PLAN_VERSION {
        return Err(plan_error(format!(
            "不支持的 layout plan 版本: {}",
            plan.version
        )));
    }
    let root = canonical_root(target_root)?;
    if root.to_string_lossy() != plan.library_root {
        return Err(plan_error("layout plan 媒体库根目录不匹配"));
    }
    let database_path = root.join(DATABASE_FILENAME);
    if file_identity(&database_path, true)? != plan.database {
        return Err(plan_error("library.db 在 plan 生成后已改变"));
    }
    validate_actions(&root, &plan.actions)?;

    let mut summary = LayoutApplySummary::default();
    for action in plan
        .actions
        .iter()
        .filter(|action| action.kind == LayoutActionKind::Move)
    {
        apply_sidecars(&root, &action.sidecars, &mut summary)?;
        let source = safe_join(&root, &action.source)?;
        let target = safe_join(&root, action.target.as_deref().unwrap_or_default())?;
        if source.exists() {
            if target.exists() {
                return Err(plan_error(format!("move 目标已存在: {}", target.display())));
            }
            fs::create_dir_all(target.parent().unwrap_or(&root))
                .map_err(|error| plan_error(format!("创建 Season 目录失败: {error}")))?;
            fs::rename(&source, &target).map_err(|error| {
                plan_error(format!("同库移动失败 {}: {error}", source.display()))
            })?;
            summary.moved += 1;
            log(&format!(
                "Moved {} -> {}",
                action.source,
                action.target.as_deref().unwrap_or_default()
            ));
        } else {
            verify_existing_target(&target, action)?;
        }
    }
    for action in plan
        .actions
        .iter()
        .filter(|action| action.kind == LayoutActionKind::Deduplicate)
    {
        apply_sidecars(&root, &action.sidecars, &mut summary)?;
        let source = safe_join(&root, &action.source)?;
        let keeper = safe_join(&root, action.keeper.as_deref().unwrap_or_default())?;
        if source.exists() {
            let expected = action
                .sha256_full
                .as_deref()
                .ok_or_else(|| plan_error("deduplicate action 缺少 full SHA-256"))?;
            if sha256_file(&source)? != expected || sha256_file(&keeper)? != expected {
                return Err(plan_error(format!(
                    "deduplicate full SHA-256 不匹配: {}",
                    action.source
                )));
            }
            fs::remove_file(&source).map_err(|error| {
                plan_error(format!("删除重复文件失败 {}: {error}", source.display()))
            })?;
            summary.deduplicated += 1;
            log(&format!("Deduplicated {}", action.source));
        } else if !keeper.exists() {
            return Err(plan_error(format!("keeper 不存在: {}", keeper.display())));
        }
    }

    update_database(&root, &plan.actions)?;
    remove_empty_source_directories(&root, &plan.actions);
    Ok(summary)
}

fn classify_path(
    path: &Path,
    relative: &str,
    components: &[&str],
    cached: Option<&CachedIdentity>,
) -> Option<(LayoutKind, LogicalIdentity, String)> {
    let (root_name, season_directory) = match components {
        [root, _file] => (*root, None),
        [root, season, _file] => {
            parse_season_directory(season).map(|number| (*root, Some(number)))?
        }
        _ => return None,
    };
    let original = FilenameParser::parse(path).is_some();
    let record = crate::library_index::LibraryIndexRecord::from_target_path(
        path.ancestors().nth(components.len())?,
        path,
    )
    .ok()??;
    let logical = cached.map_or_else(
        || LogicalIdentity {
            bangumi_id: None,
            series: record.series_title.clone(),
            season: season_directory.unwrap_or(record.season),
            episode: record.episode,
        },
        |cached| cached.logical.clone(),
    );
    if logical.season <= 0 || !logical.episode.is_finite() {
        return None;
    }
    let series_root = if season_directory.is_some() {
        root_name.to_string()
    } else {
        let split = split_series_and_season(root_name).0;
        if split.trim().is_empty() {
            record.series_title.clone()
        } else {
            split
        }
    };
    let layout = match (season_directory.is_some(), original) {
        (false, false) => LayoutKind::FlatGenerated,
        (true, false) => LayoutKind::SeasonGenerated,
        (true, true) => LayoutKind::SeasonOriginal,
        (false, true) => LayoutKind::FlatOriginal,
    };
    let file_name = relative.rsplit('/').next()?;
    let target = format!("{series_root}/Season {}/{file_name}", logical.season);
    Some((layout, logical, target))
}

fn identify_duplicates(root: &Path, items: &mut [InventoryItem], force_rehash: bool) -> Result<()> {
    let mut candidates: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, item) in items.iter().enumerate() {
        let Some(identity) = &item.action.identity else {
            continue;
        };
        let Some(prefix) = &item.action.sha256_prefix_63m else {
            continue;
        };
        candidates
            .entry(format!(
                "{}|{}|{:016x}|{}|{}",
                logical_key(identity),
                identity.season,
                identity.episode.to_bits(),
                item.action.size,
                prefix
            ))
            .or_default()
            .push(index);
    }
    for group in candidates.values().filter(|group| group.len() > 1) {
        for index in group {
            if force_rehash || items[*index].action.sha256_full.is_none() {
                let path = safe_join(root, &items[*index].action.source)?;
                items[*index].action.sha256_full = Some(sha256_file(&path)?);
            }
        }
        let mut by_full: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for index in group {
            by_full
                .entry(items[*index].action.sha256_full.clone().unwrap_or_default())
                .or_default()
                .push(*index);
        }
        for identical in by_full.values().filter(|identical| identical.len() > 1) {
            let keeper = *identical
                .iter()
                .max_by(|left, right| {
                    items[**left]
                        .action
                        .layout
                        .priority()
                        .cmp(&items[**right].action.layout.priority())
                        .then_with(|| {
                            items[**right]
                                .action
                                .source
                                .cmp(&items[**left].action.source)
                        })
                })
                .unwrap();
            let keeper_target = items[keeper].target.clone().unwrap();
            items[keeper].action.kind = if items[keeper].action.source == keeper_target {
                LayoutActionKind::Keep
            } else {
                LayoutActionKind::Move
            };
            items[keeper].action.target = Some(keeper_target.clone());
            for index in identical.iter().copied().filter(|index| *index != keeper) {
                items[index].action.kind = LayoutActionKind::Deduplicate;
                items[index].action.keeper = Some(keeper_target.clone());
                items[index].action.target = None;
                items[index].target = None;
            }
        }
    }
    Ok(())
}

fn identify_target_conflicts(root: &Path, items: &mut [InventoryItem]) {
    let source_owners = items
        .iter()
        .enumerate()
        .map(|(index, item)| (item.action.source.clone(), index))
        .collect::<HashMap<_, _>>();
    let mut targets: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, item) in items.iter().enumerate() {
        if matches!(
            item.action.kind,
            LayoutActionKind::Keep | LayoutActionKind::Move
        ) {
            if let Some(target) = &item.target {
                targets.entry(target.clone()).or_default().push(index);
            }
        }
    }
    for (target, indexes) in targets {
        let occupied = safe_join(root, &target).is_ok_and(|path| path.exists())
            && source_owners
                .get(&target)
                .is_none_or(|owner| !indexes.contains(owner));
        if indexes.len() > 1 || occupied {
            for index in indexes {
                items[index].action.kind = LayoutActionKind::Conflict;
                items[index].action.reason =
                    Some(format!("多个内容或外部文件占用目标路径 {target}"));
            }
        }
    }
}

fn attach_sidecars(root: &Path, items: &mut [InventoryItem]) -> Result<()> {
    for item in items {
        let destination_video = match item.action.kind {
            LayoutActionKind::Move => item.action.target.as_deref(),
            LayoutActionKind::Deduplicate => item.action.keeper.as_deref(),
            _ => None,
        };
        let Some(destination_video) = destination_video else {
            continue;
        };
        let source_video = safe_join(root, &item.action.source)?;
        let destination_video = safe_join(root, destination_video)?;
        let Some(source_stem) = source_video.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        let Some(destination_stem) = destination_video
            .file_stem()
            .and_then(|value| value.to_str())
        else {
            continue;
        };
        let Some(parent) = source_video.parent() else {
            continue;
        };
        for entry in fs::read_dir(parent)
            .map_err(|error| plan_error(format!("读取 sidecar 目录失败: {error}")))?
        {
            let path = entry
                .map_err(|error| plan_error(format!("读取 sidecar 失败: {error}")))?
                .path();
            if !path.is_file() || path == source_video {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            let Some(suffix) = file_name
                .strip_prefix(source_stem)
                .filter(|suffix| suffix.starts_with('.'))
            else {
                continue;
            };
            let extension = path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            if !SIDECAR_EXTENSIONS
                .iter()
                .any(|allowed| extension.eq_ignore_ascii_case(allowed))
            {
                continue;
            }
            let target_name = format!("{destination_stem}{suffix}");
            let target = destination_video.parent().unwrap_or(root).join(target_name);
            let source_relative = relative_path(root, &path)?;
            let target_relative = relative_path(root, &target)?;
            let identity = file_identity(&path, true)?;
            let kind = if path == target {
                SidecarActionKind::Keep
            } else if target.exists() {
                if sha256_file(&target)? == identity.sha256.as_deref().unwrap_or_default() {
                    SidecarActionKind::Deduplicate
                } else {
                    SidecarActionKind::Conflict
                }
            } else {
                SidecarActionKind::Move
            };
            item.action.sidecars.push(SidecarAction {
                kind,
                source: source_relative,
                target: target_relative,
                size: identity.size,
                modified_time: identity.modified_time,
                sha256_full: identity.sha256.unwrap(),
            });
        }
        item.action
            .sidecars
            .sort_by(|left, right| left.source.cmp(&right.source));
    }
    Ok(())
}

fn resolve_sidecar_conflicts(items: &mut [InventoryItem]) {
    let mut targets: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
    for (item_index, item) in items.iter().enumerate() {
        for (sidecar_index, sidecar) in item.action.sidecars.iter().enumerate() {
            if matches!(
                sidecar.kind,
                SidecarActionKind::Move | SidecarActionKind::Deduplicate
            ) {
                targets
                    .entry(sidecar.target.clone())
                    .or_default()
                    .push((item_index, sidecar_index));
            }
        }
    }
    for locations in targets.values().filter(|locations| locations.len() > 1) {
        let hashes = locations
            .iter()
            .map(|(item, sidecar)| items[*item].action.sidecars[*sidecar].sha256_full.as_str())
            .collect::<BTreeSet<_>>();
        if hashes.len() > 1 {
            for (item, sidecar) in locations {
                items[*item].action.sidecars[*sidecar].kind = SidecarActionKind::Conflict;
            }
            continue;
        }
        let mut ordered = locations.clone();
        ordered.sort_by(|left, right| {
            let execution_order = |item: usize| match items[item].action.kind {
                LayoutActionKind::Move => 0,
                LayoutActionKind::Deduplicate => 1,
                _ => 2,
            };
            execution_order(left.0)
                .cmp(&execution_order(right.0))
                .then_with(|| {
                    items[left.0].action.sidecars[left.1]
                        .source
                        .cmp(&items[right.0].action.sidecars[right.1].source)
                })
        });
        let existing_target = ordered.iter().any(|(item, sidecar)| {
            items[*item].action.sidecars[*sidecar].kind == SidecarActionKind::Deduplicate
        });
        for (position, (item, sidecar)) in ordered.into_iter().enumerate() {
            items[item].action.sidecars[sidecar].kind = if existing_target || position > 0 {
                SidecarActionKind::Deduplicate
            } else {
                SidecarActionKind::Move
            };
        }
    }
}

fn validate_actions(root: &Path, actions: &[LayoutAction]) -> Result<()> {
    for sidecar in actions
        .iter()
        .flat_map(|action| &action.sidecars)
        .filter(|sidecar| {
            matches!(
                sidecar.kind,
                SidecarActionKind::Move | SidecarActionKind::Deduplicate
            )
        })
    {
        let source = safe_join(root, &sidecar.source)?;
        let target = safe_join(root, &sidecar.target)?;
        if source.exists() {
            let identity = file_identity(&source, true)?;
            if identity.size != sidecar.size
                || identity.modified_time != sidecar.modified_time
                || identity.sha256.as_deref() != Some(&sidecar.sha256_full)
            {
                return Err(plan_error(format!(
                    "plan sidecar 已改变: {}",
                    source.display()
                )));
            }
        } else if !target.exists() || sha256_file(&target)? != sidecar.sha256_full {
            return Err(plan_error(format!(
                "plan sidecar 不存在: {}",
                source.display()
            )));
        }
    }
    for action in actions.iter().filter(|action| {
        matches!(
            action.kind,
            LayoutActionKind::Move | LayoutActionKind::Deduplicate
        )
    }) {
        let source = safe_join(root, &action.source)?;
        if !source.exists() {
            if action.kind == LayoutActionKind::Move {
                let target = safe_join(root, action.target.as_deref().unwrap_or_default())?;
                if target.exists() {
                    continue;
                }
            } else if action.kind == LayoutActionKind::Deduplicate {
                let keeper = safe_join(root, action.keeper.as_deref().unwrap_or_default())?;
                if keeper.exists() {
                    continue;
                }
            }
            return Err(plan_error(format!(
                "plan 源文件不存在: {}",
                source.display()
            )));
        }
        let identity = file_identity(&source, false)?;
        if identity.size != action.size || identity.modified_time != action.modified_time {
            return Err(plan_error(format!(
                "plan 源文件已改变: {}",
                source.display()
            )));
        }
    }
    Ok(())
}

fn apply_sidecars(
    root: &Path,
    sidecars: &[SidecarAction],
    summary: &mut LayoutApplySummary,
) -> Result<()> {
    for sidecar in sidecars {
        let source = safe_join(root, &sidecar.source)?;
        let target = safe_join(root, &sidecar.target)?;
        match sidecar.kind {
            SidecarActionKind::Move if source.exists() => {
                let identity = file_identity(&source, true)?;
                if identity.size != sidecar.size
                    || identity.modified_time != sidecar.modified_time
                    || identity.sha256.as_deref() != Some(&sidecar.sha256_full)
                {
                    return Err(plan_error(format!("sidecar 已改变: {}", source.display())));
                }
                if target.exists() {
                    return Err(plan_error(format!(
                        "sidecar 目标已存在: {}",
                        target.display()
                    )));
                }
                fs::create_dir_all(target.parent().unwrap_or(root))
                    .map_err(|error| plan_error(format!("创建 sidecar 目录失败: {error}")))?;
                fs::rename(&source, &target)
                    .map_err(|error| plan_error(format!("移动 sidecar 失败: {error}")))?;
                summary.sidecars_moved += 1;
            }
            SidecarActionKind::Deduplicate if source.exists() => {
                if sha256_file(&source)? != sidecar.sha256_full
                    || sha256_file(&target)? != sidecar.sha256_full
                {
                    return Err(plan_error(format!(
                        "sidecar 去重 hash 不匹配: {}",
                        source.display()
                    )));
                }
                fs::remove_file(&source)
                    .map_err(|error| plan_error(format!("删除重复 sidecar 失败: {error}")))?;
                summary.sidecars_deduplicated += 1;
            }
            _ => {}
        }
    }
    Ok(())
}

fn update_database(root: &Path, actions: &[LayoutAction]) -> Result<()> {
    update_staged_database(root, |conn| {
        let tx = conn
            .transaction()
            .map_err(|error| plan_error(format!("开始 layout DB 事务失败: {error}")))?;
        for action in actions
            .iter()
            .filter(|action| action.kind == LayoutActionKind::Deduplicate)
        {
            tx.execute(
                "DELETE FROM media_file WHERE path = ?1",
                params![action.source],
            )
            .map_err(|error| plan_error(format!("删除重复媒体索引失败: {error}")))?;
        }
        for action in actions {
            match action.kind {
                LayoutActionKind::Move => {
                    tx.execute(
                        "UPDATE media_file SET path = ?1, sha256_prefix_63m = ?2, sha256_full = ?3 \
                         WHERE path = ?4",
                        params![
                            action.target,
                            action.sha256_prefix_63m,
                            action.sha256_full,
                            action.source
                        ],
                    )
                    .map_err(|error| plan_error(format!("更新媒体路径索引失败: {error}")))?;
                }
                LayoutActionKind::Keep => {
                    tx.execute(
                        "UPDATE media_file SET sha256_prefix_63m = ?1, sha256_full = ?2 WHERE path = ?3",
                        params![action.sha256_prefix_63m, action.sha256_full, action.source],
                    )
                    .map_err(|error| plan_error(format!("更新媒体 hash 索引失败: {error}")))?;
                }
                _ => {}
            }
        }
        for action in actions {
            for sidecar in action
                .sidecars
                .iter()
                .filter(|sidecar| is_subtitle_path(&sidecar.source))
            {
                if action.kind == LayoutActionKind::Deduplicate
                    && matches!(
                        sidecar.kind,
                        SidecarActionKind::Move | SidecarActionKind::Deduplicate
                    )
                {
                    tx.execute(
                        "INSERT OR IGNORE INTO media_subtitle (media_file_id, path, sort_order) \
                         SELECT id, ?1, 0 FROM media_file WHERE path = ?2",
                        params![sidecar.target, action.keeper],
                    )
                    .map_err(|error| plan_error(format!("绑定 keeper 字幕失败: {error}")))?;
                    continue;
                }
                if action.kind == LayoutActionKind::Move
                    && matches!(
                        sidecar.kind,
                        SidecarActionKind::Move | SidecarActionKind::Deduplicate
                    )
                {
                    tx.execute(
                        "UPDATE OR IGNORE media_subtitle SET path = ?1 WHERE path = ?2",
                        params![sidecar.target, sidecar.source],
                    )
                    .map_err(|error| plan_error(format!("更新字幕路径失败: {error}")))?;
                    if sidecar.kind == SidecarActionKind::Deduplicate {
                        tx.execute(
                            "DELETE FROM media_subtitle WHERE path = ?1",
                            params![sidecar.source],
                        )
                        .map_err(|error| plan_error(format!("删除重复字幕索引失败: {error}")))?;
                    }
                }
            }
        }
        tx.execute(
            "DELETE FROM episode WHERE NOT EXISTS \
             (SELECT 1 FROM media_file WHERE media_file.episode_id = episode.id)",
            [],
        )
        .map_err(|error| plan_error(format!("清理空 episode 失败: {error}")))?;
        tx.execute(
            "DELETE FROM series WHERE NOT EXISTS \
             (SELECT 1 FROM episode WHERE episode.series_id = series.id) \
             AND NOT EXISTS (SELECT 1 FROM media_extra WHERE media_extra.series_id = series.id)",
            [],
        )
        .map_err(|error| plan_error(format!("清理空 series 失败: {error}")))?;
        tx.commit()
            .map_err(|error| plan_error(format!("提交 layout DB 事务失败: {error}")))?;
        Ok(())
    })
}

fn load_cache(db_path: &Path) -> Result<HashMap<String, CachedIdentity>> {
    let conn = Connection::open(db_path)
        .map_err(|error| plan_error(format!("打开 library.db 失败: {error}")))?;
    let has_prefix = column_exists(&conn, "sha256_prefix_63m")?;
    let has_full = column_exists(&conn, "sha256_full")?;
    let prefix = if has_prefix {
        "media_file.sha256_prefix_63m"
    } else {
        "NULL"
    };
    let full = if has_full {
        "media_file.sha256_full"
    } else {
        "NULL"
    };
    let sql = format!(
        "SELECT media_file.path, series.title, episode.season, episode.episode, \
         media_file.size, media_file.modified_time, {prefix}, {full}, \
         (SELECT value FROM series_external_id \
          WHERE series_external_id.series_id = series.id AND provider = 1 LIMIT 1) \
         FROM media_file \
         JOIN episode ON episode.id = media_file.episode_id \
         JOIN series ON series.id = episode.series_id"
    );
    let mut statement = conn
        .prepare(&sql)
        .map_err(|error| plan_error(format!("读取 library.db identity 失败: {error}")))?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                CachedIdentity {
                    logical: LogicalIdentity {
                        series: row.get(1)?,
                        season: row.get(2)?,
                        episode: row.get(3)?,
                        bangumi_id: row.get(8)?,
                    },
                    size: row.get(4)?,
                    modified_time: row.get(5)?,
                    prefix: row.get(6)?,
                    full: row.get(7)?,
                },
            ))
        })
        .map_err(|error| plan_error(format!("查询 library.db identity 失败: {error}")))?;
    let mut cache = HashMap::new();
    for row in rows {
        let (path, identity) =
            row.map_err(|error| plan_error(format!("解析 library.db identity 失败: {error}")))?;
        cache.insert(path, identity);
    }
    Ok(cache)
}

fn column_exists(conn: &Connection, column: &str) -> Result<bool> {
    conn.query_row(
        "SELECT 1 FROM pragma_table_info('media_file') WHERE name = ?1",
        params![column],
        |_| Ok(()),
    )
    .optional()
    .map(|value| value.is_some())
    .map_err(|error| plan_error(format!("读取 media_file schema 失败: {error}")))
}

fn parse_season_directory(value: &str) -> Option<i64> {
    value
        .strip_prefix("Season ")
        .or_else(|| value.strip_prefix("season "))?
        .trim()
        .parse::<i64>()
        .ok()
        .filter(|season| *season > 0)
}

fn is_subtitle_path(relative: &str) -> bool {
    Path::new(relative)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| {
            ["srt", "ass", "ssa", "sub", "vtt"]
                .iter()
                .any(|allowed| extension.eq_ignore_ascii_case(allowed))
        })
}

fn is_video(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| {
            VIDEO_EXTENSIONS
                .iter()
                .any(|allowed| extension.eq_ignore_ascii_case(allowed))
        })
}

fn file_identity(path: &Path, hash: bool) -> Result<FileIdentity> {
    let metadata = fs::metadata(path)
        .map_err(|error| plan_error(format!("读取文件信息失败 {}: {error}", path.display())))?;
    Ok(FileIdentity {
        size: metadata.len(),
        modified_time: metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .and_then(|duration| i64::try_from(duration.as_secs()).ok()),
        sha256: hash.then(|| sha256_file(path)).transpose()?,
    })
}

fn canonical_root(path: &Path) -> Result<PathBuf> {
    path.canonicalize()
        .map_err(|error| plan_error(format!("媒体库根目录无效 {}: {error}", path.display())))
}

fn relative_path(root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| plan_error(format!("路径不在媒体库内: {}", path.display())))?;
    Ok(relative
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/"))
}

fn safe_join(root: &Path, relative: &str) -> Result<PathBuf> {
    let path = Path::new(relative);
    if relative.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(plan_error(format!("plan 包含非法相对路径: {relative}")));
    }
    Ok(root.join(path))
}

fn verify_existing_target(target: &Path, action: &LayoutAction) -> Result<()> {
    let identity = file_identity(target, action.sha256_full.is_some())?;
    if identity.size != action.size
        || action
            .sha256_full
            .as_deref()
            .is_some_and(|hash| identity.sha256.as_deref() != Some(hash))
    {
        return Err(plan_error(format!(
            "已存在目标与 plan 不匹配: {}",
            target.display()
        )));
    }
    Ok(())
}

fn remove_empty_source_directories(root: &Path, actions: &[LayoutAction]) {
    let mut directories = actions
        .iter()
        .filter(|action| {
            matches!(
                action.kind,
                LayoutActionKind::Move | LayoutActionKind::Deduplicate
            )
        })
        .filter_map(|action| safe_join(root, &action.source).ok())
        .filter_map(|path| path.parent().map(Path::to_path_buf))
        .collect::<Vec<_>>();
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    directories.dedup();
    for directory in directories {
        if directory != root {
            let _ = fs::remove_dir(&directory);
        }
    }
}

fn summarize(actions: &[LayoutAction]) -> LayoutPlanSummary {
    let mut summary = LayoutPlanSummary::default();
    for action in actions {
        match action.kind {
            LayoutActionKind::Keep => summary.keep += 1,
            LayoutActionKind::Move => {
                summary.move_files += 1;
                summary.bytes_to_move = summary.bytes_to_move.saturating_add(action.size);
            }
            LayoutActionKind::Deduplicate => {
                summary.deduplicate += 1;
                summary.bytes_to_release = summary.bytes_to_release.saturating_add(action.size);
            }
            LayoutActionKind::Conflict => summary.conflict += 1,
            LayoutActionKind::Unresolved => summary.unresolved += 1,
        }
    }
    summary
}

fn logical_key(identity: &LogicalIdentity) -> String {
    identity.bangumi_id.as_ref().map_or_else(
        || normalize_key(&split_series_and_season(&identity.series).0),
        |id| format!("bangumi:{id}"),
    )
}

fn normalize_key(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn plan_error(message: impl Into<String>) -> AppError {
    AppError::LibraryIndexError(message.into())
}
