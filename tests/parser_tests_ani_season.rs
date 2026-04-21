//! ANi publisher season parsing tests
//!
//! Tests season detection for various ANi filename formats.
//! Covers: numeric suffix, Chinese season markers, Roman numerals, English "Season N" and "SN" patterns.

use anime_organizer::parser::FilenameParser;

// Numeric suffix format: "Anime 2", "Anime 12"
#[test]
fn test_ani_season_numeric_suffix() {
    let path = "[ANi] Test Anime 2 - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(2));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_numeric_suffix_23() {
    let path = "[ANi] Test Anime 23 - 03 [720P].mkv";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(23));
    assert_eq!(info.series_name(), "Test Anime");
}

// Chinese format: "第2季", "第二季"
#[test]
fn test_ani_season_chinese_numeric() {
    let path = "[ANi] Test Anime 第2季 - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(2));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_chinese_cjk() {
    let path = "[ANi] Test Anime 第二季 - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(2));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_chinese_fourth() {
    let path = "[ANi] Test Anime 第四季 - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(4));
    assert_eq!(info.series_name(), "Test Anime");
}

// Period format: "2期", "3期"
#[test]
fn test_ani_season_period_format() {
    let path = "[ANi] Test Anime 2期 - 05 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(2));
    assert_eq!(info.series_name(), "Test Anime");
}

// Roman numeral format: "II", "III", "IV", "V"
#[test]
fn test_ani_season_roman_er() {
    let path = "[ANi] Test Anime 貳 - 07 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(2));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_roman_alt_er() {
    let path = "[ANi] Test Anime 贰 - 07 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(2));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_roman_tierce() {
    let path = "[ANi] Test Anime III - 03 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(3));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_roman_iv() {
    let path = "[ANi] Test Anime IV - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(4));
    assert_eq!(info.series_name(), "Test Anime");
}

// Period-style CJK: "二期", "三期", "四期"
#[test]
fn test_ani_season_cjk_period_second() {
    let path = "[ANi] Test Anime 二期 - 10 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(2));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_cjk_period_third() {
    let path = "[ANi] Test Anime 三期 - 12 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(3));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_cjk_period_fourth() {
    let path = "[ANi] Test Anime 四期 - 08 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(4));
    assert_eq!(info.series_name(), "Test Anime");
}

// English "Season N" format
#[test]
fn test_ani_season_english() {
    let path = "[ANi] Test Anime Season 2 - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(2));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_english_lowercase() {
    let path = "[ANi] Test Anime season 3 - 05 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(3));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_english_no_space() {
    let path = "[ANi] Test Anime Season10 - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(10));
    assert_eq!(info.series_name(), "Test Anime");
}

// English "SN" format (S02, S03, etc.)
#[test]
fn test_ani_season_short_s02() {
    let path = "[ANi] Test Anime S02 - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(2));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_short_s03() {
    let path = "[ANi] Test Anime s03 - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(3));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_short_s10() {
    let path = "[ANi] Test Anime S10 - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(10));
    assert_eq!(info.series_name(), "Test Anime");
}

// English ordinal format: "2nd season", "3rd season"
#[test]
fn test_ani_season_ordinal_2nd() {
    let path = "[ANi] Test Anime 2nd Season - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(2));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_ordinal_3rd() {
    let path = "[ANi] Test Anime 3rd Season - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(3));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_ordinal_4th() {
    let path = "[ANi] Test Anime 4th Season - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(4));
    assert_eq!(info.series_name(), "Test Anime");
}

// Negative tests: "Anime 1" should NOT be treated as season
#[test]
fn test_ani_no_season_suffix_1() {
    let path = "[ANi] Anime 1 - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    // "1" alone is NOT a season marker (only >= 2 is)
    assert_eq!(info.season_number(), None);
    // The whole thing is the anime name
    assert_eq!(info.series_name(), "Anime 1");
}

#[test]
fn test_ani_no_season_suffix_single_digit_1() {
    let path = "[ANi] Test Anime 1 - 05 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    // Single digit "1" after title is NOT a season
    assert_eq!(info.season_number(), None);
    assert_eq!(info.series_name(), "Test Anime 1");
}

// Real ANi filenames from DMHY
#[test]
fn test_ani_real_dmhy_season_2() {
    // 異世界悠閒農家 2 - 03.mp4
    let path = "[ANi] 異世界悠閒農家 2 - 03.mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(2));
    assert_eq!(info.series_name(), "異世界悠閒農家");
}

#[test]
fn test_ani_real_dmhy_fourth_season() {
    // 關於我轉生變成史萊姆這檔事 第四季 - 74.mp4
    let path = "[ANi] 關於我轉生變成史萊姆這檔事 第四季 - 74.mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(4));
    assert_eq!(info.series_name(), "關於我轉生變成史萊姆這檔事");
}

#[test]
fn test_ani_real_dmhy_season_3() {
    // SPY×FAMILY 間諜家家酒 Season 3 - 50.mp4
    let path = "[ANi] SPY×FAMILY 間諜家家酒 Season 3 - 50.mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(3));
    assert_eq!(info.series_name(), "SPY×FAMILY 間諜家家酒");
}

#[test]
fn test_ani_real_dmhy_second_season_cjk() {
    // MIX 第二季 ~第二個夏天，邁向晴空~ - 24.mp4
    let path = "[ANi] MIX 第二季 ~第二個夏天，邁向晴空~ - 24.mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(2));
    assert_eq!(info.series_name(), "MIX");
}

// Chinese number parsing: 一, 二, 三, 四, 五, 六, 七, 八, 九, 十
#[test]
fn test_ani_season_chinese_ten() {
    let path = "[ANi] Test Anime 第十季 - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(10));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_chinese_eleven() {
    let path = "[ANi] Test Anime 第十一季 - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(11));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_chinese_twelve() {
    let path = "[ANi] Test Anime 第十二季 - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(12));
    assert_eq!(info.series_name(), "Test Anime");
}

#[test]
fn test_ani_season_chinese_twenty() {
    let path = "[ANi] Test Anime 第二十季 - 01 [1080P].mp4";
    let info = FilenameParser::parse(path).unwrap();
    assert_eq!(info.season_number(), Some(20));
    assert_eq!(info.series_name(), "Test Anime");
}
