//! Generated parser tests - 北宇治字幕组
use anime_organizer::parser::FilenameParser;
use std::path::PathBuf;

#[test]
fn test_parse_kitauji_euphonium_01() {
    let path = PathBuf::from("[KitaujiSub] Hibike! Euphonium - 01 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Hibike! Euphonium");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_kitauji_euphonium_02() {
    let path = PathBuf::from("[KitaujiSub] Hibike! Euphonium - 02 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Hibike! Euphonium");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_kitauji_euphonium_03() {
    let path = PathBuf::from("[KitaujiSub] Hibike! Euphonium - 03 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Hibike! Euphonium");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_kitauji_tamako_01() {
    let path = PathBuf::from("[KitaujiSub] Tamako Market - 01 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Tamako Market");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_kitauji_tamako_02() {
    let path = PathBuf::from("[KitaujiSub] Tamako Market - 02 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Tamako Market");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_kitauji_tamako_03() {
    let path = PathBuf::from("[KitaujiSub] Tamako Market - 03 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Tamako Market");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_kitauji_liz_01() {
    let path = PathBuf::from("[KitaujiSub] Liz to Aoi Tori - 01 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Liz to Aoi Tori");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_kitauji_liz_02() {
    let path = PathBuf::from("[KitaujiSub] Liz to Aoi Tori - 02 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Liz to Aoi Tori");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_kitauji_liz_03() {
    let path = PathBuf::from("[KitaujiSub] Liz to Aoi Tori - 03 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Liz to Aoi Tori");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_kitauji_kumodefu_01() {
    let path = PathBuf::from("[KitaujiSub] Kuma Kuma Kuma - 01 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Kuma Kuma Kuma");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_kitauji_kumodefu_02() {
    let path = PathBuf::from("[KitaujiSub] Kuma Kuma Kuma - 02 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Kuma Kuma Kuma");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_kitauji_kumodefu_03() {
    let path = PathBuf::from("[KitaujiSub] Kuma Kuma Kuma - 03 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Kuma Kuma Kuma");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_kitauji_love_live_01() {
    let path = PathBuf::from("[KitaujiSub] Love Live! Sunshine!! - 01 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Love Live! Sunshine!!");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_kitauji_love_live_02() {
    let path = PathBuf::from("[KitaujiSub] Love Live! Sunshine!! - 02 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Love Live! Sunshine!!");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_kitauji_love_live_03() {
    let path = PathBuf::from("[KitaujiSub] Love Live! Sunshine!! - 03 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Love Live! Sunshine!!");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_kitauji_frieren_01() {
    let path = PathBuf::from("[北宇治字幕组] Sousou no Frieren - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.anime_name, "Sousou no Frieren");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_kitauji_frieren_02() {
    let path = PathBuf::from("[北宇治字幕组] Sousou no Frieren - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.anime_name, "Sousou no Frieren");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_kitauji_frieren_03() {
    let path = PathBuf::from("[北宇治字幕组] Sousou no Frieren - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.anime_name, "Sousou no Frieren");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_kitauji_bangdream_01() {
    let path = PathBuf::from("[KitaujiSub] BanG Dream! - 01 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "BanG Dream!");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_kitauji_bangdream_02() {
    let path = PathBuf::from("[KitaujiSub] BanG Dream! - 02 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "BanG Dream!");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_kitauji_bangdream_03() {
    let path = PathBuf::from("[KitaujiSub] BanG Dream! - 03 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "BanG Dream!");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_kitauji_angel_01() {
    let path = PathBuf::from("[KitaujiSub] Angel Beats - 01 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Angel Beats");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_kitauji_angel_02() {
    let path = PathBuf::from("[KitaujiSub] Angel Beats - 02 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Angel Beats");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_kitauji_angel_03() {
    let path = PathBuf::from("[KitaujiSub] Angel Beats - 03 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Angel Beats");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_kitauji_clannad_01() {
    let path = PathBuf::from("[KitaujiSub] Clannad - 01 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Clannad");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_kitauji_clannad_02() {
    let path = PathBuf::from("[KitaujiSub] Clannad - 02 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Clannad");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_kitauji_clannad_03() {
    let path = PathBuf::from("[KitaujiSub] Clannad - 03 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Clannad");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_kitauji_sakura_01() {
    let path = PathBuf::from("[KitaujiSub] Sakura Trick - 01 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Sakura Trick");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_kitauji_sakura_02() {
    let path = PathBuf::from("[KitaujiSub] Sakura Trick - 02 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Sakura Trick");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_kitauji_sakura_03() {
    let path = PathBuf::from("[KitaujiSub] Sakura Trick - 03 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Sakura Trick");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_kitauji_kobayashi_01() {
    let path = PathBuf::from("[KitaujiSub] Kobayashi-san Chi no Maid Dragon - 01 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Kobayashi-san Chi no Maid Dragon");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_kitauji_kobayashi_02() {
    let path = PathBuf::from("[KitaujiSub] Kobayashi-san Chi no Maid Dragon - 02 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Kobayashi-san Chi no Maid Dragon");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_kitauji_kobayashi_03() {
    let path = PathBuf::from("[KitaujiSub] Kobayashi-san Chi no Maid Dragon - 03 [720p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "KitaujiSub");
    assert_eq!(i.anime_name, "Kobayashi-san Chi no Maid Dragon");
    assert_eq!(i.episode, "03");
}
