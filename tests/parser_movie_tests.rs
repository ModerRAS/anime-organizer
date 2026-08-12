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

#[test]
fn explicit_unnumbered_movie_and_special_titles_map_to_single_media_episode() {
    for (filename, expected_title) in [
        (
            "[LoliHouse] Gekijouban Ansatsu Kyoushitsu Minna no Jikan [WebRip 1080p HEVC-10bit AAC].mkv",
            "Gekijouban Ansatsu Kyoushitsu Minna no Jikan",
        ),
        (
            "[LoliHouse] ONE PIECE HEROINES [WebRip 1080p HEVC-10bit AAC SRTx2].mkv",
            "ONE PIECE HEROINES",
        ),
        (
            "[Skymoon-Raws] Crayon Shin-chan the Movie：Super Magificent! Scorching Kasukabe Dancers [UVOD][WEB-DL][CHT][1080p][AVC AAC].mp4",
            "Crayon Shin-chan the Movie：Super Magificent! Scorching Kasukabe Dancers",
        ),
        (
            "[Skymoon-Raws] Kimetsu no Yaiba Infinity Castle 2025 [ViuTV][WEB-DL][CHT][SRT][1080p][AVC AAC].mkv",
            "Kimetsu no Yaiba Infinity Castle",
        ),
        (
            "[Skymoon-Raws][Kimetsu no Yaiba Infinity Castle 2025][Baha][WEB-DL][1080p][AVC AAC][CHT][MP4].mp4",
            "Kimetsu no Yaiba Infinity Castle",
        ),
        (
            "[Skymoon-Raws][劇場版 我與機器子][Me & Roboco the Movie][Baha][WEB-DL][1080p][AVC AAC][CHT][MP4].mp4",
            "劇場版 我與機器子",
        ),
        (
            "[Skymoon-Raws][成為星星的少女][Trapezium][Baha][WEB-DL][1080p][AVC AAC][CHT][MP4].mp4",
            "成為星星的少女",
        ),
    ] {
        let info = FilenameParser::parse(filename).unwrap();
        assert_eq!(info.anime_name, expected_title);
        assert_eq!(info.episode, "01");
    }
}
