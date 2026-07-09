//! AniFileBERT ONNX fallback parser.
//!
//! The rule parser remains the default. This module is loaded only when the
//! `anifilebert` feature is enabled and the CLI asks for it.

use crate::parser::AnimeFileInfo;
use flate2::read::GzDecoder;
use ort::{
    ep::ExecutionProviderDispatch,
    session::{builder::GraphOptimizationLevel, Session},
    value::Tensor,
};
use serde::Deserialize;
use std::collections::HashMap;
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

static PARSER: OnceLock<Result<Mutex<AniFileBertParser>, String>> = OnceLock::new();

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
    let parser = PARSER.get_or_init(|| AniFileBertParser::new().map(Mutex::new));
    let parser = parser.as_ref().map_err(Clone::clone)?;
    let mut parser = parser
        .lock()
        .map_err(|_| "AniFileBERT parser lock poisoned".to_string())?;
    parser.parse_path(path)
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

        let mut builder = Session::builder()
            .map_err(|error| format!("failed to create ONNX Runtime session builder: {error}"))?;
        builder = builder
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|error| format!("failed to configure ONNX graph optimization: {error}"))?;

        let providers = execution_providers();
        if !providers.is_empty() {
            builder = builder
                .with_execution_providers(providers)
                .map_err(|error| {
                    format!("failed to configure ONNX execution providers: {error}")
                })?;
        }

        let session = builder
            .commit_from_memory(&model)
            .map_err(|error| format!("failed to load embedded AniFileBERT ONNX model: {error}"))?;

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
        if chars.is_empty() {
            return Ok(None);
        }

        let encoded = encode_chars(&self.vocab, &chars, self.max_seq_len);
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
        let labels = argmax_labels(shape, logits, &self.id2label)?;
        let labels = labels
            .into_iter()
            .skip(1)
            .take(encoded.char_count)
            .collect::<Vec<_>>();
        let entities = collect_entities(&chars[..encoded.char_count], &labels);

        Ok(info_from_entities(path, file_name, &chars, &entities))
    }
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
    let title = choose_title(entities)?;
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
        original_path: path.to_path_buf(),
    })
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

fn execution_providers() -> Vec<ExecutionProviderDispatch> {
    #[cfg(all(feature = "anifilebert-amd-npu", target_os = "windows"))]
    {
        use ort::ep;

        vec![ep::DirectML::default()
            .with_device_filter(ep::directml::DeviceFilter::Npu)
            .build()]
    }

    #[cfg(not(all(feature = "anifilebert-amd-npu", target_os = "windows")))]
    {
        Vec::new()
    }
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
}
