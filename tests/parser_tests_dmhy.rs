use std::path::PathBuf;

use anime_organizer::FilenameParser;

#[test]
fn test_parse_dmhy_anime_01() {
    let path = PathBuf::from("[DMHY] 你的名字 - 01 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "你的名字");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_dmhy_anime_02() {
    let path = PathBuf::from("[DMHY] 你的名字 - 02 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "你的名字");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_dmhy_anime_03() {
    let path = PathBuf::from("[DMHY] 你的名字 - 03 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "你的名字");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_dmhy_anime_04() {
    let path = PathBuf::from("[DMHY] 你的名字 - 04 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "你的名字");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_dmhy_anime_05() {
    let path = PathBuf::from("[DMHY] 你的名字 - 05 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "你的名字");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_dmhy_anime_06() {
    let path = PathBuf::from("[DMHY] 你的名字 - 06 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "你的名字");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_dmhy_anime_07() {
    let path = PathBuf::from("[DMHY] 你的名字 - 07 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "你的名字");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_dmhy_anime_08() {
    let path = PathBuf::from("[DMHY] 你的名字 - 08 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "你的名字");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_dmhy_anime_09() {
    let path = PathBuf::from("[DMHY] 你的名字 - 09 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "你的名字");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_dmhy_anime_10() {
    let path = PathBuf::from("[DMHY] 你的名字 - 10 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "你的名字");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_dmhy_anime_11() {
    let path = PathBuf::from("[DMHY] 你的名字 - 11 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "你的名字");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_dmhy_anime_12() {
    let path = PathBuf::from("[DMHY] 你的名字 - 12 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "你的名字");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_dmhy_weathering_01() {
    let path = PathBuf::from("[DMHY] 天氣之子 - 01 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "天氣之子");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_dmhy_weathering_02() {
    let path = PathBuf::from("[DMHY] 天氣之子 - 02 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "天氣之子");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_dmhy_weathering_03() {
    let path = PathBuf::from("[DMHY] 天氣之子 - 03 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "天氣之子");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_dmhy_weathering_04() {
    let path = PathBuf::from("[DMHY] 天氣之子 - 04 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "天氣之子");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_dmhy_weathering_05() {
    let path = PathBuf::from("[DMHY] 天氣之子 - 05 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "天氣之子");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_dmhy_weathering_06() {
    let path = PathBuf::from("[DMHY] 天氣之子 - 06 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "天氣之子");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_dmhy_weathering_07() {
    let path = PathBuf::from("[DMHY] 天氣之子 - 07 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "天氣之子");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_dmhy_weathering_08() {
    let path = PathBuf::from("[DMHY] 天氣之子 - 08 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "天氣之子");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_dmhy_weathering_09() {
    let path = PathBuf::from("[DMHY] 天氣之子 - 09 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "天氣之子");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_dmhy_weathering_10() {
    let path = PathBuf::from("[DMHY] 天氣之子 - 10 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "天氣之子");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_dmhy_weathering_11() {
    let path = PathBuf::from("[DMHY] 天氣之子 - 11 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "天氣之子");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_dmhy_weathering_12() {
    let path = PathBuf::from("[DMHY] 天氣之子 - 12 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "天氣之子");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_dmhy_suzume_01() {
    let path = PathBuf::from("[DMHY] 鈴芽之旅 - 01 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "鈴芽之旅");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_dmhy_suzume_02() {
    let path = PathBuf::from("[DMHY] 鈴芽之旅 - 02 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "鈴芽之旅");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_dmhy_suzume_03() {
    let path = PathBuf::from("[DMHY] 鈴芽之旅 - 03 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "鈴芽之旅");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_dmhy_suzume_04() {
    let path = PathBuf::from("[DMHY] 鈴芽之旅 - 04 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "鈴芽之旅");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_dmhy_suzume_05() {
    let path = PathBuf::from("[DMHY] 鈴芽之旅 - 05 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "鈴芽之旅");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_dmhy_suzume_06() {
    let path = PathBuf::from("[DMHY] 鈴芽之旅 - 06 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "鈴芽之旅");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_dmhy_suzume_07() {
    let path = PathBuf::from("[DMHY] 鈴芽之旅 - 07 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "鈴芽之旅");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_dmhy_suzume_08() {
    let path = PathBuf::from("[DMHY] 鈴芽之旅 - 08 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "鈴芽之旅");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_dmhy_suzume_09() {
    let path = PathBuf::from("[DMHY] 鈴芽之旅 - 09 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "鈴芽之旅");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_dmhy_suzume_10() {
    let path = PathBuf::from("[DMHY] 鈴芽之旅 - 10 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "鈴芽之旅");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_dmhy_suzume_11() {
    let path = PathBuf::from("[DMHY] 鈴芽之旅 - 11 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "鈴芽之旅");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_dmhy_suzume_12() {
    let path = PathBuf::from("[DMHY] 鈴芽之旅 - 12 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "DMHY");
    assert_eq!(info.anime_name, "鈴芽之旅");
    assert_eq!(info.episode, "12");
}
