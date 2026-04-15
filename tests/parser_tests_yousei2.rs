use std::path::PathBuf;

use anime_organizer::FilenameParser;

#[test]
fn test_parse_yousei2_kimetsu_01() {
    let path = PathBuf::from("[Yousei-raws2] 鬼灭之刃 - 01 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "鬼灭之刃");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_yousei2_kimetsu_02() {
    let path = PathBuf::from("[Yousei-raws2] 鬼灭之刃 - 02 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "鬼灭之刃");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_yousei2_kimetsu_03() {
    let path = PathBuf::from("[Yousei-raws2] 鬼灭之刃 - 03 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "鬼灭之刃");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_yousei2_kimetsu_04() {
    let path = PathBuf::from("[Yousei-raws2] 鬼灭之刃 - 04 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "鬼灭之刃");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_yousei2_kimetsu_05() {
    let path = PathBuf::from("[Yousei-raws2] 鬼灭之刃 - 05 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "鬼灭之刃");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_yousei2_kimetsu_06() {
    let path = PathBuf::from("[Yousei-raws2] 鬼灭之刃 - 06 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "鬼灭之刃");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_yousei2_kimetsu_07() {
    let path = PathBuf::from("[Yousei-raws2] 鬼灭之刃 - 07 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "鬼灭之刃");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_yousei2_kimetsu_08() {
    let path = PathBuf::from("[Yousei-raws2] 鬼灭之刃 - 08 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "鬼灭之刃");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_yousei2_kimetsu_09() {
    let path = PathBuf::from("[Yousei-raws2] 鬼灭之刃 - 09 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "鬼灭之刃");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_yousei2_kimetsu_10() {
    let path = PathBuf::from("[Yousei-raws2] 鬼灭之刃 - 10 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "鬼灭之刃");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_yousei2_kimetsu_11() {
    let path = PathBuf::from("[Yousei-raws2] 鬼灭之刃 - 11 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "鬼灭之刃");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_yousei2_kimetsu_12() {
    let path = PathBuf::from("[Yousei-raws2] 鬼灭之刃 - 12 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "鬼灭之刃");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_yousei2_jujutsu_01() {
    let path = PathBuf::from("[Yousei-raws2] 咒术回战 - 01 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "咒术回战");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_yousei2_jujutsu_02() {
    let path = PathBuf::from("[Yousei-raws2] 咒术回战 - 02 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "咒术回战");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_yousei2_jujutsu_03() {
    let path = PathBuf::from("[Yousei-raws2] 咒术回战 - 03 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "咒术回战");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_yousei2_jujutsu_04() {
    let path = PathBuf::from("[Yousei-raws2] 咒术回战 - 04 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "咒术回战");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_yousei2_jujutsu_05() {
    let path = PathBuf::from("[Yousei-raws2] 咒术回战 - 05 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "咒术回战");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_yousei2_jujutsu_06() {
    let path = PathBuf::from("[Yousei-raws2] 咒术回战 - 06 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "咒术回战");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_yousei2_jujutsu_07() {
    let path = PathBuf::from("[Yousei-raws2] 咒术回战 - 07 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "咒术回战");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_yousei2_jujutsu_08() {
    let path = PathBuf::from("[Yousei-raws2] 咒术回战 - 08 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "咒术回战");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_yousei2_jujutsu_09() {
    let path = PathBuf::from("[Yousei-raws2] 咒术回战 - 09 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "咒术回战");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_yousei2_jujutsu_10() {
    let path = PathBuf::from("[Yousei-raws2] 咒术回战 - 10 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "咒术回战");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_yousei2_jujutsu_11() {
    let path = PathBuf::from("[Yousei-raws2] 咒术回战 - 11 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "咒术回战");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_yousei2_jujutsu_12() {
    let path = PathBuf::from("[Yousei-raws2] 咒术回战 - 12 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "咒术回战");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_yousei2_spy_family_01() {
    let path = PathBuf::from("[Yousei-raws2] SPY×FAMILY - 01 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "SPY×FAMILY");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_yousei2_spy_family_02() {
    let path = PathBuf::from("[Yousei-raws2] SPY×FAMILY - 02 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "SPY×FAMILY");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_yousei2_spy_family_03() {
    let path = PathBuf::from("[Yousei-raws2] SPY×FAMILY - 03 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "SPY×FAMILY");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_yousei2_spy_family_04() {
    let path = PathBuf::from("[Yousei-raws2] SPY×FAMILY - 04 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "SPY×FAMILY");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_yousei2_spy_family_05() {
    let path = PathBuf::from("[Yousei-raws2] SPY×FAMILY - 05 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "SPY×FAMILY");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_yousei2_spy_family_06() {
    let path = PathBuf::from("[Yousei-raws2] SPY×FAMILY - 06 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "SPY×FAMILY");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_yousei2_spy_family_07() {
    let path = PathBuf::from("[Yousei-raws2] SPY×FAMILY - 07 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "SPY×FAMILY");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_yousei2_spy_family_08() {
    let path = PathBuf::from("[Yousei-raws2] SPY×FAMILY - 08 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "SPY×FAMILY");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_yousei2_spy_family_09() {
    let path = PathBuf::from("[Yousei-raws2] SPY×FAMILY - 09 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "SPY×FAMILY");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_yousei2_spy_family_10() {
    let path = PathBuf::from("[Yousei-raws2] SPY×FAMILY - 10 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "SPY×FAMILY");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_yousei2_spy_family_11() {
    let path = PathBuf::from("[Yousei-raws2] SPY×FAMILY - 11 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "SPY×FAMILY");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_yousei2_spy_family_12() {
    let path = PathBuf::from("[Yousei-raws2] SPY×FAMILY - 12 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws2");
    assert_eq!(info.anime_name, "SPY×FAMILY");
    assert_eq!(info.episode, "12");
}
