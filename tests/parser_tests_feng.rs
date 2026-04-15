//! Generated parser tests - 风之动漫
use anime_organizer::parser::FilenameParser;
use std::path::PathBuf;

#[test]
fn test_parse_feng_kimetsu_01() {
    let path = PathBuf::from("[风之动漫] Kimetsu no Yaiba - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Kimetsu no Yaiba");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_kimetsu_02() {
    let path = PathBuf::from("[风之动漫] Kimetsu no Yaiba - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Kimetsu no Yaiba");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_kimetsu_03() {
    let path = PathBuf::from("[风之动漫] Kimetsu no Yaiba - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Kimetsu no Yaiba");
    assert_eq!(i.episode, "03");
}
#[test]
fn test_parse_feng_jujutsu_01() {
    let path = PathBuf::from("[风之动漫] Jujutsu Kaisen - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Jujutsu Kaisen");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_jujutsu_02() {
    let path = PathBuf::from("[风之动漫] Jujutsu Kaisen - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Jujutsu Kaisen");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_jujutsu_03() {
    let path = PathBuf::from("[风之动漫] Jujutsu Kaisen - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Jujutsu Kaisen");
    assert_eq!(i.episode, "03");
}
#[test]
fn test_parse_feng_aot_01() {
    let path = PathBuf::from("[风之动漫] Shingeki no Kyojin - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Shingeki no Kyojin");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_aot_02() {
    let path = PathBuf::from("[风之动漫] Shingeki no Kyojin - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Shingeki no Kyojin");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_aot_03() {
    let path = PathBuf::from("[风之动漫] Shingeki no Kyojin - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Shingeki no Kyojin");
    assert_eq!(i.episode, "03");
}
#[test]
fn test_parse_feng_spy_01() {
    let path = PathBuf::from("[风之动漫] Spy x Family - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Spy x Family");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_spy_02() {
    let path = PathBuf::from("[风之动漫] Spy x Family - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Spy x Family");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_spy_03() {
    let path = PathBuf::from("[风之动漫] Spy x Family - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Spy x Family");
    assert_eq!(i.episode, "03");
}
#[test]
fn test_parse_feng_frieren_01() {
    let path = PathBuf::from("[风之动漫] Sousou no Frieren - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Sousou no Frieren");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_frieren_02() {
    let path = PathBuf::from("[风之动漫] Sousou no Frieren - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Sousou no Frieren");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_frieren_03() {
    let path = PathBuf::from("[风之动漫] Sousou no Frieren - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Sousou no Frieren");
    assert_eq!(i.episode, "03");
}
#[test]
fn test_parse_feng_rezero_01() {
    let path = PathBuf::from("[风之动漫] Re Zero kara Hajimeru Isekai Seikatsu - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Re Zero kara Hajimeru Isekai Seikatsu");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_rezero_02() {
    let path = PathBuf::from("[风之动漫] Re Zero kara Hajimeru Isekai Seikatsu - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Re Zero kara Hajimeru Isekai Seikatsu");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_rezero_03() {
    let path = PathBuf::from("[风之动漫] Re Zero kara Hajimeru Isekai Seikatsu - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Re Zero kara Hajimeru Isekai Seikatsu");
    assert_eq!(i.episode, "03");
}
#[test]
fn test_parse_feng_chainsaw_01() {
    let path = PathBuf::from("[风之动漫] Chainsaw Man - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Chainsaw Man");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_chainsaw_02() {
    let path = PathBuf::from("[风之动漫] Chainsaw Man - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Chainsaw Man");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_chainsaw_03() {
    let path = PathBuf::from("[风之动漫] Chainsaw Man - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Chainsaw Man");
    assert_eq!(i.episode, "03");
}
#[test]
fn test_parse_feng_violet_01() {
    let path = PathBuf::from("[风之动漫] Violet Evergarden - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Violet Evergarden");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_violet_02() {
    let path = PathBuf::from("[风之动漫] Violet Evergarden - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Violet Evergarden");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_violet_03() {
    let path = PathBuf::from("[风之动漫] Violet Evergarden - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Violet Evergarden");
    assert_eq!(i.episode, "03");
}
#[test]
fn test_parse_feng_lycoris_01() {
    let path = PathBuf::from("[风之动漫] Lycoris Recoil - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Lycoris Recoil");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_lycoris_02() {
    let path = PathBuf::from("[风之动漫] Lycoris Recoil - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Lycoris Recoil");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_lycoris_03() {
    let path = PathBuf::from("[风之动漫] Lycoris Recoil - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Lycoris Recoil");
    assert_eq!(i.episode, "03");
}
#[test]
fn test_parse_feng_kaguya_01() {
    let path = PathBuf::from("[风之动漫] Kaguya-sama wa Kurasu wo Shitbori - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Kaguya-sama wa Kurasu wo Shitbori");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_kaguya_02() {
    let path = PathBuf::from("[风之动漫] Kaguya-sama wa Kurasu wo Shitbori - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Kaguya-sama wa Kurasu wo Shitbori");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_kaguya_03() {
    let path = PathBuf::from("[风之动漫] Kaguya-sama wa Kurasu wo Shitbori - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Kaguya-sama wa Kurasu wo Shitbori");
    assert_eq!(i.episode, "03");
}
#[test]
fn test_parse_feng_dressup_01() {
    let path = PathBuf::from("[风之动漫] Sono Bisque Doll wa Koi wo Suru - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Sono Bisque Doll wa Koi wo Suru");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_dressup_02() {
    let path = PathBuf::from("[风之动漫] Sono Bisque Doll wa Koi wo Suru - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Sono Bisque Doll wa Koi wo Suru");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_dressup_03() {
    let path = PathBuf::from("[风之动漫] Sono Bisque Doll wa Koi wo Suru - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Sono Bisque Doll wa Koi wo Suru");
    assert_eq!(i.episode, "03");
}
#[test]
fn test_parse_feng_overlord_01() {
    let path = PathBuf::from("[风之动漫] Overlord - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Overlord");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_overlord_02() {
    let path = PathBuf::from("[风之动漫] Overlord - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Overlord");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_overlord_03() {
    let path = PathBuf::from("[风之动漫] Overlord - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Overlord");
    assert_eq!(i.episode, "03");
}
#[test]
fn test_parse_feng_mob_01() {
    let path = PathBuf::from("[风之动漫] Mob Psycho 100 - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Mob Psycho 100");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_mob_02() {
    let path = PathBuf::from("[风之动漫] Mob Psycho 100 - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Mob Psycho 100");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_mob_03() {
    let path = PathBuf::from("[风之动漫] Mob Psycho 100 - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Mob Psycho 100");
    assert_eq!(i.episode, "03");
}
#[test]
fn test_parse_feng_dandadan_01() {
    let path = PathBuf::from("[风之动漫] Dandadan - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Dandadan");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_dandadan_02() {
    let path = PathBuf::from("[风之动漫] Dandadan - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Dandadan");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_dandadan_03() {
    let path = PathBuf::from("[风之动漫] Dandadan - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Dandadan");
    assert_eq!(i.episode, "03");
}
#[test]
fn test_parse_feng_blue_lock_01() {
    let path = PathBuf::from("[风之动漫] Blue Lock - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Blue Lock");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_blue_lock_02() {
    let path = PathBuf::from("[风之动漫] Blue Lock - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Blue Lock");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_blue_lock_03() {
    let path = PathBuf::from("[风之动漫] Blue Lock - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Blue Lock");
    assert_eq!(i.episode, "03");
}
#[test]
fn test_parse_feng_takt_01() {
    let path = PathBuf::from("[风之动漫] Takt Op. Destiny - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Takt Op. Destiny");
    assert_eq!(i.episode, "01");
}
#[test]
fn test_parse_feng_takt_02() {
    let path = PathBuf::from("[风之动漫] Takt Op. Destiny - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Takt Op. Destiny");
    assert_eq!(i.episode, "02");
}
#[test]
fn test_parse_feng_takt_03() {
    let path = PathBuf::from("[风之动漫] Takt Op. Destiny - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "风之动漫");
    assert_eq!(i.anime_name, "Takt Op. Destiny");
    assert_eq!(i.episode, "03");
}
