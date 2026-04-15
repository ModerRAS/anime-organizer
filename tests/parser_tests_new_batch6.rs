//! Tests for anime publishers (Batch 6 - Additional subtitle groups and edge cases)
//! Contains 500+ test functions covering 30+ different publishers
use anime_organizer::parser::FilenameParser;
use std::path::PathBuf;

// ============================================================================
// AniLibria tests
// ============================================================================

#[test]
fn test_parse_anilibria_rezero_01() {
    let path = PathBuf::from("[AniLibria] ReZero - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AniLibria format");
    assert_eq!(info.publisher, "AniLibria");
    assert_eq!(info.anime_name, "ReZero");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_anilibria_rezero_02() {
    let path = PathBuf::from("[AniLibria] ReZero - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_anilibria_rezero_03() {
    let path = PathBuf::from("[AniLibria] ReZero - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_anilibria_rezero_04() {
    let path = PathBuf::from("[AniLibria] ReZero - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_anilibria_rezero_05() {
    let path = PathBuf::from("[AniLibria] ReZero - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_anilibria_rezero_06() {
    let path = PathBuf::from("[AniLibria] ReZero - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_anilibria_rezero_07() {
    let path = PathBuf::from("[AniLibria] ReZero - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_anilibria_rezero_08() {
    let path = PathBuf::from("[AniLibria] ReZero - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_anilibria_rezero_09() {
    let path = PathBuf::from("[AniLibria] ReZero - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_anilibria_rezero_10() {
    let path = PathBuf::from("[AniLibria] ReZero - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_anilibria_overlord_01() {
    let path = PathBuf::from("[AniLibria] Overlord - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AniLibria format");
    assert_eq!(info.publisher, "AniLibria");
    assert_eq!(info.anime_name, "Overlord");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_anilibria_overlord_02() {
    let path = PathBuf::from("[AniLibria] Overlord - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_anilibria_overlord_03() {
    let path = PathBuf::from("[AniLibria] Overlord - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_anilibria_overlord_04() {
    let path = PathBuf::from("[AniLibria] Overlord - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_anilibria_overlord_05() {
    let path = PathBuf::from("[AniLibria] Overlord - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_anilibria_overlord_06() {
    let path = PathBuf::from("[AniLibria] Overlord - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_anilibria_overlord_07() {
    let path = PathBuf::from("[AniLibria] Overlord - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_anilibria_overlord_08() {
    let path = PathBuf::from("[AniLibria] Overlord - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_anilibria_overlord_09() {
    let path = PathBuf::from("[AniLibria] Overlord - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_anilibria_overlord_10() {
    let path = PathBuf::from("[AniLibria] Overlord - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_anilibria_sword_art_online_01() {
    let path = PathBuf::from("[AniLibria] Sword Art Online - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AniLibria format");
    assert_eq!(info.publisher, "AniLibria");
    assert_eq!(info.anime_name, "Sword Art Online");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_anilibria_sword_art_online_02() {
    let path = PathBuf::from("[AniLibria] Sword Art Online - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_anilibria_sword_art_online_03() {
    let path = PathBuf::from("[AniLibria] Sword Art Online - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_anilibria_sword_art_online_04() {
    let path = PathBuf::from("[AniLibria] Sword Art Online - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_anilibria_sword_art_online_05() {
    let path = PathBuf::from("[AniLibria] Sword Art Online - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_anilibria_sword_art_online_06() {
    let path = PathBuf::from("[AniLibria] Sword Art Online - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_anilibria_sword_art_online_07() {
    let path = PathBuf::from("[AniLibria] Sword Art Online - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_anilibria_sword_art_online_08() {
    let path = PathBuf::from("[AniLibria] Sword Art Online - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_anilibria_sword_art_online_09() {
    let path = PathBuf::from("[AniLibria] Sword Art Online - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_anilibria_sword_art_online_10() {
    let path = PathBuf::from("[AniLibria] Sword Art Online - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// AnimeYorke tests
// ============================================================================

#[test]
fn test_parse_animeyorke_one_punch_man_01() {
    let path = PathBuf::from("[AnimeYorke] One Punch Man - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AnimeYorke format");
    assert_eq!(info.publisher, "AnimeYorke");
    assert_eq!(info.anime_name, "One Punch Man");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_animeyorke_one_punch_man_02() {
    let path = PathBuf::from("[AnimeYorke] One Punch Man - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_animeyorke_one_punch_man_03() {
    let path = PathBuf::from("[AnimeYorke] One Punch Man - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_animeyorke_one_punch_man_04() {
    let path = PathBuf::from("[AnimeYorke] One Punch Man - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_animeyorke_one_punch_man_05() {
    let path = PathBuf::from("[AnimeYorke] One Punch Man - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_animeyorke_one_punch_man_06() {
    let path = PathBuf::from("[AnimeYorke] One Punch Man - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_animeyorke_one_punch_man_07() {
    let path = PathBuf::from("[AnimeYorke] One Punch Man - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_animeyorke_one_punch_man_08() {
    let path = PathBuf::from("[AnimeYorke] One Punch Man - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_animeyorke_one_punch_man_09() {
    let path = PathBuf::from("[AnimeYorke] One Punch Man - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_animeyorke_one_punch_man_10() {
    let path = PathBuf::from("[AnimeYorke] One Punch Man - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_animeyorke_mob_psycho_100_01() {
    let path = PathBuf::from("[AnimeYorke] Mob Psycho 100 - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AnimeYorke format");
    assert_eq!(info.publisher, "AnimeYorke");
    assert_eq!(info.anime_name, "Mob Psycho 100");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_animeyorke_mob_psycho_100_02() {
    let path = PathBuf::from("[AnimeYorke] Mob Psycho 100 - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_animeyorke_mob_psycho_100_03() {
    let path = PathBuf::from("[AnimeYorke] Mob Psycho 100 - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_animeyorke_mob_psycho_100_04() {
    let path = PathBuf::from("[AnimeYorke] Mob Psycho 100 - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_animeyorke_mob_psycho_100_05() {
    let path = PathBuf::from("[AnimeYorke] Mob Psycho 100 - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_animeyorke_mob_psycho_100_06() {
    let path = PathBuf::from("[AnimeYorke] Mob Psycho 100 - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_animeyorke_mob_psycho_100_07() {
    let path = PathBuf::from("[AnimeYorke] Mob Psycho 100 - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_animeyorke_mob_psycho_100_08() {
    let path = PathBuf::from("[AnimeYorke] Mob Psycho 100 - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_animeyorke_mob_psycho_100_09() {
    let path = PathBuf::from("[AnimeYorke] Mob Psycho 100 - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_animeyorke_mob_psycho_100_10() {
    let path = PathBuf::from("[AnimeYorke] Mob Psycho 100 - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// AnimeZeitgeist tests
// ============================================================================

#[test]
fn test_parse_animezeitgeist_attack_on_titan_01() {
    let path = PathBuf::from("[AnimeZeitgeist] Attack on Titan - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AnimeZeitgeist format");
    assert_eq!(info.publisher, "AnimeZeitgeist");
    assert_eq!(info.anime_name, "Attack on Titan");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_animezeitgeist_attack_on_titan_02() {
    let path = PathBuf::from("[AnimeZeitgeist] Attack on Titan - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_animezeitgeist_attack_on_titan_03() {
    let path = PathBuf::from("[AnimeZeitgeist] Attack on Titan - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_animezeitgeist_attack_on_titan_04() {
    let path = PathBuf::from("[AnimeZeitgeist] Attack on Titan - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_animezeitgeist_attack_on_titan_05() {
    let path = PathBuf::from("[AnimeZeitgeist] Attack on Titan - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_animezeitgeist_attack_on_titan_06() {
    let path = PathBuf::from("[AnimeZeitgeist] Attack on Titan - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_animezeitgeist_attack_on_titan_07() {
    let path = PathBuf::from("[AnimeZeitgeist] Attack on Titan - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_animezeitgeist_attack_on_titan_08() {
    let path = PathBuf::from("[AnimeZeitgeist] Attack on Titan - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_animezeitgeist_attack_on_titan_09() {
    let path = PathBuf::from("[AnimeZeitgeist] Attack on Titan - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_animezeitgeist_attack_on_titan_10() {
    let path = PathBuf::from("[AnimeZeitgeist] Attack on Titan - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_animezeitgeist_fullmetal_alchemist_01() {
    let path = PathBuf::from("[AnimeZeitgeist] Fullmetal Alchemist - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AnimeZeitgeist format");
    assert_eq!(info.publisher, "AnimeZeitgeist");
    assert_eq!(info.anime_name, "Fullmetal Alchemist");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_animezeitgeist_fullmetal_alchemist_02() {
    let path = PathBuf::from("[AnimeZeitgeist] Fullmetal Alchemist - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_animezeitgeist_fullmetal_alchemist_03() {
    let path = PathBuf::from("[AnimeZeitgeist] Fullmetal Alchemist - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_animezeitgeist_fullmetal_alchemist_04() {
    let path = PathBuf::from("[AnimeZeitgeist] Fullmetal Alchemist - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_animezeitgeist_fullmetal_alchemist_05() {
    let path = PathBuf::from("[AnimeZeitgeist] Fullmetal Alchemist - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_animezeitgeist_fullmetal_alchemist_06() {
    let path = PathBuf::from("[AnimeZeitgeist] Fullmetal Alchemist - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_animezeitgeist_fullmetal_alchemist_07() {
    let path = PathBuf::from("[AnimeZeitgeist] Fullmetal Alchemist - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_animezeitgeist_fullmetal_alchemist_08() {
    let path = PathBuf::from("[AnimeZeitgeist] Fullmetal Alchemist - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_animezeitgeist_fullmetal_alchemist_09() {
    let path = PathBuf::from("[AnimeZeitgeist] Fullmetal Alchemist - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_animezeitgeist_fullmetal_alchemist_10() {
    let path = PathBuf::from("[AnimeZeitgeist] Fullmetal Alchemist - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// SubsUrbana tests
// ============================================================================

#[test]
fn test_parse_subsurbana_kaguya_sama_love_is_war_01() {
    let path = PathBuf::from("[SubsUrbana] Kaguya-sama Love is War - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle SubsUrbana format");
    assert_eq!(info.publisher, "SubsUrbana");
    assert_eq!(info.anime_name, "Kaguya-sama Love is War");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_subsurbana_kaguya_sama_love_is_war_02() {
    let path = PathBuf::from("[SubsUrbana] Kaguya-sama Love is War - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_subsurbana_kaguya_sama_love_is_war_03() {
    let path = PathBuf::from("[SubsUrbana] Kaguya-sama Love is War - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_subsurbana_kaguya_sama_love_is_war_04() {
    let path = PathBuf::from("[SubsUrbana] Kaguya-sama Love is War - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_subsurbana_kaguya_sama_love_is_war_05() {
    let path = PathBuf::from("[SubsUrbana] Kaguya-sama Love is War - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_subsurbana_kaguya_sama_love_is_war_06() {
    let path = PathBuf::from("[SubsUrbana] Kaguya-sama Love is War - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_subsurbana_kaguya_sama_love_is_war_07() {
    let path = PathBuf::from("[SubsUrbana] Kaguya-sama Love is War - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_subsurbana_kaguya_sama_love_is_war_08() {
    let path = PathBuf::from("[SubsUrbana] Kaguya-sama Love is War - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_subsurbana_kaguya_sama_love_is_war_09() {
    let path = PathBuf::from("[SubsUrbana] Kaguya-sama Love is War - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_subsurbana_kaguya_sama_love_is_war_10() {
    let path = PathBuf::from("[SubsUrbana] Kaguya-sama Love is War - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_subsurbana_toradora_01() {
    let path = PathBuf::from("[SubsUrbana] Toradora - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle SubsUrbana format");
    assert_eq!(info.publisher, "SubsUrbana");
    assert_eq!(info.anime_name, "Toradora");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_subsurbana_toradora_02() {
    let path = PathBuf::from("[SubsUrbana] Toradora - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_subsurbana_toradora_03() {
    let path = PathBuf::from("[SubsUrbana] Toradora - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_subsurbana_toradora_04() {
    let path = PathBuf::from("[SubsUrbana] Toradora - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_subsurbana_toradora_05() {
    let path = PathBuf::from("[SubsUrbana] Toradora - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_subsurbana_toradora_06() {
    let path = PathBuf::from("[SubsUrbana] Toradora - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_subsurbana_toradora_07() {
    let path = PathBuf::from("[SubsUrbana] Toradora - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_subsurbana_toradora_08() {
    let path = PathBuf::from("[SubsUrbana] Toradora - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_subsurbana_toradora_09() {
    let path = PathBuf::from("[SubsUrbana] Toradora - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_subsurbana_toradora_10() {
    let path = PathBuf::from("[SubsUrbana] Toradora - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// Daiz tests
// ============================================================================

#[test]
fn test_parse_daiz_naruto_01() {
    let path = PathBuf::from("[Daiz] Naruto - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Daiz format");
    assert_eq!(info.publisher, "Daiz");
    assert_eq!(info.anime_name, "Naruto");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_daiz_naruto_02() {
    let path = PathBuf::from("[Daiz] Naruto - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_daiz_naruto_03() {
    let path = PathBuf::from("[Daiz] Naruto - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_daiz_naruto_04() {
    let path = PathBuf::from("[Daiz] Naruto - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_daiz_naruto_05() {
    let path = PathBuf::from("[Daiz] Naruto - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_daiz_naruto_06() {
    let path = PathBuf::from("[Daiz] Naruto - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_daiz_naruto_07() {
    let path = PathBuf::from("[Daiz] Naruto - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_daiz_naruto_08() {
    let path = PathBuf::from("[Daiz] Naruto - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_daiz_naruto_09() {
    let path = PathBuf::from("[Daiz] Naruto - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_daiz_naruto_10() {
    let path = PathBuf::from("[Daiz] Naruto - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_daiz_bleach_01() {
    let path = PathBuf::from("[Daiz] Bleach - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Daiz format");
    assert_eq!(info.publisher, "Daiz");
    assert_eq!(info.anime_name, "Bleach");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_daiz_bleach_02() {
    let path = PathBuf::from("[Daiz] Bleach - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_daiz_bleach_03() {
    let path = PathBuf::from("[Daiz] Bleach - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_daiz_bleach_04() {
    let path = PathBuf::from("[Daiz] Bleach - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_daiz_bleach_05() {
    let path = PathBuf::from("[Daiz] Bleach - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_daiz_bleach_06() {
    let path = PathBuf::from("[Daiz] Bleach - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_daiz_bleach_07() {
    let path = PathBuf::from("[Daiz] Bleach - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_daiz_bleach_08() {
    let path = PathBuf::from("[Daiz] Bleach - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_daiz_bleach_09() {
    let path = PathBuf::from("[Daiz] Bleach - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_daiz_bleach_10() {
    let path = PathBuf::from("[Daiz] Bleach - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// HorribleSubs tests
// ============================================================================

#[test]
fn test_parse_horriblesubs_darling_in_the_franxx_01() {
    let path = PathBuf::from("[HorribleSubs] Darling in the Franxx - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle HorribleSubs format");
    assert_eq!(info.publisher, "HorribleSubs");
    assert_eq!(info.anime_name, "Darling in the Franxx");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_horriblesubs_darling_in_the_franxx_02() {
    let path = PathBuf::from("[HorribleSubs] Darling in the Franxx - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_horriblesubs_darling_in_the_franxx_03() {
    let path = PathBuf::from("[HorribleSubs] Darling in the Franxx - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_horriblesubs_darling_in_the_franxx_04() {
    let path = PathBuf::from("[HorribleSubs] Darling in the Franxx - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_horriblesubs_darling_in_the_franxx_05() {
    let path = PathBuf::from("[HorribleSubs] Darling in the Franxx - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_horriblesubs_darling_in_the_franxx_06() {
    let path = PathBuf::from("[HorribleSubs] Darling in the Franxx - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_horriblesubs_darling_in_the_franxx_07() {
    let path = PathBuf::from("[HorribleSubs] Darling in the Franxx - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_horriblesubs_darling_in_the_franxx_08() {
    let path = PathBuf::from("[HorribleSubs] Darling in the Franxx - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_horriblesubs_darling_in_the_franxx_09() {
    let path = PathBuf::from("[HorribleSubs] Darling in the Franxx - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_horriblesubs_darling_in_the_franxx_10() {
    let path = PathBuf::from("[HorribleSubs] Darling in the Franxx - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_horriblesubs_bungo_stray_dogs_01() {
    let path = PathBuf::from("[HorribleSubs] Bungo Stray Dogs - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle HorribleSubs format");
    assert_eq!(info.publisher, "HorribleSubs");
    assert_eq!(info.anime_name, "Bungo Stray Dogs");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_horriblesubs_bungo_stray_dogs_02() {
    let path = PathBuf::from("[HorribleSubs] Bungo Stray Dogs - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_horriblesubs_bungo_stray_dogs_03() {
    let path = PathBuf::from("[HorribleSubs] Bungo Stray Dogs - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_horriblesubs_bungo_stray_dogs_04() {
    let path = PathBuf::from("[HorribleSubs] Bungo Stray Dogs - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_horriblesubs_bungo_stray_dogs_05() {
    let path = PathBuf::from("[HorribleSubs] Bungo Stray Dogs - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_horriblesubs_bungo_stray_dogs_06() {
    let path = PathBuf::from("[HorribleSubs] Bungo Stray Dogs - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_horriblesubs_bungo_stray_dogs_07() {
    let path = PathBuf::from("[HorribleSubs] Bungo Stray Dogs - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_horriblesubs_bungo_stray_dogs_08() {
    let path = PathBuf::from("[HorribleSubs] Bungo Stray Dogs - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_horriblesubs_bungo_stray_dogs_09() {
    let path = PathBuf::from("[HorribleSubs] Bungo Stray Dogs - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_horriblesubs_bungo_stray_dogs_10() {
    let path = PathBuf::from("[HorribleSubs] Bungo Stray Dogs - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// Elysium tests
// ============================================================================

#[test]
fn test_parse_elysium_dr_stone_01() {
    let path = PathBuf::from("[Elysium] Dr Stone - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Elysium format");
    assert_eq!(info.publisher, "Elysium");
    assert_eq!(info.anime_name, "Dr Stone");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_elysium_dr_stone_02() {
    let path = PathBuf::from("[Elysium] Dr Stone - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_elysium_dr_stone_03() {
    let path = PathBuf::from("[Elysium] Dr Stone - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_elysium_dr_stone_04() {
    let path = PathBuf::from("[Elysium] Dr Stone - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_elysium_dr_stone_05() {
    let path = PathBuf::from("[Elysium] Dr Stone - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_elysium_dr_stone_06() {
    let path = PathBuf::from("[Elysium] Dr Stone - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_elysium_dr_stone_07() {
    let path = PathBuf::from("[Elysium] Dr Stone - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_elysium_dr_stone_08() {
    let path = PathBuf::from("[Elysium] Dr Stone - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_elysium_dr_stone_09() {
    let path = PathBuf::from("[Elysium] Dr Stone - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_elysium_dr_stone_10() {
    let path = PathBuf::from("[Elysium] Dr Stone - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_elysium_the_promised_neverland_01() {
    let path = PathBuf::from("[Elysium] The Promised Neverland - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Elysium format");
    assert_eq!(info.publisher, "Elysium");
    assert_eq!(info.anime_name, "The Promised Neverland");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_elysium_the_promised_neverland_02() {
    let path = PathBuf::from("[Elysium] The Promised Neverland - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_elysium_the_promised_neverland_03() {
    let path = PathBuf::from("[Elysium] The Promised Neverland - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_elysium_the_promised_neverland_04() {
    let path = PathBuf::from("[Elysium] The Promised Neverland - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_elysium_the_promised_neverland_05() {
    let path = PathBuf::from("[Elysium] The Promised Neverland - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_elysium_the_promised_neverland_06() {
    let path = PathBuf::from("[Elysium] The Promised Neverland - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_elysium_the_promised_neverland_07() {
    let path = PathBuf::from("[Elysium] The Promised Neverland - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_elysium_the_promised_neverland_08() {
    let path = PathBuf::from("[Elysium] The Promised Neverland - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_elysium_the_promised_neverland_09() {
    let path = PathBuf::from("[Elysium] The Promised Neverland - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_elysium_the_promised_neverland_10() {
    let path = PathBuf::from("[Elysium] The Promised Neverland - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// Kametsu tests
// ============================================================================

#[test]
fn test_parse_kametsu_violet_evergarden_01() {
    let path = PathBuf::from("[Kametsu] Violet Evergarden - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Kametsu format");
    assert_eq!(info.publisher, "Kametsu");
    assert_eq!(info.anime_name, "Violet Evergarden");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_kametsu_violet_evergarden_02() {
    let path = PathBuf::from("[Kametsu] Violet Evergarden - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_kametsu_violet_evergarden_03() {
    let path = PathBuf::from("[Kametsu] Violet Evergarden - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_kametsu_violet_evergarden_04() {
    let path = PathBuf::from("[Kametsu] Violet Evergarden - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_kametsu_violet_evergarden_05() {
    let path = PathBuf::from("[Kametsu] Violet Evergarden - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_kametsu_violet_evergarden_06() {
    let path = PathBuf::from("[Kametsu] Violet Evergarden - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_kametsu_violet_evergarden_07() {
    let path = PathBuf::from("[Kametsu] Violet Evergarden - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_kametsu_violet_evergarden_08() {
    let path = PathBuf::from("[Kametsu] Violet Evergarden - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_kametsu_violet_evergarden_09() {
    let path = PathBuf::from("[Kametsu] Violet Evergarden - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_kametsu_violet_evergarden_10() {
    let path = PathBuf::from("[Kametsu] Violet Evergarden - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_kametsu_your_lie_in_april_01() {
    let path = PathBuf::from("[Kametsu] Your Lie in April - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Kametsu format");
    assert_eq!(info.publisher, "Kametsu");
    assert_eq!(info.anime_name, "Your Lie in April");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_kametsu_your_lie_in_april_02() {
    let path = PathBuf::from("[Kametsu] Your Lie in April - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_kametsu_your_lie_in_april_03() {
    let path = PathBuf::from("[Kametsu] Your Lie in April - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_kametsu_your_lie_in_april_04() {
    let path = PathBuf::from("[Kametsu] Your Lie in April - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_kametsu_your_lie_in_april_05() {
    let path = PathBuf::from("[Kametsu] Your Lie in April - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_kametsu_your_lie_in_april_06() {
    let path = PathBuf::from("[Kametsu] Your Lie in April - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_kametsu_your_lie_in_april_07() {
    let path = PathBuf::from("[Kametsu] Your Lie in April - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_kametsu_your_lie_in_april_08() {
    let path = PathBuf::from("[Kametsu] Your Lie in April - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_kametsu_your_lie_in_april_09() {
    let path = PathBuf::from("[Kametsu] Your Lie in April - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_kametsu_your_lie_in_april_10() {
    let path = PathBuf::from("[Kametsu] Your Lie in April - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// AnimeWorld tests
// ============================================================================

#[test]
fn test_parse_animeworld_that_time_i_got_reincarnated_as_a_slime_01() {
    let path = PathBuf::from("[AnimeWorld] That Time I Got Reincarnated as a Slime - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AnimeWorld format");
    assert_eq!(info.publisher, "AnimeWorld");
    assert_eq!(info.anime_name, "That Time I Got Reincarnated as a Slime");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_animeworld_that_time_i_got_reincarnated_as_a_slime_02() {
    let path = PathBuf::from("[AnimeWorld] That Time I Got Reincarnated as a Slime - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_animeworld_that_time_i_got_reincarnated_as_a_slime_03() {
    let path = PathBuf::from("[AnimeWorld] That Time I Got Reincarnated as a Slime - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_animeworld_that_time_i_got_reincarnated_as_a_slime_04() {
    let path = PathBuf::from("[AnimeWorld] That Time I Got Reincarnated as a Slime - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_animeworld_that_time_i_got_reincarnated_as_a_slime_05() {
    let path = PathBuf::from("[AnimeWorld] That Time I Got Reincarnated as a Slime - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_animeworld_that_time_i_got_reincarnated_as_a_slime_06() {
    let path = PathBuf::from("[AnimeWorld] That Time I Got Reincarnated as a Slime - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_animeworld_that_time_i_got_reincarnated_as_a_slime_07() {
    let path = PathBuf::from("[AnimeWorld] That Time I Got Reincarnated as a Slime - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_animeworld_that_time_i_got_reincarnated_as_a_slime_08() {
    let path = PathBuf::from("[AnimeWorld] That Time I Got Reincarnated as a Slime - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_animeworld_that_time_i_got_reincarnated_as_a_slime_09() {
    let path = PathBuf::from("[AnimeWorld] That Time I Got Reincarnated as a Slime - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_animeworld_that_time_i_got_reincarnated_as_a_slime_10() {
    let path = PathBuf::from("[AnimeWorld] That Time I Got Reincarnated as a Slime - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_animeworld_mushoku_tensei_01() {
    let path = PathBuf::from("[AnimeWorld] Mushoku Tensei - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AnimeWorld format");
    assert_eq!(info.publisher, "AnimeWorld");
    assert_eq!(info.anime_name, "Mushoku Tensei");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_animeworld_mushoku_tensei_02() {
    let path = PathBuf::from("[AnimeWorld] Mushoku Tensei - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_animeworld_mushoku_tensei_03() {
    let path = PathBuf::from("[AnimeWorld] Mushoku Tensei - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_animeworld_mushoku_tensei_04() {
    let path = PathBuf::from("[AnimeWorld] Mushoku Tensei - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_animeworld_mushoku_tensei_05() {
    let path = PathBuf::from("[AnimeWorld] Mushoku Tensei - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_animeworld_mushoku_tensei_06() {
    let path = PathBuf::from("[AnimeWorld] Mushoku Tensei - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_animeworld_mushoku_tensei_07() {
    let path = PathBuf::from("[AnimeWorld] Mushoku Tensei - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_animeworld_mushoku_tensei_08() {
    let path = PathBuf::from("[AnimeWorld] Mushoku Tensei - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_animeworld_mushoku_tensei_09() {
    let path = PathBuf::from("[AnimeWorld] Mushoku Tensei - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_animeworld_mushoku_tensei_10() {
    let path = PathBuf::from("[AnimeWorld] Mushoku Tensei - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// Sw4mp tests
// ============================================================================

#[test]
fn test_parse_sw4mp_demon_slayer_01() {
    let path = PathBuf::from("[Sw4mp] Demon Slayer - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Sw4mp format");
    assert_eq!(info.publisher, "Sw4mp");
    assert_eq!(info.anime_name, "Demon Slayer");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_sw4mp_demon_slayer_02() {
    let path = PathBuf::from("[Sw4mp] Demon Slayer - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_sw4mp_demon_slayer_03() {
    let path = PathBuf::from("[Sw4mp] Demon Slayer - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_sw4mp_demon_slayer_04() {
    let path = PathBuf::from("[Sw4mp] Demon Slayer - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_sw4mp_demon_slayer_05() {
    let path = PathBuf::from("[Sw4mp] Demon Slayer - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_sw4mp_demon_slayer_06() {
    let path = PathBuf::from("[Sw4mp] Demon Slayer - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_sw4mp_demon_slayer_07() {
    let path = PathBuf::from("[Sw4mp] Demon Slayer - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_sw4mp_demon_slayer_08() {
    let path = PathBuf::from("[Sw4mp] Demon Slayer - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_sw4mp_demon_slayer_09() {
    let path = PathBuf::from("[Sw4mp] Demon Slayer - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_sw4mp_demon_slayer_10() {
    let path = PathBuf::from("[Sw4mp] Demon Slayer - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_sw4mp_jujutsu_kaisen_01() {
    let path = PathBuf::from("[Sw4mp] Jujutsu Kaisen - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Sw4mp format");
    assert_eq!(info.publisher, "Sw4mp");
    assert_eq!(info.anime_name, "Jujutsu Kaisen");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_sw4mp_jujutsu_kaisen_02() {
    let path = PathBuf::from("[Sw4mp] Jujutsu Kaisen - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_sw4mp_jujutsu_kaisen_03() {
    let path = PathBuf::from("[Sw4mp] Jujutsu Kaisen - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_sw4mp_jujutsu_kaisen_04() {
    let path = PathBuf::from("[Sw4mp] Jujutsu Kaisen - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_sw4mp_jujutsu_kaisen_05() {
    let path = PathBuf::from("[Sw4mp] Jujutsu Kaisen - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_sw4mp_jujutsu_kaisen_06() {
    let path = PathBuf::from("[Sw4mp] Jujutsu Kaisen - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_sw4mp_jujutsu_kaisen_07() {
    let path = PathBuf::from("[Sw4mp] Jujutsu Kaisen - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_sw4mp_jujutsu_kaisen_08() {
    let path = PathBuf::from("[Sw4mp] Jujutsu Kaisen - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_sw4mp_jujutsu_kaisen_09() {
    let path = PathBuf::from("[Sw4mp] Jujutsu Kaisen - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_sw4mp_jujutsu_kaisen_10() {
    let path = PathBuf::from("[Sw4mp] Jujutsu Kaisen - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// AniTomo tests
// ============================================================================

#[test]
fn test_parse_anitomo_my_hero_academia_01() {
    let path = PathBuf::from("[AniTomo] My Hero Academia - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AniTomo format");
    assert_eq!(info.publisher, "AniTomo");
    assert_eq!(info.anime_name, "My Hero Academia");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_anitomo_my_hero_academia_02() {
    let path = PathBuf::from("[AniTomo] My Hero Academia - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_anitomo_my_hero_academia_03() {
    let path = PathBuf::from("[AniTomo] My Hero Academia - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_anitomo_my_hero_academia_04() {
    let path = PathBuf::from("[AniTomo] My Hero Academia - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_anitomo_my_hero_academia_05() {
    let path = PathBuf::from("[AniTomo] My Hero Academia - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_anitomo_my_hero_academia_06() {
    let path = PathBuf::from("[AniTomo] My Hero Academia - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_anitomo_my_hero_academia_07() {
    let path = PathBuf::from("[AniTomo] My Hero Academia - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_anitomo_my_hero_academia_08() {
    let path = PathBuf::from("[AniTomo] My Hero Academia - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_anitomo_my_hero_academia_09() {
    let path = PathBuf::from("[AniTomo] My Hero Academia - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_anitomo_my_hero_academia_10() {
    let path = PathBuf::from("[AniTomo] My Hero Academia - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_anitomo_black_clover_01() {
    let path = PathBuf::from("[AniTomo] Black Clover - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AniTomo format");
    assert_eq!(info.publisher, "AniTomo");
    assert_eq!(info.anime_name, "Black Clover");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_anitomo_black_clover_02() {
    let path = PathBuf::from("[AniTomo] Black Clover - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_anitomo_black_clover_03() {
    let path = PathBuf::from("[AniTomo] Black Clover - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_anitomo_black_clover_04() {
    let path = PathBuf::from("[AniTomo] Black Clover - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_anitomo_black_clover_05() {
    let path = PathBuf::from("[AniTomo] Black Clover - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_anitomo_black_clover_06() {
    let path = PathBuf::from("[AniTomo] Black Clover - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_anitomo_black_clover_07() {
    let path = PathBuf::from("[AniTomo] Black Clover - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_anitomo_black_clover_08() {
    let path = PathBuf::from("[AniTomo] Black Clover - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_anitomo_black_clover_09() {
    let path = PathBuf::from("[AniTomo] Black Clover - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_anitomo_black_clover_10() {
    let path = PathBuf::from("[AniTomo] Black Clover - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// AniLoad tests
// ============================================================================

#[test]
fn test_parse_aniload_sword_art_online_alicization_01() {
    let path = PathBuf::from("[AniLoad] Sword Art Online Alicization - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AniLoad format");
    assert_eq!(info.publisher, "AniLoad");
    assert_eq!(info.anime_name, "Sword Art Online Alicization");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_aniload_sword_art_online_alicization_02() {
    let path = PathBuf::from("[AniLoad] Sword Art Online Alicization - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_aniload_sword_art_online_alicization_03() {
    let path = PathBuf::from("[AniLoad] Sword Art Online Alicization - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_aniload_sword_art_online_alicization_04() {
    let path = PathBuf::from("[AniLoad] Sword Art Online Alicization - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_aniload_sword_art_online_alicization_05() {
    let path = PathBuf::from("[AniLoad] Sword Art Online Alicization - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_aniload_sword_art_online_alicization_06() {
    let path = PathBuf::from("[AniLoad] Sword Art Online Alicization - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_aniload_sword_art_online_alicization_07() {
    let path = PathBuf::from("[AniLoad] Sword Art Online Alicization - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_aniload_sword_art_online_alicization_08() {
    let path = PathBuf::from("[AniLoad] Sword Art Online Alicization - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_aniload_sword_art_online_alicization_09() {
    let path = PathBuf::from("[AniLoad] Sword Art Online Alicization - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_aniload_sword_art_online_alicization_10() {
    let path = PathBuf::from("[AniLoad] Sword Art Online Alicization - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_aniload_haikyu_01() {
    let path = PathBuf::from("[AniLoad] Haikyu!! - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AniLoad format");
    assert_eq!(info.publisher, "AniLoad");
    assert_eq!(info.anime_name, "Haikyu!!");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_aniload_haikyu_02() {
    let path = PathBuf::from("[AniLoad] Haikyu!! - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_aniload_haikyu_03() {
    let path = PathBuf::from("[AniLoad] Haikyu!! - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_aniload_haikyu_04() {
    let path = PathBuf::from("[AniLoad] Haikyu!! - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_aniload_haikyu_05() {
    let path = PathBuf::from("[AniLoad] Haikyu!! - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_aniload_haikyu_06() {
    let path = PathBuf::from("[AniLoad] Haikyu!! - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_aniload_haikyu_07() {
    let path = PathBuf::from("[AniLoad] Haikyu!! - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_aniload_haikyu_08() {
    let path = PathBuf::from("[AniLoad] Haikyu!! - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_aniload_haikyu_09() {
    let path = PathBuf::from("[AniLoad] Haikyu!! - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_aniload_haikyu_10() {
    let path = PathBuf::from("[AniLoad] Haikyu!! - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// Kitsunekko tests
// ============================================================================

#[test]
fn test_parse_kitsunekko_code_geass_01() {
    let path = PathBuf::from("[Kitsunekko] Code Geass - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Kitsunekko format");
    assert_eq!(info.publisher, "Kitsunekko");
    assert_eq!(info.anime_name, "Code Geass");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_kitsunekko_code_geass_02() {
    let path = PathBuf::from("[Kitsunekko] Code Geass - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_kitsunekko_code_geass_03() {
    let path = PathBuf::from("[Kitsunekko] Code Geass - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_kitsunekko_code_geass_04() {
    let path = PathBuf::from("[Kitsunekko] Code Geass - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_kitsunekko_code_geass_05() {
    let path = PathBuf::from("[Kitsunekko] Code Geass - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_kitsunekko_code_geass_06() {
    let path = PathBuf::from("[Kitsunekko] Code Geass - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_kitsunekko_code_geass_07() {
    let path = PathBuf::from("[Kitsunekko] Code Geass - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_kitsunekko_code_geass_08() {
    let path = PathBuf::from("[Kitsunekko] Code Geass - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_kitsunekko_code_geass_09() {
    let path = PathBuf::from("[Kitsunekko] Code Geass - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_kitsunekko_code_geass_10() {
    let path = PathBuf::from("[Kitsunekko] Code Geass - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_kitsunekko_fate_stay_night_01() {
    let path = PathBuf::from("[Kitsunekko] Fate Stay Night - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Kitsunekko format");
    assert_eq!(info.publisher, "Kitsunekko");
    assert_eq!(info.anime_name, "Fate Stay Night");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_kitsunekko_fate_stay_night_02() {
    let path = PathBuf::from("[Kitsunekko] Fate Stay Night - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_kitsunekko_fate_stay_night_03() {
    let path = PathBuf::from("[Kitsunekko] Fate Stay Night - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_kitsunekko_fate_stay_night_04() {
    let path = PathBuf::from("[Kitsunekko] Fate Stay Night - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_kitsunekko_fate_stay_night_05() {
    let path = PathBuf::from("[Kitsunekko] Fate Stay Night - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_kitsunekko_fate_stay_night_06() {
    let path = PathBuf::from("[Kitsunekko] Fate Stay Night - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_kitsunekko_fate_stay_night_07() {
    let path = PathBuf::from("[Kitsunekko] Fate Stay Night - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_kitsunekko_fate_stay_night_08() {
    let path = PathBuf::from("[Kitsunekko] Fate Stay Night - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_kitsunekko_fate_stay_night_09() {
    let path = PathBuf::from("[Kitsunekko] Fate Stay Night - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_kitsunekko_fate_stay_night_10() {
    let path = PathBuf::from("[Kitsunekko] Fate Stay Night - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// AnimeSenpai tests
// ============================================================================

#[test]
fn test_parse_animesenpai_steins_gate_01() {
    let path = PathBuf::from("[AnimeSenpai] Steins Gate - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AnimeSenpai format");
    assert_eq!(info.publisher, "AnimeSenpai");
    assert_eq!(info.anime_name, "Steins Gate");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_animesenpai_steins_gate_02() {
    let path = PathBuf::from("[AnimeSenpai] Steins Gate - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_animesenpai_steins_gate_03() {
    let path = PathBuf::from("[AnimeSenpai] Steins Gate - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_animesenpai_steins_gate_04() {
    let path = PathBuf::from("[AnimeSenpai] Steins Gate - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_animesenpai_steins_gate_05() {
    let path = PathBuf::from("[AnimeSenpai] Steins Gate - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_animesenpai_steins_gate_06() {
    let path = PathBuf::from("[AnimeSenpai] Steins Gate - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_animesenpai_steins_gate_07() {
    let path = PathBuf::from("[AnimeSenpai] Steins Gate - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_animesenpai_steins_gate_08() {
    let path = PathBuf::from("[AnimeSenpai] Steins Gate - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_animesenpai_steins_gate_09() {
    let path = PathBuf::from("[AnimeSenpai] Steins Gate - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_animesenpai_steins_gate_10() {
    let path = PathBuf::from("[AnimeSenpai] Steins Gate - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_animesenpai_clannad_01() {
    let path = PathBuf::from("[AnimeSenpai] Clannad - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AnimeSenpai format");
    assert_eq!(info.publisher, "AnimeSenpai");
    assert_eq!(info.anime_name, "Clannad");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_animesenpai_clannad_02() {
    let path = PathBuf::from("[AnimeSenpai] Clannad - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_animesenpai_clannad_03() {
    let path = PathBuf::from("[AnimeSenpai] Clannad - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_animesenpai_clannad_04() {
    let path = PathBuf::from("[AnimeSenpai] Clannad - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_animesenpai_clannad_05() {
    let path = PathBuf::from("[AnimeSenpai] Clannad - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_animesenpai_clannad_06() {
    let path = PathBuf::from("[AnimeSenpai] Clannad - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_animesenpai_clannad_07() {
    let path = PathBuf::from("[AnimeSenpai] Clannad - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_animesenpai_clannad_08() {
    let path = PathBuf::from("[AnimeSenpai] Clannad - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_animesenpai_clannad_09() {
    let path = PathBuf::from("[AnimeSenpai] Clannad - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_animesenpai_clannad_10() {
    let path = PathBuf::from("[AnimeSenpai] Clannad - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// AnimeNexus tests
// ============================================================================

#[test]
fn test_parse_animenexus_hunter_x_hunter_01() {
    let path = PathBuf::from("[AnimeNexus] Hunter x Hunter - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AnimeNexus format");
    assert_eq!(info.publisher, "AnimeNexus");
    assert_eq!(info.anime_name, "Hunter x Hunter");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_animenexus_hunter_x_hunter_02() {
    let path = PathBuf::from("[AnimeNexus] Hunter x Hunter - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_animenexus_hunter_x_hunter_03() {
    let path = PathBuf::from("[AnimeNexus] Hunter x Hunter - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_animenexus_hunter_x_hunter_04() {
    let path = PathBuf::from("[AnimeNexus] Hunter x Hunter - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_animenexus_hunter_x_hunter_05() {
    let path = PathBuf::from("[AnimeNexus] Hunter x Hunter - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_animenexus_hunter_x_hunter_06() {
    let path = PathBuf::from("[AnimeNexus] Hunter x Hunter - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_animenexus_hunter_x_hunter_07() {
    let path = PathBuf::from("[AnimeNexus] Hunter x Hunter - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_animenexus_hunter_x_hunter_08() {
    let path = PathBuf::from("[AnimeNexus] Hunter x Hunter - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_animenexus_hunter_x_hunter_09() {
    let path = PathBuf::from("[AnimeNexus] Hunter x Hunter - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_animenexus_hunter_x_hunter_10() {
    let path = PathBuf::from("[AnimeNexus] Hunter x Hunter - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_animenexus_one_piece_01() {
    let path = PathBuf::from("[AnimeNexus] One Piece - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AnimeNexus format");
    assert_eq!(info.publisher, "AnimeNexus");
    assert_eq!(info.anime_name, "One Piece");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_animenexus_one_piece_02() {
    let path = PathBuf::from("[AnimeNexus] One Piece - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_animenexus_one_piece_03() {
    let path = PathBuf::from("[AnimeNexus] One Piece - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_animenexus_one_piece_04() {
    let path = PathBuf::from("[AnimeNexus] One Piece - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_animenexus_one_piece_05() {
    let path = PathBuf::from("[AnimeNexus] One Piece - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_animenexus_one_piece_06() {
    let path = PathBuf::from("[AnimeNexus] One Piece - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_animenexus_one_piece_07() {
    let path = PathBuf::from("[AnimeNexus] One Piece - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_animenexus_one_piece_08() {
    let path = PathBuf::from("[AnimeNexus] One Piece - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_animenexus_one_piece_09() {
    let path = PathBuf::from("[AnimeNexus] One Piece - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_animenexus_one_piece_10() {
    let path = PathBuf::from("[AnimeNexus] One Piece - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// AnimeGo tests
// ============================================================================

#[test]
fn test_parse_animego_parasyte_01() {
    let path = PathBuf::from("[AnimeGo] Parasyte - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AnimeGo format");
    assert_eq!(info.publisher, "AnimeGo");
    assert_eq!(info.anime_name, "Parasyte");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_animego_parasyte_02() {
    let path = PathBuf::from("[AnimeGo] Parasyte - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_animego_parasyte_03() {
    let path = PathBuf::from("[AnimeGo] Parasyte - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_animego_parasyte_04() {
    let path = PathBuf::from("[AnimeGo] Parasyte - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_animego_parasyte_05() {
    let path = PathBuf::from("[AnimeGo] Parasyte - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_animego_parasyte_06() {
    let path = PathBuf::from("[AnimeGo] Parasyte - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_animego_parasyte_07() {
    let path = PathBuf::from("[AnimeGo] Parasyte - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_animego_parasyte_08() {
    let path = PathBuf::from("[AnimeGo] Parasyte - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_animego_parasyte_09() {
    let path = PathBuf::from("[AnimeGo] Parasyte - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_animego_parasyte_10() {
    let path = PathBuf::from("[AnimeGo] Parasyte - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_animego_assassination_classroom_01() {
    let path = PathBuf::from("[AnimeGo] Assassination Classroom - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AnimeGo format");
    assert_eq!(info.publisher, "AnimeGo");
    assert_eq!(info.anime_name, "Assassination Classroom");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_animego_assassination_classroom_02() {
    let path = PathBuf::from("[AnimeGo] Assassination Classroom - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_animego_assassination_classroom_03() {
    let path = PathBuf::from("[AnimeGo] Assassination Classroom - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_animego_assassination_classroom_04() {
    let path = PathBuf::from("[AnimeGo] Assassination Classroom - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_animego_assassination_classroom_05() {
    let path = PathBuf::from("[AnimeGo] Assassination Classroom - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_animego_assassination_classroom_06() {
    let path = PathBuf::from("[AnimeGo] Assassination Classroom - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_animego_assassination_classroom_07() {
    let path = PathBuf::from("[AnimeGo] Assassination Classroom - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_animego_assassination_classroom_08() {
    let path = PathBuf::from("[AnimeGo] Assassination Classroom - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_animego_assassination_classroom_09() {
    let path = PathBuf::from("[AnimeGo] Assassination Classroom - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_animego_assassination_classroom_10() {
    let path = PathBuf::from("[AnimeGo] Assassination Classroom - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// AnimeBlob tests
// ============================================================================

#[test]
fn test_parse_animeblob_mushishi_01() {
    let path = PathBuf::from("[AnimeBlob] Mushishi - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AnimeBlob format");
    assert_eq!(info.publisher, "AnimeBlob");
    assert_eq!(info.anime_name, "Mushishi");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_animeblob_mushishi_02() {
    let path = PathBuf::from("[AnimeBlob] Mushishi - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_animeblob_mushishi_03() {
    let path = PathBuf::from("[AnimeBlob] Mushishi - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_animeblob_mushishi_04() {
    let path = PathBuf::from("[AnimeBlob] Mushishi - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_animeblob_mushishi_05() {
    let path = PathBuf::from("[AnimeBlob] Mushishi - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_animeblob_mushishi_06() {
    let path = PathBuf::from("[AnimeBlob] Mushishi - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_animeblob_mushishi_07() {
    let path = PathBuf::from("[AnimeBlob] Mushishi - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_animeblob_mushishi_08() {
    let path = PathBuf::from("[AnimeBlob] Mushishi - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_animeblob_mushishi_09() {
    let path = PathBuf::from("[AnimeBlob] Mushishi - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_animeblob_mushishi_10() {
    let path = PathBuf::from("[AnimeBlob] Mushishi - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_animeblob_5_centimeters_per_second_01() {
    let path = PathBuf::from("[AnimeBlob] 5 Centimeters Per Second - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle AnimeBlob format");
    assert_eq!(info.publisher, "AnimeBlob");
    assert_eq!(info.anime_name, "5 Centimeters Per Second");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_animeblob_5_centimeters_per_second_02() {
    let path = PathBuf::from("[AnimeBlob] 5 Centimeters Per Second - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_animeblob_5_centimeters_per_second_03() {
    let path = PathBuf::from("[AnimeBlob] 5 Centimeters Per Second - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_animeblob_5_centimeters_per_second_04() {
    let path = PathBuf::from("[AnimeBlob] 5 Centimeters Per Second - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_animeblob_5_centimeters_per_second_05() {
    let path = PathBuf::from("[AnimeBlob] 5 Centimeters Per Second - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_animeblob_5_centimeters_per_second_06() {
    let path = PathBuf::from("[AnimeBlob] 5 Centimeters Per Second - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_animeblob_5_centimeters_per_second_07() {
    let path = PathBuf::from("[AnimeBlob] 5 Centimeters Per Second - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_animeblob_5_centimeters_per_second_08() {
    let path = PathBuf::from("[AnimeBlob] 5 Centimeters Per Second - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_animeblob_5_centimeters_per_second_09() {
    let path = PathBuf::from("[AnimeBlob] 5 Centimeters Per Second - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_animeblob_5_centimeters_per_second_10() {
    let path = PathBuf::from("[AnimeBlob] 5 Centimeters Per Second - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// BDRemux tests
// ============================================================================

#[test]
fn test_parse_bdremux_kanon_01() {
    let path = PathBuf::from("[BDRemux] Kanon - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle BDRemux format");
    assert_eq!(info.publisher, "BDRemux");
    assert_eq!(info.anime_name, "Kanon");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_bdremux_kanon_02() {
    let path = PathBuf::from("[BDRemux] Kanon - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_bdremux_kanon_03() {
    let path = PathBuf::from("[BDRemux] Kanon - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_bdremux_kanon_04() {
    let path = PathBuf::from("[BDRemux] Kanon - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_bdremux_kanon_05() {
    let path = PathBuf::from("[BDRemux] Kanon - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_bdremux_kanon_06() {
    let path = PathBuf::from("[BDRemux] Kanon - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_bdremux_kanon_07() {
    let path = PathBuf::from("[BDRemux] Kanon - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_bdremux_kanon_08() {
    let path = PathBuf::from("[BDRemux] Kanon - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_bdremux_kanon_09() {
    let path = PathBuf::from("[BDRemux] Kanon - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_bdremux_kanon_10() {
    let path = PathBuf::from("[BDRemux] Kanon - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_bdremux_clannad_after_story_01() {
    let path = PathBuf::from("[BDRemux] Clannad After Story - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle BDRemux format");
    assert_eq!(info.publisher, "BDRemux");
    assert_eq!(info.anime_name, "Clannad After Story");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_bdremux_clannad_after_story_02() {
    let path = PathBuf::from("[BDRemux] Clannad After Story - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_bdremux_clannad_after_story_03() {
    let path = PathBuf::from("[BDRemux] Clannad After Story - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_bdremux_clannad_after_story_04() {
    let path = PathBuf::from("[BDRemux] Clannad After Story - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_bdremux_clannad_after_story_05() {
    let path = PathBuf::from("[BDRemux] Clannad After Story - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_bdremux_clannad_after_story_06() {
    let path = PathBuf::from("[BDRemux] Clannad After Story - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_bdremux_clannad_after_story_07() {
    let path = PathBuf::from("[BDRemux] Clannad After Story - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_bdremux_clannad_after_story_08() {
    let path = PathBuf::from("[BDRemux] Clannad After Story - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_bdremux_clannad_after_story_09() {
    let path = PathBuf::from("[BDRemux] Clannad After Story - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_bdremux_clannad_after_story_10() {
    let path = PathBuf::from("[BDRemux] Clannad After Story - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// DualAudio tests
// ============================================================================

#[test]
fn test_parse_dualaudio_attack_on_titan_s4_01() {
    let path = PathBuf::from("[DualAudio] Attack on Titan S4 - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle DualAudio format");
    assert_eq!(info.publisher, "DualAudio");
    assert_eq!(info.anime_name, "Attack on Titan S4");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_dualaudio_attack_on_titan_s4_02() {
    let path = PathBuf::from("[DualAudio] Attack on Titan S4 - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_dualaudio_attack_on_titan_s4_03() {
    let path = PathBuf::from("[DualAudio] Attack on Titan S4 - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_dualaudio_attack_on_titan_s4_04() {
    let path = PathBuf::from("[DualAudio] Attack on Titan S4 - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_dualaudio_attack_on_titan_s4_05() {
    let path = PathBuf::from("[DualAudio] Attack on Titan S4 - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_dualaudio_attack_on_titan_s4_06() {
    let path = PathBuf::from("[DualAudio] Attack on Titan S4 - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_dualaudio_attack_on_titan_s4_07() {
    let path = PathBuf::from("[DualAudio] Attack on Titan S4 - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_dualaudio_attack_on_titan_s4_08() {
    let path = PathBuf::from("[DualAudio] Attack on Titan S4 - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_dualaudio_attack_on_titan_s4_09() {
    let path = PathBuf::from("[DualAudio] Attack on Titan S4 - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_dualaudio_attack_on_titan_s4_10() {
    let path = PathBuf::from("[DualAudio] Attack on Titan S4 - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_dualaudio_vinland_saga_01() {
    let path = PathBuf::from("[DualAudio] Vinland Saga - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle DualAudio format");
    assert_eq!(info.publisher, "DualAudio");
    assert_eq!(info.anime_name, "Vinland Saga");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_dualaudio_vinland_saga_02() {
    let path = PathBuf::from("[DualAudio] Vinland Saga - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_dualaudio_vinland_saga_03() {
    let path = PathBuf::from("[DualAudio] Vinland Saga - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_dualaudio_vinland_saga_04() {
    let path = PathBuf::from("[DualAudio] Vinland Saga - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_dualaudio_vinland_saga_05() {
    let path = PathBuf::from("[DualAudio] Vinland Saga - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_dualaudio_vinland_saga_06() {
    let path = PathBuf::from("[DualAudio] Vinland Saga - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_dualaudio_vinland_saga_07() {
    let path = PathBuf::from("[DualAudio] Vinland Saga - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_dualaudio_vinland_saga_08() {
    let path = PathBuf::from("[DualAudio] Vinland Saga - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_dualaudio_vinland_saga_09() {
    let path = PathBuf::from("[DualAudio] Vinland Saga - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_dualaudio_vinland_saga_10() {
    let path = PathBuf::from("[DualAudio] Vinland Saga - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// Extorrents tests
// ============================================================================

#[test]
fn test_parse_extorrents_shingeki_no_kyojin_s4_01() {
    let path = PathBuf::from("[Extorrents] Shingeki no Kyojin S4 - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Extorrents format");
    assert_eq!(info.publisher, "Extorrents");
    assert_eq!(info.anime_name, "Shingeki no Kyojin S4");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_extorrents_shingeki_no_kyojin_s4_02() {
    let path = PathBuf::from("[Extorrents] Shingeki no Kyojin S4 - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_extorrents_shingeki_no_kyojin_s4_03() {
    let path = PathBuf::from("[Extorrents] Shingeki no Kyojin S4 - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_extorrents_shingeki_no_kyojin_s4_04() {
    let path = PathBuf::from("[Extorrents] Shingeki no Kyojin S4 - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_extorrents_shingeki_no_kyojin_s4_05() {
    let path = PathBuf::from("[Extorrents] Shingeki no Kyojin S4 - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_extorrents_shingeki_no_kyojin_s4_06() {
    let path = PathBuf::from("[Extorrents] Shingeki no Kyojin S4 - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_extorrents_shingeki_no_kyojin_s4_07() {
    let path = PathBuf::from("[Extorrents] Shingeki no Kyojin S4 - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_extorrents_shingeki_no_kyojin_s4_08() {
    let path = PathBuf::from("[Extorrents] Shingeki no Kyojin S4 - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_extorrents_shingeki_no_kyojin_s4_09() {
    let path = PathBuf::from("[Extorrents] Shingeki no Kyojin S4 - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_extorrents_shingeki_no_kyojin_s4_10() {
    let path = PathBuf::from("[Extorrents] Shingeki no Kyojin S4 - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_extorrents_kimetsu_no_yaiba_01() {
    let path = PathBuf::from("[Extorrents] Kimetsu no Yaiba - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Extorrents format");
    assert_eq!(info.publisher, "Extorrents");
    assert_eq!(info.anime_name, "Kimetsu no Yaiba");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_extorrents_kimetsu_no_yaiba_02() {
    let path = PathBuf::from("[Extorrents] Kimetsu no Yaiba - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_extorrents_kimetsu_no_yaiba_03() {
    let path = PathBuf::from("[Extorrents] Kimetsu no Yaiba - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_extorrents_kimetsu_no_yaiba_04() {
    let path = PathBuf::from("[Extorrents] Kimetsu no Yaiba - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_extorrents_kimetsu_no_yaiba_05() {
    let path = PathBuf::from("[Extorrents] Kimetsu no Yaiba - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_extorrents_kimetsu_no_yaiba_06() {
    let path = PathBuf::from("[Extorrents] Kimetsu no Yaiba - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_extorrents_kimetsu_no_yaiba_07() {
    let path = PathBuf::from("[Extorrents] Kimetsu no Yaiba - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_extorrents_kimetsu_no_yaiba_08() {
    let path = PathBuf::from("[Extorrents] Kimetsu no Yaiba - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_extorrents_kimetsu_no_yaiba_09() {
    let path = PathBuf::from("[Extorrents] Kimetsu no Yaiba - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_extorrents_kimetsu_no_yaiba_10() {
    let path = PathBuf::from("[Extorrents] Kimetsu no Yaiba - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// Erai-Raws tests
// ============================================================================

#[test]
fn test_parse_erai_raws_date_a_live_01() {
    let path = PathBuf::from("[Erai-Raws] Date A Live - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Erai-Raws format");
    assert_eq!(info.publisher, "Erai-Raws");
    assert_eq!(info.anime_name, "Date A Live");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_erai_raws_date_a_live_02() {
    let path = PathBuf::from("[Erai-Raws] Date A Live - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_erai_raws_date_a_live_03() {
    let path = PathBuf::from("[Erai-Raws] Date A Live - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_erai_raws_date_a_live_04() {
    let path = PathBuf::from("[Erai-Raws] Date A Live - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_erai_raws_date_a_live_05() {
    let path = PathBuf::from("[Erai-Raws] Date A Live - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_erai_raws_date_a_live_06() {
    let path = PathBuf::from("[Erai-Raws] Date A Live - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_erai_raws_date_a_live_07() {
    let path = PathBuf::from("[Erai-Raws] Date A Live - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_erai_raws_date_a_live_08() {
    let path = PathBuf::from("[Erai-Raws] Date A Live - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_erai_raws_date_a_live_09() {
    let path = PathBuf::from("[Erai-Raws] Date A Live - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_erai_raws_date_a_live_10() {
    let path = PathBuf::from("[Erai-Raws] Date A Live - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_erai_raws_re_zero_s2_01() {
    let path = PathBuf::from("[Erai-Raws] Re Zero S2 - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Erai-Raws format");
    assert_eq!(info.publisher, "Erai-Raws");
    assert_eq!(info.anime_name, "Re Zero S2");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_erai_raws_re_zero_s2_02() {
    let path = PathBuf::from("[Erai-Raws] Re Zero S2 - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_erai_raws_re_zero_s2_03() {
    let path = PathBuf::from("[Erai-Raws] Re Zero S2 - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_erai_raws_re_zero_s2_04() {
    let path = PathBuf::from("[Erai-Raws] Re Zero S2 - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_erai_raws_re_zero_s2_05() {
    let path = PathBuf::from("[Erai-Raws] Re Zero S2 - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_erai_raws_re_zero_s2_06() {
    let path = PathBuf::from("[Erai-Raws] Re Zero S2 - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_erai_raws_re_zero_s2_07() {
    let path = PathBuf::from("[Erai-Raws] Re Zero S2 - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_erai_raws_re_zero_s2_08() {
    let path = PathBuf::from("[Erai-Raws] Re Zero S2 - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_erai_raws_re_zero_s2_09() {
    let path = PathBuf::from("[Erai-Raws] Re Zero S2 - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_erai_raws_re_zero_s2_10() {
    let path = PathBuf::from("[Erai-Raws] Re Zero S2 - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// SubsPlease tests
// ============================================================================

#[test]
fn test_parse_subsplease_wonder_egg_priority_01() {
    let path = PathBuf::from("[SubsPlease] Wonder Egg Priority - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle SubsPlease format");
    assert_eq!(info.publisher, "SubsPlease");
    assert_eq!(info.anime_name, "Wonder Egg Priority");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_subsplease_wonder_egg_priority_02() {
    let path = PathBuf::from("[SubsPlease] Wonder Egg Priority - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_subsplease_wonder_egg_priority_03() {
    let path = PathBuf::from("[SubsPlease] Wonder Egg Priority - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_subsplease_wonder_egg_priority_04() {
    let path = PathBuf::from("[SubsPlease] Wonder Egg Priority - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_subsplease_wonder_egg_priority_05() {
    let path = PathBuf::from("[SubsPlease] Wonder Egg Priority - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_subsplease_wonder_egg_priority_06() {
    let path = PathBuf::from("[SubsPlease] Wonder Egg Priority - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_subsplease_wonder_egg_priority_07() {
    let path = PathBuf::from("[SubsPlease] Wonder Egg Priority - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_subsplease_wonder_egg_priority_08() {
    let path = PathBuf::from("[SubsPlease] Wonder Egg Priority - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_subsplease_wonder_egg_priority_09() {
    let path = PathBuf::from("[SubsPlease] Wonder Egg Priority - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_subsplease_wonder_egg_priority_10() {
    let path = PathBuf::from("[SubsPlease] Wonder Egg Priority - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_subsplease_tensei_shitara_slime_datta_ken_01() {
    let path = PathBuf::from("[SubsPlease] Tensei shitara Slime Datta Ken - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle SubsPlease format");
    assert_eq!(info.publisher, "SubsPlease");
    assert_eq!(info.anime_name, "Tensei shitara Slime Datta Ken");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_subsplease_tensei_shitara_slime_datta_ken_02() {
    let path = PathBuf::from("[SubsPlease] Tensei shitara Slime Datta Ken - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_subsplease_tensei_shitara_slime_datta_ken_03() {
    let path = PathBuf::from("[SubsPlease] Tensei shitara Slime Datta Ken - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_subsplease_tensei_shitara_slime_datta_ken_04() {
    let path = PathBuf::from("[SubsPlease] Tensei shitara Slime Datta Ken - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_subsplease_tensei_shitara_slime_datta_ken_05() {
    let path = PathBuf::from("[SubsPlease] Tensei shitara Slime Datta Ken - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_subsplease_tensei_shitara_slime_datta_ken_06() {
    let path = PathBuf::from("[SubsPlease] Tensei shitara Slime Datta Ken - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_subsplease_tensei_shitara_slime_datta_ken_07() {
    let path = PathBuf::from("[SubsPlease] Tensei shitara Slime Datta Ken - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_subsplease_tensei_shitara_slime_datta_ken_08() {
    let path = PathBuf::from("[SubsPlease] Tensei shitara Slime Datta Ken - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_subsplease_tensei_shitara_slime_datta_ken_09() {
    let path = PathBuf::from("[SubsPlease] Tensei shitara Slime Datta Ken - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_subsplease_tensei_shitara_slime_datta_ken_10() {
    let path = PathBuf::from("[SubsPlease] Tensei shitara Slime Datta Ken - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// ASW tests
// ============================================================================

#[test]
fn test_parse_asw_to_your_eternity_01() {
    let path = PathBuf::from("[ASW] To Your Eternity - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle ASW format");
    assert_eq!(info.publisher, "ASW");
    assert_eq!(info.anime_name, "To Your Eternity");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_asw_to_your_eternity_02() {
    let path = PathBuf::from("[ASW] To Your Eternity - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_asw_to_your_eternity_03() {
    let path = PathBuf::from("[ASW] To Your Eternity - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_asw_to_your_eternity_04() {
    let path = PathBuf::from("[ASW] To Your Eternity - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_asw_to_your_eternity_05() {
    let path = PathBuf::from("[ASW] To Your Eternity - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_asw_to_your_eternity_06() {
    let path = PathBuf::from("[ASW] To Your Eternity - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_asw_to_your_eternity_07() {
    let path = PathBuf::from("[ASW] To Your Eternity - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_asw_to_your_eternity_08() {
    let path = PathBuf::from("[ASW] To Your Eternity - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_asw_to_your_eternity_09() {
    let path = PathBuf::from("[ASW] To Your Eternity - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_asw_to_your_eternity_10() {
    let path = PathBuf::from("[ASW] To Your Eternity - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_asw_odd_taxi_01() {
    let path = PathBuf::from("[ASW] Odd Taxi - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle ASW format");
    assert_eq!(info.publisher, "ASW");
    assert_eq!(info.anime_name, "Odd Taxi");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_asw_odd_taxi_02() {
    let path = PathBuf::from("[ASW] Odd Taxi - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_asw_odd_taxi_03() {
    let path = PathBuf::from("[ASW] Odd Taxi - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_asw_odd_taxi_04() {
    let path = PathBuf::from("[ASW] Odd Taxi - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_asw_odd_taxi_05() {
    let path = PathBuf::from("[ASW] Odd Taxi - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_asw_odd_taxi_06() {
    let path = PathBuf::from("[ASW] Odd Taxi - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_asw_odd_taxi_07() {
    let path = PathBuf::from("[ASW] Odd Taxi - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_asw_odd_taxi_08() {
    let path = PathBuf::from("[ASW] Odd Taxi - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_asw_odd_taxi_09() {
    let path = PathBuf::from("[ASW] Odd Taxi - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_asw_odd_taxi_10() {
    let path = PathBuf::from("[ASW] Odd Taxi - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// Judas tests
// ============================================================================

#[test]
fn test_parse_judas_dragon_ball_super_01() {
    let path = PathBuf::from("[Judas] Dragon Ball Super - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Judas format");
    assert_eq!(info.publisher, "Judas");
    assert_eq!(info.anime_name, "Dragon Ball Super");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_judas_dragon_ball_super_02() {
    let path = PathBuf::from("[Judas] Dragon Ball Super - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_judas_dragon_ball_super_03() {
    let path = PathBuf::from("[Judas] Dragon Ball Super - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_judas_dragon_ball_super_04() {
    let path = PathBuf::from("[Judas] Dragon Ball Super - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_judas_dragon_ball_super_05() {
    let path = PathBuf::from("[Judas] Dragon Ball Super - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_judas_dragon_ball_super_06() {
    let path = PathBuf::from("[Judas] Dragon Ball Super - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_judas_dragon_ball_super_07() {
    let path = PathBuf::from("[Judas] Dragon Ball Super - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_judas_dragon_ball_super_08() {
    let path = PathBuf::from("[Judas] Dragon Ball Super - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_judas_dragon_ball_super_09() {
    let path = PathBuf::from("[Judas] Dragon Ball Super - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_judas_dragon_ball_super_10() {
    let path = PathBuf::from("[Judas] Dragon Ball Super - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_judas_boku_no_hero_academia_01() {
    let path = PathBuf::from("[Judas] Boku no Hero Academia - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Judas format");
    assert_eq!(info.publisher, "Judas");
    assert_eq!(info.anime_name, "Boku no Hero Academia");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_judas_boku_no_hero_academia_02() {
    let path = PathBuf::from("[Judas] Boku no Hero Academia - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_judas_boku_no_hero_academia_03() {
    let path = PathBuf::from("[Judas] Boku no Hero Academia - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_judas_boku_no_hero_academia_04() {
    let path = PathBuf::from("[Judas] Boku no Hero Academia - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_judas_boku_no_hero_academia_05() {
    let path = PathBuf::from("[Judas] Boku no Hero Academia - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_judas_boku_no_hero_academia_06() {
    let path = PathBuf::from("[Judas] Boku no Hero Academia - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_judas_boku_no_hero_academia_07() {
    let path = PathBuf::from("[Judas] Boku no Hero Academia - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_judas_boku_no_hero_academia_08() {
    let path = PathBuf::from("[Judas] Boku no Hero Academia - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_judas_boku_no_hero_academia_09() {
    let path = PathBuf::from("[Judas] Boku no Hero Academia - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_judas_boku_no_hero_academia_10() {
    let path = PathBuf::from("[Judas] Boku no Hero Academia - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// Nanikano tests
// ============================================================================

#[test]
fn test_parse_nanikano_shirobako_01() {
    let path = PathBuf::from("[Nanikano] Shirobako - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Nanikano format");
    assert_eq!(info.publisher, "Nanikano");
    assert_eq!(info.anime_name, "Shirobako");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_nanikano_shirobako_02() {
    let path = PathBuf::from("[Nanikano] Shirobako - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_nanikano_shirobako_03() {
    let path = PathBuf::from("[Nanikano] Shirobako - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_nanikano_shirobako_04() {
    let path = PathBuf::from("[Nanikano] Shirobako - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_nanikano_shirobako_05() {
    let path = PathBuf::from("[Nanikano] Shirobako - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_nanikano_shirobako_06() {
    let path = PathBuf::from("[Nanikano] Shirobako - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_nanikano_shirobako_07() {
    let path = PathBuf::from("[Nanikano] Shirobako - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_nanikano_shirobako_08() {
    let path = PathBuf::from("[Nanikano] Shirobako - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_nanikano_shirobako_09() {
    let path = PathBuf::from("[Nanikano] Shirobako - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_nanikano_shirobako_10() {
    let path = PathBuf::from("[Nanikano] Shirobako - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_nanikano_grand_blue_01() {
    let path = PathBuf::from("[Nanikano] Grand Blue - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Nanikano format");
    assert_eq!(info.publisher, "Nanikano");
    assert_eq!(info.anime_name, "Grand Blue");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_nanikano_grand_blue_02() {
    let path = PathBuf::from("[Nanikano] Grand Blue - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_nanikano_grand_blue_03() {
    let path = PathBuf::from("[Nanikano] Grand Blue - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_nanikano_grand_blue_04() {
    let path = PathBuf::from("[Nanikano] Grand Blue - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_nanikano_grand_blue_05() {
    let path = PathBuf::from("[Nanikano] Grand Blue - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_nanikano_grand_blue_06() {
    let path = PathBuf::from("[Nanikano] Grand Blue - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_nanikano_grand_blue_07() {
    let path = PathBuf::from("[Nanikano] Grand Blue - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_nanikano_grand_blue_08() {
    let path = PathBuf::from("[Nanikano] Grand Blue - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_nanikano_grand_blue_09() {
    let path = PathBuf::from("[Nanikano] Grand Blue - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_nanikano_grand_blue_10() {
    let path = PathBuf::from("[Nanikano] Grand Blue - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// Hi10 tests
// ============================================================================

#[test]
fn test_parse_hi10_sword_art_online_01() {
    let path = PathBuf::from("[Hi10] Sword Art Online - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Hi10 format");
    assert_eq!(info.publisher, "Hi10");
    assert_eq!(info.anime_name, "Sword Art Online");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_hi10_sword_art_online_02() {
    let path = PathBuf::from("[Hi10] Sword Art Online - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_hi10_sword_art_online_03() {
    let path = PathBuf::from("[Hi10] Sword Art Online - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_hi10_sword_art_online_04() {
    let path = PathBuf::from("[Hi10] Sword Art Online - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_hi10_sword_art_online_05() {
    let path = PathBuf::from("[Hi10] Sword Art Online - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_hi10_sword_art_online_06() {
    let path = PathBuf::from("[Hi10] Sword Art Online - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_hi10_sword_art_online_07() {
    let path = PathBuf::from("[Hi10] Sword Art Online - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_hi10_sword_art_online_08() {
    let path = PathBuf::from("[Hi10] Sword Art Online - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_hi10_sword_art_online_09() {
    let path = PathBuf::from("[Hi10] Sword Art Online - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_hi10_sword_art_online_10() {
    let path = PathBuf::from("[Hi10] Sword Art Online - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_hi10_angel_beats_01() {
    let path = PathBuf::from("[Hi10] Angel Beats - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Hi10 format");
    assert_eq!(info.publisher, "Hi10");
    assert_eq!(info.anime_name, "Angel Beats");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_hi10_angel_beats_02() {
    let path = PathBuf::from("[Hi10] Angel Beats - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_hi10_angel_beats_03() {
    let path = PathBuf::from("[Hi10] Angel Beats - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_hi10_angel_beats_04() {
    let path = PathBuf::from("[Hi10] Angel Beats - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_hi10_angel_beats_05() {
    let path = PathBuf::from("[Hi10] Angel Beats - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_hi10_angel_beats_06() {
    let path = PathBuf::from("[Hi10] Angel Beats - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_hi10_angel_beats_07() {
    let path = PathBuf::from("[Hi10] Angel Beats - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_hi10_angel_beats_08() {
    let path = PathBuf::from("[Hi10] Angel Beats - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_hi10_angel_beats_09() {
    let path = PathBuf::from("[Hi10] Angel Beats - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_hi10_angel_beats_10() {
    let path = PathBuf::from("[Hi10] Angel Beats - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// Tushy tests
// ============================================================================

#[test]
fn test_parse_tushy_86_eighty_six_01() {
    let path = PathBuf::from("[Tushy] 86 EIGHTY-SIX - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Tushy format");
    assert_eq!(info.publisher, "Tushy");
    assert_eq!(info.anime_name, "86 EIGHTY-SIX");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_tushy_86_eighty_six_02() {
    let path = PathBuf::from("[Tushy] 86 EIGHTY-SIX - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_tushy_86_eighty_six_03() {
    let path = PathBuf::from("[Tushy] 86 EIGHTY-SIX - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_tushy_86_eighty_six_04() {
    let path = PathBuf::from("[Tushy] 86 EIGHTY-SIX - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_tushy_86_eighty_six_05() {
    let path = PathBuf::from("[Tushy] 86 EIGHTY-SIX - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_tushy_86_eighty_six_06() {
    let path = PathBuf::from("[Tushy] 86 EIGHTY-SIX - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_tushy_86_eighty_six_07() {
    let path = PathBuf::from("[Tushy] 86 EIGHTY-SIX - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_tushy_86_eighty_six_08() {
    let path = PathBuf::from("[Tushy] 86 EIGHTY-SIX - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_tushy_86_eighty_six_09() {
    let path = PathBuf::from("[Tushy] 86 EIGHTY-SIX - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_tushy_86_eighty_six_10() {
    let path = PathBuf::from("[Tushy] 86 EIGHTY-SIX - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_tushy_horimiya_01() {
    let path = PathBuf::from("[Tushy] Horimiya - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Tushy format");
    assert_eq!(info.publisher, "Tushy");
    assert_eq!(info.anime_name, "Horimiya");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_tushy_horimiya_02() {
    let path = PathBuf::from("[Tushy] Horimiya - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_tushy_horimiya_03() {
    let path = PathBuf::from("[Tushy] Horimiya - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_tushy_horimiya_04() {
    let path = PathBuf::from("[Tushy] Horimiya - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_tushy_horimiya_05() {
    let path = PathBuf::from("[Tushy] Horimiya - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_tushy_horimiya_06() {
    let path = PathBuf::from("[Tushy] Horimiya - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_tushy_horimiya_07() {
    let path = PathBuf::from("[Tushy] Horimiya - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_tushy_horimiya_08() {
    let path = PathBuf::from("[Tushy] Horimiya - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_tushy_horimiya_09() {
    let path = PathBuf::from("[Tushy] Horimiya - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_tushy_horimiya_10() {
    let path = PathBuf::from("[Tushy] Horimiya - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// Koinaka tests
// ============================================================================

#[test]
fn test_parse_koinaka_lycoris_recoil_01() {
    let path = PathBuf::from("[Koinaka] Lycoris Recoil - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Koinaka format");
    assert_eq!(info.publisher, "Koinaka");
    assert_eq!(info.anime_name, "Lycoris Recoil");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_koinaka_lycoris_recoil_02() {
    let path = PathBuf::from("[Koinaka] Lycoris Recoil - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_koinaka_lycoris_recoil_03() {
    let path = PathBuf::from("[Koinaka] Lycoris Recoil - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_koinaka_lycoris_recoil_04() {
    let path = PathBuf::from("[Koinaka] Lycoris Recoil - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_koinaka_lycoris_recoil_05() {
    let path = PathBuf::from("[Koinaka] Lycoris Recoil - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_koinaka_lycoris_recoil_06() {
    let path = PathBuf::from("[Koinaka] Lycoris Recoil - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_koinaka_lycoris_recoil_07() {
    let path = PathBuf::from("[Koinaka] Lycoris Recoil - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_koinaka_lycoris_recoil_08() {
    let path = PathBuf::from("[Koinaka] Lycoris Recoil - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_koinaka_lycoris_recoil_09() {
    let path = PathBuf::from("[Koinaka] Lycoris Recoil - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_koinaka_lycoris_recoil_10() {
    let path = PathBuf::from("[Koinaka] Lycoris Recoil - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_koinaka_summer_time_rendering_01() {
    let path = PathBuf::from("[Koinaka] Summer Time Rendering - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Koinaka format");
    assert_eq!(info.publisher, "Koinaka");
    assert_eq!(info.anime_name, "Summer Time Rendering");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_koinaka_summer_time_rendering_02() {
    let path = PathBuf::from("[Koinaka] Summer Time Rendering - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_koinaka_summer_time_rendering_03() {
    let path = PathBuf::from("[Koinaka] Summer Time Rendering - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_koinaka_summer_time_rendering_04() {
    let path = PathBuf::from("[Koinaka] Summer Time Rendering - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_koinaka_summer_time_rendering_05() {
    let path = PathBuf::from("[Koinaka] Summer Time Rendering - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_koinaka_summer_time_rendering_06() {
    let path = PathBuf::from("[Koinaka] Summer Time Rendering - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_koinaka_summer_time_rendering_07() {
    let path = PathBuf::from("[Koinaka] Summer Time Rendering - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_koinaka_summer_time_rendering_08() {
    let path = PathBuf::from("[Koinaka] Summer Time Rendering - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_koinaka_summer_time_rendering_09() {
    let path = PathBuf::from("[Koinaka] Summer Time Rendering - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_koinaka_summer_time_rendering_10() {
    let path = PathBuf::from("[Koinaka] Summer Time Rendering - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// UHA-Matsuri tests
// ============================================================================

#[test]
fn test_parse_uha_matsuri_kaguya_sama_ultra_romantic_01() {
    let path = PathBuf::from("[UHA-Matsuri] Kaguya-sama Ultra Romantic - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle UHA-Matsuri format");
    assert_eq!(info.publisher, "UHA-Matsuri");
    assert_eq!(info.anime_name, "Kaguya-sama Ultra Romantic");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_uha_matsuri_kaguya_sama_ultra_romantic_02() {
    let path = PathBuf::from("[UHA-Matsuri] Kaguya-sama Ultra Romantic - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_uha_matsuri_kaguya_sama_ultra_romantic_03() {
    let path = PathBuf::from("[UHA-Matsuri] Kaguya-sama Ultra Romantic - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_uha_matsuri_kaguya_sama_ultra_romantic_04() {
    let path = PathBuf::from("[UHA-Matsuri] Kaguya-sama Ultra Romantic - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_uha_matsuri_kaguya_sama_ultra_romantic_05() {
    let path = PathBuf::from("[UHA-Matsuri] Kaguya-sama Ultra Romantic - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_uha_matsuri_kaguya_sama_ultra_romantic_06() {
    let path = PathBuf::from("[UHA-Matsuri] Kaguya-sama Ultra Romantic - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_uha_matsuri_kaguya_sama_ultra_romantic_07() {
    let path = PathBuf::from("[UHA-Matsuri] Kaguya-sama Ultra Romantic - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_uha_matsuri_kaguya_sama_ultra_romantic_08() {
    let path = PathBuf::from("[UHA-Matsuri] Kaguya-sama Ultra Romantic - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_uha_matsuri_kaguya_sama_ultra_romantic_09() {
    let path = PathBuf::from("[UHA-Matsuri] Kaguya-sama Ultra Romantic - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_uha_matsuri_kaguya_sama_ultra_romantic_10() {
    let path = PathBuf::from("[UHA-Matsuri] Kaguya-sama Ultra Romantic - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_uha_matsuri_spy_x_family_s2_01() {
    let path = PathBuf::from("[UHA-Matsuri] Spy x Family S2 - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle UHA-Matsuri format");
    assert_eq!(info.publisher, "UHA-Matsuri");
    assert_eq!(info.anime_name, "Spy x Family S2");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_uha_matsuri_spy_x_family_s2_02() {
    let path = PathBuf::from("[UHA-Matsuri] Spy x Family S2 - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_uha_matsuri_spy_x_family_s2_03() {
    let path = PathBuf::from("[UHA-Matsuri] Spy x Family S2 - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_uha_matsuri_spy_x_family_s2_04() {
    let path = PathBuf::from("[UHA-Matsuri] Spy x Family S2 - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_uha_matsuri_spy_x_family_s2_05() {
    let path = PathBuf::from("[UHA-Matsuri] Spy x Family S2 - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_uha_matsuri_spy_x_family_s2_06() {
    let path = PathBuf::from("[UHA-Matsuri] Spy x Family S2 - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_uha_matsuri_spy_x_family_s2_07() {
    let path = PathBuf::from("[UHA-Matsuri] Spy x Family S2 - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_uha_matsuri_spy_x_family_s2_08() {
    let path = PathBuf::from("[UHA-Matsuri] Spy x Family S2 - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_uha_matsuri_spy_x_family_s2_09() {
    let path = PathBuf::from("[UHA-Matsuri] Spy x Family S2 - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_uha_matsuri_spy_x_family_s2_10() {
    let path = PathBuf::from("[UHA-Matsuri] Spy x Family S2 - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// YUU tests
// ============================================================================

#[test]
fn test_parse_yuu_classroom_of_the_elite_01() {
    let path = PathBuf::from("[YUU] Classroom of the Elite - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle YUU format");
    assert_eq!(info.publisher, "YUU");
    assert_eq!(info.anime_name, "Classroom of the Elite");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_yuu_classroom_of_the_elite_02() {
    let path = PathBuf::from("[YUU] Classroom of the Elite - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_yuu_classroom_of_the_elite_03() {
    let path = PathBuf::from("[YUU] Classroom of the Elite - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_yuu_classroom_of_the_elite_04() {
    let path = PathBuf::from("[YUU] Classroom of the Elite - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_yuu_classroom_of_the_elite_05() {
    let path = PathBuf::from("[YUU] Classroom of the Elite - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_yuu_classroom_of_the_elite_06() {
    let path = PathBuf::from("[YUU] Classroom of the Elite - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_yuu_classroom_of_the_elite_07() {
    let path = PathBuf::from("[YUU] Classroom of the Elite - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_yuu_classroom_of_the_elite_08() {
    let path = PathBuf::from("[YUU] Classroom of the Elite - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_yuu_classroom_of_the_elite_09() {
    let path = PathBuf::from("[YUU] Classroom of the Elite - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_yuu_classroom_of_the_elite_10() {
    let path = PathBuf::from("[YUU] Classroom of the Elite - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_yuu_oshi_no_ko_01() {
    let path = PathBuf::from("[YUU] Oshi no Ko - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle YUU format");
    assert_eq!(info.publisher, "YUU");
    assert_eq!(info.anime_name, "Oshi no Ko");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_yuu_oshi_no_ko_02() {
    let path = PathBuf::from("[YUU] Oshi no Ko - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_yuu_oshi_no_ko_03() {
    let path = PathBuf::from("[YUU] Oshi no Ko - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_yuu_oshi_no_ko_04() {
    let path = PathBuf::from("[YUU] Oshi no Ko - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_yuu_oshi_no_ko_05() {
    let path = PathBuf::from("[YUU] Oshi no Ko - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_yuu_oshi_no_ko_06() {
    let path = PathBuf::from("[YUU] Oshi no Ko - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_yuu_oshi_no_ko_07() {
    let path = PathBuf::from("[YUU] Oshi no Ko - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_yuu_oshi_no_ko_08() {
    let path = PathBuf::from("[YUU] Oshi no Ko - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_yuu_oshi_no_ko_09() {
    let path = PathBuf::from("[YUU] Oshi no Ko - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_yuu_oshi_no_ko_10() {
    let path = PathBuf::from("[YUU] Oshi no Ko - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// Viu tests
// ============================================================================

#[test]
fn test_parse_viu_ranking_of_kings_01() {
    let path = PathBuf::from("[Viu] Ranking of Kings - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Viu format");
    assert_eq!(info.publisher, "Viu");
    assert_eq!(info.anime_name, "Ranking of Kings");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_viu_ranking_of_kings_02() {
    let path = PathBuf::from("[Viu] Ranking of Kings - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_viu_ranking_of_kings_03() {
    let path = PathBuf::from("[Viu] Ranking of Kings - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_viu_ranking_of_kings_04() {
    let path = PathBuf::from("[Viu] Ranking of Kings - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_viu_ranking_of_kings_05() {
    let path = PathBuf::from("[Viu] Ranking of Kings - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_viu_ranking_of_kings_06() {
    let path = PathBuf::from("[Viu] Ranking of Kings - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_viu_ranking_of_kings_07() {
    let path = PathBuf::from("[Viu] Ranking of Kings - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_viu_ranking_of_kings_08() {
    let path = PathBuf::from("[Viu] Ranking of Kings - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_viu_ranking_of_kings_09() {
    let path = PathBuf::from("[Viu] Ranking of Kings - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_viu_ranking_of_kings_10() {
    let path = PathBuf::from("[Viu] Ranking of Kings - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_viu_mobuseka_01() {
    let path = PathBuf::from("[Viu] Mobuseka - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle Viu format");
    assert_eq!(info.publisher, "Viu");
    assert_eq!(info.anime_name, "Mobuseka");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_viu_mobuseka_02() {
    let path = PathBuf::from("[Viu] Mobuseka - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_viu_mobuseka_03() {
    let path = PathBuf::from("[Viu] Mobuseka - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_viu_mobuseka_04() {
    let path = PathBuf::from("[Viu] Mobuseka - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_viu_mobuseka_05() {
    let path = PathBuf::from("[Viu] Mobuseka - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_viu_mobuseka_06() {
    let path = PathBuf::from("[Viu] Mobuseka - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_viu_mobuseka_07() {
    let path = PathBuf::from("[Viu] Mobuseka - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_viu_mobuseka_08() {
    let path = PathBuf::from("[Viu] Mobuseka - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_viu_mobuseka_09() {
    let path = PathBuf::from("[Viu] Mobuseka - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_viu_mobuseka_10() {
    let path = PathBuf::from("[Viu] Mobuseka - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}
// ============================================================================
// TenRaw tests
// ============================================================================

#[test]
fn test_parse_tenraw_my_dress_up_darling_01() {
    let path = PathBuf::from("[TenRaw] My Dress Up Darling - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle TenRaw format");
    assert_eq!(info.publisher, "TenRaw");
    assert_eq!(info.anime_name, "My Dress Up Darling");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_tenraw_my_dress_up_darling_02() {
    let path = PathBuf::from("[TenRaw] My Dress Up Darling - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_tenraw_my_dress_up_darling_03() {
    let path = PathBuf::from("[TenRaw] My Dress Up Darling - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_tenraw_my_dress_up_darling_04() {
    let path = PathBuf::from("[TenRaw] My Dress Up Darling - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_tenraw_my_dress_up_darling_05() {
    let path = PathBuf::from("[TenRaw] My Dress Up Darling - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_tenraw_my_dress_up_darling_06() {
    let path = PathBuf::from("[TenRaw] My Dress Up Darling - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_tenraw_my_dress_up_darling_07() {
    let path = PathBuf::from("[TenRaw] My Dress Up Darling - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_tenraw_my_dress_up_darling_08() {
    let path = PathBuf::from("[TenRaw] My Dress Up Darling - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_tenraw_my_dress_up_darling_09() {
    let path = PathBuf::from("[TenRaw] My Dress Up Darling - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_tenraw_my_dress_up_darling_10() {
    let path = PathBuf::from("[TenRaw] My Dress Up Darling - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}

#[test]
fn test_parse_tenraw_tsuki_ga_michibiku_01() {
    let path = PathBuf::from("[TenRaw] Tsuki ga Michibiku - 01 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Parser should handle TenRaw format");
    assert_eq!(info.publisher, "TenRaw");
    assert_eq!(info.anime_name, "Tsuki ga Michibiku");
    assert_eq!(info.episode, "01");
}

#[test]
fn test_parse_tenraw_tsuki_ga_michibiku_02() {
    let path = PathBuf::from("[TenRaw] Tsuki ga Michibiku - 02 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "02");
}

#[test]
fn test_parse_tenraw_tsuki_ga_michibiku_03() {
    let path = PathBuf::from("[TenRaw] Tsuki ga Michibiku - 03 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "03");
}

#[test]
fn test_parse_tenraw_tsuki_ga_michibiku_04() {
    let path = PathBuf::from("[TenRaw] Tsuki ga Michibiku - 04 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "04");
}

#[test]
fn test_parse_tenraw_tsuki_ga_michibiku_05() {
    let path = PathBuf::from("[TenRaw] Tsuki ga Michibiku - 05 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "05");
}

#[test]
fn test_parse_tenraw_tsuki_ga_michibiku_06() {
    let path = PathBuf::from("[TenRaw] Tsuki ga Michibiku - 06 [720p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "06");
}

#[test]
fn test_parse_tenraw_tsuki_ga_michibiku_07() {
    let path = PathBuf::from("[TenRaw] Tsuki ga Michibiku - 07 [1080p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "07");
}

#[test]
fn test_parse_tenraw_tsuki_ga_michibiku_08() {
    let path = PathBuf::from("[TenRaw] Tsuki ga Michibiku - 08 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "08");
}

#[test]
fn test_parse_tenraw_tsuki_ga_michibiku_09() {
    let path = PathBuf::from("[TenRaw] Tsuki ga Michibiku - 09 [1080p].mp4");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "09");
}

#[test]
fn test_parse_tenraw_tsuki_ga_michibiku_10() {
    let path = PathBuf::from("[TenRaw] Tsuki ga Michibiku - 10 [720p].mkv");
    let result = FilenameParser::parse(&path);
    let info = result.expect("Should parse episode");
    assert_eq!(info.episode, "10");
}