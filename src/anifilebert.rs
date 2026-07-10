//! AniFileBERT ONNX fallback parser.
//!
//! The rule parser remains the default. This module is loaded only when the
//! `anifilebert` feature is enabled and the CLI asks for it.

use crate::parser::{split_series_and_season, AnimeFileInfo};
use flate2::read::GzDecoder;
use ort::{
    ep::ExecutionProviderDispatch,
    session::{builder::GraphOptimizationLevel, Session},
    value::Tensor,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::io::Read;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

const MODEL_GZ: &[u8] = include_bytes!("../assets/anifilebert/anime_filename_parser.onnx.gz");
const CONFIG_JSON: &str = include_str!("../assets/anifilebert/config.json");
const VOCAB_JSON: &str = include_str!("../assets/anifilebert/vocab.json");
const DEFAULT_MAX_SEQ_LEN: usize = 128;
const PAD_TOKEN: i64 = 0;
const UNK_TOKEN: i64 = 1;
const CLS_TOKEN: i64 = 2;
const SEP_TOKEN: i64 = 3;
const ENV_PROVIDER: &str = "ANIORG_BERT_PROVIDER";
#[cfg(all(feature = "anifilebert-directml", target_os = "windows"))]
const ENV_DIRECTML_DEVICE_ID: &str = "ANIORG_DIRECTML_DEVICE_ID";
const ENV_INTRA_THREADS: &str = "ANIORG_ORT_INTRA_THREADS";
const ENV_INTER_THREADS: &str = "ANIORG_ORT_INTER_THREADS";

static PARSER: OnceLock<Result<Mutex<AniFileBertParser>, String>> = OnceLock::new();
static SEASON_CACHE: OnceLock<Mutex<HashMap<String, Option<u32>>>> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct ModelConfig {
    max_seq_length: Option<usize>,
    id2label: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Entity {
    label: String,
    value: String,
    start: usize,
    end: usize,
}

pub fn parse_path(path: &Path) -> Result<Option<AnimeFileInfo>, String> {
    with_parser(|parser| parser.parse_path(path))
}

/// 从标题文本中识别季号；使用文件名形状保持与模型训练分布一致。
pub fn detect_season(value: &str) -> Result<Option<u32>, String> {
    let cache = SEASON_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(season) = cache
        .lock()
        .map_err(|_| "AniFileBERT season cache lock poisoned".to_string())?
        .get(value)
        .copied()
    {
        return Ok(season);
    }

    let input = format!("[Search] {value} - 01.mkv");
    let season = with_parser(|parser| parser.detect_season(&input))?;
    cache
        .lock()
        .map_err(|_| "AniFileBERT season cache lock poisoned".to_string())?
        .insert(value.to_string(), season);
    Ok(season)
}

fn with_parser<T>(
    run: impl FnOnce(&mut AniFileBertParser) -> Result<T, String>,
) -> Result<T, String> {
    let parser = PARSER.get_or_init(|| AniFileBertParser::new().map(Mutex::new));
    let parser = parser.as_ref().map_err(Clone::clone)?;
    let mut parser = parser
        .lock()
        .map_err(|_| "AniFileBERT parser lock poisoned".to_string())?;
    run(&mut parser)
}

struct AniFileBertParser {
    vocab: HashMap<String, i64>,
    id2label: Vec<String>,
    max_seq_len: usize,
    session: Session,
}

impl AniFileBertParser {
    fn new() -> Result<Self, String> {
        let config: ModelConfig = serde_json::from_str(CONFIG_JSON)
            .map_err(|error| format!("failed to parse AniFileBERT config: {error}"))?;
        let vocab: HashMap<String, i64> = serde_json::from_str(VOCAB_JSON)
            .map_err(|error| format!("failed to parse AniFileBERT vocab: {error}"))?;
        let id2label = labels_from_config(&config.id2label)?;
        let max_seq_len = config.max_seq_length.unwrap_or(DEFAULT_MAX_SEQ_LEN);
        if max_seq_len < 3 {
            return Err(format!("invalid AniFileBERT max_seq_length: {max_seq_len}"));
        }

        let mut model = Vec::new();
        GzDecoder::new(MODEL_GZ)
            .read_to_end(&mut model)
            .map_err(|error| format!("failed to decompress AniFileBERT ONNX model: {error}"))?;

        let session = create_session(&model, provider_mode()?)?;

        Ok(Self {
            vocab,
            id2label,
            max_seq_len,
            session,
        })
    }

    fn parse_path(&mut self, path: &Path) -> Result<Option<AnimeFileInfo>, String> {
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("invalid filename: {}", path.display()))?;
        let chars: Vec<char> = file_name.chars().collect();
        let entities = self.predict_entities(&chars)?;
        Ok(info_from_entities(path, file_name, &chars, &entities))
    }

    fn detect_season(&mut self, value: &str) -> Result<Option<u32>, String> {
        let chars: Vec<char> = value.chars().collect();
        let entities = self.predict_entities(&chars)?;
        Ok(season_from_entities(&entities))
    }

    fn predict_entities(&mut self, chars: &[char]) -> Result<Vec<Entity>, String> {
        if chars.is_empty() {
            return Ok(Vec::new());
        }

        let encoded = encode_chars(&self.vocab, chars, self.max_seq_len);
        let char_count = encoded.char_count;
        let input_ids = Tensor::from_array(([1usize, self.max_seq_len], encoded.input_ids))
            .map_err(|error| format!("failed to create input_ids tensor: {error}"))?;
        let attention_mask =
            Tensor::from_array(([1usize, self.max_seq_len], encoded.attention_mask))
                .map_err(|error| format!("failed to create attention_mask tensor: {error}"))?;

        let outputs = self
            .session
            .run(ort::inputs! {
                "input_ids" => input_ids,
                "attention_mask" => attention_mask,
            })
            .map_err(|error| format!("AniFileBERT inference failed: {error}"))?;
        let (shape, logits) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|error| format!("failed to read AniFileBERT logits: {error}"))?;
        let labels = argmax_labels(shape, logits, &self.id2label)?
            .into_iter()
            .skip(1)
            .take(char_count)
            .collect::<Vec<_>>();
        Ok(collect_entities(&chars[..char_count], &labels))
    }
}

fn create_session(model: &[u8], mode: ProviderMode) -> Result<Session, String> {
    match mode {
        ProviderMode::Cpu => build_session_with_providers(model, Vec::new()),
        ProviderMode::Auto if directml_available_in_build() => {
            let providers = directml_provider(DirectMlDeviceFilter::Gpu, false)?;
            match build_session_with_providers(model, providers) {
                Ok(session) => Ok(session),
                Err(directml_error) => build_session_with_providers(model, Vec::new())
                    .map_err(|cpu_error| {
                        format!(
                            "DirectML auto initialization failed: {directml_error}; CPU fallback failed: {cpu_error}"
                        )
                    }),
            }
        }
        ProviderMode::Auto => build_session_with_providers(model, Vec::new()),
        ProviderMode::DirectMl(filter) => {
            let providers = directml_provider(filter, true)?;
            build_session_with_providers(model, providers)
        }
    }
}

fn build_session_with_providers(
    model: &[u8],
    providers: Vec<ExecutionProviderDispatch>,
) -> Result<Session, String> {
    let mut builder = Session::builder()
        .map_err(|error| format!("failed to create ONNX Runtime session builder: {error}"))?;
    builder = builder
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|error| format!("failed to configure ONNX graph optimization: {error}"))?;
    builder = builder
        .with_intra_threads(env_thread_count(ENV_INTRA_THREADS)?.unwrap_or(1))
        .map_err(|error| format!("failed to configure ONNX intra-op threads: {error}"))?;
    builder = builder
        .with_inter_threads(env_thread_count(ENV_INTER_THREADS)?.unwrap_or(1))
        .map_err(|error| format!("failed to configure ONNX inter-op threads: {error}"))?;
    builder = builder
        .with_parallel_execution(false)
        .map_err(|error| format!("failed to configure ONNX execution mode: {error}"))?;

    if !providers.is_empty() {
        builder = builder
            .with_memory_pattern(false)
            .map_err(|error| format!("failed to configure ONNX memory pattern: {error}"))?;
        builder = builder
            .with_execution_providers(providers)
            .map_err(|error| format!("failed to configure ONNX execution providers: {error}"))?;
    }

    builder
        .commit_from_memory(model)
        .map_err(|error| format!("failed to load embedded AniFileBERT ONNX model: {error}"))
}

struct EncodedInput {
    input_ids: Vec<i64>,
    attention_mask: Vec<i64>,
    char_count: usize,
}

fn encode_chars(vocab: &HashMap<String, i64>, chars: &[char], max_seq_len: usize) -> EncodedInput {
    let char_capacity = max_seq_len.saturating_sub(2);
    let char_count = chars.len().min(char_capacity);
    let mut input_ids = vec![PAD_TOKEN; max_seq_len];
    let mut attention_mask = vec![0; max_seq_len];

    input_ids[0] = CLS_TOKEN;
    attention_mask[0] = 1;
    for (index, ch) in chars.iter().take(char_count).enumerate() {
        input_ids[index + 1] = token_id(vocab, *ch);
        attention_mask[index + 1] = 1;
    }
    input_ids[char_count + 1] = SEP_TOKEN;
    attention_mask[char_count + 1] = 1;

    EncodedInput {
        input_ids,
        attention_mask,
        char_count,
    }
}

fn token_id(vocab: &HashMap<String, i64>, ch: char) -> i64 {
    let token = ch.to_string();
    if let Some(id) = vocab.get(&token) {
        return *id;
    }
    let lower = ch.to_lowercase().to_string();
    vocab.get(&lower).copied().unwrap_or(UNK_TOKEN)
}

fn argmax_labels(
    shape: &[i64],
    logits: &[f32],
    id2label: &[String],
) -> Result<Vec<String>, String> {
    let (seq_len, label_count, base_offset) = match shape {
        [1, seq_len, label_count] => (*seq_len as usize, *label_count as usize, 0usize),
        [seq_len, label_count] => (*seq_len as usize, *label_count as usize, 0usize),
        other => return Err(format!("unexpected AniFileBERT logits shape: {other:?}")),
    };
    if label_count == 0 || logits.len() < base_offset + seq_len * label_count {
        return Err(format!(
            "invalid AniFileBERT logits length: shape={shape:?}, len={}",
            logits.len()
        ));
    }

    let mut labels = Vec::with_capacity(seq_len);
    for position in 0..seq_len {
        let start = base_offset + position * label_count;
        let row = &logits[start..start + label_count];
        let best = row
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.total_cmp(right.1))
            .map(|(index, _)| index)
            .unwrap_or(0);
        labels.push(
            id2label
                .get(best)
                .cloned()
                .unwrap_or_else(|| "O".to_string()),
        );
    }
    Ok(labels)
}

fn collect_entities(chars: &[char], labels: &[String]) -> Vec<Entity> {
    let mut entities = Vec::new();
    let mut current_label: Option<String> = None;
    let mut current_value = String::new();
    let mut current_start = 0usize;

    for (index, (ch, label)) in chars.iter().zip(labels.iter()).enumerate() {
        let Some((prefix, base_label)) = split_bio_label(label) else {
            flush_entity(
                &mut entities,
                &mut current_label,
                &mut current_value,
                current_start,
                index,
            );
            continue;
        };

        let should_continue = prefix == "I" && current_label.as_deref() == Some(base_label);
        if !should_continue {
            flush_entity(
                &mut entities,
                &mut current_label,
                &mut current_value,
                current_start,
                index,
            );
            current_label = Some(base_label.to_string());
            current_start = index;
        }
        current_value.push(*ch);
    }

    flush_entity(
        &mut entities,
        &mut current_label,
        &mut current_value,
        current_start,
        chars.len(),
    );
    entities
}

fn split_bio_label(label: &str) -> Option<(&str, &str)> {
    label
        .split_once('-')
        .filter(|(prefix, base)| (*prefix == "B" || *prefix == "I") && !base.is_empty())
}

fn flush_entity(
    entities: &mut Vec<Entity>,
    current_label: &mut Option<String>,
    current_value: &mut String,
    start: usize,
    end: usize,
) {
    if let Some(label) = current_label.take() {
        let value = clean_value(current_value);
        if !value.is_empty() {
            entities.push(Entity {
                label,
                value,
                start,
                end,
            });
        }
    }
    current_value.clear();
}

fn info_from_entities(
    path: &Path,
    file_name: &str,
    chars: &[char],
    entities: &[Entity],
) -> Option<AnimeFileInfo> {
    let mut title = choose_title(entities)?;
    if split_series_and_season(&title).1.is_none() {
        if let Some(season) = season_entity(entities) {
            let title_with_season = format!("{title} {}", season.value);
            if split_series_and_season(&title_with_season).1.is_some() {
                title = title_with_season;
            }
        }
    }
    let episode = entities
        .iter()
        .find(|entity| entity.label == "EPISODE")
        .map(|entity| entity.value.as_str())
        .and_then(format_episode)?;
    let episode_span = entities.iter().find(|entity| entity.label == "EPISODE")?;
    let publisher = entities
        .iter()
        .find(|entity| entity.label == "GROUP")
        .map(|entity| entity.value.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| format!(".{}", value.to_lowercase()))
        .unwrap_or_default();
    let tags = tags_after_episode(file_name, chars, episode_span, &extension)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| tag_entities(entities));

    Some(AnimeFileInfo {
        publisher,
        anime_name: title,
        episode,
        tags,
        extension,
        original_path: path.to_string_lossy().to_string(),
    })
}

fn season_from_entities(entities: &[Entity]) -> Option<u32> {
    let season = season_entity(entities)?;
    split_series_and_season(&format!("Title {}", season.value)).1
}

fn season_entity(entities: &[Entity]) -> Option<&Entity> {
    entities
        .iter()
        .find(|entity| matches!(entity.label.as_str(), "SEASON" | "PATH_SEASON"))
}

fn choose_title(entities: &[Entity]) -> Option<String> {
    let title = join_entities(
        entities
            .iter()
            .filter(|entity| entity.label.starts_with("TITLE_")),
    );
    if !title.is_empty() {
        return Some(title);
    }

    let path_title = join_entities(
        entities
            .iter()
            .filter(|entity| entity.label.starts_with("PATH_TITLE_")),
    );
    if path_title.is_empty() {
        None
    } else {
        Some(path_title)
    }
}

fn join_entities<'a>(entities: impl Iterator<Item = &'a Entity>) -> String {
    let mut values = Vec::new();
    for entity in entities {
        if values.last() != Some(&entity.value) {
            values.push(entity.value.clone());
        }
    }
    values.join(" ").trim().to_string()
}

fn tags_after_episode(
    file_name: &str,
    chars: &[char],
    episode_span: &Entity,
    extension: &str,
) -> Option<String> {
    let stem_char_len = if extension.is_empty() {
        chars.len()
    } else {
        let stem = file_name.strip_suffix(extension).or_else(|| {
            file_name
                .len()
                .checked_sub(extension.len())
                .and_then(|len| file_name.get(..len))
        })?;
        stem.chars().count()
    };
    if episode_span.end >= stem_char_len {
        return Some(String::new());
    }

    let value = chars[episode_span.end..stem_char_len]
        .iter()
        .collect::<String>();
    Some(clean_leading_separators(&value))
}

fn tag_entities(entities: &[Entity]) -> String {
    join_entities(entities.iter().filter(|entity| {
        matches!(
            entity.label.as_str(),
            "RESOLUTION" | "SOURCE" | "SPECIAL" | "TAG" | "SEASON"
        )
    }))
}

fn labels_from_config(labels: &HashMap<String, String>) -> Result<Vec<String>, String> {
    let max_id = labels
        .keys()
        .filter_map(|key| key.parse::<usize>().ok())
        .max()
        .ok_or_else(|| "AniFileBERT config has no id2label entries".to_string())?;
    let mut id2label = vec!["O".to_string(); max_id + 1];
    for (key, label) in labels {
        let id = key
            .parse::<usize>()
            .map_err(|error| format!("invalid AniFileBERT label id {key}: {error}"))?;
        id2label[id] = label.clone();
    }
    Ok(id2label)
}

fn format_episode(raw: &str) -> Option<String> {
    let episode = raw.trim().trim_matches(['[', ']']);
    if episode.is_empty() {
        return None;
    }
    if episode.contains('.') {
        return Some(episode.to_string());
    }
    if episode.chars().all(|ch| ch.is_ascii_digit()) {
        Some(format!("{:0>2}", episode))
    } else {
        Some(episode.to_string())
    }
}

fn clean_value(value: &str) -> String {
    value
        .trim()
        .trim_matches(['[', ']', '【', '】'])
        .trim()
        .to_string()
}

fn clean_leading_separators(value: &str) -> String {
    value
        .trim()
        .trim_start_matches(['-', '_', '.', ' '])
        .trim()
        .to_string()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProviderMode {
    Cpu,
    Auto,
    DirectMl(DirectMlDeviceFilter),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DirectMlDeviceFilter {
    Gpu,
    Npu,
    Any,
}

fn provider_mode() -> Result<ProviderMode, String> {
    provider_mode_from_str(env::var(ENV_PROVIDER).ok().as_deref())
}

fn provider_mode_from_str(value: Option<&str>) -> Result<ProviderMode, String> {
    let value = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| default_provider_mode().to_string());
    let normalized = value.to_ascii_lowercase().replace('_', "-");

    match normalized.as_str() {
        "cpu" => Ok(ProviderMode::Cpu),
        "auto" => Ok(ProviderMode::Auto),
        "directml" | "directml-gpu" | "dml" | "gpu" => {
            Ok(ProviderMode::DirectMl(DirectMlDeviceFilter::Gpu))
        }
        "directml-npu" | "dml-npu" | "npu" => {
            Ok(ProviderMode::DirectMl(DirectMlDeviceFilter::Npu))
        }
        "directml-any" | "dml-any" => Ok(ProviderMode::DirectMl(DirectMlDeviceFilter::Any)),
        _ => Err(format!(
            "unsupported {ENV_PROVIDER}={value}; expected cpu, auto, directml-gpu, directml-npu, or directml-any"
        )),
    }
}

fn default_provider_mode() -> &'static str {
    if directml_available_in_build() {
        "auto"
    } else {
        "cpu"
    }
}

fn directml_available_in_build() -> bool {
    cfg!(all(feature = "anifilebert-directml", target_os = "windows"))
}

fn directml_provider(
    filter: DirectMlDeviceFilter,
    required: bool,
) -> Result<Vec<ExecutionProviderDispatch>, String> {
    directml_provider_impl(filter, required)
}

#[cfg(all(feature = "anifilebert-directml", target_os = "windows"))]
fn directml_provider_impl(
    filter: DirectMlDeviceFilter,
    required: bool,
) -> Result<Vec<ExecutionProviderDispatch>, String> {
    use ort::ep;

    let mut provider = ep::DirectML::default();
    if let Some(device_id) = env_i32(ENV_DIRECTML_DEVICE_ID)? {
        provider = provider.with_device_id(device_id);
    } else {
        provider = provider.with_device_filter(match filter {
            DirectMlDeviceFilter::Gpu => ep::directml::DeviceFilter::Gpu,
            DirectMlDeviceFilter::Npu => ep::directml::DeviceFilter::Npu,
            DirectMlDeviceFilter::Any => ep::directml::DeviceFilter::Any,
        });
    }

    let provider = provider.build();
    let provider = if required {
        provider.error_on_failure()
    } else {
        provider.fail_silently()
    };
    Ok(vec![provider])
}

#[cfg(not(all(feature = "anifilebert-directml", target_os = "windows")))]
fn directml_provider_impl(
    _filter: DirectMlDeviceFilter,
    _required: bool,
) -> Result<Vec<ExecutionProviderDispatch>, String> {
    Err(format!(
        "{ENV_PROVIDER}=directml requires a Windows build with --features anifilebert-directml"
    ))
}

fn env_thread_count(name: &str) -> Result<Option<usize>, String> {
    let Some(value) = env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    let count = value
        .parse::<usize>()
        .map_err(|error| format!("invalid {name}={value}: {error}"))?;
    if count == 0 {
        return Err(format!(
            "invalid {name}=0: thread count must be greater than zero"
        ));
    }
    Ok(Some(count))
}

#[cfg(all(feature = "anifilebert-directml", target_os = "windows"))]
fn env_i32(name: &str) -> Result<Option<i32>, String> {
    let Some(value) = env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    let id = value
        .parse::<i32>()
        .map_err(|error| format!("invalid {name}={value}: {error}"))?;
    if id < 0 {
        return Err(format!(
            "invalid {name}={value}: device id must be non-negative"
        ));
    }
    Ok(Some(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_bio_entities() {
        let chars: Vec<char> = "[Grp] Anime - 12 [1080p]".chars().collect();
        let labels = [
            "O",
            "B-GROUP",
            "I-GROUP",
            "I-GROUP",
            "O",
            "O",
            "B-TITLE_LATIN",
            "I-TITLE_LATIN",
            "I-TITLE_LATIN",
            "I-TITLE_LATIN",
            "I-TITLE_LATIN",
            "O",
            "O",
            "O",
            "B-EPISODE",
            "I-EPISODE",
            "O",
            "B-RESOLUTION",
            "I-RESOLUTION",
            "I-RESOLUTION",
            "I-RESOLUTION",
            "I-RESOLUTION",
            "I-RESOLUTION",
            "O",
        ]
        .into_iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();

        let entities = collect_entities(&chars, &labels);

        assert_eq!(
            entities,
            vec![
                Entity {
                    label: "GROUP".to_string(),
                    value: "Grp".to_string(),
                    start: 1,
                    end: 4,
                },
                Entity {
                    label: "TITLE_LATIN".to_string(),
                    value: "Anime".to_string(),
                    start: 6,
                    end: 11,
                },
                Entity {
                    label: "EPISODE".to_string(),
                    value: "12".to_string(),
                    start: 14,
                    end: 16,
                },
                Entity {
                    label: "RESOLUTION".to_string(),
                    value: "1080p".to_string(),
                    start: 17,
                    end: 23,
                },
            ]
        );
    }

    #[test]
    fn builds_current_parser_shape() {
        let path = Path::new("[Grp] Anime - 1 [1080p].mkv");
        let chars: Vec<char> = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .chars()
            .collect();
        let entities = vec![
            Entity {
                label: "GROUP".to_string(),
                value: "Grp".to_string(),
                start: 1,
                end: 4,
            },
            Entity {
                label: "TITLE_LATIN".to_string(),
                value: "Anime".to_string(),
                start: 6,
                end: 11,
            },
            Entity {
                label: "EPISODE".to_string(),
                value: "1".to_string(),
                start: 14,
                end: 15,
            },
        ];

        let info = info_from_entities(
            path,
            path.file_name().unwrap().to_str().unwrap(),
            &chars,
            &entities,
        )
        .unwrap();

        assert_eq!(info.publisher, "Grp");
        assert_eq!(info.anime_name, "Anime");
        assert_eq!(info.episode, "01");
        assert_eq!(info.tags, "[1080p]");
        assert_eq!(info.extension, ".mkv");
    }

    #[test]
    fn preserves_model_season_entity_in_parser_result() {
        let path = Path::new("[Grp] Anime S2 - 01 [1080p].mkv");
        let chars: Vec<char> = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .chars()
            .collect();
        let entities = vec![
            Entity {
                label: "GROUP".to_string(),
                value: "Grp".to_string(),
                start: 1,
                end: 4,
            },
            Entity {
                label: "TITLE_LATIN".to_string(),
                value: "Anime".to_string(),
                start: 6,
                end: 11,
            },
            Entity {
                label: "SEASON".to_string(),
                value: "S2".to_string(),
                start: 12,
                end: 14,
            },
            Entity {
                label: "EPISODE".to_string(),
                value: "01".to_string(),
                start: 17,
                end: 19,
            },
        ];

        let info = info_from_entities(
            path,
            path.file_name().unwrap().to_str().unwrap(),
            &chars,
            &entities,
        )
        .unwrap();

        assert_eq!(season_from_entities(&entities), Some(2));
        assert_eq!(info.anime_name, "Anime S2");
        assert_eq!(info.season_number(), Some(2));
    }

    #[test]
    fn parses_provider_modes() {
        assert_eq!(
            provider_mode_from_str(Some("cpu")).unwrap(),
            ProviderMode::Cpu
        );
        assert_eq!(
            provider_mode_from_str(Some("auto")).unwrap(),
            ProviderMode::Auto
        );
        assert_eq!(
            provider_mode_from_str(Some("gpu")).unwrap(),
            ProviderMode::DirectMl(DirectMlDeviceFilter::Gpu)
        );
        assert_eq!(
            provider_mode_from_str(Some("directml_npu")).unwrap(),
            ProviderMode::DirectMl(DirectMlDeviceFilter::Npu)
        );
        assert!(provider_mode_from_str(Some("vitis")).is_err());
    }

    #[test]
    fn runs_embedded_model_parser_smoke() {
        let mut parser = AniFileBertParser::new().unwrap();
        let path =
            Path::new("[GM-Team][国漫][神印王座][Throne of Seal][2022][200][AVC][GB][1080P].mp4");

        parser.parse_path(path).unwrap();
    }

    #[test]
    fn embedded_model_detects_candidate_season() {
        assert_eq!(
            detect_season("恋爱游戏世界对路人角色很不友好 第二季").unwrap(),
            Some(2)
        );
        assert_eq!(
            detect_season("恋爱游戏世界对路人角色很不友好").unwrap(),
            None
        );
        assert_eq!(
            detect_season("乙女ゲー世界はモブに厳しい世界です2").unwrap(),
            Some(2)
        );
        assert_eq!(
            detect_season("乙女ゲー世界はモブに厳しい世界です").unwrap(),
            None
        );
    }
}
