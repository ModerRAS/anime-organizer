pub mod dmhy;
pub mod nyaa;
pub mod types;

pub const MAX_SCRAPE_PAGES: u32 = 2000;

pub fn clamp_pages(pages: u32) -> u32 {
    pages.clamp(1, MAX_SCRAPE_PAGES)
}

pub use types::{sorted_unique_title_lines, sorted_unique_title_text, ScrapedTitle, TorrentSource};

#[cfg(test)]
mod tests {
    use super::clamp_pages;

    #[test]
    fn pages_are_clamped_to_one_through_2000() {
        assert_eq!(clamp_pages(0), 1);
        assert_eq!(clamp_pages(1), 1);
        assert_eq!(clamp_pages(2000), 2000);
        assert_eq!(clamp_pages(2001), 2000);
    }
}
