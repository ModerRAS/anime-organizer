use anime_organizer::library_index::{
    Artwork, ArtworkKind, ExternalProvider, LibraryIndex, LibraryIndexRecord,
};
use image::{codecs::png::PngEncoder, ExtendedColorType, ImageEncoder};
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: generate_mlip_v4_fixture <output-directory>");
    let root = std::env::temp_dir().join("aniorg-mlip-v4-shared-fixture");
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&output);
    fs::create_dir_all(root.join("Fixture Show/Season 1")).unwrap();
    fs::write(root.join("Fixture Show/Season 1/01.mkv"), b"fixture video").unwrap();

    let poster = png(2, 3, [0x10, 0x60, 0xc0, 0xff]);
    let episode = png(3, 2, [0xc0, 0x30, 0x40, 0xff]);
    fs::write(root.join("Fixture Show/poster.png"), poster).unwrap();
    fs::write(root.join("Fixture Show/episode.png"), episode).unwrap();
    fs::write(
        root.join("Fixture Show/legacy.jpg"),
        b"legacy path fallback",
    )
    .unwrap();

    let video = root.join("Fixture Show/Season 1/01.mkv");
    let mut record = LibraryIndexRecord::from_target_path(&root, &video)
        .unwrap()
        .unwrap();
    record.series_artwork = vec![
        Artwork::new(ArtworkKind::Poster, "Fixture Show/poster.png").with_source(
            ExternalProvider::Bangumi,
            424242,
            Some("https://lain.bgm.tv/pic/cover/l/fixture-original.jpg".to_string()),
            Some("2026-07-23T00:00:00Z".to_string()),
        ),
        Artwork::new(ArtworkKind::SeasonPoster, "Fixture Show/poster.png"),
        Artwork::new(ArtworkKind::Logo, "Fixture Show/legacy.jpg"),
    ];
    record.episode_artwork = vec![Artwork::new(ArtworkKind::Thumb, "Fixture Show/episode.png")];

    LibraryIndex::rebuild_v4_staging(&root, &[record.clone()], &[]).unwrap();
    copy_snapshot(&root, &output.join("base"));

    fs::write(
        root.join("Fixture Show/fanart.png"),
        png(4, 3, [0x20, 0xa0, 0x50, 0xff]),
    )
    .unwrap();
    record
        .series_artwork
        .push(Artwork::new(ArtworkKind::Fanart, "Fixture Show/fanart.png"));
    LibraryIndex::rebuild_v4_staging(&root, &[record], &[]).unwrap();
    copy_snapshot(&root, &output.join("incremental"));
    fs::remove_dir_all(root).unwrap();
}

fn png(width: u32, height: u32, pixel: [u8; 4]) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for _ in 0..width * height {
        pixels.extend_from_slice(&pixel);
    }
    let mut output = Vec::new();
    PngEncoder::new(&mut output)
        .write_image(&pixels, width, height, ExtendedColorType::Rgba8)
        .unwrap();
    output
}

fn copy_snapshot(root: &Path, output: &Path) {
    fs::create_dir_all(output.join("MLIP-Artwork")).unwrap();
    fs::copy(root.join("library.db"), output.join("library.db")).unwrap();
    for entry in fs::read_dir(root.join("MLIP-Artwork")).unwrap() {
        let entry = entry.unwrap();
        fs::copy(
            entry.path(),
            output.join("MLIP-Artwork").join(entry.file_name()),
        )
        .unwrap();
    }
}
