use std::path::PathBuf;

use anime_organizer::FilenameParser;

#[test]
fn test_parse_yousei_one_piece_01() {
    let path = PathBuf::from("[Yousei-raws] ONE PIECE - 01 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "ONE PIECE");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_yousei_one_piece_02() {
    let path = PathBuf::from("[Yousei-raws] ONE PIECE - 02 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "ONE PIECE");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_yousei_one_piece_03() {
    let path = PathBuf::from("[Yousei-raws] ONE PIECE - 03 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "ONE PIECE");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_yousei_one_piece_04() {
    let path = PathBuf::from("[Yousei-raws] ONE PIECE - 04 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "ONE PIECE");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_yousei_one_piece_05() {
    let path = PathBuf::from("[Yousei-raws] ONE PIECE - 05 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "ONE PIECE");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_yousei_one_piece_06() {
    let path = PathBuf::from("[Yousei-raws] ONE PIECE - 06 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "ONE PIECE");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_yousei_one_piece_07() {
    let path = PathBuf::from("[Yousei-raws] ONE PIECE - 07 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "ONE PIECE");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_yousei_one_piece_08() {
    let path = PathBuf::from("[Yousei-raws] ONE PIECE - 08 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "ONE PIECE");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_yousei_one_piece_09() {
    let path = PathBuf::from("[Yousei-raws] ONE PIECE - 09 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "ONE PIECE");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_yousei_one_piece_10() {
    let path = PathBuf::from("[Yousei-raws] ONE PIECE - 10 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "ONE PIECE");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_yousei_one_piece_11() {
    let path = PathBuf::from("[Yousei-raws] ONE PIECE - 11 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "ONE PIECE");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_yousei_one_piece_12() {
    let path = PathBuf::from("[Yousei-raws] ONE PIECE - 12 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "ONE PIECE");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_yousei_hxh_01() {
    let path = PathBuf::from("[Yousei-raws] 全职猎人 - 01 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "全职猎人");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_yousei_hxh_02() {
    let path = PathBuf::from("[Yousei-raws] 全职猎人 - 02 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "全职猎人");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_yousei_hxh_03() {
    let path = PathBuf::from("[Yousei-raws] 全职猎人 - 03 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "全职猎人");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_yousei_hxh_04() {
    let path = PathBuf::from("[Yousei-raws] 全职猎人 - 04 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "全职猎人");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_yousei_hxh_05() {
    let path = PathBuf::from("[Yousei-raws] 全职猎人 - 05 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "全职猎人");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_yousei_hxh_06() {
    let path = PathBuf::from("[Yousei-raws] 全职猎人 - 06 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "全职猎人");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_yousei_hxh_07() {
    let path = PathBuf::from("[Yousei-raws] 全职猎人 - 07 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "全职猎人");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_yousei_hxh_08() {
    let path = PathBuf::from("[Yousei-raws] 全职猎人 - 08 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "全职猎人");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_yousei_hxh_09() {
    let path = PathBuf::from("[Yousei-raws] 全职猎人 - 09 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "全职猎人");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_yousei_hxh_10() {
    let path = PathBuf::from("[Yousei-raws] 全职猎人 - 10 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "全职猎人");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_yousei_hxh_11() {
    let path = PathBuf::from("[Yousei-raws] 全职猎人 - 11 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "全职猎人");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_yousei_hxh_12() {
    let path = PathBuf::from("[Yousei-raws] 全职猎人 - 12 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "全职猎人");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_yousei_fma_01() {
    let path = PathBuf::from("[Yousei-raws] 钢之炼金术师 - 01 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "钢之炼金术师");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_yousei_fma_02() {
    let path = PathBuf::from("[Yousei-raws] 钢之炼金术师 - 02 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "钢之炼金术师");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_yousei_fma_03() {
    let path = PathBuf::from("[Yousei-raws] 钢之炼金术师 - 03 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "钢之炼金术师");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_yousei_fma_04() {
    let path = PathBuf::from("[Yousei-raws] 钢之炼金术师 - 04 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "钢之炼金术师");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_yousei_fma_05() {
    let path = PathBuf::from("[Yousei-raws] 钢之炼金术师 - 05 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "钢之炼金术师");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_yousei_fma_06() {
    let path = PathBuf::from("[Yousei-raws] 钢之炼金术师 - 06 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "钢之炼金术师");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_yousei_fma_07() {
    let path = PathBuf::from("[Yousei-raws] 钢之炼金术师 - 07 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "钢之炼金术师");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_yousei_fma_08() {
    let path = PathBuf::from("[Yousei-raws] 钢之炼金术师 - 08 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "钢之炼金术师");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_yousei_fma_09() {
    let path = PathBuf::from("[Yousei-raws] 钢之炼金术师 - 09 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "钢之炼金术师");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_yousei_fma_10() {
    let path = PathBuf::from("[Yousei-raws] 钢之炼金术师 - 10 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "钢之炼金术师");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_yousei_fma_11() {
    let path = PathBuf::from("[Yousei-raws] 钢之炼金术师 - 11 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "钢之炼金术师");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_yousei_fma_12() {
    let path = PathBuf::from("[Yousei-raws] 钢之炼金术师 - 12 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Yousei-raws");
    assert_eq!(info.anime_name, "钢之炼金术师");
    assert_eq!(info.episode, "12");
}
