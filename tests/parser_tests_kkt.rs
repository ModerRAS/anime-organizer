use std::path::PathBuf;

use anime_organizer::FilenameParser;

#[test]
fn test_parse_kkt_odd_01() {
    let path = PathBuf::from("[KKT] 怪物事变 - 01 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "怪物事变");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_kkt_odd_02() {
    let path = PathBuf::from("[KKT] 怪物事变 - 02 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "怪物事变");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_kkt_odd_03() {
    let path = PathBuf::from("[KKT] 怪物事变 - 03 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "怪物事变");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_kkt_odd_04() {
    let path = PathBuf::from("[KKT] 怪物事变 - 04 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "怪物事变");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_kkt_odd_05() {
    let path = PathBuf::from("[KKT] 怪物事变 - 05 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "怪物事变");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_kkt_odd_06() {
    let path = PathBuf::from("[KKT] 怪物事变 - 06 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "怪物事变");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_kkt_odd_07() {
    let path = PathBuf::from("[KKT] 怪物事变 - 07 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "怪物事变");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_kkt_odd_08() {
    let path = PathBuf::from("[KKT] 怪物事变 - 08 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "怪物事变");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_kkt_odd_09() {
    let path = PathBuf::from("[KKT] 怪物事变 - 09 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "怪物事变");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_kkt_odd_10() {
    let path = PathBuf::from("[KKT] 怪物事变 - 10 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "怪物事变");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_kkt_odd_11() {
    let path = PathBuf::from("[KKT] 怪物事变 - 11 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "怪物事变");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_kkt_odd_12() {
    let path = PathBuf::from("[KKT] 怪物事变 - 12 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "怪物事变");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_kkt_promo_01() {
    let path = PathBuf::from("[KKT] 平稳世代的韦驮天 - 01 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "平稳世代的韦驮天");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_kkt_promo_02() {
    let path = PathBuf::from("[KKT] 平稳世代的韦驮天 - 02 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "平稳世代的韦驮天");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_kkt_promo_03() {
    let path = PathBuf::from("[KKT] 平稳世代的韦驮天 - 03 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "平稳世代的韦驮天");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_kkt_promo_04() {
    let path = PathBuf::from("[KKT] 平稳世代的韦驮天 - 04 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "平稳世代的韦驮天");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_kkt_promo_05() {
    let path = PathBuf::from("[KKT] 平稳世代的韦驮天 - 05 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "平稳世代的韦驮天");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_kkt_promo_06() {
    let path = PathBuf::from("[KKT] 平稳世代的韦驮天 - 06 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "平稳世代的韦驮天");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_kkt_promo_07() {
    let path = PathBuf::from("[KKT] 平稳世代的韦驮天 - 07 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "平稳世代的韦驮天");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_kkt_promo_08() {
    let path = PathBuf::from("[KKT] 平稳世代的韦驮天 - 08 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "平稳世代的韦驮天");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_kkt_promo_09() {
    let path = PathBuf::from("[KKT] 平稳世代的韦驮天 - 09 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "平稳世代的韦驮天");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_kkt_promo_10() {
    let path = PathBuf::from("[KKT] 平稳世代的韦驮天 - 10 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "平稳世代的韦驮天");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_kkt_promo_11() {
    let path = PathBuf::from("[KKT] 平稳世代的韦驮天 - 11 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "平稳世代的韦驮天");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_kkt_promo_12() {
    let path = PathBuf::from("[KKT] 平稳世代的韦驮天 - 12 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "平稳世代的韦驮天");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_kkt_shibuya_01() {
    let path = PathBuf::from("[KKT] 涩谷事变 - 01 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "涩谷事变");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_kkt_shibuya_02() {
    let path = PathBuf::from("[KKT] 涩谷事变 - 02 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "涩谷事变");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_kkt_shibuya_03() {
    let path = PathBuf::from("[KKT] 涩谷事变 - 03 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "涩谷事变");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_kkt_shibuya_04() {
    let path = PathBuf::from("[KKT] 涩谷事变 - 04 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "涩谷事变");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_kkt_shibuya_05() {
    let path = PathBuf::from("[KKT] 涩谷事变 - 05 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "涩谷事变");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_kkt_shibuya_06() {
    let path = PathBuf::from("[KKT] 涩谷事变 - 06 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "涩谷事变");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_kkt_shibuya_07() {
    let path = PathBuf::from("[KKT] 涩谷事变 - 07 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "涩谷事变");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_kkt_shibuya_08() {
    let path = PathBuf::from("[KKT] 涩谷事变 - 08 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "涩谷事变");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_kkt_shibuya_09() {
    let path = PathBuf::from("[KKT] 涩谷事变 - 09 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "涩谷事变");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_kkt_shibuya_10() {
    let path = PathBuf::from("[KKT] 涩谷事变 - 10 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "涩谷事变");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_kkt_shibuya_11() {
    let path = PathBuf::from("[KKT] 涩谷事变 - 11 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "涩谷事变");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_kkt_shibuya_12() {
    let path = PathBuf::from("[KKT] 涩谷事变 - 12 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "KKT");
    assert_eq!(info.anime_name, "涩谷事变");
    assert_eq!(info.episode, "12");
}
