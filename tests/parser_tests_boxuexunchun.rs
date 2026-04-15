//! Tests for 拨雪寻春 publisher
use anime_organizer::parser::FilenameParser;
use std::path::PathBuf;

#[test]
fn test_parse_boxuexunchun_尖帽子的魔法工房_01() {
    let path = PathBuf::from("[拨雪寻春] 尖帽子的魔法工房 - 01 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "尖帽子的魔法工房");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_boxuexunchun_尖帽子的魔法工房_02() {
    let path = PathBuf::from("[拨雪寻春] 尖帽子的魔法工房 - 02 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "尖帽子的魔法工房");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_boxuexunchun_尖帽子的魔法工房_03() {
    let path = PathBuf::from("[拨雪寻春] 尖帽子的魔法工房 - 03 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "尖帽子的魔法工房");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_boxuexunchun_尖帽子的魔法工房_04() {
    let path = PathBuf::from("[拨雪寻春] 尖帽子的魔法工房 - 04 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "尖帽子的魔法工房");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_boxuexunchun_尖帽子的魔法工房_05() {
    let path = PathBuf::from("[拨雪寻春] 尖帽子的魔法工房 - 05 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "尖帽子的魔法工房");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_boxuexunchun_尖帽子的魔法工房_06() {
    let path = PathBuf::from("[拨雪寻春] 尖帽子的魔法工房 - 06 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "尖帽子的魔法工房");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_boxuexunchun_尖帽子的魔法工房_07() {
    let path = PathBuf::from("[拨雪寻春] 尖帽子的魔法工房 - 07 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "尖帽子的魔法工房");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_boxuexunchun_尖帽子的魔法工房_08() {
    let path = PathBuf::from("[拨雪寻春] 尖帽子的魔法工房 - 08 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "尖帽子的魔法工房");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_boxuexunchun_尖帽子的魔法工房_09() {
    let path = PathBuf::from("[拨雪寻春] 尖帽子的魔法工房 - 09 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "尖帽子的魔法工房");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_boxuexunchun_尖帽子的魔法工房_10() {
    let path = PathBuf::from("[拨雪寻春] 尖帽子的魔法工房 - 10 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "尖帽子的魔法工房");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_boxuexunchun_尖帽子的魔法工房_11() {
    let path = PathBuf::from("[拨雪寻春] 尖帽子的魔法工房 - 11 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "尖帽子的魔法工房");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_boxuexunchun_尖帽子的魔法工房_12() {
    let path = PathBuf::from("[拨雪寻春] 尖帽子的魔法工房 - 12 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "尖帽子的魔法工房");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_boxuexunchun_命运奇异赝品_01() {
    let path = PathBuf::from("[拨雪寻春] 命运-奇异赝品 - 01 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "命运-奇异赝品");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_boxuexunchun_命运奇异赝品_02() {
    let path = PathBuf::from("[拨雪寻春] 命运-奇异赝品 - 02 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "命运-奇异赝品");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_boxuexunchun_命运奇异赝品_03() {
    let path = PathBuf::from("[拨雪寻春] 命运-奇异赝品 - 03 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "命运-奇异赝品");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_boxuexunchun_命运奇异赝品_04() {
    let path = PathBuf::from("[拨雪寻春] 命运-奇异赝品 - 04 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "命运-奇异赝品");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_boxuexunchun_命运奇异赝品_05() {
    let path = PathBuf::from("[拨雪寻春] 命运-奇异赝品 - 05 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "命运-奇异赝品");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_boxuexunchun_命运奇异赝品_06() {
    let path = PathBuf::from("[拨雪寻春] 命运-奇异赝品 - 06 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "命运-奇异赝品");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_boxuexunchun_命运奇异赝品_07() {
    let path = PathBuf::from("[拨雪寻春] 命运-奇异赝品 - 07 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "命运-奇异赝品");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_boxuexunchun_命运奇异赝品_08() {
    let path = PathBuf::from("[拨雪寻春] 命运-奇异赝品 - 08 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "命运-奇异赝品");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_boxuexunchun_命运奇异赝品_09() {
    let path = PathBuf::from("[拨雪寻春] 命运-奇异赝品 - 09 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "命运-奇异赝品");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_boxuexunchun_命运奇异赝品_10() {
    let path = PathBuf::from("[拨雪寻春] 命运-奇异赝品 - 10 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "命运-奇异赝品");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_boxuexunchun_命运奇异赝品_11() {
    let path = PathBuf::from("[拨雪寻春] 命运-奇异赝品 - 11 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "命运-奇异赝品");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_boxuexunchun_命运奇异赝品_12() {
    let path = PathBuf::from("[拨雪寻春] 命运-奇异赝品 - 12 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "命运-奇异赝品");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_boxuexunchun_异国日记_01() {
    let path = PathBuf::from("[拨雪寻春] 异国日记 - 01 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "异国日记");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_boxuexunchun_异国日记_02() {
    let path = PathBuf::from("[拨雪寻春] 异国日记 - 02 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "异国日记");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_boxuexunchun_异国日记_03() {
    let path = PathBuf::from("[拨雪寻春] 异国日记 - 03 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "异国日记");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_boxuexunchun_异国日记_04() {
    let path = PathBuf::from("[拨雪寻春] 异国日记 - 04 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "异国日记");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_boxuexunchun_异国日记_05() {
    let path = PathBuf::from("[拨雪寻春] 异国日记 - 05 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "异国日记");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_boxuexunchun_异国日记_06() {
    let path = PathBuf::from("[拨雪寻春] 异国日记 - 06 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "异国日记");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_boxuexunchun_异国日记_07() {
    let path = PathBuf::from("[拨雪寻春] 异国日记 - 07 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "异国日记");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_boxuexunchun_异国日记_08() {
    let path = PathBuf::from("[拨雪寻春] 异国日记 - 08 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "异国日记");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_boxuexunchun_异国日记_09() {
    let path = PathBuf::from("[拨雪寻春] 异国日记 - 09 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "异国日记");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_boxuexunchun_异国日记_10() {
    let path = PathBuf::from("[拨雪寻春] 异国日记 - 10 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "异国日记");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_boxuexunchun_异国日记_11() {
    let path = PathBuf::from("[拨雪寻春] 异国日记 - 11 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "异国日记");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_boxuexunchun_异国日记_12() {
    let path = PathBuf::from("[拨雪寻春] 异国日记 - 12 [1080P].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle 拨雪寻春 format");
    assert_eq!(info.publisher, "拨雪寻春");
    assert_eq!(info.anime_name, "异国日记");
    assert_eq!(info.episode, "12");
}
