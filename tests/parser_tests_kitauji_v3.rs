//! Tests for 北宇治字幕组 publisher format (v3)
//!
//! Format: `[北宇治字幕组] Anime Name - 01 [1080P].mkv`

use anime_organizer::parser::FilenameParser;
use std::path::PathBuf;

#[test]
fn test_kitauji_v3_anime_01() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 01 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.anime_name, "Anime Name");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_kitauji_v3_anime_02() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 02 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_kitauji_v3_anime_03() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 03 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_kitauji_v3_anime_04() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 04 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "04");
}

#[test]
fn test_kitauji_v3_anime_05() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 05 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "05");
}

#[test]
fn test_kitauji_v3_anime_06() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 06 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "06");
}

#[test]
fn test_kitauji_v3_anime_07() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 07 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "07");
}

#[test]
fn test_kitauji_v3_anime_08() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 08 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "08");
}

#[test]
fn test_kitauji_v3_anime_09() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 09 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "09");
}

#[test]
fn test_kitauji_v3_anime_10() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 10 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "10");
}

#[test]
fn test_kitauji_v3_anime_11() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 11 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "11");
}

#[test]
fn test_kitauji_v3_anime_12() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 12 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "12");
}

#[test]
fn test_kitauji_v3_anime_13() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 13 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "13");
}

#[test]
fn test_kitauji_v3_anime_14() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 14 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "14");
}

#[test]
fn test_kitauji_v3_anime_15() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 15 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "15");
}

#[test]
fn test_kitauji_v3_anime_16() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 16 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "16");
}

#[test]
fn test_kitauji_v3_anime_17() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 17 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "17");
}

#[test]
fn test_kitauji_v3_anime_18() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 18 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "18");
}

#[test]
fn test_kitauji_v3_anime_19() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 19 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "19");
}

#[test]
fn test_kitauji_v3_anime_20() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 20 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "20");
}

#[test]
fn test_kitauji_v3_anime_21() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 21 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "21");
}

#[test]
fn test_kitauji_v3_anime_22() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 22 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "22");
}

#[test]
fn test_kitauji_v3_anime_23() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 23 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "23");
}

#[test]
fn test_kitauji_v3_anime_24() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 24 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "24");
}

#[test]
fn test_kitauji_v3_anime_25() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 25 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "25");
}

#[test]
fn test_kitauji_v3_anime_26() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 26 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "26");
}

#[test]
fn test_kitauji_v3_anime_27() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 27 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "27");
}

#[test]
fn test_kitauji_v3_anime_28() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 28 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "28");
}

#[test]
fn test_kitauji_v3_anime_29() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 29 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "29");
}

#[test]
fn test_kitauji_v3_anime_30() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 30 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "30");
}

#[test]
fn test_kitauji_v3_anime_31() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 31 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "31");
}

#[test]
fn test_kitauji_v3_anime_32() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 32 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "32");
}

#[test]
fn test_kitauji_v3_anime_33() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 33 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "33");
}

#[test]
fn test_kitauji_v3_anime_34() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 34 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "34");
}

#[test]
fn test_kitauji_v3_anime_35() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 35 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "35");
}

#[test]
fn test_kitauji_v3_anime_36() {
    let path = PathBuf::from("[北宇治字幕组] Anime Name - 36 [1080P].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "北宇治字幕组");
    assert_eq!(i.episode, "36");
}
