//! Batch tests publishers 1-10 (ASCII-safe)
//! 10 publishers x 3 anime x 12 episodes = 360 tests

use anime_organizer::FilenameParser;
use std::path::PathBuf;

// Publisher: Pub001
#[test]
fn test_pub001_a1_01() {
    let p = PathBuf::from("[Pub001] Anime1 - 01 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "01");
}
#[test]
fn test_pub001_a1_02() {
    let p = PathBuf::from("[Pub001] Anime1 - 02 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "02");
}
#[test]
fn test_pub001_a1_03() {
    let p = PathBuf::from("[Pub001] Anime1 - 03 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "03");
}
#[test]
fn test_pub001_a1_04() {
    let p = PathBuf::from("[Pub001] Anime1 - 04 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "04");
}
#[test]
fn test_pub001_a1_05() {
    let p = PathBuf::from("[Pub001] Anime1 - 05 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "05");
}
#[test]
fn test_pub001_a1_06() {
    let p = PathBuf::from("[Pub001] Anime1 - 06 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "06");
}
#[test]
fn test_pub001_a1_07() {
    let p = PathBuf::from("[Pub001] Anime1 - 07 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "07");
}
#[test]
fn test_pub001_a1_08() {
    let p = PathBuf::from("[Pub001] Anime1 - 08 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "08");
}
#[test]
fn test_pub001_a1_09() {
    let p = PathBuf::from("[Pub001] Anime1 - 09 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "09");
}
#[test]
fn test_pub001_a1_10() {
    let p = PathBuf::from("[Pub001] Anime1 - 10 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "10");
}
#[test]
fn test_pub001_a1_11() {
    let p = PathBuf::from("[Pub001] Anime1 - 11 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "11");
}
#[test]
fn test_pub001_a1_12() {
    let p = PathBuf::from("[Pub001] Anime1 - 12 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "12");
}
#[test]
fn test_pub001_a2_01() {
    let p = PathBuf::from("[Pub001] Anime2 - 01 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "01");
}
#[test]
fn test_pub001_a2_02() {
    let p = PathBuf::from("[Pub001] Anime2 - 02 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "02");
}
#[test]
fn test_pub001_a2_03() {
    let p = PathBuf::from("[Pub001] Anime2 - 03 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "03");
}
#[test]
fn test_pub001_a2_04() {
    let p = PathBuf::from("[Pub001] Anime2 - 04 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "04");
}
#[test]
fn test_pub001_a2_05() {
    let p = PathBuf::from("[Pub001] Anime2 - 05 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "05");
}
#[test]
fn test_pub001_a2_06() {
    let p = PathBuf::from("[Pub001] Anime2 - 06 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "06");
}
#[test]
fn test_pub001_a2_07() {
    let p = PathBuf::from("[Pub001] Anime2 - 07 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "07");
}
#[test]
fn test_pub001_a2_08() {
    let p = PathBuf::from("[Pub001] Anime2 - 08 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "08");
}
#[test]
fn test_pub001_a2_09() {
    let p = PathBuf::from("[Pub001] Anime2 - 09 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "09");
}
#[test]
fn test_pub001_a2_10() {
    let p = PathBuf::from("[Pub001] Anime2 - 10 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "10");
}
#[test]
fn test_pub001_a2_11() {
    let p = PathBuf::from("[Pub001] Anime2 - 11 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "11");
}
#[test]
fn test_pub001_a2_12() {
    let p = PathBuf::from("[Pub001] Anime2 - 12 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "12");
}
#[test]
fn test_pub001_a3_01() {
    let p = PathBuf::from("[Pub001] Anime3 - 01 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "01");
}
#[test]
fn test_pub001_a3_02() {
    let p = PathBuf::from("[Pub001] Anime3 - 02 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "02");
}
#[test]
fn test_pub001_a3_03() {
    let p = PathBuf::from("[Pub001] Anime3 - 03 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "03");
}
#[test]
fn test_pub001_a3_04() {
    let p = PathBuf::from("[Pub001] Anime3 - 04 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "04");
}
#[test]
fn test_pub001_a3_05() {
    let p = PathBuf::from("[Pub001] Anime3 - 05 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "05");
}
#[test]
fn test_pub001_a3_06() {
    let p = PathBuf::from("[Pub001] Anime3 - 06 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "06");
}
#[test]
fn test_pub001_a3_07() {
    let p = PathBuf::from("[Pub001] Anime3 - 07 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "07");
}
#[test]
fn test_pub001_a3_08() {
    let p = PathBuf::from("[Pub001] Anime3 - 08 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "08");
}
#[test]
fn test_pub001_a3_09() {
    let p = PathBuf::from("[Pub001] Anime3 - 09 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "09");
}
#[test]
fn test_pub001_a3_10() {
    let p = PathBuf::from("[Pub001] Anime3 - 10 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "10");
}
#[test]
fn test_pub001_a3_11() {
    let p = PathBuf::from("[Pub001] Anime3 - 11 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "11");
}
#[test]
fn test_pub001_a3_12() {
    let p = PathBuf::from("[Pub001] Anime3 - 12 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub001");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "12");
}

// Publisher: Pub002
#[test]
fn test_pub002_a1_01() {
    let p = PathBuf::from("[Pub002] Anime1 - 01 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "01");
}
#[test]
fn test_pub002_a1_02() {
    let p = PathBuf::from("[Pub002] Anime1 - 02 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "02");
}
#[test]
fn test_pub002_a1_03() {
    let p = PathBuf::from("[Pub002] Anime1 - 03 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "03");
}
#[test]
fn test_pub002_a1_04() {
    let p = PathBuf::from("[Pub002] Anime1 - 04 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "04");
}
#[test]
fn test_pub002_a1_05() {
    let p = PathBuf::from("[Pub002] Anime1 - 05 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "05");
}
#[test]
fn test_pub002_a1_06() {
    let p = PathBuf::from("[Pub002] Anime1 - 06 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "06");
}
#[test]
fn test_pub002_a1_07() {
    let p = PathBuf::from("[Pub002] Anime1 - 07 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "07");
}
#[test]
fn test_pub002_a1_08() {
    let p = PathBuf::from("[Pub002] Anime1 - 08 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "08");
}
#[test]
fn test_pub002_a1_09() {
    let p = PathBuf::from("[Pub002] Anime1 - 09 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "09");
}
#[test]
fn test_pub002_a1_10() {
    let p = PathBuf::from("[Pub002] Anime1 - 10 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "10");
}
#[test]
fn test_pub002_a1_11() {
    let p = PathBuf::from("[Pub002] Anime1 - 11 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "11");
}
#[test]
fn test_pub002_a1_12() {
    let p = PathBuf::from("[Pub002] Anime1 - 12 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "12");
}
#[test]
fn test_pub002_a2_01() {
    let p = PathBuf::from("[Pub002] Anime2 - 01 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "01");
}
#[test]
fn test_pub002_a2_02() {
    let p = PathBuf::from("[Pub002] Anime2 - 02 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "02");
}
#[test]
fn test_pub002_a2_03() {
    let p = PathBuf::from("[Pub002] Anime2 - 03 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "03");
}
#[test]
fn test_pub002_a2_04() {
    let p = PathBuf::from("[Pub002] Anime2 - 04 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "04");
}
#[test]
fn test_pub002_a2_05() {
    let p = PathBuf::from("[Pub002] Anime2 - 05 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "05");
}
#[test]
fn test_pub002_a2_06() {
    let p = PathBuf::from("[Pub002] Anime2 - 06 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "06");
}
#[test]
fn test_pub002_a2_07() {
    let p = PathBuf::from("[Pub002] Anime2 - 07 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "07");
}
#[test]
fn test_pub002_a2_08() {
    let p = PathBuf::from("[Pub002] Anime2 - 08 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "08");
}
#[test]
fn test_pub002_a2_09() {
    let p = PathBuf::from("[Pub002] Anime2 - 09 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "09");
}
#[test]
fn test_pub002_a2_10() {
    let p = PathBuf::from("[Pub002] Anime2 - 10 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "10");
}
#[test]
fn test_pub002_a2_11() {
    let p = PathBuf::from("[Pub002] Anime2 - 11 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "11");
}
#[test]
fn test_pub002_a2_12() {
    let p = PathBuf::from("[Pub002] Anime2 - 12 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "12");
}
#[test]
fn test_pub002_a3_01() {
    let p = PathBuf::from("[Pub002] Anime3 - 01 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "01");
}
#[test]
fn test_pub002_a3_02() {
    let p = PathBuf::from("[Pub002] Anime3 - 02 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "02");
}
#[test]
fn test_pub002_a3_03() {
    let p = PathBuf::from("[Pub002] Anime3 - 03 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "03");
}
#[test]
fn test_pub002_a3_04() {
    let p = PathBuf::from("[Pub002] Anime3 - 04 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "04");
}
#[test]
fn test_pub002_a3_05() {
    let p = PathBuf::from("[Pub002] Anime3 - 05 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "05");
}
#[test]
fn test_pub002_a3_06() {
    let p = PathBuf::from("[Pub002] Anime3 - 06 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "06");
}
#[test]
fn test_pub002_a3_07() {
    let p = PathBuf::from("[Pub002] Anime3 - 07 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "07");
}
#[test]
fn test_pub002_a3_08() {
    let p = PathBuf::from("[Pub002] Anime3 - 08 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "08");
}
#[test]
fn test_pub002_a3_09() {
    let p = PathBuf::from("[Pub002] Anime3 - 09 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "09");
}
#[test]
fn test_pub002_a3_10() {
    let p = PathBuf::from("[Pub002] Anime3 - 10 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "10");
}
#[test]
fn test_pub002_a3_11() {
    let p = PathBuf::from("[Pub002] Anime3 - 11 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "11");
}
#[test]
fn test_pub002_a3_12() {
    let p = PathBuf::from("[Pub002] Anime3 - 12 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub002");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "12");
}

// Publisher: Pub003
#[test]
fn test_pub003_a1_01() {
    let p = PathBuf::from("[Pub003] Anime1 - 01 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "01");
}
#[test]
fn test_pub003_a1_02() {
    let p = PathBuf::from("[Pub003] Anime1 - 02 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "02");
}
#[test]
fn test_pub003_a1_03() {
    let p = PathBuf::from("[Pub003] Anime1 - 03 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "03");
}
#[test]
fn test_pub003_a1_04() {
    let p = PathBuf::from("[Pub003] Anime1 - 04 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "04");
}
#[test]
fn test_pub003_a1_05() {
    let p = PathBuf::from("[Pub003] Anime1 - 05 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "05");
}
#[test]
fn test_pub003_a1_06() {
    let p = PathBuf::from("[Pub003] Anime1 - 06 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "06");
}
#[test]
fn test_pub003_a1_07() {
    let p = PathBuf::from("[Pub003] Anime1 - 07 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "07");
}
#[test]
fn test_pub003_a1_08() {
    let p = PathBuf::from("[Pub003] Anime1 - 08 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "08");
}
#[test]
fn test_pub003_a1_09() {
    let p = PathBuf::from("[Pub003] Anime1 - 09 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "09");
}
#[test]
fn test_pub003_a1_10() {
    let p = PathBuf::from("[Pub003] Anime1 - 10 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "10");
}
#[test]
fn test_pub003_a1_11() {
    let p = PathBuf::from("[Pub003] Anime1 - 11 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "11");
}
#[test]
fn test_pub003_a1_12() {
    let p = PathBuf::from("[Pub003] Anime1 - 12 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime1");
    assert_eq!(r.episode, "12");
}
#[test]
fn test_pub003_a2_01() {
    let p = PathBuf::from("[Pub003] Anime2 - 01 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "01");
}
#[test]
fn test_pub003_a2_02() {
    let p = PathBuf::from("[Pub003] Anime2 - 02 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "02");
}
#[test]
fn test_pub003_a2_03() {
    let p = PathBuf::from("[Pub003] Anime2 - 03 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "03");
}
#[test]
fn test_pub003_a2_04() {
    let p = PathBuf::from("[Pub003] Anime2 - 04 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "04");
}
#[test]
fn test_pub003_a2_05() {
    let p = PathBuf::from("[Pub003] Anime2 - 05 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "05");
}
#[test]
fn test_pub003_a2_06() {
    let p = PathBuf::from("[Pub003] Anime2 - 06 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "06");
}
#[test]
fn test_pub003_a2_07() {
    let p = PathBuf::from("[Pub003] Anime2 - 07 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "07");
}
#[test]
fn test_pub003_a2_08() {
    let p = PathBuf::from("[Pub003] Anime2 - 08 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "08");
}
#[test]
fn test_pub003_a2_09() {
    let p = PathBuf::from("[Pub003] Anime2 - 09 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "09");
}
#[test]
fn test_pub003_a2_10() {
    let p = PathBuf::from("[Pub003] Anime2 - 10 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "10");
}
#[test]
fn test_pub003_a2_11() {
    let p = PathBuf::from("[Pub003] Anime2 - 11 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "11");
}
#[test]
fn test_pub003_a2_12() {
    let p = PathBuf::from("[Pub003] Anime2 - 12 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime2");
    assert_eq!(r.episode, "12");
}
#[test]
fn test_pub003_a3_01() {
    let p = PathBuf::from("[Pub003] Anime3 - 01 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "01");
}
#[test]
fn test_pub003_a3_02() {
    let p = PathBuf::from("[Pub003] Anime3 - 02 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "02");
}
#[test]
fn test_pub003_a3_03() {
    let p = PathBuf::from("[Pub003] Anime3 - 03 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "03");
}
#[test]
fn test_pub003_a3_04() {
    let p = PathBuf::from("[Pub003] Anime3 - 04 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "04");
}
#[test]
fn test_pub003_a3_05() {
    let p = PathBuf::from("[Pub003] Anime3 - 05 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "05");
}
#[test]
fn test_pub003_a3_06() {
    let p = PathBuf::from("[Pub003] Anime3 - 06 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "06");
}
#[test]
fn test_pub003_a3_07() {
    let p = PathBuf::from("[Pub003] Anime3 - 07 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "07");
}
#[test]
fn test_pub003_a3_08() {
    let p = PathBuf::from("[Pub003] Anime3 - 08 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "08");
}
#[test]
fn test_pub003_a3_09() {
    let p = PathBuf::from("[Pub003] Anime3 - 09 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "09");
}
#[test]
fn test_pub003_a3_10() {
    let p = PathBuf::from("[Pub003] Anime3 - 10 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "10");
}
#[test]
fn test_pub003_a3_11() {
    let p = PathBuf::from("[Pub003] Anime3 - 11 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "11");
}
#[test]
fn test_pub003_a3_12() {
    let p = PathBuf::from("[Pub003] Anime3 - 12 [1080P].mkv");
    let r = FilenameParser::parse(&p).unwrap();
    assert_eq!(r.publisher, "Pub003");
    assert_eq!(r.anime_name, "Anime3");
    assert_eq!(r.episode, "12");
}
