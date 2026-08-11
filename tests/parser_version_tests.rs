use anime_organizer::FilenameParser;

#[test]
fn parses_hyphenated_episode_revision_suffixes() {
    for (filename, expected_title, expected_episode) in [
        (
            "[Skymoon-Raws] Shin Samurai-den YAIBA - 15v2 [ViuTV][WEB-DL][1080p].mkv",
            "Shin Samurai-den YAIBA",
            "15",
        ),
        (
            "[Skymoon-Raws] Shin Samurai-den YAIBA - 15v3 [ViuTV][WEB-DL][1080p].mkv",
            "Shin Samurai-den YAIBA",
            "15",
        ),
        (
            "[Skymoon-Raws] Rooster Fighter - 09v2 [ViuTV][WEB-DL][1080p].mkv",
            "Rooster Fighter",
            "09",
        ),
    ] {
        let parsed = FilenameParser::parse(filename).expect("versioned episode should parse");
        assert_eq!(parsed.anime_name, expected_title);
        assert_eq!(parsed.episode, expected_episode);
    }
}

#[test]
fn parses_bracketed_episode_revision_suffix() {
    let parsed = FilenameParser::parse("[Skymoon] 魔法光源股份有限公司第二季 [04v2].mp4")
        .expect("bracketed versioned episode should parse");

    assert_eq!(parsed.anime_name, "魔法光源股份有限公司第二季");
    assert_eq!(parsed.episode, "04");
}
