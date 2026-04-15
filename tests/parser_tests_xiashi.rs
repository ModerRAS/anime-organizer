use std::path::PathBuf;

use anime_organizer::FilenameParser;

#[test]
fn test_parse_xiashi_fate_01() {
    let path = PathBuf::from("[Xiashi] Fate/Stay Night - 01 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Stay Night");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_xiashi_fate_02() {
    let path = PathBuf::from("[Xiashi] Fate/Stay Night - 02 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Stay Night");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_xiashi_fate_03() {
    let path = PathBuf::from("[Xiashi] Fate/Stay Night - 03 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Stay Night");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_xiashi_fate_04() {
    let path = PathBuf::from("[Xiashi] Fate/Stay Night - 04 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Stay Night");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_xiashi_fate_05() {
    let path = PathBuf::from("[Xiashi] Fate/Stay Night - 05 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Stay Night");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_xiashi_fate_06() {
    let path = PathBuf::from("[Xiashi] Fate/Stay Night - 06 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Stay Night");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_xiashi_fate_07() {
    let path = PathBuf::from("[Xiashi] Fate/Stay Night - 07 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Stay Night");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_xiashi_fate_08() {
    let path = PathBuf::from("[Xiashi] Fate/Stay Night - 08 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Stay Night");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_xiashi_fate_09() {
    let path = PathBuf::from("[Xiashi] Fate/Stay Night - 09 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Stay Night");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_xiashi_fate_10() {
    let path = PathBuf::from("[Xiashi] Fate/Stay Night - 10 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Stay Night");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_xiashi_fate_11() {
    let path = PathBuf::from("[Xiashi] Fate/Stay Night - 11 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Stay Night");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_xiashi_fate_12() {
    let path = PathBuf::from("[Xiashi] Fate/Stay Night - 12 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Stay Night");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_xiashi_fate_zero_01() {
    let path = PathBuf::from("[Xiashi] Fate/Zero - 01 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Zero");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_xiashi_fate_zero_02() {
    let path = PathBuf::from("[Xiashi] Fate/Zero - 02 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Zero");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_xiashi_fate_zero_03() {
    let path = PathBuf::from("[Xiashi] Fate/Zero - 03 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Zero");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_xiashi_fate_zero_04() {
    let path = PathBuf::from("[Xiashi] Fate/Zero - 04 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Zero");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_xiashi_fate_zero_05() {
    let path = PathBuf::from("[Xiashi] Fate/Zero - 05 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Zero");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_xiashi_fate_zero_06() {
    let path = PathBuf::from("[Xiashi] Fate/Zero - 06 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Zero");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_xiashi_fate_zero_07() {
    let path = PathBuf::from("[Xiashi] Fate/Zero - 07 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Zero");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_xiashi_fate_zero_08() {
    let path = PathBuf::from("[Xiashi] Fate/Zero - 08 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Zero");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_xiashi_fate_zero_09() {
    let path = PathBuf::from("[Xiashi] Fate/Zero - 09 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Zero");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_xiashi_fate_zero_10() {
    let path = PathBuf::from("[Xiashi] Fate/Zero - 10 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Zero");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_xiashi_fate_zero_11() {
    let path = PathBuf::from("[Xiashi] Fate/Zero - 11 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Zero");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_xiashi_fate_zero_12() {
    let path = PathBuf::from("[Xiashi] Fate/Zero - 12 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/Zero");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_xiashi_fate_ubw_01() {
    let path = PathBuf::from("[Xiashi] Fate/UBW - 01 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/UBW");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_xiashi_fate_ubw_02() {
    let path = PathBuf::from("[Xiashi] Fate/UBW - 02 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/UBW");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_xiashi_fate_ubw_03() {
    let path = PathBuf::from("[Xiashi] Fate/UBW - 03 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/UBW");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_xiashi_fate_ubw_04() {
    let path = PathBuf::from("[Xiashi] Fate/UBW - 04 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/UBW");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_xiashi_fate_ubw_05() {
    let path = PathBuf::from("[Xiashi] Fate/UBW - 05 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/UBW");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_xiashi_fate_ubw_06() {
    let path = PathBuf::from("[Xiashi] Fate/UBW - 06 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/UBW");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_xiashi_fate_ubw_07() {
    let path = PathBuf::from("[Xiashi] Fate/UBW - 07 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/UBW");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_xiashi_fate_ubw_08() {
    let path = PathBuf::from("[Xiashi] Fate/UBW - 08 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/UBW");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_xiashi_fate_ubw_09() {
    let path = PathBuf::from("[Xiashi] Fate/UBW - 09 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/UBW");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_xiashi_fate_ubw_10() {
    let path = PathBuf::from("[Xiashi] Fate/UBW - 10 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/UBW");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_xiashi_fate_ubw_11() {
    let path = PathBuf::from("[Xiashi] Fate/UBW - 11 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/UBW");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_xiashi_fate_ubw_12() {
    let path = PathBuf::from("[Xiashi] Fate/UBW - 12 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Xiashi");
    assert_eq!(info.anime_name, "Fate/UBW");
    assert_eq!(info.episode, "12");
}
