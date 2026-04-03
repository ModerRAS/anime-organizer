//! Bangumi dump 客户端模块
//!
//! 提供 Bangumi Archive dump 的下载、加载和查询功能。

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Bangumi subject 数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiSubject {
    /// Subject ID
    pub id: u32,
    /// 类型（2 = 动画）
    #[serde(rename = "type")]
    pub subject_type: u32,
    /// 日文名
    pub name: String,
    /// 中文名
    pub name_cn: Option<String>,
    /// Infobox wiki 文本
    pub infobox: Option<String>,
    /// 简介
    pub summary: Option<String>,
    /// 首播日期
    pub date: Option<String>,
    /// 评分
    pub score: Option<f32>,
    /// 平台
    pub platform: Option<u32>,
}

/// Bangumi 剧集数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiEpisode {
    /// Episode ID
    pub id: u32,
    /// 所属 Subject ID
    pub subject_id: u32,
    /// 集数
    #[serde(default)]
    pub sort: f32,
    /// 标题
    pub name: Option<String>,
    /// 中文标题
    pub name_cn: Option<String>,
    /// 播出日期
    pub airdate: Option<String>,
    /// 时长
    pub duration: Option<String>,
}

/// Bangumi 客户端
///
/// 加载和查询 Bangumi Archive dump 数据。
pub struct BangumiClient {
    /// Subject 索引（by ID）
    subjects_by_id: HashMap<u32, BangumiSubject>,
    /// Subject 索引（by name）
    subjects_by_name: HashMap<String, u32>,
    /// Subject 索引（by name_cn）
    subjects_by_name_cn: HashMap<String, u32>,
    /// 缓存目录
    cache_dir: PathBuf,
}

impl BangumiClient {
    /// 创建新的 Bangumi 客户端
    ///
    /// # 参数
    ///
    /// * `cache_dir` - Bangumi dump 的缓存目录
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Self {
        Self {
            subjects_by_id: HashMap::new(),
            subjects_by_name: HashMap::new(),
            subjects_by_name_cn: HashMap::new(),
            cache_dir: cache_dir.as_ref().to_path_buf(),
        }
    }

    /// 加载 subject.jsonlines 数据
    ///
    /// 仅加载 type=2（动画）的 subject。
    ///
    /// # 参数
    ///
    /// * `path` - subject.jsonlines 文件路径
    pub fn load_subjects<P: AsRef<Path>>(&mut self, path: P) -> Result<usize> {
        let file = std::fs::File::open(path.as_ref()).map_err(|e| {
            AppError::BangumiParseError(format!(
                "无法打开文件 {}: {e}",
                path.as_ref().display()
            ))
        })?;

        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line.map_err(|e| AppError::BangumiParseError(format!("读取行失败: {e}")))?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let subject: BangumiSubject = match serde_json::from_str(line) {
                Ok(s) => s,
                Err(_) => continue, // 跳过无法解析的行
            };

            // 仅加载动画类型
            if subject.subject_type != 2 {
                continue;
            }

            let id = subject.id;
            self.subjects_by_name
                .insert(subject.name.clone(), subject.id);
            if let Some(ref name_cn) = subject.name_cn {
                if !name_cn.is_empty() {
                    self.subjects_by_name_cn.insert(name_cn.clone(), subject.id);
                }
            }
            self.subjects_by_id.insert(id, subject);
            count += 1;
        }

        Ok(count)
    }

    /// 根据 ID 查找 subject
    pub fn find_by_id(&self, id: u32) -> Option<&BangumiSubject> {
        self.subjects_by_id.get(&id)
    }

    /// 根据名称精确查找
    pub fn find_by_name(&self, name: &str) -> Option<&BangumiSubject> {
        // 先查日文名
        if let Some(id) = self.subjects_by_name.get(name) {
            return self.subjects_by_id.get(id);
        }
        // 再查中文名
        if let Some(id) = self.subjects_by_name_cn.get(name) {
            return self.subjects_by_id.get(id);
        }
        None
    }

    /// 模糊搜索
    ///
    /// 返回名称中包含查询字符串的所有 subject。
    pub fn search(&self, query: &str) -> Vec<&BangumiSubject> {
        let query_lower = query.to_lowercase();
        self.subjects_by_id
            .values()
            .filter(|s| {
                s.name.to_lowercase().contains(&query_lower)
                    || s.name_cn
                        .as_deref()
                        .map_or(false, |cn| cn.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// 获取缓存目录
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// 获取已加载的 subject 数量
    pub fn subject_count(&self) -> usize {
        self.subjects_by_id.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_test_dump(dir: &Path) -> PathBuf {
        let path = dir.join("subject.jsonlines");
        let mut file = std::fs::File::create(&path).unwrap();

        // 动画类型
        writeln!(file, r#"{{"id":8,"type":2,"name":"コードギアス 反逆のルルーシュR2","name_cn":"Code Geass 反叛的鲁路修R2","summary":"test summary","date":"2008-04-06","score":9.0,"infobox":"","platform":0}}"#).unwrap();
        writeln!(file, r#"{{"id":265,"type":2,"name":"進撃の巨人","name_cn":"进击的巨人","summary":"进击的巨人简介","date":"2013-04-07","score":8.5,"infobox":"","platform":0}}"#).unwrap();
        // 非动画类型（应被跳过）
        writeln!(
            file,
            r#"{{"id":100,"type":1,"name":"某漫画","name_cn":"某漫画","summary":"","date":"2020-01-01","score":7.0,"infobox":"","platform":0}}"#
        )
        .unwrap();

        path
    }

    #[test]
    fn test_load_subjects() {
        let dir = tempfile::TempDir::new().unwrap();
        let dump_path = create_test_dump(dir.path());

        let mut client = BangumiClient::new(dir.path());
        let count = client.load_subjects(&dump_path).unwrap();

        assert_eq!(count, 2); // 只加载动画类型
        assert_eq!(client.subject_count(), 2);
    }

    #[test]
    fn test_find_by_id() {
        let dir = tempfile::TempDir::new().unwrap();
        let dump_path = create_test_dump(dir.path());
        let mut client = BangumiClient::new(dir.path());
        client.load_subjects(&dump_path).unwrap();

        let subject = client.find_by_id(8).unwrap();
        assert_eq!(subject.name, "コードギアス 反逆のルルーシュR2");
    }

    #[test]
    fn test_find_by_name_japanese() {
        let dir = tempfile::TempDir::new().unwrap();
        let dump_path = create_test_dump(dir.path());
        let mut client = BangumiClient::new(dir.path());
        client.load_subjects(&dump_path).unwrap();

        let subject = client.find_by_name("進撃の巨人").unwrap();
        assert_eq!(subject.id, 265);
    }

    #[test]
    fn test_find_by_name_chinese() {
        let dir = tempfile::TempDir::new().unwrap();
        let dump_path = create_test_dump(dir.path());
        let mut client = BangumiClient::new(dir.path());
        client.load_subjects(&dump_path).unwrap();

        let subject = client.find_by_name("进击的巨人").unwrap();
        assert_eq!(subject.id, 265);
    }

    #[test]
    fn test_search() {
        let dir = tempfile::TempDir::new().unwrap();
        let dump_path = create_test_dump(dir.path());
        let mut client = BangumiClient::new(dir.path());
        client.load_subjects(&dump_path).unwrap();

        let results = client.search("巨人");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "進撃の巨人");
    }

    #[test]
    fn test_find_not_found() {
        let dir = tempfile::TempDir::new().unwrap();
        let dump_path = create_test_dump(dir.path());
        let mut client = BangumiClient::new(dir.path());
        client.load_subjects(&dump_path).unwrap();

        assert!(client.find_by_id(99999).is_none());
        assert!(client.find_by_name("不存在").is_none());
    }
}
