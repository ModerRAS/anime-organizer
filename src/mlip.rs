use anime_organizer::metadata::{
    bangumi::{BangumiEpisode, BangumiSubject},
    tmdb::TmdbTvShow,
    AliasLookup, BangumiClient, TmdbClient,
};
use anime_organizer::nfo::{EpisodeNfo, NfoWriter, UniqueId};
use anime_organizer::{
    parser::split_series_and_season, AnimeFileInfo, AnimeMetadata, LibraryIndexRecord,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use zhhz::{Config, Converter};

static TRADITIONAL_TO_SIMPLIFIED: LazyLock<Mutex<Converter>> =
    LazyLock::new(|| Mutex::new(Converter::new(Config::T2s)));

#[cfg(feature = "metadata")]
pub(crate) async fn fetch_anime_metadata(
    anime_name: &str,
    series_name: &str,
    season_hint: Option<u32>,
    alias_lookup: &AliasLookup,
    bangumi: &BangumiClient,
    tmdb: Option<&TmdbClient>,
    verbose: bool,
) -> Option<AnimeMetadata> {
    let mut metadata = None;
    let mut anidb_id = None;
    let lookup_queries = metadata_search_queries(series_name, Some(anime_name), None, season_hint);

    let alias = lookup_queries.iter().find_map(|query| {
        alias_lookup
            .find(query)
            .or_else(|| alias_lookup.find_fuzzy(query))
    });

    if let Some(entry) = alias {
        if verbose {
            eprintln!(
                "别名匹配: {} -> {} (bangumi_id={})",
                anime_name, entry.name, entry.bangumi_id
            );
        }

        match bangumi.fetch_metadata(entry.bangumi_id).await {
            Ok(mut meta) => {
                meta.tmdb_id = entry.tmdb_id;
                meta.anidb_id = entry.anidb_id;
                metadata = Some(meta);
                anidb_id = entry.anidb_id;
            }
            Err(error) => {
                if verbose {
                    eprintln!("Bangumi 获取失败 {}: {error}", entry.bangumi_id);
                }
            }
        }
    }

    if metadata.is_none() {
        let subject = lookup_queries
            .iter()
            .find_map(|query| bangumi.find_by_name(query).ok().flatten());

        if let Some(subject) = subject {
            if verbose {
                eprintln!("Bangumi 名称匹配: {} -> {}", anime_name, subject.name);
            }
            if let Ok(meta) = bangumi.fetch_metadata(subject.id).await {
                metadata = Some(meta);
            }
        }
    }

    if metadata.is_none() {
        let mut best_subject = None;
        for query in &lookup_queries {
            if let Ok(candidates) = bangumi.find_local_subjects(0.72, 10, |subject| {
                bangumi_subject_match_score(query, subject, false)
            }) {
                if let Some((subject, score)) = choose_local_bangumi_subject(query, candidates) {
                    if best_subject
                        .as_ref()
                        .is_none_or(|(_, best_score, _)| score > *best_score)
                    {
                        best_subject = Some((subject, score, query.clone()));
                    }
                }
            }
        }

        if let Some((subject, score, query)) = best_subject {
            if verbose {
                eprintln!(
                    "Bangumi 本地搜索: {} -> {} (score={score:.2})",
                    query, subject.name
                );
            }
            if let Ok(meta) = bangumi.fetch_metadata(subject.id).await {
                metadata = Some(meta);
            }
        }
    }

    if metadata.is_none() {
        let mut best_subject = None;
        for query in &lookup_queries {
            match bangumi.search_subjects(query, 10).await {
                Ok(subjects) => {
                    if let Some((subject, score)) = choose_bangumi_subject(query, subjects) {
                        if best_subject
                            .as_ref()
                            .is_none_or(|(_, best_score, _)| score > *best_score)
                        {
                            best_subject = Some((subject, score, query.clone()));
                        }
                    } else if verbose {
                        eprintln!("Bangumi 在线搜索未找到可靠匹配: {query}");
                    }
                }
                Err(error) => {
                    if verbose {
                        eprintln!("Bangumi 在线搜索失败 '{}': {error}", query);
                    }
                }
            }
        }

        if let Some((subject, score, query)) = best_subject {
            if verbose {
                eprintln!(
                    "Bangumi 在线搜索: {} -> {} (score={score:.2})",
                    query, subject.name
                );
            }
            if let Ok(meta) = bangumi.fetch_metadata(subject.id).await {
                metadata = Some(meta);
            }
        }
    }

    let mut metadata = metadata?;
    if metadata.anidb_id.is_none() {
        metadata.anidb_id = anidb_id;
    }

    if metadata.tmdb_id.is_none() {
        if let Some(tmdb_client) = tmdb {
            let year = metadata.air_date.as_deref().and_then(parse_year);
            for candidate in unique_titles(
                &metadata.title,
                metadata.title_cn.as_deref(),
                Some(&metadata.original_title),
            ) {
                match tmdb_client.find_by_title(&candidate, year).await {
                    Ok(Some(show)) => {
                        metadata.tmdb_id = Some(show.id);
                        if verbose {
                            eprintln!(
                                "TMDB 搜索匹配: {} -> {} (tmdb_id={})",
                                candidate, show.name, show.id
                            );
                        }
                        break;
                    }
                    Ok(None) => continue,
                    Err(error) => {
                        if verbose {
                            eprintln!("TMDB 搜索失败 {}: {error}", candidate);
                        }
                    }
                }
            }
        }
    }

    Some(metadata)
}

#[cfg(feature = "metadata")]
pub(crate) async fn download_images(
    meta: &AnimeMetadata,
    anime_root: &Path,
    season_number: u32,
    bangumi: &BangumiClient,
    tmdb: Option<&TmdbClient>,
    force: bool,
    verbose: bool,
) {
    let root_poster_path = anime_root.join("poster.jpg");
    let season_poster_path = anime_root.join(format!("season{season_number:02}-poster.jpg"));
    let fanart_path = anime_root.join("fanart.jpg");
    let needs_root = force || !root_poster_path.exists();
    let needs_season = force || !season_poster_path.exists();
    let needs_fanart = force || !fanart_path.exists();
    let mut poster_written = !needs_root && !needs_season;

    if !poster_written {
        if let Some(url) = meta
            .poster_url
            .as_deref()
            .filter(|url| !url.trim().is_empty())
        {
            match bangumi.download_image_bytes(url).await {
                Ok(bytes) => {
                    write_poster_images(
                        &bytes,
                        &root_poster_path,
                        &season_poster_path,
                        needs_root,
                        needs_season,
                        "Bangumi",
                        verbose,
                    );
                    poster_written = true;
                }
                Err(error) => {
                    if verbose {
                        eprintln!("Bangumi 海报下载失败: {error}");
                    }
                }
            }
        }
    }

    let tmdb_show = match (tmdb, !poster_written || needs_fanart) {
        (Some(tmdb_client), true) => resolve_tmdb_show(meta, tmdb_client, verbose).await,
        _ => None,
    };

    if !poster_written {
        if let (Some(tmdb_client), Some(show)) = (tmdb, tmdb_show.as_ref()) {
            match tmdb_client.best_poster_url(show).await {
                Ok(Some(url)) => match tmdb_client.download_image_bytes(&url).await {
                    Ok(bytes) => {
                        write_poster_images(
                            &bytes,
                            &root_poster_path,
                            &season_poster_path,
                            needs_root,
                            needs_season,
                            "TMDB",
                            verbose,
                        );
                        poster_written = true;
                    }
                    Err(error) => eprintln!("TMDB 海报下载失败: {error}"),
                },
                Ok(None) => {}
                Err(error) => eprintln!("TMDB 海报获取失败: {error}"),
            }
        }
    }

    if let (Some(tmdb_client), Some(show)) = (tmdb, tmdb_show.as_ref()) {
        if needs_fanart {
            match tmdb_client.best_backdrop_url(show).await {
                Ok(Some(url)) => match tmdb_client.download_image_bytes(&url).await {
                    Ok(bytes) => {
                        if let Err(error) = NfoWriter::write_image(&fanart_path, &bytes) {
                            eprintln!("背景图写入失败: {error}");
                        } else if verbose {
                            eprintln!("已下载 TMDB 背景图: {}", fanart_path.display());
                        }
                    }
                    Err(error) => eprintln!("TMDB 背景图下载失败: {error}"),
                },
                Ok(None) => {}
                Err(error) => eprintln!("TMDB 背景图获取失败: {error}"),
            }
        }
    }

    if !poster_written && (force || !root_poster_path.exists()) {
        if let (Some(tmdb_client), Some(anidb_id)) = (tmdb, meta.anidb_id) {
            match tmdb_client
                .download_anidb_poster(anidb_id, &root_poster_path)
                .await
            {
                Ok(()) => {
                    if verbose {
                        eprintln!("已从 AniDB 下载海报: {}", root_poster_path.display());
                    }

                    if force || !season_poster_path.exists() {
                        match std::fs::read(&root_poster_path) {
                            Ok(bytes) => {
                                if let Err(error) =
                                    NfoWriter::write_image(&season_poster_path, &bytes)
                                {
                                    eprintln!("季海报写入失败: {error}");
                                }
                            }
                            Err(error) => eprintln!("读取 AniDB 海报失败: {error}"),
                        }
                    }
                }
                Err(error) => {
                    if verbose {
                        eprintln!("AniDB 回退失败 (aid={anidb_id}): {error}");
                    }
                }
            }
        }
    }
}

#[cfg(feature = "metadata")]
fn write_poster_images(
    bytes: &[u8],
    root_poster_path: &Path,
    season_poster_path: &Path,
    needs_root: bool,
    needs_season: bool,
    source: &str,
    verbose: bool,
) {
    if needs_root {
        if let Err(error) = NfoWriter::write_image(root_poster_path, bytes) {
            eprintln!("海报写入失败: {error}");
        } else if verbose {
            eprintln!("已下载 {source} 海报: {}", root_poster_path.display());
        }
    }

    if needs_season {
        if let Err(error) = NfoWriter::write_image(season_poster_path, bytes) {
            eprintln!("季海报写入失败: {error}");
        } else if verbose {
            eprintln!("已下载 {source} 季海报: {}", season_poster_path.display());
        }
    }
}

#[cfg(feature = "metadata")]
async fn resolve_tmdb_show(
    meta: &AnimeMetadata,
    tmdb_client: &TmdbClient,
    verbose: bool,
) -> Option<TmdbTvShow> {
    if let Some(tmdb_id) = meta.tmdb_id {
        match tmdb_client.find_by_tmdb_id(tmdb_id).await {
            Ok(show) => return Some(show),
            Err(error) => {
                if verbose {
                    eprintln!("TMDB 详情获取失败 (tmdb_id={tmdb_id}): {error}");
                }
            }
        }
    }

    let year = meta.air_date.as_deref().and_then(parse_year);
    for title in unique_titles(
        &meta.title,
        meta.title_cn.as_deref(),
        Some(&meta.original_title),
    ) {
        match tmdb_client.find_by_title(&title, year).await {
            Ok(Some(show)) => return Some(show),
            Ok(None) => continue,
            Err(error) => {
                if verbose {
                    eprintln!("TMDB 搜索失败 {}: {error}", title);
                }
            }
        }
    }

    None
}

#[cfg(feature = "metadata")]
pub(crate) fn create_episode_nfo(file: &AnimeFileInfo, meta: &AnimeMetadata) -> EpisodeNfo {
    let episode_number = file.episode.trim().parse().unwrap_or(0);

    EpisodeNfo {
        title: format!("Episode {}", file.episode.trim()),
        season: file.season_number().unwrap_or(1),
        episode: episode_number,
        plot: None,
        aired: meta.air_date.clone(),
        runtime: None,
        displayseason: None,
        displayepisode: None,
        uniqueid: vec![UniqueId {
            id_type: "bangumi".to_string(),
            default: true,
            value: meta.bangumi_id.to_string(),
        }],
        credits: Vec::new(),
        director: meta.director.iter().cloned().collect(),
        actor: Vec::new(),
        tagline: None,
        playcount: None,
        lastplayed: None,
    }
}

#[allow(dead_code)]
#[cfg(feature = "metadata")]
pub(crate) async fn fetch_bangumi_episodes_cached(
    bangumi_id: u32,
    bangumi: &BangumiClient,
    episode_cache: &mut HashMap<u32, Option<Vec<BangumiEpisode>>>,
    verbose: bool,
) -> Option<Vec<BangumiEpisode>> {
    if let Some(cached) = episode_cache.get(&bangumi_id) {
        return cached.clone();
    }

    let fetched = match bangumi.fetch_episodes(bangumi_id).await {
        Ok(episodes) => {
            if verbose {
                eprintln!("Bangumi 分集加载: {bangumi_id} -> {} 集", episodes.len());
            }
            Some(episodes)
        }
        Err(error) => {
            if verbose {
                eprintln!("Bangumi 分集加载失败 {bangumi_id}: {error}");
            }
            None
        }
    };
    episode_cache.insert(bangumi_id, fetched.clone());
    fetched
}

#[cfg(feature = "metadata")]
pub(crate) fn apply_bangumi_episode_details(
    record: &mut LibraryIndexRecord,
    episodes: Option<&[BangumiEpisode]>,
    min_episode: Option<f64>,
) {
    let Some(episode) = episodes.and_then(|items| find_bangumi_episode(record, items, min_episode))
    else {
        return;
    };

    if let Some(title) = episode.display_title() {
        record.episode_title = Some(title);
    }
    if let Some(summary) = episode.cleaned_summary() {
        record.episode_summary = Some(summary);
    }
    if let Some(seconds) = episode.duration_seconds.filter(|seconds| *seconds > 0) {
        record.runtime = Some(i64::from(seconds));
    }
}

#[cfg(feature = "metadata")]
fn find_bangumi_episode<'a>(
    record: &LibraryIndexRecord,
    episodes: &'a [BangumiEpisode],
    min_episode: Option<f64>,
) -> Option<&'a BangumiEpisode> {
    let number = record.episode;
    episodes
        .iter()
        .find(|episode| {
            episode
                .sort
                .is_some_and(|sort| episode_number_matches(sort, number))
        })
        .or_else(|| {
            episodes.iter().find(|episode| {
                episode
                    .ep
                    .is_some_and(|ep| episode_number_matches(ep, number))
            })
        })
        .or_else(|| {
            let local_number = min_episode
                .filter(|min| *min > 1.0 && number >= *min)
                .map(|min| number - min + 1.0)?;
            episodes.iter().find(|episode| {
                episode
                    .sort
                    .is_some_and(|sort| episode_number_matches(sort, local_number))
                    || episode
                        .ep
                        .is_some_and(|ep| episode_number_matches(ep, local_number))
            })
        })
}

#[cfg(feature = "metadata")]
fn episode_number_matches(left: f64, right: f64) -> bool {
    (left - right).abs() < 0.001
}

#[cfg(feature = "metadata")]
pub(crate) fn anime_group_min_episode(files: &[AnimeFileInfo]) -> Option<f64> {
    files
        .iter()
        .filter_map(|file| file.episode.trim().parse::<f64>().ok())
        .filter(|episode| episode.is_finite())
        .min_by(|left, right| left.total_cmp(right))
}

#[cfg(feature = "metadata")]
pub(crate) fn min_episode_by_series(records: &[LibraryIndexRecord]) -> HashMap<(String, i64), f64> {
    let mut min_episodes = HashMap::new();
    for record in records {
        if !record.episode.is_finite() {
            continue;
        }
        min_episodes
            .entry((record.series_title.clone(), record.season))
            .and_modify(|episode| {
                if record.episode < *episode {
                    *episode = record.episode;
                }
            })
            .or_insert(record.episode);
    }
    min_episodes
}

fn parse_year(value: &str) -> Option<i32> {
    value.get(0..4)?.parse().ok()
}

#[cfg(feature = "metadata")]
fn metadata_search_queries(
    primary: &str,
    secondary: Option<&str>,
    tertiary: Option<&str>,
    season_hint: Option<u32>,
) -> Vec<String> {
    let titles = unique_titles(primary, secondary, tertiary);
    let mut queries = Vec::new();

    for title in titles {
        if let Some(season) = season_hint.filter(|season| *season > 1) {
            if title_season_hint(&title).is_some() || split_series_and_season(&title).1.is_some() {
                push_unique_title(&mut queries, title);
            } else {
                if let Some(number) = cjk_season_number(season) {
                    push_unique_title(&mut queries, format!("{title} 第{number}季"));
                }
                push_unique_title(&mut queries, format!("{title} Season {season}"));
            }
        } else {
            push_unique_title(&mut queries, title);
        }
    }

    for query in queries.clone() {
        let simplified = simplify_title_text(&query);
        push_unique_title(&mut queries, simplified);
    }
    queries
}

#[cfg(feature = "metadata")]
fn push_unique_title(titles: &mut Vec<String>, value: String) {
    let value = value.trim();
    if !value.is_empty() && !titles.iter().any(|item| item == value) {
        titles.push(value.to_string());
    }
}

#[cfg(feature = "metadata")]
fn cjk_season_number(season: u32) -> Option<&'static str> {
    match season {
        1 => Some("一"),
        2 => Some("二"),
        3 => Some("三"),
        4 => Some("四"),
        5 => Some("五"),
        6 => Some("六"),
        7 => Some("七"),
        8 => Some("八"),
        9 => Some("九"),
        10 => Some("十"),
        _ => None,
    }
}

#[cfg(feature = "metadata")]
fn choose_bangumi_subject(
    query: &str,
    subjects: Vec<BangumiSubject>,
) -> Option<(BangumiSubject, f32)> {
    subjects
        .into_iter()
        .enumerate()
        .filter_map(|(index, subject)| {
            let score = bangumi_subject_match_score(query, &subject, true);
            let ranked_score = score - (index as f32 * 0.12);
            (score >= 0.45).then_some((subject, ranked_score))
        })
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
}

#[cfg(feature = "metadata")]
fn choose_local_bangumi_subject(
    query: &str,
    subjects: Vec<(BangumiSubject, f32)>,
) -> Option<(BangumiSubject, f32)> {
    subjects
        .into_iter()
        .filter_map(|(subject, _)| {
            let score = bangumi_subject_match_score(query, &subject, true);
            (score >= 0.72).then_some((subject, score))
        })
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
}

#[cfg(feature = "metadata")]
fn bangumi_subject_match_score(
    query: &str,
    subject: &BangumiSubject,
    use_bert_season: bool,
) -> f32 {
    let query_season = title_season_hint(query)
        .or_else(|| use_bert_season.then(|| bert_season_hint(query)).flatten());
    let ascii_tokens = distinctive_ascii_tokens(query);
    [Some(subject.name.as_str()), subject.name_cn.as_deref()]
        .into_iter()
        .flatten()
        .map(|title| {
            let candidate_season = title_season_hint(title)
                .or_else(|| use_bert_season.then(|| bert_season_hint(title)).flatten())
                .or_else(|| {
                    query_season.filter(|season| has_trailing_season_number(title, *season))
                });
            let lexical_score = title_match_score(query, title);
            if lexical_score < 0.8
                && !ascii_tokens.is_empty()
                && !candidate_has_any_ascii_token(title, &ascii_tokens)
            {
                return 0.0;
            }
            title_match_score_with_known_seasons(query, title, query_season, candidate_season)
        })
        .fold(0.0, f32::max)
}

#[cfg(all(feature = "metadata", feature = "anifilebert"))]
fn bert_season_hint(value: &str) -> Option<u32> {
    anime_organizer::anifilebert::detect_season(value)
        .ok()
        .flatten()
}

#[cfg(all(feature = "metadata", not(feature = "anifilebert")))]
fn bert_season_hint(_value: &str) -> Option<u32> {
    None
}

#[cfg(feature = "metadata")]
fn distinctive_ascii_tokens(value: &str) -> Vec<String> {
    simplify_title_text(value)
        .to_ascii_lowercase()
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| token.len() >= 3 && !is_generic_ascii_token(token))
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(feature = "metadata")]
fn candidate_has_any_ascii_token(candidate: &str, tokens: &[String]) -> bool {
    let candidate = distinctive_ascii_tokens(candidate);
    tokens
        .iter()
        .any(|token| candidate.iter().any(|item| item == token))
}

#[cfg(feature = "metadata")]
fn is_generic_ascii_token(value: &str) -> bool {
    matches!(
        value,
        "and"
            | "for"
            | "from"
            | "into"
            | "movie"
            | "ona"
            | "ova"
            | "season"
            | "special"
            | "the"
            | "tv"
            | "tvsp"
            | "with"
    )
}

#[cfg(all(test, feature = "metadata"))]
fn title_match_score_with_season(query: &str, candidate: &str, query_season: Option<u32>) -> f32 {
    title_match_score_with_known_seasons(
        query,
        candidate,
        query_season,
        title_season_hint(candidate),
    )
}

#[cfg(feature = "metadata")]
fn title_match_score_with_known_seasons(
    query: &str,
    candidate: &str,
    query_season: Option<u32>,
    candidate_season: Option<u32>,
) -> f32 {
    if query_season.is_some() && candidate_season.is_some() && query_season != candidate_season {
        return 0.0;
    }

    if query_season.is_some() && candidate_season.is_none() {
        let query_text = normalized_match_text(query);
        let candidate_text = normalized_match_text(candidate);
        let query_len = query_text.chars().count();
        let candidate_len = candidate_text.chars().count();
        if candidate_len >= 4
            && query_len > candidate_len
            && query_text.contains(&candidate_text)
            && candidate_len * 4 < query_len * 3
        {
            return 0.0;
        }
    }

    let score = title_match_score(query, candidate);
    if query_season.is_some() && query_season == candidate_season {
        (score + 0.25).min(1.0)
    } else {
        score
    }
}

#[cfg(feature = "metadata")]
fn title_season_hint(value: &str) -> Option<u32> {
    let normalized = simplify_title_text(value).to_ascii_lowercase();
    let chars: Vec<char> = normalized.chars().collect();

    for (index, ch) in chars.iter().enumerate() {
        if *ch != '第' {
            continue;
        }
        let Some(end) = chars[index + 1..]
            .iter()
            .position(|item| *item == '季' || *item == '期')
        else {
            continue;
        };
        let raw: String = chars[index + 1..index + 1 + end].iter().collect();
        if let Some(season) = parse_season_hint_number(&raw) {
            return Some(season);
        }
    }

    let words: Vec<&str> = normalized
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    for pair in words.windows(2) {
        if pair[0] == "season" {
            if let Ok(season) = pair[1].parse::<u32>() {
                return Some(season);
            }
        }
        if pair[1] == "season" {
            if let Some(season) = parse_ordinal_number(pair[0]) {
                return Some(season);
            }
        }
    }
    words
        .iter()
        .find_map(|word| word.strip_prefix('s')?.parse::<u32>().ok())
}

fn parse_season_hint_number(value: &str) -> Option<u32> {
    value.parse::<u32>().ok().or(match value {
        "一" => Some(1),
        "二" => Some(2),
        "三" => Some(3),
        "四" => Some(4),
        "五" => Some(5),
        "六" => Some(6),
        "七" => Some(7),
        "八" => Some(8),
        "九" => Some(9),
        "十" => Some(10),
        _ => None,
    })
}

fn parse_ordinal_number(value: &str) -> Option<u32> {
    value
        .strip_suffix("st")
        .or_else(|| value.strip_suffix("nd"))
        .or_else(|| value.strip_suffix("rd"))
        .or_else(|| value.strip_suffix("th"))
        .and_then(|number| number.parse::<u32>().ok())
}

#[cfg(feature = "metadata")]
fn has_trailing_season_number(value: &str, expected: u32) -> bool {
    let value = simplify_title_text(value).trim().to_ascii_lowercase();
    let suffix = expected.to_string();
    value
        .strip_suffix(&suffix)
        .and_then(|prefix| prefix.chars().next_back())
        .is_some_and(|previous| !previous.is_ascii_digit())
}

#[cfg(feature = "metadata")]
fn title_match_score(query: &str, candidate: &str) -> f32 {
    let query = normalized_match_text(query);
    let candidate = normalized_match_text(candidate);
    if query.is_empty() || candidate.is_empty() {
        return 0.0;
    }
    if query == candidate {
        return 1.0;
    }

    let query_len = query.chars().count();
    let candidate_len = candidate.chars().count();
    let shorter = query_len.min(candidate_len) as f32;
    let longer = query_len.max(candidate_len) as f32;
    if shorter >= 4.0 && (query.contains(&candidate) || candidate.contains(&query)) {
        return 0.8 + 0.2 * (shorter / longer);
    }

    let query_chars: HashSet<char> = query.chars().collect();
    let candidate_chars: HashSet<char> = candidate.chars().collect();
    let overlap = query_chars.intersection(&candidate_chars).count() as f32;
    if overlap == 0.0 {
        return 0.0;
    }

    let query_coverage = overlap / query_chars.len() as f32;
    let candidate_coverage = overlap / candidate_chars.len() as f32;
    query_coverage.min(candidate_coverage)
}

#[cfg(feature = "metadata")]
fn normalized_match_text(value: &str) -> String {
    simplify_title_text(value)
        .chars()
        .filter_map(|ch| {
            let ch = ch.to_ascii_lowercase();
            ch.is_alphanumeric().then_some(ch)
        })
        .collect()
}

#[cfg(feature = "metadata")]
fn simplify_title_text(value: &str) -> String {
    TRADITIONAL_TO_SIMPLIFIED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .convert(value)
        .chars()
        .map(simplify_title_char)
        .collect()
}

#[cfg(feature = "metadata")]
fn simplify_title_char(ch: char) -> char {
    match ch {
        '國' => '国',
        '強' => '强',
        '從' => '从',
        '後' => '后',
        '復' => '复',
        '戀' => '恋',
        '戰' => '战',
        '戲' => '戏',
        '戶' => '户',
        '數' => '数',
        '斷' => '断',
        '時' => '时',
        '會' => '会',
        '機' => '机',
        '氣' => '气',
        '漢' => '汉',
        '燈' => '灯',
        '爾' => '尔',
        '獸' => '兽',
        '現' => '现',
        '異' => '异',
        '當' => '当',
        '盜' => '盗',
        '禮' => '礼',
        '種' => '种',
        '穩' => '稳',
        '級' => '级',
        '經' => '经',
        '續' => '续',
        '聖' => '圣',
        '聲' => '声',
        '職' => '职',
        '脫' => '脱',
        '臨' => '临',
        '與' => '与',
        '舊' => '旧',
        '萬' => '万',
        '處' => '处',
        '術' => '术',
        '衛' => '卫',
        '裡' => '里',
        '覺' => '觉',
        '討' => '讨',
        '譚' => '谭',
        '變' => '变',
        '貓' => '猫',
        '貴' => '贵',
        '賢' => '贤',
        '轉' => '转',
        '輕' => '轻',
        '輩' => '辈',
        '邊' => '边',
        '醫' => '医',
        '醬' => '酱',
        '釋' => '释',
        '銀' => '银',
        '錄' => '录',
        '長' => '长',
        '間' => '间',
        '關' => '关',
        '闇' => '暗',
        '隊' => '队',
        '階' => '阶',
        '險' => '险',
        '雜' => '杂',
        '靈' => '灵',
        '領' => '领',
        '顧' => '顾',
        '顯' => '显',
        '騎' => '骑',
        '體' => '体',
        '鬥' => '斗',
        '鬆' => '松',
        '魔' => '魔',
        '鳥' => '鸟',
        '點' => '点',
        '齡' => '龄',
        '龍' => '龙',
        '龐' => '庞',
        '乾' => '干',
        '亂' => '乱',
        '亞' => '亚',
        '優' => '优',
        '傳' => '传',
        '兒' => '儿',
        '兩' => '两',
        '內' => '内',
        '劍' => '剑',
        '動' => '动',
        '勞' => '劳',
        '勝' => '胜',
        '單' => '单',
        '喪' => '丧',
        '嚮' => '向',
        '圖' => '图',
        '圓' => '圆',
        '壞' => '坏',
        '壓' => '压',
        '壘' => '垒',
        '壯' => '壮',
        '夢' => '梦',
        '夥' => '伙',
        '奪' => '夺',
        '學' => '学',
        '寶' => '宝',
        '實' => '实',
        '將' => '将',
        '專' => '专',
        '尋' => '寻',
        '對' => '对',
        '導' => '导',
        '屬' => '属',
        '歲' => '岁',
        '幫' => '帮',
        '幾' => '几',
        '庫' => '库',
        '廳' => '厅',
        '廣' => '广',
        '廢' => '废',
        '開' => '开',
        '張' => '张',
        '彈' => '弹',
        '彙' => '汇',
        '徵' => '征',
        '德' => '德',
        '憑' => '凭',
        '應' => '应',
        '懶' => '懒',
        '懲' => '惩',
        '滅' => '灭',
        '灣' => '湾',
        '為' => '为',
        '無' => '无',
        '爭' => '争',
        '產' => '产',
        '疊' => '叠',
        '發' => '发',
        '盡' => '尽',
        '砲' => '炮',
        '禦' => '御',
        '稱' => '称',
        '穫' => '获',
        '竊' => '窃',
        '紅' => '红',
        '紳' => '绅',
        '給' => '给',
        '絕' => '绝',
        '絲' => '丝',
        '緣' => '缘',
        '縱' => '纵',
        '繼' => '继',
        '罰' => '罚',
        '罵' => '骂',
        '義' => '义',
        '習' => '习',
        '聽' => '听',
        '肅' => '肃',
        '脈' => '脉',
        '腦' => '脑',
        '臺' => '台',
        '莊' => '庄',
        '著' => '着',
        '藍' => '蓝',
        '虛' => '虚',
        '號' => '号',
        '蟲' => '虫',
        '蠻' => '蛮',
        '計' => '计',
        '詛' => '诅',
        '話' => '话',
        '該' => '该',
        '誕' => '诞',
        '語' => '语',
        '說' => '说',
        '調' => '调',
        '諸' => '诸',
        '謎' => '谜',
        '識' => '识',
        '護' => '护',
        '讚' => '赞',
        '豬' => '猪',
        '貳' => '贰',
        '買' => '买',
        '賽' => '赛',
        '贖' => '赎',
        '軍' => '军',
        '農' => '农',
        '迴' => '回',
        '選' => '选',
        '遺' => '遗',
        '遲' => '迟',
        '還' => '还',
        '邁' => '迈',
        '鄉' => '乡',
        '鄰' => '邻',
        '釀' => '酿',
        '鈴' => '铃',
        '鍊' => '炼',
        '鎖' => '锁',
        '鐘' => '钟',
        '鐵' => '铁',
        '鑰' => '钥',
        '門' => '门',
        '闆' => '板',
        '陣' => '阵',
        '陸' => '陆',
        '雙' => '双',
        '雞' => '鸡',
        '離' => '离',
        '難' => '难',
        '電' => '电',
        '霧' => '雾',
        '靜' => '静',
        '頁' => '页',
        '風' => '风',
        '飛' => '飞',
        '飢' => '饥',
        '館' => '馆',
        '馬' => '马',
        '驅' => '驱',
        '驗' => '验',
        '髮' => '发',
        '鬧' => '闹',
        '魯' => '鲁',
        '鮮' => '鲜',
        '鳴' => '鸣',
        '鷹' => '鹰',
        '麼' => '么',
        _ => ch,
    }
}

fn unique_titles(primary: &str, secondary: Option<&str>, tertiary: Option<&str>) -> Vec<String> {
    let mut titles = Vec::new();

    for value in [Some(primary), secondary, tertiary].into_iter().flatten() {
        let trimmed = value.trim();
        if !trimmed.is_empty() && !titles.iter().any(|item| item == trimmed) {
            titles.push(trimmed.to_string());
        }
    }

    titles
}

#[cfg(all(test, feature = "metadata"))]
mod tests {
    use super::*;

    #[test]
    fn bangumi_search_score_accepts_reliable_matches() {
        assert!(
            title_match_score_with_season(
                "Re：從零開始的異世界生活 第四季",
                "Re：从零开始的异世界生活 第四季 丧失篇",
                title_season_hint("Re：從零開始的異世界生活 第四季")
            ) >= 0.45
        );
        assert!(title_match_score("咒術迴戰 死滅迴游 前篇", "咒术回战 死灭回游 前篇") >= 0.45);
        assert!(title_match_score("少女怪獸焦糖戀心", "少女怪兽焦糖味") >= 0.45);
    }

    fn bangumi_subject(id: u32, name: &str, name_cn: &str) -> BangumiSubject {
        BangumiSubject {
            id,
            subject_type: 2,
            name: name.to_string(),
            name_cn: Some(name_cn.to_string()),
            summary: None,
            date: None,
            score: None,
            eps: None,
            infobox: None,
            tags: Vec::new(),
            meta_tags: Vec::new(),
            images: None,
        }
    }

    #[test]
    fn season_directory_queries_do_not_fall_back_to_bare_title() {
        let queries =
            metadata_search_queries("女性向遊戲世界對路人角色很不友好", None, None, Some(2));

        assert!(queries
            .iter()
            .any(|query| query == "女性向遊戲世界對路人角色很不友好 第二季"));
        assert!(queries
            .iter()
            .any(|query| query == "女性向遊戲世界對路人角色很不友好 Season 2"));
        assert!(!queries
            .iter()
            .any(|query| query == "女性向遊戲世界對路人角色很不友好"));

        let named_season = metadata_search_queries("妖怪旅館營業中 貳", None, None, Some(2));
        assert!(named_season
            .iter()
            .any(|query| query == "妖怪旅館營業中 貳"));
        assert!(named_season
            .iter()
            .all(|query| !query.ends_with("第二季") && !query.ends_with("Season 2")));

        let numeric_season = metadata_search_queries(
            "關於我在無意間被隔壁的天使變成廢柴這件事 2",
            None,
            None,
            Some(2),
        );
        assert!(numeric_season
            .iter()
            .any(|query| query == "關於我在無意間被隔壁的天使變成廢柴這件事 2"));
        assert!(numeric_season
            .iter()
            .all(|query| !query.ends_with("2 第二季") && !query.ends_with("2 Season 2")));
    }

    #[test]
    fn local_search_matches_traditional_movie_title() {
        let query = "劇場版 關於我轉生變成史萊姆這檔事 蒼海之淚篇";
        let subject = bangumi_subject(
            515595,
            "劇場版 転生したらスライムだった件 蒼海の涙編",
            "剧场版 关于我转生变成史莱姆这档事 苍海之泪篇",
        );

        assert_eq!(
            simplify_title_text(query),
            "剧场版 关于我转生变成史莱姆这档事 苍海之泪篇"
        );
        assert_eq!(bangumi_subject_match_score(query, &subject, false), 1.0);
        assert_eq!(
            choose_local_bangumi_subject(query, vec![(subject, 1.0)])
                .unwrap()
                .0
                .id,
            515595
        );
    }

    #[test]
    fn local_search_prefers_matching_season() {
        let (subject, _) = choose_local_bangumi_subject(
            "女性向遊戲世界對路人角色很不友好 第二季",
            vec![
                (
                    bangumi_subject(
                        359980,
                        "乙女ゲー世界はモブに厳しい世界です",
                        "恋爱游戏世界对路人角色很不友好",
                    ),
                    0.8,
                ),
                (
                    bangumi_subject(
                        412144,
                        "乙女ゲー世界はモブに厳しい世界です2",
                        "恋爱游戏世界对路人角色很不友好 第二季",
                    ),
                    1.0,
                ),
            ],
        )
        .unwrap();

        assert_eq!(subject.id, 412144);
    }

    #[test]
    fn min_episode_isolated_by_season() {
        let records = vec![
            LibraryIndexRecord::new(
                "Anime".to_string(),
                1,
                13.0,
                "Anime/Season 1/13.mkv".to_string(),
                std::path::Path::new("13.mkv"),
            ),
            LibraryIndexRecord::new(
                "Anime".to_string(),
                2,
                1.0,
                "Anime/Season 2/01.mkv".to_string(),
                std::path::Path::new("01.mkv"),
            ),
        ];

        let minimums = min_episode_by_series(&records);
        assert_eq!(minimums.get(&("Anime".to_string(), 1)), Some(&13.0));
        assert_eq!(minimums.get(&("Anime".to_string(), 2)), Some(&1.0));
    }

    #[test]
    fn bangumi_search_score_rejects_bad_first_results() {
        assert!(title_match_score("Re：從零開始的異世界生活 第四季", "Re: 甜心战士") < 0.45);
        assert!(
            title_match_score_with_season(
                "Re：從零開始的異世界生活 第四季",
                "Re：从零开始的异世界生活 第四季 丧失篇",
                title_season_hint("Re：從零開始的異世界生活 第四季")
            ) > title_match_score_with_season(
                "Re：從零開始的異世界生活 第四季",
                "Re：从零开始的异世界生活",
                title_season_hint("Re：從零開始的異世界生活 第四季")
            )
        );
        assert!(
            title_match_score(
                "Animatica「北斗之拳 拳王軍雜兵們的輓歌」",
                "莎士比亚名剧动画"
            ) < 0.45
        );
        assert!(
            title_match_score("咒術迴戰 死滅迴游 前篇", "咒术回战 真实版4D ～轮转的钟楼～") < 0.45
        );
        assert_eq!(
            title_match_score_with_season(
                "Dr.STONE 新石紀 第四季",
                "Dr.STONE",
                title_season_hint("Dr.STONE 新石紀 第四季")
            ),
            0.0
        );
        assert!(choose_bangumi_subject(
            "假面騎士 ZEZTZ",
            vec![bangumi_subject(
                48352,
                "仮面ライダーSD 怪奇!?クモ男",
                "假面骑士SD 怪奇!?蜘蛛男"
            )]
        )
        .is_none());
        assert!(choose_bangumi_subject(
            "THE WORLD IS DANCING 世界在起舞",
            vec![bangumi_subject(
                622633,
                "ワールド イズ ダンシング",
                "世界在起舞"
            )]
        )
        .is_some());
        let unrelated_season = bangumi_subject(
            568800,
            "MUZIK TIGER In the Forest 第2期",
            "MUZIK TIGER In the Forest 第二季",
        );
        assert_eq!(
            bangumi_subject_match_score(
                "Maou Gakuin no Futekigousha Season 2",
                &unrelated_season,
                true,
            ),
            0.0
        );
    }

    fn bangumi_episode(ep: f64, sort: f64, title: &str, duration_seconds: u32) -> BangumiEpisode {
        BangumiEpisode {
            id: sort as u32,
            subject_id: 1,
            ep: Some(ep),
            sort: Some(sort),
            name: Some(format!("JP {title}")),
            name_cn: Some(title.to_string()),
            summary: Some(format!("{title} summary")),
            duration_seconds: Some(duration_seconds),
            episode_type: 0,
        }
    }

    #[test]
    fn bangumi_episode_details_match_global_or_local_numbering() {
        let episodes = vec![
            bangumi_episode(1.0, 1.0, "第一集", 1440),
            bangumi_episode(2.0, 2.0, "第二集", 1441),
        ];
        let mut record = LibraryIndexRecord::new(
            "Dr.STONE 新石紀 第四季".to_string(),
            4,
            25.0,
            "Dr.STONE 新石紀 第四季/25.mkv".to_string(),
            std::path::Path::new("25.mkv"),
        );

        apply_bangumi_episode_details(&mut record, Some(&episodes), Some(25.0));

        assert_eq!(record.episode_title.as_deref(), Some("第一集"));
        assert_eq!(record.episode_summary.as_deref(), Some("第一集 summary"));
        assert_eq!(record.runtime, Some(1440));

        let episodes = vec![bangumi_episode(1.0, 13.0, "第十三集", 0)];
        let mut record = LibraryIndexRecord::new(
            "終究，與你相戀。第二季".to_string(),
            2,
            13.0,
            "終究，與你相戀。第二季/13.mkv".to_string(),
            std::path::Path::new("13.mkv"),
        );

        apply_bangumi_episode_details(&mut record, Some(&episodes), Some(13.0));

        assert_eq!(record.episode_title.as_deref(), Some("第十三集"));
        assert_eq!(record.runtime, None);
    }

    #[test]
    fn bangumi_search_prefers_api_rank_for_close_scores() {
        let (subject, _) = choose_bangumi_subject(
            "Dr.STONE 新石紀 第四季",
            vec![
                bangumi_subject(471578, "Dr.STONE SCIENCE FUTURE", "石纪元 科学与未来"),
                bangumi_subject(
                    424372,
                    "Dr.STONE NEW WORLD 第2クール",
                    "石纪元 新世界 第2部分",
                ),
            ],
        )
        .unwrap();

        assert_eq!(subject.id, 471578);
    }
}
