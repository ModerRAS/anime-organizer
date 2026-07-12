use anime_organizer::parser::FilenameParser;

#[test]
fn ani_movie_marker_maps_to_single_media_episode() {
    for (filename, expected_title) in [
        (
            "[ANi] 劇場版 歌之☆王子殿下♪ TABOO NIGHT XXXX - 電影 [1080P][Baha].mp4",
            "劇場版 歌之☆王子殿下♪ TABOO NIGHT XXXX",
        ),
        (
            "[ANi] 劇場版 關於我轉生變成史萊姆這檔事 蒼海之淚篇 - 電影 [1080P][Baha].mp4",
            "劇場版 關於我轉生變成史萊姆這檔事 蒼海之淚篇",
        ),
        (
            "[ANi] 魯邦三世 不死身的血族 - 電影 [1080P][Baha].mp4",
            "魯邦三世 不死身的血族",
        ),
    ] {
        let info = FilenameParser::parse(filename).unwrap();
        assert_eq!(info.anime_name, expected_title);
        assert_eq!(info.episode, "01");
        assert_eq!(info.target_filename(), "01 [1080P][Baha].mp4");
    }
}
