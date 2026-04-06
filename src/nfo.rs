//! NFO 文件生成模块
//!
//! 生成 Kodi 兼容的 NFO 文件（XML 格式）。
//!
//! ## 文件类型
//!
//! - `tvshow.nfo` - 电视剧级别元数据
//! - `episodedetails.nfo` - 单集级别元数据
//!
//! ## 目录结构
//!
//! ```text
//! Anime Name/
//! ├── tvshow.nfo
//! ├── poster.jpg
//! ├── fanart.jpg
//! └── Season 1/
//!     ├── 01 [1080P].mkv
//!     └── 01 [1080P].nfo
//! ```

use std::fmt::Write as FmtWrite;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// 电视剧 NFO 数据
///
/// 对应 Kodi 的 `<tvshow>` 格式。
///
/// 符合 Kodi Wiki 规范：https://kodi.wiki/view/NFO_files/TV_shows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvShowNfo {
    /// 标题（必需）
    pub title: String,
    /// 原始标题
    pub originaltitle: String,
    /// 排序标题
    pub sorttitle: Option<String>,
    /// 简介
    pub plot: String,
    /// 类型标签
    pub genre: Vec<String>,
    /// 首播日期（YYYY-MM-DD）
    pub premiered: Option<String>,
    /// 状态 (Continuing/Ended)
    pub status: Option<String>,
    /// 制作公司
    pub studio: Option<String>,
    /// 评分
    pub rating: Option<Rating>,
    /// 用户个人评分
    pub userrating: Option<f32>,
    /// IMDB Top 250 排名
    pub top250: Option<u32>,
    /// 唯一标识（必需）
    pub uniqueid: Vec<UniqueId>,
    /// 演员/声优
    pub actor: Vec<Actor>,
    /// 季数
    pub season: Option<u32>,
    /// 总集数
    pub episode: Option<u32>,
    /// 简短标语（v22+）
    pub tagline: Option<String>,
    /// MPAA 分级
    pub mpaa: Option<String>,
    /// 播放次数
    pub playcount: Option<u32>,
    /// 最后播放日期（YYYY-MM-DD HH:MM:SS）
    pub lastplayed: Option<String>,
    /// 标签
    pub tag: Vec<String>,
}

/// 评分信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
    /// 评分来源名称
    pub name: String,
    /// 最大分值
    pub max: u32,
    /// 是否为默认评分
    pub default: bool,
    /// 分数
    pub value: f32,
    /// 投票数
    pub votes: Option<u32>,
}

/// 唯一标识
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniqueId {
    /// 标识类型（如 bangumi, tmdb, anidb）
    #[serde(rename = "type")]
    pub id_type: String,
    /// 是否为默认标识
    pub default: bool,
    /// 标识值
    pub value: String,
}

/// 演员/声优信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    /// 姓名
    pub name: String,
    /// 角色
    pub role: Option<String>,
    /// 排序序号
    pub order: Option<u32>,
    /// 头像图片路径
    pub thumb: Option<String>,
}

/// 单集 NFO 数据
///
/// 对应 Kodi 的 `<episodedetails>` 格式。
///
/// 符合 Kodi Wiki 规范：https://kodi.wiki/view/NFO_files/Episodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeNfo {
    /// 集标题（必需）
    pub title: String,
    /// 季号（从文件名读取，可选）
    pub season: u32,
    /// 集号（从文件名读取，可选）
    pub episode: u32,
    /// 简介
    pub plot: Option<String>,
    /// 播出日期（YYYY-MM-DD）
    pub aired: Option<String>,
    /// 时长（分钟）
    pub runtime: Option<u32>,
    /// 显示季号（Specials 排序用）
    pub displayseason: Option<i32>,
    /// 显示集号（Specials 排序用）
    pub displayepisode: Option<i32>,
    /// 唯一标识（必需）
    pub uniqueid: Vec<UniqueId>,
    /// 编剧
    pub credits: Vec<String>,
    /// 导演
    pub director: Vec<String>,
    /// 演员
    pub actor: Vec<Actor>,
    /// 集标语
    pub tagline: Option<String>,
    /// 播放次数
    pub playcount: Option<u32>,
    /// 最后播放日期（YYYY-MM-DD HH:MM:SS）
    pub lastplayed: Option<String>,
}

impl TvShowNfo {
    /// 序列化为 XML 字符串
    pub fn to_xml(&self) -> Result<String> {
        let mut xml = String::new();
        writeln!(
            xml,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#
        )
        .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
        writeln!(xml, "<tvshow>")
            .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;

        write_xml_tag(&mut xml, "title", &self.title)?;
        write_xml_tag(&mut xml, "originaltitle", &self.originaltitle)?;

        if let Some(ref sort) = self.sorttitle {
            write_xml_tag(&mut xml, "sorttitle", sort)?;
        }

        write_xml_tag(&mut xml, "plot", &self.plot)?;

        for genre in &self.genre {
            write_xml_tag(&mut xml, "genre", genre)?;
        }

        if let Some(ref premiered) = self.premiered {
            write_xml_tag(&mut xml, "premiered", premiered)?;
        }

        if let Some(ref status) = self.status {
            write_xml_tag(&mut xml, "status", status)?;
        }

        if let Some(ref studio) = self.studio {
            write_xml_tag(&mut xml, "studio", studio)?;
        }

        // 简短标语（v22+）
        if let Some(ref tagline) = self.tagline {
            write_xml_tag(&mut xml, "tagline", tagline)?;
        }

        // MPAA 分级
        if let Some(ref mpaa) = self.mpaa {
            write_xml_tag(&mut xml, "mpaa", mpaa)?;
        }

        if let Some(ref season) = self.season {
            write_xml_tag(&mut xml, "season", &season.to_string())?;
        }

        if let Some(ref ep) = self.episode {
            write_xml_tag(&mut xml, "episode", &ep.to_string())?;
        }

        // 评分
        if let Some(ref rating) = self.rating {
            writeln!(xml, "  <ratings>")
                .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
            writeln!(
                xml,
                r#"    <rating name="{}" max="{}" default="{}">"#,
                xml_escape(&rating.name),
                rating.max,
                rating.default
            )
            .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
            writeln!(xml, "      <value>{}</value>", rating.value)
                .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
            if let Some(votes) = rating.votes {
                writeln!(xml, "      <votes>{votes}</votes>")
                    .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
            }
            writeln!(xml, "    </rating>")
                .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
            writeln!(xml, "  </ratings>")
                .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
        }

        // 唯一标识
        for uid in &self.uniqueid {
            writeln!(
                xml,
                r#"  <uniqueid type="{}" default="{}">{}</uniqueid>"#,
                xml_escape(&uid.id_type),
                uid.default,
                xml_escape(&uid.value)
            )
            .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
        }

        // 用户评分
        if let Some(userrating) = self.userrating {
            writeln!(xml, "  <userrating>{}</userrating>", userrating)
                .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败：{e}")))?;
        }

        // IMDB Top 250
        if let Some(top250) = self.top250 {
            writeln!(xml, "  <top250>{}</top250>", top250)
                .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败：{e}")))?;
        }

        // 播放次数
        if let Some(playcount) = self.playcount {
            writeln!(xml, "  <playcount>{}</playcount>", playcount)
                .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败：{e}")))?;
        }

        // 最后播放日期
        if let Some(ref lastplayed) = self.lastplayed {
            write_xml_tag(&mut xml, "lastplayed", lastplayed)?;
        }

        // 标签
        for tag in &self.tag {
            write_xml_tag(&mut xml, "tag", tag)?;
        }

        // 演员
        for a in &self.actor {
            writeln!(xml, "  <actor>")
                .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
            writeln!(xml, "    <name>{}</name>", xml_escape(&a.name))
                .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
            if let Some(ref role) = a.role {
                writeln!(xml, "    <role>{}</role>", xml_escape(role))
                    .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
            }
            if let Some(order) = a.order {
                writeln!(xml, "    <order>{order}</order>")
                    .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
            }
            if let Some(ref thumb) = a.thumb {
                writeln!(xml, "    <thumb>{}</thumb>", xml_escape(thumb))
                    .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
            }
            writeln!(xml, "  </actor>")
                .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
        }

        writeln!(xml, "</tvshow>")
            .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;

        Ok(xml)
    }
}

impl EpisodeNfo {
    /// 序列化为 XML 字符串
    pub fn to_xml(&self) -> Result<String> {
        let mut xml = String::new();
        writeln!(
            xml,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#
        )
        .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
        writeln!(xml, "<episodedetails>")
            .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;

        write_xml_tag(&mut xml, "title", &self.title)?;
        write_xml_tag(&mut xml, "season", &self.season.to_string())?;
        write_xml_tag(&mut xml, "episode", &self.episode.to_string())?;

        if let Some(ref plot) = self.plot {
            write_xml_tag(&mut xml, "plot", plot)?;
        }

        if let Some(ref aired) = self.aired {
            write_xml_tag(&mut xml, "aired", aired)?;
        }

        if let Some(runtime) = self.runtime {
            write_xml_tag(&mut xml, "runtime", &runtime.to_string())?;
        }

        if let Some(ds) = self.displayseason {
            write_xml_tag(&mut xml, "displayseason", &ds.to_string())?;
        }

        if let Some(de) = self.displayepisode {
            write_xml_tag(&mut xml, "displayepisode", &de.to_string())?;
        }

        // 集标语（v22+）
        if let Some(ref tagline) = self.tagline {
            write_xml_tag(&mut xml, "tagline", tagline)?;
        }

        // 播放次数
        if let Some(playcount) = self.playcount {
            writeln!(xml, "  <playcount>{}</playcount>", playcount)
                .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败：{e}")))?;
        }

        // 最后播放日期
        if let Some(ref lastplayed) = self.lastplayed {
            write_xml_tag(&mut xml, "lastplayed", lastplayed)?;
        }

        // 唯一标识
        for uid in &self.uniqueid {
            writeln!(
                xml,
                r#"  <uniqueid type="{}" default="{}">{}</uniqueid>"#,
                xml_escape(&uid.id_type),
                uid.default,
                xml_escape(&uid.value)
            )
            .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
        }

        for credit in &self.credits {
            write_xml_tag(&mut xml, "credits", credit)?;
        }

        for dir in &self.director {
            write_xml_tag(&mut xml, "director", dir)?;
        }

        for a in &self.actor {
            writeln!(xml, "  <actor>")
                .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
            writeln!(xml, "    <name>{}</name>", xml_escape(&a.name))
                .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
            if let Some(ref role) = a.role {
                writeln!(xml, "    <role>{}</role>", xml_escape(role))
                    .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
            }
            writeln!(xml, "  </actor>")
                .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
        }

        writeln!(xml, "</episodedetails>")
            .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;

        Ok(xml)
    }
}

/// NFO 文件写入器
pub struct NfoWriter;

impl NfoWriter {
    /// 写入 tvshow.nfo 到指定目录
    pub fn write_tvshow(dir: &Path, nfo: &TvShowNfo) -> Result<()> {
        std::fs::create_dir_all(dir)
            .map_err(|e| AppError::NfoGenerationError(format!("创建目录失败: {e}")))?;
        let xml = nfo.to_xml()?;
        let path = dir.join("tvshow.nfo");
        std::fs::write(&path, xml.as_bytes())
            .map_err(|e| AppError::NfoGenerationError(format!("写入 tvshow.nfo 失败: {e}")))?;
        Ok(())
    }

    /// 写入 episode.nfo 到指定路径
    ///
    /// 文件名与视频文件同名但扩展名为 `.nfo`。
    pub fn write_episode(path: &Path, nfo: &EpisodeNfo) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::NfoGenerationError(format!("创建目录失败: {e}")))?;
        }
        let xml = nfo.to_xml()?;
        std::fs::write(path, xml.as_bytes())
            .map_err(|e| AppError::NfoGenerationError(format!("写入 episode.nfo 失败: {e}")))?;
        Ok(())
    }

    /// 保存图片文件
    pub fn write_image(path: &Path, data: &[u8]) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::NfoGenerationError(format!("创建目录失败: {e}")))?;
        }
        std::fs::write(path, data)
            .map_err(|e| AppError::NfoGenerationError(format!("保存图片失败: {e}")))?;
        Ok(())
    }
}

/// 从 AnimeMetadata 创建 TvShowNfo
impl From<&crate::metadata::AnimeMetadata> for TvShowNfo {
    fn from(meta: &crate::metadata::AnimeMetadata) -> Self {
        let mut uniqueid = vec![UniqueId {
            id_type: "bangumi".to_string(),
            default: true,
            value: meta.bangumi_id.to_string(),
        }];

        if let Some(tmdb_id) = meta.tmdb_id {
            uniqueid.push(UniqueId {
                id_type: "tmdb".to_string(),
                default: false,
                value: tmdb_id.to_string(),
            });
        }

        if let Some(anidb_id) = meta.anidb_id {
            uniqueid.push(UniqueId {
                id_type: "anidb".to_string(),
                default: false,
                value: anidb_id.to_string(),
            });
        }

        let rating = if meta.rating > 0.0 {
            Some(Rating {
                name: "bangumi".to_string(),
                max: 10,
                default: true,
                value: meta.rating,
                votes: None,
            })
        } else {
            None
        };

        Self {
            title: meta.title_cn.clone().unwrap_or_else(|| meta.title.clone()),
            originaltitle: meta.original_title.clone(),
            sorttitle: None,
            plot: meta.summary.clone(),
            genre: meta.genre.clone(),
            premiered: meta.air_date.clone(),
            status: None,
            studio: meta.studio.clone(),
            rating,
            userrating: None,
            top250: None,
            uniqueid,
            actor: Vec::new(),
            season: Some(1),
            episode: Some(meta.episode_count),
            tagline: None,
            mpaa: None,
            playcount: None,
            lastplayed: None,
            tag: Vec::new(),
        }
    }
}

/// 写入 XML 标签（带缩进和转义）
fn write_xml_tag(xml: &mut String, tag: &str, value: &str) -> Result<()> {
    writeln!(xml, "  <{tag}>{}</{tag}>", xml_escape(value))
        .map_err(|e| AppError::NfoGenerationError(format!("XML 写入失败: {e}")))?;
    Ok(())
}

/// XML 特殊字符转义
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::AnimeMetadata;

    fn sample_metadata() -> AnimeMetadata {
        let mut meta = AnimeMetadata::new(253395, "進撃の巨人".to_string());
        meta.title_cn = Some("进击的巨人".to_string());
        meta.original_title = "進撃の巨人".to_string();
        meta.summary = "那一天，人类终于回想起了...".to_string();
        meta.genre = vec!["动作".to_string(), "奇幻".to_string()];
        meta.studio = Some("WIT STUDIO".to_string());
        meta.director = Some("荒木哲郎".to_string());
        meta.episode_count = 25;
        meta.air_date = Some("2013-04-07".to_string());
        meta.rating = 8.5;
        meta.tmdb_id = Some(1429);
        meta
    }

    #[test]
    fn test_tvshow_nfo_to_xml() {
        let meta = sample_metadata();
        let nfo = TvShowNfo::from(&meta);
        let xml = nfo.to_xml().unwrap();

        assert!(xml.contains(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#));
        assert!(xml.contains("<tvshow>"));
        assert!(xml.contains("</tvshow>"));
        assert!(xml.contains("<title>进击的巨人</title>"));
        assert!(xml.contains("<originaltitle>進撃の巨人</originaltitle>"));
        assert!(xml.contains("<plot>那一天，人类终于回想起了...</plot>"));
        assert!(xml.contains("<genre>动作</genre>"));
        assert!(xml.contains("<genre>奇幻</genre>"));
        assert!(xml.contains("<premiered>2013-04-07</premiered>"));
        assert!(xml.contains("<studio>WIT STUDIO</studio>"));
        assert!(xml.contains(r#"<uniqueid type="bangumi" default="true">253395</uniqueid>"#));
        assert!(xml.contains(r#"<uniqueid type="tmdb" default="false">1429</uniqueid>"#));
        assert!(xml.contains("<value>8.5</value>"));
    }

    #[test]
    fn test_episode_nfo_to_xml() {
        let nfo = EpisodeNfo {
            title: "致两千年后的你".to_string(),
            season: 1,
            episode: 1,
            plot: Some("那一天，人类终于回想起了...".to_string()),
            aired: Some("2013-04-07".to_string()),
            runtime: Some(24),
            displayseason: Some(-1),
            displayepisode: Some(1),
            uniqueid: vec![UniqueId {
                id_type: "bangumi".to_string(),
                default: true,
                value: "12345".to_string(),
            }],
            credits: vec!["小林靖子".to_string()],
            director: vec!["荒木哲郎".to_string()],
            actor: Vec::new(),
            tagline: None,
            playcount: None,
            lastplayed: None,
        };

        let xml = nfo.to_xml().unwrap();

        assert!(xml.contains("<episodedetails>"));
        assert!(xml.contains("</episodedetails>"));
        assert!(xml.contains("<title>致两千年后的你</title>"));
        assert!(xml.contains("<season>1</season>"));
        assert!(xml.contains("<episode>1</episode>"));
        assert!(xml.contains("<aired>2013-04-07</aired>"));
        assert!(xml.contains("<runtime>24</runtime>"));
        assert!(xml.contains("<displayseason>-1</displayseason>"));
        assert!(xml.contains("<displayepisode>1</displayepisode>"));
        assert!(xml.contains("<credits>小林靖子</credits>"));
        assert!(xml.contains("<director>荒木哲郎</director>"));
    }

    #[test]
    fn test_episode_nfo_absolute_ordering() {
        let nfo = EpisodeNfo {
            title: "Episode 26".to_string(),
            season: 1,
            episode: 26,
            plot: None,
            aired: None,
            runtime: None,
            displayseason: Some(-1),
            displayepisode: Some(26),
            uniqueid: Vec::new(),
            credits: Vec::new(),
            director: Vec::new(),
            actor: Vec::new(),
            tagline: None,
            playcount: None,
            lastplayed: None,
        };

        let xml = nfo.to_xml().unwrap();
        assert!(xml.contains("<displayseason>-1</displayseason>"));
        assert!(xml.contains("<displayepisode>26</displayepisode>"));
    }

    #[test]
    fn test_xml_escape_special_chars() {
        let meta = AnimeMetadata::new(1, "Test & <Anime> \"Name\"".to_string());
        let nfo = TvShowNfo::from(&meta);
        let xml = nfo.to_xml().unwrap();

        assert!(xml.contains("Test &amp; &lt;Anime&gt; &quot;Name&quot;"));
    }

    #[test]
    fn test_nfo_writer_tvshow() {
        let dir = tempfile::tempdir().unwrap();
        let meta = sample_metadata();
        let nfo = TvShowNfo::from(&meta);

        NfoWriter::write_tvshow(dir.path(), &nfo).unwrap();

        let nfo_path = dir.path().join("tvshow.nfo");
        assert!(nfo_path.exists());

        let content = std::fs::read_to_string(&nfo_path).unwrap();
        assert!(content.contains("<tvshow>"));
        assert!(content.contains("进击的巨人"));
    }

    #[test]
    fn test_nfo_writer_episode() {
        let dir = tempfile::tempdir().unwrap();
        let nfo = EpisodeNfo {
            title: "Test Episode".to_string(),
            season: 1,
            episode: 1,
            plot: None,
            aired: None,
            runtime: None,
            displayseason: None,
            displayepisode: None,
            uniqueid: Vec::new(),
            credits: Vec::new(),
            director: Vec::new(),
            actor: Vec::new(),
            tagline: None,
            playcount: None,
            lastplayed: None,
        };

        let path = dir.path().join("Season 1").join("01.nfo");
        NfoWriter::write_episode(&path, &nfo).unwrap();

        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("<episodedetails>"));
        assert!(content.contains("Test Episode"));
    }

    #[test]
    fn test_nfo_writer_image() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("poster.jpg");
        let data = b"fake image data";

        NfoWriter::write_image(&path, data).unwrap();
        assert!(path.exists());
        assert_eq!(std::fs::read(&path).unwrap(), data);
    }

    /// Kodi 兼容性验证测试
    ///
    /// 确保生成的 NFO 完全符合 Kodi Wiki 官方规范：
    /// - https://kodi.wiki/view/NFO_files/TV_shows
    /// - https://kodi.wiki/view/NFO_files/Episodes
    #[test]
    fn test_kodi_nfo_compliance() {
        let meta = sample_metadata();
        let tvshow_nfo = TvShowNfo::from(&meta);
        let xml = tvshow_nfo.to_xml().unwrap();

        // 验证必需字段 (Required tags per Kodi spec)
        assert!(
            xml.contains(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#),
            "Missing XML declaration with UTF-8 encoding"
        );
        assert!(xml.contains("<tvshow>"), "Missing <tvshow> root element");
        assert!(xml.contains("</tvshow>"), "Missing </tvshow> closing tag");

        // 标题是必需的
        assert!(
            xml.matches("<title>").count() >= 1,
            "Missing required <title> field"
        );

        // uniqueid 是必需的（至少一个）
        let uniqueid_count = xml.matches("<uniqueid type=").count();
        assert!(
            uniqueid_count >= 1,
            "Missing required <uniqueid> field (at least one required)"
        );

        // 验证 Kodi 支持的可选字段都存在且格式正确
        assert!(xml.contains("</tvshow>"), "NFO must end with </tvshow>");

        // 验证 XML 结构完整性（配对标签）
        assert!(
            xml.matches("<tvshow>").count() == xml.matches("</tvshow>").count(),
            "XML structure error: mismatched <tvshow> tags"
        );

        println!("Generated NFO:\n{}", xml);
    }

    /// Episode NFO Kodi 兼容性验证测试
    #[test]
    fn test_kodi_episode_nfo_compliance() {
        let nfo = EpisodeNfo {
            title: "Test Episode".to_string(),
            season: 1,
            episode: 1,
            plot: Some("Episode description".to_string()),
            aired: Some("2024-01-01".to_string()),
            runtime: Some(24),
            displayseason: None,
            displayepisode: None,
            uniqueid: vec![UniqueId {
                id_type: "bangumi".to_string(),
                default: true,
                value: "12345".to_string(),
            }],
            credits: Vec::new(),
            director: Vec::new(),
            actor: Vec::new(),
            tagline: None,
            playcount: None,
            lastplayed: None,
        };

        let xml = nfo.to_xml().unwrap();

        // 验证 Kodi Episode NFO 必需字段
        assert!(
            xml.contains(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#),
            "Missing XML declaration"
        );
        assert!(
            xml.contains("<episodedetails>"),
            "Missing <episodedetails> root element"
        );
        assert!(
            xml.contains("</episodedetails>"),
            "Missing </episodedetails> closing tag"
        );

        // title 是必需的
        assert!(
            xml.matches("<title>").count() >= 1,
            "Missing required <title> field"
        );

        println!("Generated Episode NFO:\n{}", xml);
    }
}
