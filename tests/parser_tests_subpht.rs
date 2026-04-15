use std::path::PathBuf;

use anime_organizer::FilenameParser;

#[test]
fn test_parse_subpht_bleach_01() {
    let path = PathBuf::from("[SubPht] Bleach - 01 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Bleach");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_subpht_bleach_02() {
    let path = PathBuf::from("[SubPht] Bleach - 02 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Bleach");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_subpht_bleach_03() {
    let path = PathBuf::from("[SubPht] Bleach - 03 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Bleach");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_subpht_bleach_04() {
    let path = PathBuf::from("[SubPht] Bleach - 04 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Bleach");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_subpht_bleach_05() {
    let path = PathBuf::from("[SubPht] Bleach - 05 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Bleach");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_subpht_bleach_06() {
    let path = PathBuf::from("[SubPht] Bleach - 06 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Bleach");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_subpht_bleach_07() {
    let path = PathBuf::from("[SubPht] Bleach - 07 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Bleach");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_subpht_bleach_08() {
    let path = PathBuf::from("[SubPht] Bleach - 08 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Bleach");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_subpht_bleach_09() {
    let path = PathBuf::from("[SubPht] Bleach - 09 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Bleach");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_subpht_bleach_10() {
    let path = PathBuf::from("[SubPht] Bleach - 10 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Bleach");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_subpht_bleach_11() {
    let path = PathBuf::from("[SubPht] Bleach - 11 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Bleach");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_subpht_bleach_12() {
    let path = PathBuf::from("[SubPht] Bleach - 12 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Bleach");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_subpht_dbz_01() {
    let path = PathBuf::from("[SubPht] Dragon Ball Z - 01 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Dragon Ball Z");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_subpht_dbz_02() {
    let path = PathBuf::from("[SubPht] Dragon Ball Z - 02 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Dragon Ball Z");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_subpht_dbz_03() {
    let path = PathBuf::from("[SubPht] Dragon Ball Z - 03 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Dragon Ball Z");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_subpht_dbz_04() {
    let path = PathBuf::from("[SubPht] Dragon Ball Z - 04 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Dragon Ball Z");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_subpht_dbz_05() {
    let path = PathBuf::from("[SubPht] Dragon Ball Z - 05 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Dragon Ball Z");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_subpht_dbz_06() {
    let path = PathBuf::from("[SubPht] Dragon Ball Z - 06 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Dragon Ball Z");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_subpht_dbz_07() {
    let path = PathBuf::from("[SubPht] Dragon Ball Z - 07 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Dragon Ball Z");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_subpht_dbz_08() {
    let path = PathBuf::from("[SubPht] Dragon Ball Z - 08 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Dragon Ball Z");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_subpht_dbz_09() {
    let path = PathBuf::from("[SubPht] Dragon Ball Z - 09 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Dragon Ball Z");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_subpht_dbz_10() {
    let path = PathBuf::from("[SubPht] Dragon Ball Z - 10 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Dragon Ball Z");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_subpht_dbz_11() {
    let path = PathBuf::from("[SubPht] Dragon Ball Z - 11 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Dragon Ball Z");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_subpht_dbz_12() {
    let path = PathBuf::from("[SubPht] Dragon Ball Z - 12 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Dragon Ball Z");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_subpht_eva_01() {
    let path = PathBuf::from("[SubPht] Evangelion - 01 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Evangelion");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_subpht_eva_02() {
    let path = PathBuf::from("[SubPht] Evangelion - 02 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Evangelion");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_subpht_eva_03() {
    let path = PathBuf::from("[SubPht] Evangelion - 03 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Evangelion");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_subpht_eva_04() {
    let path = PathBuf::from("[SubPht] Evangelion - 04 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Evangelion");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_subpht_eva_05() {
    let path = PathBuf::from("[SubPht] Evangelion - 05 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Evangelion");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_subpht_eva_06() {
    let path = PathBuf::from("[SubPht] Evangelion - 06 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Evangelion");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_subpht_eva_07() {
    let path = PathBuf::from("[SubPht] Evangelion - 07 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Evangelion");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_subpht_eva_08() {
    let path = PathBuf::from("[SubPht] Evangelion - 08 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Evangelion");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_subpht_eva_09() {
    let path = PathBuf::from("[SubPht] Evangelion - 09 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Evangelion");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_subpht_eva_10() {
    let path = PathBuf::from("[SubPht] Evangelion - 10 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Evangelion");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_subpht_eva_11() {
    let path = PathBuf::from("[SubPht] Evangelion - 11 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Evangelion");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_subpht_eva_12() {
    let path = PathBuf::from("[SubPht] Evangelion - 12 [BD].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "SubPht");
    assert_eq!(info.anime_name, "Evangelion");
    assert_eq!(info.episode, "12");
}
