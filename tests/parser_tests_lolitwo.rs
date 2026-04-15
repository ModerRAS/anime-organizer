//! Generated parser tests - 萝莉社活动室
use anime_organizer::parser::FilenameParser;
use std::path::PathBuf;

#[test]
fn test_parse_loli_asmr_01() {
    let path = PathBuf::from("[萝莉社活动室] ASMR Special - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "ASMR Special");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_loli_asmr_02() {
    let path = PathBuf::from("[萝莉社活动室] ASMR Special - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "ASMR Special");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_loli_asmr_03() {
    let path = PathBuf::from("[萝莉社活动室] ASMR Special - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "ASMR Special");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_loli_kimetsu_01() {
    let path = PathBuf::from("[萝莉社活动室] Kimetsu no Yaiba - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Kimetsu no Yaiba");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_loli_kimetsu_02() {
    let path = PathBuf::from("[萝莉社活动室] Kimetsu no Yaiba - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Kimetsu no Yaiba");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_loli_kimetsu_03() {
    let path = PathBuf::from("[萝莉社活动室] Kimetsu no Yaiba - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Kimetsu no Yaiba");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_loli_jujutsu_01() {
    let path = PathBuf::from("[萝莉社活动室] Jujutsu Kaisen - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Jujutsu Kaisen");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_loli_jujutsu_02() {
    let path = PathBuf::from("[萝莉社活动室] Jujutsu Kaisen - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Jujutsu Kaisen");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_loli_jujutsu_03() {
    let path = PathBuf::from("[萝莉社活动室] Jujutsu Kaisen - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Jujutsu Kaisen");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_loli_aot_01() {
    let path = PathBuf::from("[萝莉社活动室] Shingeki no Kyojin - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Shingeki no Kyojin");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_loli_aot_02() {
    let path = PathBuf::from("[萝莉社活动室] Shingeki no Kyojin - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Shingeki no Kyojin");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_loli_aot_03() {
    let path = PathBuf::from("[萝莉社活动室] Shingeki no Kyojin - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Shingeki no Kyojin");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_loli_frieren_01() {
    let path = PathBuf::from("[萝莉社活动室] Sousou no Frieren - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Sousou no Frieren");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_loli_frieren_02() {
    let path = PathBuf::from("[萝莉社活动室] Sousou no Frieren - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Sousou no Frieren");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_loli_frieren_03() {
    let path = PathBuf::from("[萝莉社活动室] Sousou no Frieren - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Sousou no Frieren");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_loli_spy_01() {
    let path = PathBuf::from("[萝莉社活动室] Spy x Family - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Spy x Family");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_loli_spy_02() {
    let path = PathBuf::from("[萝莉社活动室] Spy x Family - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Spy x Family");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_loli_spy_03() {
    let path = PathBuf::from("[萝莉社活动室] Spy x Family - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Spy x Family");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_loli_rezero_01() {
    let path =
        PathBuf::from("[萝莉社活动室] Re Zero kara Hajimeru Isekai Seikatsu - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Re Zero kara Hajimeru Isekai Seikatsu");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_loli_rezero_02() {
    let path =
        PathBuf::from("[萝莉社活动室] Re Zero kara Hajimeru Isekai Seikatsu - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Re Zero kara Hajimeru Isekai Seikatsu");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_loli_rezero_03() {
    let path =
        PathBuf::from("[萝莉社活动室] Re Zero kara Hajimeru Isekai Seikatsu - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Re Zero kara Hajimeru Isekai Seikatsu");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_loli_chainsaw_01() {
    let path = PathBuf::from("[萝莉社活动室] Chainsaw Man - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Chainsaw Man");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_loli_chainsaw_02() {
    let path = PathBuf::from("[萝莉社活动室] Chainsaw Man - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Chainsaw Man");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_loli_chainsaw_03() {
    let path = PathBuf::from("[萝莉社活动室] Chainsaw Man - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Chainsaw Man");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_loli_violet_01() {
    let path = PathBuf::from("[萝莉社活动室] Violet Evergarden - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Violet Evergarden");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_loli_violet_02() {
    let path = PathBuf::from("[萝莉社活动室] Violet Evergarden - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Violet Evergarden");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_loli_violet_03() {
    let path = PathBuf::from("[萝莉社活动室] Violet Evergarden - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Violet Evergarden");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_loli_lycoris_01() {
    let path = PathBuf::from("[萝莉社活动室] Lycoris Recoil - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Lycoris Recoil");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_loli_lycoris_02() {
    let path = PathBuf::from("[萝莉社活动室] Lycoris Recoil - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Lycoris Recoil");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_loli_lycoris_03() {
    let path = PathBuf::from("[萝莉社活动室] Lycoris Recoil - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Lycoris Recoil");
    assert_eq!(i.episode, "03");
}

#[test]
fn test_parse_loli_kaguya_01() {
    let path = PathBuf::from("[萝莉社活动室] Kaguya-sama wa Kurasu wo Shitbori - 01 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Kaguya-sama wa Kurasu wo Shitbori");
    assert_eq!(i.episode, "01");
}

#[test]
fn test_parse_loli_kaguya_02() {
    let path = PathBuf::from("[萝莉社活动室] Kaguya-sama wa Kurasu wo Shitbori - 02 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Kaguya-sama wa Kurasu wo Shitbori");
    assert_eq!(i.episode, "02");
}

#[test]
fn test_parse_loli_kaguya_03() {
    let path = PathBuf::from("[萝莉社活动室] Kaguya-sama wa Kurasu wo Shitbori - 03 [1080p].mkv");
    let r = FilenameParser::parse(&path);
    let i = r.expect("parse failed");
    assert_eq!(i.publisher, "萝莉社活动室");
    assert_eq!(i.anime_name, "Kaguya-sama wa Kurasu wo Shitbori");
    assert_eq!(i.episode, "03");
}
