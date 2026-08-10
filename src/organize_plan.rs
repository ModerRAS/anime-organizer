//! Pure destination-directory planning shared by local and remote organizers.

use crate::parser::AnimeFileInfo;

/// Returns platform-neutral destination directory components for an anime file.
///
/// The original media filename is intentionally not included or changed. Local
/// callers can join the components onto a [`std::path::Path`], while CloudDrive
/// callers can join them with `/` for a remote path.
#[must_use]
pub fn organize_directory_components(anime_file: &AnimeFileInfo, season_mode: bool) -> Vec<String> {
    if season_mode {
        vec![anime_file.series_name(), anime_file.season_dir_name()]
    } else {
        vec![anime_file.anime_name.clone()]
    }
}

#[cfg(test)]
mod tests {
    use super::organize_directory_components;
    use crate::parser::AnimeFileInfo;

    fn anime_file(name: &str) -> AnimeFileInfo {
        AnimeFileInfo {
            publisher: "ANi".to_string(),
            anime_name: name.to_string(),
            episode: "01".to_string(),
            tags: "[1080P]".to_string(),
            extension: ".mkv".to_string(),
            original_path: "/downloads/[ANi] example - 01 [1080P].mkv".to_string(),
        }
    }

    #[test]
    fn plans_season_directory_components() {
        let components = organize_directory_components(&anime_file("Test Anime 第2季"), true);

        assert_eq!(components, ["Test Anime", "Season 2"]);
        assert_eq!(components.join("/"), "Test Anime/Season 2");
    }

    #[test]
    fn plans_flat_directory_without_renaming_media() {
        let file = anime_file("Test Anime 第2季");

        assert_eq!(
            organize_directory_components(&file, false),
            ["Test Anime 第2季"]
        );
        assert_eq!(
            file.original_path.rsplit(['/', '\\']).next(),
            Some("[ANi] example - 01 [1080P].mkv")
        );
    }
}
