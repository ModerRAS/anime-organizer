use anime_organizer::LibraryIndex;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = std::env::args()
        .nth(1)
        .ok_or("usage: cargo run --example migrate_mlip_v3_to_v4 -- <target-root>")?;
    let stats = LibraryIndex::migrate_v3_to_v4(Path::new(&target))?;
    println!(
        "MLIP v4 migration finished: {} series, {} episodes, {} media files, {} extras",
        stats.series, stats.episodes, stats.media_files, stats.extras
    );
    Ok(())
}
