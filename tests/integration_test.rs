//! 集成测试
//!
//! 测试从别名查找到 NFO 生成的完整流程。

use anime_organizer::metadata::wiki::WikiParser;
use anime_organizer::metadata::AnimeMetadata;
use anime_organizer::nfo::{NfoWriter, TvShowNfo};

#[cfg(feature = "metadata")]
use anime_organizer::metadata::AliasLookup;

/// 测试完整流程：别名查找 → 模拟元数据 → NFO 生成
#[test]
fn test_alias_to_nfo_flow() {
    // 1. 加载别名库
    let lookup = AliasLookup::load(None).unwrap();
    assert!(!lookup.is_empty(), "别名库不应为空");

    // 2. 查找已知动画
    let entry = lookup.find("进击的巨人");
    assert!(entry.is_some(), "应能找到'进击的巨人'");
    let entry = entry.unwrap();
    assert!(entry.bangumi_id > 0);

    // 3. 模拟从 Bangumi 获取元数据
    let meta = AnimeMetadata {
        bangumi_id: entry.bangumi_id,
        title: entry.name.clone(),
        title_cn: Some("进击的巨人".to_string()),
        original_title: entry.name.clone(),
        summary: "人类与巨人的战斗".to_string(),
        genre: vec!["动作".to_string(), "奇幻".to_string()],
        studio: Some("WIT STUDIO".to_string()),
        director: Some("荒木哲郎".to_string()),
        episode_count: 25,
        air_date: Some("2013-04-07".to_string()),
        rating: 8.5,
        tmdb_id: entry.tmdb_id,
        anidb_id: entry.anidb_id,
    };

    // 4. 生成 TvShowNfo
    let nfo = TvShowNfo::from(&meta);
    // title_cn 存在时优先使用中文标题
    assert_eq!(nfo.title, "进击的巨人");
    assert_eq!(nfo.originaltitle, meta.original_title);

    // 5. 序列化为 XML
    let xml = nfo.to_xml().unwrap();
    assert!(xml.contains("<tvshow>"));
    assert!(xml.contains("</tvshow>"));
    assert!(xml.contains("<title>"));
    assert!(xml.contains("<plot>"));
    assert!(xml.contains("bangumi"));

    // 6. 写入临时文件
    let dir = tempfile::tempdir().unwrap();
    NfoWriter::write_tvshow(dir.path(), &nfo).unwrap();

    let nfo_path = dir.path().join("tvshow.nfo");
    assert!(nfo_path.exists(), "tvshow.nfo 应已创建");

    let content = std::fs::read_to_string(&nfo_path).unwrap();
    assert!(content.contains("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"));
    assert!(content.contains("<tvshow>"));
}

/// 测试 WikiParser 完整解析流程
#[test]
fn test_wiki_parse_to_metadata_conversion() {
    let infobox = r#"{{Infobox animanga/TVAnime
|中文名= 孤独摇滚！
|别名={
[ぼっち・ざ・ろっく！]
[Bocchi the Rock!]
}
|话数= 12
|放送开始= 2022年10月8日
|放送结束= 2022年12月24日
|动画制作= CloverWorks
|导演= 斎藤圭一郎
|系列构成= 吉田恵里香
|音乐= 菊谷知樹
}}"#;

    let parser = WikiParser::new();
    let info = parser.parse_anime_infobox(infobox).unwrap();

    // 从 Infobox 转换为 AnimeMetadata
    let meta = AnimeMetadata {
        bangumi_id: 378862,
        title: "ぼっち・ざ・ろっく！".to_string(),
        title_cn: info.name_cn.clone(),
        original_title: "ぼっち・ざ・ろっく！".to_string(),
        summary: String::new(),
        genre: Vec::new(),
        studio: info.studio.clone(),
        director: info.director.clone(),
        episode_count: info.episode_count.unwrap_or(0),
        air_date: info.air_date_start.clone(),
        rating: 0.0,
        tmdb_id: None,
        anidb_id: None,
    };

    assert_eq!(meta.title_cn.as_deref(), Some("孤独摇滚！"));
    assert_eq!(meta.studio.as_deref(), Some("CloverWorks"));
    assert_eq!(meta.director.as_deref(), Some("斎藤圭一郎"));
    assert_eq!(meta.episode_count, 12);

    // 生成 NFO 并验证
    let nfo = TvShowNfo::from(&meta);
    let xml = nfo.to_xml().unwrap();
    assert!(xml.contains("CloverWorks"));
    assert!(xml.contains("ぼっち・ざ・ろっく！"));
}

/// 测试别名模糊匹配 + NFO 生成
#[test]
fn test_fuzzy_alias_to_nfo() {
    let lookup = AliasLookup::load(None).unwrap();

    // 测试多个已知别名
    let test_cases = vec!["鬼灭之刃", "间谍过家家", "咒术回战", "电锯人"];

    for name in &test_cases {
        if let Some(entry) = lookup.find(name).or_else(|| lookup.find_fuzzy(name)) {
            let meta = AnimeMetadata::new(entry.bangumi_id, entry.name.clone());
            let nfo = TvShowNfo::from(&meta);
            let xml = nfo.to_xml().unwrap();
            assert!(
                xml.contains("<tvshow>"),
                "NFO for '{}' should be valid XML",
                name
            );
        }
    }
}

/// 测试 EpisodeNfo 生成正确结构
#[test]
fn test_episode_nfo_generation() {
    use anime_organizer::nfo::{EpisodeNfo, UniqueId};

    let ep = EpisodeNfo {
        title: "Episode 01".to_string(),
        season: 1,
        episode: 1,
        plot: Some("第一集剧情".to_string()),
        aired: Some("2022-10-08".to_string()),
        runtime: Some(24),
        displayseason: None,
        displayepisode: None,
        uniqueid: vec![UniqueId {
            id_type: "bangumi".to_string(),
            default: true,
            value: "378862".to_string(),
        }],
        credits: vec!["吉田恵里香".to_string()],
        director: vec!["斎藤圭一郎".to_string()],
        actor: Vec::new(),
    };

    let xml = ep.to_xml().unwrap();
    assert!(xml.contains("<episodedetails>"));
    assert!(xml.contains("<title>Episode 01</title>"));
    assert!(xml.contains("<season>1</season>"));
    assert!(xml.contains("<episode>1</episode>"));
    assert!(xml.contains("<aired>2022-10-08</aired>"));
    assert!(xml.contains("<runtime>24</runtime>"));
    assert!(xml.contains("<credits>吉田恵里香</credits>"));
    assert!(xml.contains("<director>斎藤圭一郎</director>"));

    // 写入临时文件
    let dir = tempfile::tempdir().unwrap();
    let ep_path = dir.path().join("01.nfo");
    NfoWriter::write_episode(&ep_path, &ep).unwrap();
    assert!(ep_path.exists());
}

/// 测试文件名解析 → 整理目标路径
#[test]
fn test_parser_to_organizer_path() {
    use anime_organizer::FilenameParser;

    let test_files = [
        "[SubsPlease] Bocchi the Rock! - 01 (1080p) [ABC123].mkv",
        "[Ember] Spy x Family - 01.mkv",
        "[ANi] Chainsaw Man - 01 [1080P].mp4",
    ];

    for filename in &test_files {
        let path = std::path::PathBuf::from(filename);
        if let Some(info) = FilenameParser::parse(&path) {
            assert!(
                !info.anime_name.is_empty(),
                "anime_name should not be empty for {}",
                filename
            );
            assert!(
                !info.episode.is_empty(),
                "episode should not be empty for {}",
                filename
            );
            let target = info.target_filename();
            assert!(
                !target.is_empty(),
                "target_filename should not be empty for {}",
                filename
            );
        }
    }
}
