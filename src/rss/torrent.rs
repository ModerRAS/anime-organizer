//! .torrent 下载和 magnet 转换模块
//!
//! 处理 .torrent 文件下载、bencode 解析和 magnet 链接构建。

use bt_bencode::Value;
use sha1::{Digest, Sha1};

use crate::error::{AppError, Result};

/// 从 .torrent 文件字节内容构建 magnet 链接
///
/// 解析 bencode 格式的 .torrent 文件，提取 info_hash 和 tracker list，
/// 构建 `magnet:?xt=urn:btih:{info_hash}&tr=...` 格式链接。
pub fn torrent_bytes_to_magnet(data: &[u8]) -> Result<String> {
    let value: Value = bt_bencode::from_slice(data)
        .map_err(|e| AppError::MetadataFetchError(format!("解析 .torrent 文件失败: {e}")))?;

    let dict = match &value {
        Value::Dict(d) => d,
        _ => {
            return Err(AppError::MetadataFetchError(
                ".torrent 文件格式无效：顶层不是字典".to_string(),
            ))
        }
    };

    // 提取 info 字典并计算 SHA1 hash
    let info_key: bt_bencode::ByteString = b"info"[..].into();
    let info_value = dict
        .get(&info_key)
        .ok_or_else(|| AppError::MetadataFetchError(".torrent 文件缺少 info 字典".to_string()))?;

    // 重新编码 info 字典以计算 hash
    let info_bytes = bt_bencode::to_vec(info_value)
        .map_err(|e| AppError::MetadataFetchError(format!("编码 info 字典失败: {e}")))?;

    let mut hasher = Sha1::new();
    hasher.update(&info_bytes);
    let info_hash = hasher.finalize();
    let info_hash_hex = hex_encode(&info_hash);

    // 构建 magnet 链接
    let mut magnet = format!("magnet:?xt=urn:btih:{info_hash_hex}");

    // 提取 tracker URLs
    let trackers = extract_trackers(dict);
    for tracker in &trackers {
        magnet.push_str("&tr=");
        magnet.push_str(&url_encode(tracker));
    }

    // 提取名称（可选）
    if let Some(name) = extract_name(dict) {
        magnet.push_str("&dn=");
        magnet.push_str(&url_encode(&name));
    }

    Ok(magnet)
}

/// 下载 .torrent 文件并转换为 magnet 链接
pub async fn download_torrent_to_magnet(
    client: &reqwest::Client,
    torrent_url: &str,
) -> Result<String> {
    let response = client
        .get(torrent_url)
        .send()
        .await
        .map_err(|e| AppError::MetadataFetchError(format!("下载 .torrent 文件失败: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::MetadataFetchError(format!(
            "下载 .torrent 文件失败，HTTP 状态码: {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::MetadataFetchError(format!("读取 .torrent 内容失败: {e}")))?;

    torrent_bytes_to_magnet(&bytes)
}

type BtDict = std::collections::BTreeMap<bt_bencode::ByteString, Value>;

/// 从 torrent 字典中提取 tracker 列表
fn extract_trackers(dict: &BtDict) -> Vec<String> {
    let mut trackers = Vec::new();

    let announce_list_key: bt_bencode::ByteString = b"announce-list"[..].into();
    let announce_key: bt_bencode::ByteString = b"announce"[..].into();

    // announce-list (多 tracker 列表)
    if let Some(Value::List(announce_list)) = dict.get(&announce_list_key) {
        for tier in announce_list {
            if let Value::List(tier_list) = tier {
                for item in tier_list {
                    if let Some(url) = value_to_string(item) {
                        if !trackers.contains(&url) {
                            trackers.push(url);
                        }
                    }
                }
            }
        }
    }

    // announce (单 tracker)
    if let Some(value) = dict.get(&announce_key) {
        if let Some(url) = value_to_string(value) {
            if !trackers.contains(&url) {
                trackers.insert(0, url);
            }
        }
    }

    trackers
}

/// 从 info 字典中提取名称
fn extract_name(dict: &BtDict) -> Option<String> {
    let info_key: bt_bencode::ByteString = b"info"[..].into();
    let name_key: bt_bencode::ByteString = b"name"[..].into();

    let info = dict.get(&info_key)?;
    if let Value::Dict(info_dict) = info {
        if let Some(value) = info_dict.get(&name_key) {
            return value_to_string(value);
        }
    }
    None
}

/// 将 Value::ByteStr 转换为 String
fn value_to_string(value: &Value) -> Option<String> {
    value
        .as_byte_str()
        .and_then(|bs| std::str::from_utf8(bs.as_slice()).ok())
        .map(|s| s.to_string())
}

/// 将字节数组编码为十六进制字符串
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// 简易 URL 编码
fn url_encode(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 3);
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{byte:02X}"));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_encode() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];
        assert_eq!(hex_encode(&bytes), "deadbeef");
    }

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("hello world"), "hello%20world");
        assert_eq!(url_encode("http://example.com"), "http%3A%2F%2Fexample.com");
        assert_eq!(url_encode("simple"), "simple");
    }

    #[test]
    fn test_torrent_bytes_invalid() {
        let result = torrent_bytes_to_magnet(b"not valid bencode");
        assert!(result.is_err());
    }

    #[test]
    fn test_torrent_bytes_to_magnet_minimal() {
        use bt_bencode::Value;
        use std::collections::BTreeMap;

        let mut info: BTreeMap<bt_bencode::ByteString, Value> = BTreeMap::new();
        info.insert(b"name"[..].into(), Value::ByteStr(b"test"[..].into()));
        info.insert(b"piece length"[..].into(), Value::Int(262144.into()));
        info.insert(b"pieces"[..].into(), Value::ByteStr(vec![0u8; 20].into()));

        let mut torrent: BTreeMap<bt_bencode::ByteString, Value> = BTreeMap::new();
        torrent.insert(
            b"announce"[..].into(),
            Value::ByteStr(b"http://tracker.example.com/announce"[..].into()),
        );
        torrent.insert(b"info"[..].into(), Value::Dict(info));

        let data = bt_bencode::to_vec(&Value::Dict(torrent)).unwrap();
        let magnet = torrent_bytes_to_magnet(&data).unwrap();

        assert!(magnet.starts_with("magnet:?xt=urn:btih:"));
        assert!(magnet.contains("&tr="));
        assert!(magnet.contains("tracker.example.com"));
        assert!(magnet.contains("&dn=test"));
    }
}
