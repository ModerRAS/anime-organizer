//! 集成测试
//!
//! 测试从别名查找到 NFO 生成的完整流程。

use anime_organizer::metadata::wiki::WikiParser;
use anime_organizer::metadata::AnimeMetadata;
use anime_organizer::nfo::{NfoWriter, TvShowNfo};

#[cfg(feature = "metadata")]
use anime_organizer::metadata::AliasLookup;

#[cfg(feature = "metadata")]
fn create_test_db() -> tempfile::TempDir {
    use rusqlite::Connection;

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let conn = Connection::open(&db_path).unwrap();

    conn.execute_batch(
        r#"
        CREATE TABLE subjects (
            id INTEGER PRIMARY KEY,
            type INTEGER NOT NULL,
            name TEXT NOT NULL,
            name_cn TEXT
        );
        CREATE TABLE aliases (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            subject_id INTEGER REFERENCES subjects(id) ON DELETE CASCADE,
            alias TEXT NOT NULL,
            UNIQUE(subject_id, alias)
        );
        "#,
    )
    .unwrap();

    conn.execute(
        "INSERT INTO subjects (id, type, name, name_cn) VALUES (?1, 2, ?2, ?3)",
        rusqlite::params![1, "進撃の巨人", "进击的巨人"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO aliases (subject_id, alias) VALUES (1, '进击的巨人')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO subjects (id, type, name, name_cn) VALUES (?1, 2, ?2, ?3)",
        rusqlite::params![2, "鬼滅の刃", "鬼灭之刃"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO aliases (subject_id, alias) VALUES (2, '鬼灭之刃')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO subjects (id, type, name, name_cn) VALUES (?1, 2, ?2, ?3)",
        rusqlite::params![3, "スパイファミリー", "间谍过家家"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO aliases (subject_id, alias) VALUES (3, '间谍过家家')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO subjects (id, type, name, name_cn) VALUES (?1, 2, ?2, ?3)",
        rusqlite::params![4, "呪術廻戦", "咒术回战"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO aliases (subject_id, alias) VALUES (4, '咒术回战')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO subjects (id, type, name, name_cn) VALUES (?1, 2, ?2, ?3)",
        rusqlite::params![5, "チェーンソーマン", "电锯人"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO aliases (subject_id, alias) VALUES (5, '电锯人')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO subjects (id, type, name, name_cn) VALUES (?1, 2, ?2, ?3)",
        rusqlite::params![6, "ぼっち・ざ・ろっく！", "孤独摇滚"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO aliases (subject_id, alias) VALUES (6, '孤独摇滚')",
        [],
    )
    .unwrap();

    dir
}

/// 测试完整流程：别名查找 → 模拟元数据 → NFO 生成
#[test]
fn test_alias_to_nfo_flow() {
    let dir = create_test_db();
    let db_path = dir.path().join("test.db");

    // 1. 加载别名库
    let lookup = AliasLookup::load(&db_path).unwrap();
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
    let dir = create_test_db();
    let db_path = dir.path().join("test.db");
    let lookup = AliasLookup::load(&db_path).unwrap();

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
        tagline: None,
        playcount: None,
        lastplayed: None,
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

/// 验证别名库数据质量：所有条目结构正确
#[test]
fn test_alias_library_data_quality() {
    let dir = create_test_db();
    let db_path = dir.path().join("test.db");
    let lookup = AliasLookup::load(&db_path).unwrap();
    assert!(
        lookup.len() >= 6,
        "测试数据库应有 >= 6 个别名，实际: {}",
        lookup.len()
    );

    // 验证每个条目结构合法
    for (key, entry) in lookup.entries() {
        assert!(!key.is_empty(), "别名 key 不应为空");
        assert!(entry.bangumi_id > 0, "bangumi_id 应为正数: {:?}", entry);
        assert!(!entry.name.is_empty(), "name 不应为空: {:?}", entry);
    }
}

/// 测试完整管道：文件名解析 → 别名查找 → 元数据构造 → NFO 生成 → 文件写入
#[test]
fn test_full_pipeline_parse_to_nfo_files() {
    use anime_organizer::FilenameParser;

    let db_dir = create_test_db();
    let db_path = db_dir.path().join("test.db");
    let lookup = AliasLookup::load(&db_path).unwrap();
    let work_dir = tempfile::tempdir().unwrap();

    let filename = "[ANi] Bocci the Rock - 05 [1080P].mkv";
    let path = std::path::PathBuf::from(filename);
    let info = FilenameParser::parse(&path).expect("应能解析文件名");

    // 用解析出的动画名查找别名
    let entry = lookup
        .find(&info.anime_name)
        .or_else(|| lookup.find_fuzzy(&info.anime_name));

    // 无论是否匹配到，都测试 NFO 生成管道
    let meta = if let Some(e) = entry {
        let mut m = AnimeMetadata::new(e.bangumi_id, e.name.clone());
        m.title_cn = Some(info.anime_name.clone());
        m.tmdb_id = e.tmdb_id;
        m
    } else {
        AnimeMetadata::new(0, info.anime_name.clone())
    };

    // 生成 tvshow.nfo
    let nfo = TvShowNfo::from(&meta);
    let anime_dir = work_dir.path().join(&info.anime_name);
    std::fs::create_dir_all(&anime_dir).unwrap();
    NfoWriter::write_tvshow(&anime_dir, &nfo).unwrap();

    let tvshow_nfo = anime_dir.join("tvshow.nfo");
    assert!(tvshow_nfo.exists(), "tvshow.nfo 应已创建");
    let content = std::fs::read_to_string(&tvshow_nfo).unwrap();
    assert!(content.contains("<tvshow>"));
    assert!(content.contains(&xml_escape(&info.anime_name)));

    // 生成 episode.nfo
    let ep = anime_organizer::nfo::EpisodeNfo {
        title: format!("Episode {}", info.episode),
        season: 1,
        episode: info.episode.parse::<u32>().unwrap_or(1),
        plot: None,
        aired: None,
        runtime: Some(24),
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

    let season_dir = anime_dir.join("Season 1");
    std::fs::create_dir_all(&season_dir).unwrap();
    let ep_nfo_path = season_dir.join(format!("{}.nfo", info.episode));
    NfoWriter::write_episode(&ep_nfo_path, &ep).unwrap();
    assert!(ep_nfo_path.exists(), "episode.nfo 应已创建");

    let ep_content = std::fs::read_to_string(&ep_nfo_path).unwrap();
    assert!(ep_content.contains("<episodedetails>"));
    assert!(ep_content.contains(&format!("<episode>{}</episode>", ep.episode)));

    // 写入模拟图片
    let poster_path = anime_dir.join("poster.jpg");
    NfoWriter::write_image(&poster_path, b"fake-poster-bytes").unwrap();
    assert!(poster_path.exists());

    let fanart_path = anime_dir.join("fanart.jpg");
    NfoWriter::write_image(&fanart_path, b"fake-fanart-bytes").unwrap();
    assert!(fanart_path.exists());
}

/// XML 转义辅助（与 nfo 模块中的逻辑一致）
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
