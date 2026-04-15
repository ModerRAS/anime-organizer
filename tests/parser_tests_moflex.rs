use std::path::PathBuf;

use anime_organizer::FilenameParser;

#[test]
fn test_parse_moflex_anime_01() {
    let path = PathBuf::from("[Moflex] 孤獨搖滾 - 01 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "孤獨搖滾");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_moflex_anime_02() {
    let path = PathBuf::from("[Moflex] 孤獨搖滾 - 02 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "孤獨搖滾");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_moflex_anime_03() {
    let path = PathBuf::from("[Moflex] 孤獨搖滾 - 03 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "孤獨搖滾");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_moflex_anime_04() {
    let path = PathBuf::from("[Moflex] 孤獨搖滾 - 04 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "孤獨搖滾");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_moflex_anime_05() {
    let path = PathBuf::from("[Moflex] 孤獨搖滾 - 05 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "孤獨搖滾");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_moflex_anime_06() {
    let path = PathBuf::from("[Moflex] 孤獨搖滾 - 06 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "孤獨搖滾");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_moflex_anime_07() {
    let path = PathBuf::from("[Moflex] 孤獨搖滾 - 07 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "孤獨搖滾");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_moflex_anime_08() {
    let path = PathBuf::from("[Moflex] 孤獨搖滾 - 08 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "孤獨搖滾");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_moflex_anime_09() {
    let path = PathBuf::from("[Moflex] 孤獨搖滾 - 09 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "孤獨搖滾");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_moflex_anime_10() {
    let path = PathBuf::from("[Moflex] 孤獨搖滾 - 10 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "孤獨搖滾");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_moflex_anime_11() {
    let path = PathBuf::from("[Moflex] 孤獨搖滾 - 11 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "孤獨搖滾");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_moflex_anime_12() {
    let path = PathBuf::from("[Moflex] 孤獨搖滾 - 12 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "孤獨搖滾");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_moflex_blue_lock_01() {
    let path = PathBuf::from("[Moflex] 藍色監獄 - 01 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "藍色監獄");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_moflex_blue_lock_02() {
    let path = PathBuf::from("[Moflex] 藍色監獄 - 02 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "藍色監獄");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_moflex_blue_lock_03() {
    let path = PathBuf::from("[Moflex] 藍色監獄 - 03 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "藍色監獄");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_moflex_blue_lock_04() {
    let path = PathBuf::from("[Moflex] 藍色監獄 - 04 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "藍色監獄");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_moflex_blue_lock_05() {
    let path = PathBuf::from("[Moflex] 藍色監獄 - 05 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "藍色監獄");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_moflex_blue_lock_06() {
    let path = PathBuf::from("[Moflex] 藍色監獄 - 06 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "藍色監獄");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_moflex_blue_lock_07() {
    let path = PathBuf::from("[Moflex] 藍色監獄 - 07 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "藍色監獄");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_moflex_blue_lock_08() {
    let path = PathBuf::from("[Moflex] 藍色監獄 - 08 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "藍色監獄");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_moflex_blue_lock_09() {
    let path = PathBuf::from("[Moflex] 藍色監獄 - 09 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "藍色監獄");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_moflex_blue_lock_10() {
    let path = PathBuf::from("[Moflex] 藍色監獄 - 10 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "藍色監獄");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_moflex_blue_lock_11() {
    let path = PathBuf::from("[Moflex] 藍色監獄 - 11 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "藍色監獄");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_moflex_blue_lock_12() {
    let path = PathBuf::from("[Moflex] 藍色監獄 - 12 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "藍色監獄");
    assert_eq!(info.episode, "12");
}

#[test]
fn test_parse_moflex_lycoris_01() {
    let path = PathBuf::from("[Moflex] Lycoris Recoil - 01 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "Lycoris Recoil");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_moflex_lycoris_02() {
    let path = PathBuf::from("[Moflex] Lycoris Recoil - 02 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "Lycoris Recoil");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_moflex_lycoris_03() {
    let path = PathBuf::from("[Moflex] Lycoris Recoil - 03 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "Lycoris Recoil");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_moflex_lycoris_04() {
    let path = PathBuf::from("[Moflex] Lycoris Recoil - 04 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "Lycoris Recoil");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_moflex_lycoris_05() {
    let path = PathBuf::from("[Moflex] Lycoris Recoil - 05 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "Lycoris Recoil");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_moflex_lycoris_06() {
    let path = PathBuf::from("[Moflex] Lycoris Recoil - 06 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "Lycoris Recoil");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_moflex_lycoris_07() {
    let path = PathBuf::from("[Moflex] Lycoris Recoil - 07 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "Lycoris Recoil");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_moflex_lycoris_08() {
    let path = PathBuf::from("[Moflex] Lycoris Recoil - 08 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "Lycoris Recoil");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_moflex_lycoris_09() {
    let path = PathBuf::from("[Moflex] Lycoris Recoil - 09 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "Lycoris Recoil");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_moflex_lycoris_10() {
    let path = PathBuf::from("[Moflex] Lycoris Recoil - 10 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "Lycoris Recoil");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_moflex_lycoris_11() {
    let path = PathBuf::from("[Moflex] Lycoris Recoil - 11 [1080P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "Lycoris Recoil");
    assert_eq!(info.episode, "11");
}

#[test]
fn test_parse_moflex_lycoris_12() {
    let path = PathBuf::from("[Moflex] Lycoris Recoil - 12 [720P].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("parse failed");
    assert_eq!(info.publisher, "Moflex");
    assert_eq!(info.anime_name, "Lycoris Recoil");
    assert_eq!(info.episode, "12");
}
