//! NFO 文件生成模块
//!
//! 生成 Kodi 兼容的 NFO 文件（tvshow.nfo 和 episode.nfo）。

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;

/// TV Show NFO 数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvShowNfo {
    /// 标题
    pub title: String,
    /// 原始标题
    pub originaltitle: String,
    /// 排序标题
    pub sorttitle: Option<String>,
    /// 剧情简介
    pub plot: String,
    /// 类型标签
    pub genre: Vec<String>,
    /// 首播日期
    pub premiered: Option<String>,
    /// 状态
    pub status: Option<String>,
    /// 制作公司
    pub studio: Option<String>,
    /// 评分
    pub rating: Option<f32>,
    /// 总集数
    pub episode_count: Option<u32>,
    /// 季数
    pub season_count: Option<u32>,
    /// Bangumi ID
    pub bangumi_id: Option<u32>,
    /// TMDB ID
    pub tmdb_id: Option<u32>,
    /// 演员列表
    pub actors: Vec<Actor>,
}

/// 演员信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    /// 演员名称
    pub name: String,
    /// 角色名称
    pub role: Option<String>,
}

/// 剧集 NFO 数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeNfo {
    /// 剧集标题
    pub title: String,
    /// 季数
    pub season: u32,
    /// 集数
    pub episode: u32,
    /// 剧情简介
    pub plot: Option<String>,
    /// 播出日期
    pub aired: Option<String>,
    /// 时长（分钟）
    pub runtime: Option<u32>,
    /// 显示季数（用于绝对编号）
    pub displayseason: Option<u32>,
    /// 显示集数（用于绝对编号）
    pub displayepisode: Option<u32>,
    /// Bangumi episode ID
    pub bangumi_id: Option<u32>,
    /// 导演
    pub director: Option<String>,
    /// 编剧
    pub credits: Option<String>,
}

/// NFO 文件写入器
pub struct NfoWriter;

impl TvShowNfo {
    /// 序列化为 Kodi 兼容的 XML 字符串
    pub fn to_xml(&self) -> Result<String> {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        xml.push_str("<tvshow>\n");

        xml.push_str(&format!("  <title>{}</title>\n", xml_escape(&self.title)));
        xml.push_str(&format!(
            "  <originaltitle>{}</originaltitle>\n",
            xml_escape(&self.originaltitle)
        ));

        if let Some(ref sorttitle) = self.sorttitle {
            xml.push_str(&format!(
                "  <sorttitle>{}</sorttitle>\n",
                xml_escape(sorttitle)
            ));
        }

        xml.push_str(&format!("  <plot>{}</plot>\n", xml_escape(&self.plot)));

        for g in &self.genre {
            xml.push_str(&format!("  <genre>{}</genre>\n", xml_escape(g)));
        }

        if let Some(ref premiered) = self.premiered {
            xml.push_str(&format!(
                "  <premiered>{}</premiered>\n",
                xml_escape(premiered)
            ));
        }

        if let Some(ref status) = self.status {
            xml.push_str(&format!("  <status>{}</status>\n", xml_escape(status)));
        }

        if let Some(ref studio) = self.studio {
            xml.push_str(&format!("  <studio>{}</studio>\n", xml_escape(studio)));
        }

        if let Some(rating) = self.rating {
            xml.push_str("  <ratings>\n");
            xml.push_str("    <rating name=\"bangumi\" max=\"10\" default=\"true\">\n");
            xml.push_str(&format!("      <value>{:.1}</value>\n", rating));
            xml.push_str("    </rating>\n");
            xml.push_str("  </ratings>\n");
        }

        if let Some(count) = self.episode_count {
            xml.push_str(&format!("  <episode>{count}</episode>\n"));
        }

        if let Some(count) = self.season_count {
            xml.push_str(&format!("  <season>{count}</season>\n"));
        }

        if let Some(id) = self.bangumi_id {
            xml.push_str(&format!(
                "  <uniqueid type=\"bangumi\" default=\"true\">{id}</uniqueid>\n"
            ));
        }

        if let Some(id) = self.tmdb_id {
            xml.push_str(&format!("  <uniqueid type=\"tmdb\">{id}</uniqueid>\n"));
        }

        for actor in &self.actors {
            xml.push_str("  <actor>\n");
            xml.push_str(&format!("    <name>{}</name>\n", xml_escape(&actor.name)));
            if let Some(ref role) = actor.role {
                xml.push_str(&format!("    <role>{}</role>\n", xml_escape(role)));
            }
            xml.push_str("  </actor>\n");
        }

        xml.push_str("</tvshow>\n");
        Ok(xml)
    }
}

impl EpisodeNfo {
    /// 序列化为 Kodi 兼容的 XML 字符串
    pub fn to_xml(&self) -> Result<String> {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        xml.push_str("<episodedetails>\n");

        xml.push_str(&format!("  <title>{}</title>\n", xml_escape(&self.title)));
        xml.push_str(&format!("  <season>{}</season>\n", self.season));
        xml.push_str(&format!("  <episode>{}</episode>\n", self.episode));

        if let Some(ref plot) = self.plot {
            xml.push_str(&format!("  <plot>{}</plot>\n", xml_escape(plot)));
        }

        if let Some(ref aired) = self.aired {
            xml.push_str(&format!("  <aired>{}</aired>\n", xml_escape(aired)));
        }

        if let Some(runtime) = self.runtime {
            xml.push_str(&format!("  <runtime>{runtime}</runtime>\n"));
        }

        if let Some(ds) = self.displayseason {
            xml.push_str(&format!("  <displayseason>{ds}</displayseason>\n"));
        }

        if let Some(de) = self.displayepisode {
            xml.push_str(&format!("  <displayepisode>{de}</displayepisode>\n"));
        }

        if let Some(id) = self.bangumi_id {
            xml.push_str(&format!(
                "  <uniqueid type=\"bangumi\">{id}</uniqueid>\n"
            ));
        }

        if let Some(ref director) = self.director {
            xml.push_str(&format!(
                "  <director>{}</director>\n",
                xml_escape(director)
            ));
        }

        if let Some(ref credits) = self.credits {
            xml.push_str(&format!("  <credits>{}</credits>\n", xml_escape(credits)));
        }

        xml.push_str("</episodedetails>\n");
        Ok(xml)
    }
}

impl NfoWriter {
    /// 写入 tvshow.nfo 到指定目录
    ///
    /// # 参数
    ///
    /// * `dir` - 动漫根目录
    /// * `nfo` - TvShow NFO 数据
    /// * `overwrite` - 是否覆盖已有文件
    pub fn write_tvshow<P: AsRef<Path>>(
        dir: P,
        nfo: &TvShowNfo,
        overwrite: bool,
    ) -> Result<()> {
        let path = dir.as_ref().join("tvshow.nfo");

        if path.exists() && !overwrite {
            return Ok(());
        }

        std::fs::create_dir_all(dir.as_ref())?;

        let xml = nfo.to_xml()?;
        let mut file = std::fs::File::create(&path).map_err(|e| {
            AppError::NfoGenerationError(format!("创建 tvshow.nfo 失败: {e}"))
        })?;
        file.write_all(xml.as_bytes()).map_err(|e| {
            AppError::NfoGenerationError(format!("写入 tvshow.nfo 失败: {e}"))
        })?;

        Ok(())
    }

    /// 写入 episode.nfo 到指定路径
    ///
    /// # 参数
    ///
    /// * `path` - episode.nfo 文件路径
    /// * `nfo` - Episode NFO 数据
    /// * `overwrite` - 是否覆盖已有文件
    pub fn write_episode<P: AsRef<Path>>(
        path: P,
        nfo: &EpisodeNfo,
        overwrite: bool,
    ) -> Result<()> {
        let path = path.as_ref();

        if path.exists() && !overwrite {
            return Ok(());
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let xml = nfo.to_xml()?;
        let mut file = std::fs::File::create(path).map_err(|e| {
            AppError::NfoGenerationError(format!("创建 episode.nfo 失败: {e}"))
        })?;
        file.write_all(xml.as_bytes()).map_err(|e| {
            AppError::NfoGenerationError(format!("写入 episode.nfo 失败: {e}"))
        })?;

        Ok(())
    }
}

/// XML 转义特殊字符
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

    fn sample_tvshow_nfo() -> TvShowNfo {
        TvShowNfo {
            title: "進撃の巨人".to_string(),
            originaltitle: "進撃の巨人".to_string(),
            sorttitle: Some("Attack on Titan".to_string()),
            plot: "进击的巨人的故事简介".to_string(),
            genre: vec!["动作".to_string(), "奇幻".to_string()],
            premiered: Some("2013-04-07".to_string()),
            status: Some("Ended".to_string()),
            studio: Some("WIT STUDIO".to_string()),
            rating: Some(8.5),
            episode_count: Some(25),
            season_count: Some(1),
            bangumi_id: Some(265),
            tmdb_id: Some(1429),
            actors: vec![Actor {
                name: "梶裕貴".to_string(),
                role: Some("エレン・イェーガー".to_string()),
            }],
        }
    }

    fn sample_episode_nfo() -> EpisodeNfo {
        EpisodeNfo {
            title: "二千年後の君へ".to_string(),
            season: 1,
            episode: 1,
            plot: Some("第一集的剧情简介".to_string()),
            aired: Some("2013-04-07".to_string()),
            runtime: Some(24),
            displayseason: None,
            displayepisode: None,
            bangumi_id: Some(12345),
            director: Some("荒木哲郎".to_string()),
            credits: None,
        }
    }

    #[test]
    fn test_tvshow_nfo_to_xml() {
        let nfo = sample_tvshow_nfo();
        let xml = nfo.to_xml().unwrap();

        assert!(xml.contains("<?xml version=\"1.0\""));
        assert!(xml.contains("<tvshow>"));
        assert!(xml.contains("<title>進撃の巨人</title>"));
        assert!(xml.contains("<originaltitle>進撃の巨人</originaltitle>"));
        assert!(xml.contains("<sorttitle>Attack on Titan</sorttitle>"));
        assert!(xml.contains("<genre>动作</genre>"));
        assert!(xml.contains("<genre>奇幻</genre>"));
        assert!(xml.contains("<premiered>2013-04-07</premiered>"));
        assert!(xml.contains("<studio>WIT STUDIO</studio>"));
        assert!(xml.contains("<value>8.5</value>"));
        assert!(xml.contains("<uniqueid type=\"bangumi\" default=\"true\">265</uniqueid>"));
        assert!(xml.contains("<uniqueid type=\"tmdb\">1429</uniqueid>"));
        assert!(xml.contains("<name>梶裕貴</name>"));
        assert!(xml.contains("<role>エレン・イェーガー</role>"));
        assert!(xml.contains("</tvshow>"));
    }

    #[test]
    fn test_episode_nfo_to_xml() {
        let nfo = sample_episode_nfo();
        let xml = nfo.to_xml().unwrap();

        assert!(xml.contains("<?xml version=\"1.0\""));
        assert!(xml.contains("<episodedetails>"));
        assert!(xml.contains("<title>二千年後の君へ</title>"));
        assert!(xml.contains("<season>1</season>"));
        assert!(xml.contains("<episode>1</episode>"));
        assert!(xml.contains("<aired>2013-04-07</aired>"));
        assert!(xml.contains("<runtime>24</runtime>"));
        assert!(xml.contains("<uniqueid type=\"bangumi\">12345</uniqueid>"));
        assert!(xml.contains("<director>荒木哲郎</director>"));
        assert!(xml.contains("</episodedetails>"));
    }

    #[test]
    fn test_episode_nfo_with_absolute_ordering() {
        let nfo = EpisodeNfo {
            title: "第1話".to_string(),
            season: 1,
            episode: 1,
            plot: None,
            aired: None,
            runtime: None,
            displayseason: Some(1),
            displayepisode: Some(26),
            bangumi_id: None,
            director: None,
            credits: None,
        };
        let xml = nfo.to_xml().unwrap();

        assert!(xml.contains("<displayseason>1</displayseason>"));
        assert!(xml.contains("<displayepisode>26</displayepisode>"));
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("A & B"), "A &amp; B");
        assert_eq!(xml_escape("<test>"), "&lt;test&gt;");
        assert_eq!(xml_escape("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_nfo_writer_tvshow() {
        let dir = tempfile::TempDir::new().unwrap();
        let nfo = sample_tvshow_nfo();

        NfoWriter::write_tvshow(dir.path(), &nfo, true).unwrap();

        let path = dir.path().join("tvshow.nfo");
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("<tvshow>"));
        assert!(content.contains("進撃の巨人"));
    }

    #[test]
    fn test_nfo_writer_episode() {
        let dir = tempfile::TempDir::new().unwrap();
        let nfo = sample_episode_nfo();
        let path = dir.path().join("episode.nfo");

        NfoWriter::write_episode(&path, &nfo, true).unwrap();

        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("<episodedetails>"));
    }

    #[test]
    fn test_nfo_writer_skip_existing() {
        let dir = tempfile::TempDir::new().unwrap();
        let nfo = sample_tvshow_nfo();

        // 写入第一次
        NfoWriter::write_tvshow(dir.path(), &nfo, true).unwrap();

        // 修改标题
        let nfo2 = TvShowNfo {
            title: "新标题".to_string(),
            ..nfo
        };

        // 不覆盖模式
        NfoWriter::write_tvshow(dir.path(), &nfo2, false).unwrap();

        let content = std::fs::read_to_string(dir.path().join("tvshow.nfo")).unwrap();
        assert!(content.contains("進撃の巨人")); // 旧标题应保留
        assert!(!content.contains("新标题"));
    }

    #[test]
    fn test_nfo_writer_overwrite_existing() {
        let dir = tempfile::TempDir::new().unwrap();
        let nfo = sample_tvshow_nfo();

        // 写入第一次
        NfoWriter::write_tvshow(dir.path(), &nfo, true).unwrap();

        // 修改标题
        let nfo2 = TvShowNfo {
            title: "新标题".to_string(),
            ..nfo
        };

        // 覆盖模式
        NfoWriter::write_tvshow(dir.path(), &nfo2, true).unwrap();

        let content = std::fs::read_to_string(dir.path().join("tvshow.nfo")).unwrap();
        assert!(content.contains("新标题"));
    }
}
