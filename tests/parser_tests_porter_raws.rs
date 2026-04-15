//! Generated parser tests - PorterRAWS
use anime_organizer::parser::FilenameParser;
use std::path::PathBuf;

#[test]
fn test_parse_porter_konosuba_01() {
    let path =
        PathBuf::from("[PorterRAWS] Kono Subarashii Sekai ni Shukufuku wo! - 01 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Kono Subarashii Sekai ni Shukufuku wo!");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_porter_konosuba_02() {
    let path =
        PathBuf::from("[PorterRAWS] Kono Subarashii Sekai ni Shukufuku wo! - 02 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Kono Subarashii Sekai ni Shukufuku wo!");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_porter_konosuba_03() {
    let path =
        PathBuf::from("[PorterRAWS] Kono Subarashii Sekai ni Shukufuku wo! - 03 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Kono Subarashii Sekai ni Shukufuku wo!");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_porter_rezero_01() {
    let path =
        PathBuf::from("[PorterRAWS] Re Zero kara Hajimeru Isekai Seikatsu - 01 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Re Zero kara Hajimeru Isekai Seikatsu");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_porter_rezero_02() {
    let path =
        PathBuf::from("[PorterRAWS] Re Zero kara Hajimeru Isekai Seikatsu - 02 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Re Zero kara Hajimeru Isekai Seikatsu");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_porter_rezero_03() {
    let path =
        PathBuf::from("[PorterRAWS] Re Zero kara Hajimeru Isekai Seikatsu - 03 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Re Zero kara Hajimeru Isekai Seikatsu");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_porter_aot_01() {
    let path = PathBuf::from("[PorterRAWS] Shingeki no Kyojin - 01 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Shingeki no Kyojin");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_porter_aot_02() {
    let path = PathBuf::from("[PorterRAWS] Shingeki no Kyojin - 02 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Shingeki no Kyojin");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_porter_aot_03() {
    let path = PathBuf::from("[PorterRAWS] Shingeki no Kyojin - 03 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Shingeki no Kyojin");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_porter_demonslayer_01() {
    let path = PathBuf::from("[PorterRAWS] Kimetsu no Yaiba - 01 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Kimetsu no Yaiba");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_porter_demonslayer_02() {
    let path = PathBuf::from("[PorterRAWS] Kimetsu no Yaiba - 02 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Kimetsu no Yaiba");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_porter_demonslayer_03() {
    let path = PathBuf::from("[PorterRAWS] Kimetsu no Yaiba - 03 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Kimetsu no Yaiba");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_porter_jujutsu_01() {
    let path = PathBuf::from("[PorterRAWS] Jujutsu Kaisen - 01 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Jujutsu Kaisen");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_porter_jujutsu_02() {
    let path = PathBuf::from("[PorterRAWS] Jujutsu Kaisen - 02 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Jujutsu Kaisen");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_porter_jujutsu_03() {
    let path = PathBuf::from("[PorterRAWS] Jujutsu Kaisen - 03 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Jujutsu Kaisen");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_porter_violet_01() {
    let path = PathBuf::from("[PorterRAWS] Violet Evergarden - 01 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Violet Evergarden");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_porter_violet_02() {
    let path = PathBuf::from("[PorterRAWS] Violet Evergarden - 02 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Violet Evergarden");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_porter_violet_03() {
    let path = PathBuf::from("[PorterRAWS] Violet Evergarden - 03 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Violet Evergarden");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_porter_spyfamily_01() {
    let path = PathBuf::from("[PorterRAWS] Spy x Family - 01 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Spy x Family");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_porter_spyfamily_02() {
    let path = PathBuf::from("[PorterRAWS] Spy x Family - 02 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Spy x Family");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_porter_spyfamily_03() {
    let path = PathBuf::from("[PorterRAWS] Spy x Family - 03 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Spy x Family");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_porter_kael_01() {
    let path = PathBuf::from("[PorterRAWS] Kaerudori - 01 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Kaerudori");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_porter_kael_02() {
    let path = PathBuf::from("[PorterRAWS] Kaerudori - 02 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Kaerudori");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_porter_kael_03() {
    let path = PathBuf::from("[PorterRAWS] Kaerudori - 03 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Kaerudori");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_porter_overlord_01() {
    let path = PathBuf::from("[PorterRAWS] Overlord - 01 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Overlord");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_porter_overlord_02() {
    let path = PathBuf::from("[PorterRAWS] Overlord - 02 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Overlord");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_porter_overlord_03() {
    let path = PathBuf::from("[PorterRAWS] Overlord - 03 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Overlord");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_porter_mushoku_01() {
    let path = PathBuf::from("[PorterRAWS] Mushoku Tensei - 01 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Mushoku Tensei");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_porter_mushoku_02() {
    let path = PathBuf::from("[PorterRAWS] Mushoku Tensei - 02 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Mushoku Tensei");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_porter_mushoku_03() {
    let path = PathBuf::from("[PorterRAWS] Mushoku Tensei - 03 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Mushoku Tensei");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_porter_kaguya_01() {
    let path = PathBuf::from("[PorterRAWS] Kaguya-sama - 01 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Kaguya-sama");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_porter_kaguya_02() {
    let path = PathBuf::from("[PorterRAWS] Kaguya-sama - 02 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Kaguya-sama");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_porter_kaguya_03() {
    let path = PathBuf::from("[PorterRAWS] Kaguya-sama - 03 [1080p][BD].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "PorterRAWS");
    assert_eq!(i.anime_name, "Kaguya-sama");
    assert_eq!(i.episode, "03");
}
