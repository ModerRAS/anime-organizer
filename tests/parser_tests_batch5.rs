// Auto-generated test file - Batch 5
// Publisher: Pub051-Pub060 (10 publishers × 3 anime × 12 episodes = 360 tests)
use anime_organizer::FilenameParser;
use std::path::PathBuf;

fn test_pub(name: &str, anime: &str, episode: &str) {
    let path = PathBuf::from(format!("[{}] {} - {} [1080P].mkv", name, anime, episode));
    let result = FilenameParser::parse(&path).unwrap();
    assert_eq!(result.publisher, name);
    assert_eq!(result.anime_name, anime);
    assert_eq!(result.episode, episode);
}

// Pub051
#[test]
fn test_pub051_a1_e01() {
    test_pub("Pub051", "Neon Genesis Evangelion", "01");
}
#[test]
fn test_pub051_a1_e02() {
    test_pub("Pub051", "Neon Genesis Evangelion", "02");
}
#[test]
fn test_pub051_a1_e03() {
    test_pub("Pub051", "Neon Genesis Evangelion", "03");
}
#[test]
fn test_pub051_a1_e04() {
    test_pub("Pub051", "Neon Genesis Evangelion", "04");
}
#[test]
fn test_pub051_a1_e05() {
    test_pub("Pub051", "Neon Genesis Evangelion", "05");
}
#[test]
fn test_pub051_a1_e06() {
    test_pub("Pub051", "Neon Genesis Evangelion", "06");
}
#[test]
fn test_pub051_a1_e07() {
    test_pub("Pub051", "Neon Genesis Evangelion", "07");
}
#[test]
fn test_pub051_a1_e08() {
    test_pub("Pub051", "Neon Genesis Evangelion", "08");
}
#[test]
fn test_pub051_a1_e09() {
    test_pub("Pub051", "Neon Genesis Evangelion", "09");
}
#[test]
fn test_pub051_a1_e10() {
    test_pub("Pub051", "Neon Genesis Evangelion", "10");
}
#[test]
fn test_pub051_a1_e11() {
    test_pub("Pub051", "Neon Genesis Evangelion", "11");
}
#[test]
fn test_pub051_a1_e12() {
    test_pub("Pub051", "Neon Genesis Evangelion", "12");
}
#[test]
fn test_pub051_a2_e01() {
    test_pub("Pub051", "Cowboy Bebop", "01");
}
#[test]
fn test_pub051_a2_e02() {
    test_pub("Pub051", "Cowboy Bebop", "02");
}
#[test]
fn test_pub051_a2_e03() {
    test_pub("Pub051", "Cowboy Bebop", "03");
}
#[test]
fn test_pub051_a2_e04() {
    test_pub("Pub051", "Cowboy Bebop", "04");
}
#[test]
fn test_pub051_a2_e05() {
    test_pub("Pub051", "Cowboy Bebop", "05");
}
#[test]
fn test_pub051_a2_e06() {
    test_pub("Pub051", "Cowboy Bebop", "06");
}
#[test]
fn test_pub051_a2_e07() {
    test_pub("Pub051", "Cowboy Bebop", "07");
}
#[test]
fn test_pub051_a2_e08() {
    test_pub("Pub051", "Cowboy Bebop", "08");
}
#[test]
fn test_pub051_a2_e09() {
    test_pub("Pub051", "Cowboy Bebop", "09");
}
#[test]
fn test_pub051_a2_e10() {
    test_pub("Pub051", "Cowboy Bebop", "10");
}
#[test]
fn test_pub051_a2_e11() {
    test_pub("Pub051", "Cowboy Bebop", "11");
}
#[test]
fn test_pub051_a2_e12() {
    test_pub("Pub051", "Cowboy Bebop", "12");
}
#[test]
fn test_pub051_a3_e01() {
    test_pub("Pub051", "Trigun", "01");
}
#[test]
fn test_pub051_a3_e02() {
    test_pub("Pub051", "Trigun", "02");
}
#[test]
fn test_pub051_a3_e03() {
    test_pub("Pub051", "Trigun", "03");
}
#[test]
fn test_pub051_a3_e04() {
    test_pub("Pub051", "Trigun", "04");
}
#[test]
fn test_pub051_a3_e05() {
    test_pub("Pub051", "Trigun", "05");
}
#[test]
fn test_pub051_a3_e06() {
    test_pub("Pub051", "Trigun", "06");
}
#[test]
fn test_pub051_a3_e07() {
    test_pub("Pub051", "Trigun", "07");
}
#[test]
fn test_pub051_a3_e08() {
    test_pub("Pub051", "Trigun", "08");
}
#[test]
fn test_pub051_a3_e09() {
    test_pub("Pub051", "Trigun", "09");
}
#[test]
fn test_pub051_a3_e10() {
    test_pub("Pub051", "Trigun", "10");
}
#[test]
fn test_pub051_a3_e11() {
    test_pub("Pub051", "Trigun", "11");
}
#[test]
fn test_pub051_a3_e12() {
    test_pub("Pub051", "Trigun", "12");
}

// Pub052
#[test]
fn test_pub052_a1_e01() {
    test_pub("Pub052", "Ghost in the Shell", "01");
}
#[test]
fn test_pub052_a1_e02() {
    test_pub("Pub052", "Ghost in the Shell", "02");
}
#[test]
fn test_pub052_a1_e03() {
    test_pub("Pub052", "Ghost in the Shell", "03");
}
#[test]
fn test_pub052_a1_e04() {
    test_pub("Pub052", "Ghost in the Shell", "04");
}
#[test]
fn test_pub052_a1_e05() {
    test_pub("Pub052", "Ghost in the Shell", "05");
}
#[test]
fn test_pub052_a1_e06() {
    test_pub("Pub052", "Ghost in the Shell", "06");
}
#[test]
fn test_pub052_a1_e07() {
    test_pub("Pub052", "Ghost in the Shell", "07");
}
#[test]
fn test_pub052_a1_e08() {
    test_pub("Pub052", "Ghost in the Shell", "08");
}
#[test]
fn test_pub052_a1_e09() {
    test_pub("Pub052", "Ghost in the Shell", "09");
}
#[test]
fn test_pub052_a1_e10() {
    test_pub("Pub052", "Ghost in the Shell", "10");
}
#[test]
fn test_pub052_a1_e11() {
    test_pub("Pub052", "Ghost in the Shell", "11");
}
#[test]
fn test_pub052_a1_e12() {
    test_pub("Pub052", "Ghost in the Shell", "12");
}
#[test]
fn test_pub052_a2_e01() {
    test_pub("Pub052", "Berserk", "01");
}
#[test]
fn test_pub052_a2_e02() {
    test_pub("Pub052", "Berserk", "02");
}
#[test]
fn test_pub052_a2_e03() {
    test_pub("Pub052", "Berserk", "03");
}
#[test]
fn test_pub052_a2_e04() {
    test_pub("Pub052", "Berserk", "04");
}
#[test]
fn test_pub052_a2_e05() {
    test_pub("Pub052", "Berserk", "05");
}
#[test]
fn test_pub052_a2_e06() {
    test_pub("Pub052", "Berserk", "06");
}
#[test]
fn test_pub052_a2_e07() {
    test_pub("Pub052", "Berserk", "07");
}
#[test]
fn test_pub052_a2_e08() {
    test_pub("Pub052", "Berserk", "08");
}
#[test]
fn test_pub052_a2_e09() {
    test_pub("Pub052", "Berserk", "09");
}
#[test]
fn test_pub052_a2_e10() {
    test_pub("Pub052", "Berserk", "10");
}
#[test]
fn test_pub052_a2_e11() {
    test_pub("Pub052", "Berserk", "11");
}
#[test]
fn test_pub052_a2_e12() {
    test_pub("Pub052", "Berserk", "12");
}
#[test]
fn test_pub052_a3_e01() {
    test_pub("Pub052", "Monster", "01");
}
#[test]
fn test_pub052_a3_e02() {
    test_pub("Pub052", "Monster", "02");
}
#[test]
fn test_pub052_a3_e03() {
    test_pub("Pub052", "Monster", "03");
}
#[test]
fn test_pub052_a3_e04() {
    test_pub("Pub052", "Monster", "04");
}
#[test]
fn test_pub052_a3_e05() {
    test_pub("Pub052", "Monster", "05");
}
#[test]
fn test_pub052_a3_e06() {
    test_pub("Pub052", "Monster", "06");
}
#[test]
fn test_pub052_a3_e07() {
    test_pub("Pub052", "Monster", "07");
}
#[test]
fn test_pub052_a3_e08() {
    test_pub("Pub052", "Monster", "08");
}
#[test]
fn test_pub052_a3_e09() {
    test_pub("Pub052", "Monster", "09");
}
#[test]
fn test_pub052_a3_e10() {
    test_pub("Pub052", "Monster", "10");
}
#[test]
fn test_pub052_a3_e11() {
    test_pub("Pub052", "Monster", "11");
}
#[test]
fn test_pub052_a3_e12() {
    test_pub("Pub052", "Monster", "12");
}

// Pub053
#[test]
fn test_pub053_a1_e01() {
    test_pub("Pub053", "Steins;Gate", "01");
}
#[test]
fn test_pub053_a1_e02() {
    test_pub("Pub053", "Steins;Gate", "02");
}
#[test]
fn test_pub053_a1_e03() {
    test_pub("Pub053", "Steins;Gate", "03");
}
#[test]
fn test_pub053_a1_e04() {
    test_pub("Pub053", "Steins;Gate", "04");
}
#[test]
fn test_pub053_a1_e05() {
    test_pub("Pub053", "Steins;Gate", "05");
}
#[test]
fn test_pub053_a1_e06() {
    test_pub("Pub053", "Steins;Gate", "06");
}
#[test]
fn test_pub053_a1_e07() {
    test_pub("Pub053", "Steins;Gate", "07");
}
#[test]
fn test_pub053_a1_e08() {
    test_pub("Pub053", "Steins;Gate", "08");
}
#[test]
fn test_pub053_a1_e09() {
    test_pub("Pub053", "Steins;Gate", "09");
}
#[test]
fn test_pub053_a1_e10() {
    test_pub("Pub053", "Steins;Gate", "10");
}
#[test]
fn test_pub053_a1_e11() {
    test_pub("Pub053", "Steins;Gate", "11");
}
#[test]
fn test_pub053_a1_e12() {
    test_pub("Pub053", "Steins;Gate", "12");
}
#[test]
fn test_pub053_a2_e01() {
    test_pub("Pub053", "Clannad", "01");
}
#[test]
fn test_pub053_a2_e02() {
    test_pub("Pub053", "Clannad", "02");
}
#[test]
fn test_pub053_a2_e03() {
    test_pub("Pub053", "Clannad", "03");
}
#[test]
fn test_pub053_a2_e04() {
    test_pub("Pub053", "Clannad", "04");
}
#[test]
fn test_pub053_a2_e05() {
    test_pub("Pub053", "Clannad", "05");
}
#[test]
fn test_pub053_a2_e06() {
    test_pub("Pub053", "Clannad", "06");
}
#[test]
fn test_pub053_a2_e07() {
    test_pub("Pub053", "Clannad", "07");
}
#[test]
fn test_pub053_a2_e08() {
    test_pub("Pub053", "Clannad", "08");
}
#[test]
fn test_pub053_a2_e09() {
    test_pub("Pub053", "Clannad", "09");
}
#[test]
fn test_pub053_a2_e10() {
    test_pub("Pub053", "Clannad", "10");
}
#[test]
fn test_pub053_a2_e11() {
    test_pub("Pub053", "Clannad", "11");
}
#[test]
fn test_pub053_a2_e12() {
    test_pub("Pub053", "Clannad", "12");
}
#[test]
fn test_pub053_a3_e01() {
    test_pub("Pub053", "Anohana", "01");
}
#[test]
fn test_pub053_a3_e02() {
    test_pub("Pub053", "Anohana", "02");
}
#[test]
fn test_pub053_a3_e03() {
    test_pub("Pub053", "Anohana", "03");
}
#[test]
fn test_pub053_a3_e04() {
    test_pub("Pub053", "Anohana", "04");
}
#[test]
fn test_pub053_a3_e05() {
    test_pub("Pub053", "Anohana", "05");
}
#[test]
fn test_pub053_a3_e06() {
    test_pub("Pub053", "Anohana", "06");
}
#[test]
fn test_pub053_a3_e07() {
    test_pub("Pub053", "Anohana", "07");
}
#[test]
fn test_pub053_a3_e08() {
    test_pub("Pub053", "Anohana", "08");
}
#[test]
fn test_pub053_a3_e09() {
    test_pub("Pub053", "Anohana", "09");
}
#[test]
fn test_pub053_a3_e10() {
    test_pub("Pub053", "Anohana", "10");
}
#[test]
fn test_pub053_a3_e11() {
    test_pub("Pub053", "Anohana", "11");
}
#[test]
fn test_pub053_a3_e12() {
    test_pub("Pub053", "Anohana", "12");
}

// Pub054
#[test]
fn test_pub054_a1_e01() {
    test_pub("Pub054", "Your Lie in April", "01");
}
#[test]
fn test_pub054_a1_e02() {
    test_pub("Pub054", "Your Lie in April", "02");
}
#[test]
fn test_pub054_a1_e03() {
    test_pub("Pub054", "Your Lie in April", "03");
}
#[test]
fn test_pub054_a1_e04() {
    test_pub("Pub054", "Your Lie in April", "04");
}
#[test]
fn test_pub054_a1_e05() {
    test_pub("Pub054", "Your Lie in April", "05");
}
#[test]
fn test_pub054_a1_e06() {
    test_pub("Pub054", "Your Lie in April", "06");
}
#[test]
fn test_pub054_a1_e07() {
    test_pub("Pub054", "Your Lie in April", "07");
}
#[test]
fn test_pub054_a1_e08() {
    test_pub("Pub054", "Your Lie in April", "08");
}
#[test]
fn test_pub054_a1_e09() {
    test_pub("Pub054", "Your Lie in April", "09");
}
#[test]
fn test_pub054_a1_e10() {
    test_pub("Pub054", "Your Lie in April", "10");
}
#[test]
fn test_pub054_a1_e11() {
    test_pub("Pub054", "Your Lie in April", "11");
}
#[test]
fn test_pub054_a1_e12() {
    test_pub("Pub054", "Your Lie in April", "12");
}
#[test]
fn test_pub054_a2_e01() {
    test_pub("Pub054", "Violet Evergarden", "01");
}
#[test]
fn test_pub054_a2_e02() {
    test_pub("Pub054", "Violet Evergarden", "02");
}
#[test]
fn test_pub054_a2_e03() {
    test_pub("Pub054", "Violet Evergarden", "03");
}
#[test]
fn test_pub054_a2_e04() {
    test_pub("Pub054", "Violet Evergarden", "04");
}
#[test]
fn test_pub054_a2_e05() {
    test_pub("Pub054", "Violet Evergarden", "05");
}
#[test]
fn test_pub054_a2_e06() {
    test_pub("Pub054", "Violet Evergarden", "06");
}
#[test]
fn test_pub054_a2_e07() {
    test_pub("Pub054", "Violet Evergarden", "07");
}
#[test]
fn test_pub054_a2_e08() {
    test_pub("Pub054", "Violet Evergarden", "08");
}
#[test]
fn test_pub054_a2_e09() {
    test_pub("Pub054", "Violet Evergarden", "09");
}
#[test]
fn test_pub054_a2_e10() {
    test_pub("Pub054", "Violet Evergarden", "10");
}
#[test]
fn test_pub054_a2_e11() {
    test_pub("Pub054", "Violet Evergarden", "11");
}
#[test]
fn test_pub054_a2_e12() {
    test_pub("Pub054", "Violet Evergarden", "12");
}
#[test]
fn test_pub054_a3_e01() {
    test_pub("Pub054", "Angel Beats", "01");
}
#[test]
fn test_pub054_a3_e02() {
    test_pub("Pub054", "Angel Beats", "02");
}
#[test]
fn test_pub054_a3_e03() {
    test_pub("Pub054", "Angel Beats", "03");
}
#[test]
fn test_pub054_a3_e04() {
    test_pub("Pub054", "Angel Beats", "04");
}
#[test]
fn test_pub054_a3_e05() {
    test_pub("Pub054", "Angel Beats", "05");
}
#[test]
fn test_pub054_a3_e06() {
    test_pub("Pub054", "Angel Beats", "06");
}
#[test]
fn test_pub054_a3_e07() {
    test_pub("Pub054", "Angel Beats", "07");
}
#[test]
fn test_pub054_a3_e08() {
    test_pub("Pub054", "Angel Beats", "08");
}
#[test]
fn test_pub054_a3_e09() {
    test_pub("Pub054", "Angel Beats", "09");
}
#[test]
fn test_pub054_a3_e10() {
    test_pub("Pub054", "Angel Beats", "10");
}
#[test]
fn test_pub054_a3_e11() {
    test_pub("Pub054", "Angel Beats", "11");
}
#[test]
fn test_pub054_a3_e12() {
    test_pub("Pub054", "Angel Beats", "12");
}

// Pub055
#[test]
fn test_pub055_a1_e01() {
    test_pub("Pub055", "A Silent Voice", "01");
}
#[test]
fn test_pub055_a1_e02() {
    test_pub("Pub055", "A Silent Voice", "02");
}
#[test]
fn test_pub055_a1_e03() {
    test_pub("Pub055", "A Silent Voice", "03");
}
#[test]
fn test_pub055_a1_e04() {
    test_pub("Pub055", "A Silent Voice", "04");
}
#[test]
fn test_pub055_a1_e05() {
    test_pub("Pub055", "A Silent Voice", "05");
}
#[test]
fn test_pub055_a1_e06() {
    test_pub("Pub055", "A Silent Voice", "06");
}
#[test]
fn test_pub055_a1_e07() {
    test_pub("Pub055", "A Silent Voice", "07");
}
#[test]
fn test_pub055_a1_e08() {
    test_pub("Pub055", "A Silent Voice", "08");
}
#[test]
fn test_pub055_a1_e09() {
    test_pub("Pub055", "A Silent Voice", "09");
}
#[test]
fn test_pub055_a1_e10() {
    test_pub("Pub055", "A Silent Voice", "10");
}
#[test]
fn test_pub055_a1_e11() {
    test_pub("Pub055", "A Silent Voice", "11");
}
#[test]
fn test_pub055_a1_e12() {
    test_pub("Pub055", "A Silent Voice", "12");
}
#[test]
fn test_pub055_a2_e01() {
    test_pub("Pub055", "Your Name", "01");
}
#[test]
fn test_pub055_a2_e02() {
    test_pub("Pub055", "Your Name", "02");
}
#[test]
fn test_pub055_a2_e03() {
    test_pub("Pub055", "Your Name", "03");
}
#[test]
fn test_pub055_a2_e04() {
    test_pub("Pub055", "Your Name", "04");
}
#[test]
fn test_pub055_a2_e05() {
    test_pub("Pub055", "Your Name", "05");
}
#[test]
fn test_pub055_a2_e06() {
    test_pub("Pub055", "Your Name", "06");
}
#[test]
fn test_pub055_a2_e07() {
    test_pub("Pub055", "Your Name", "07");
}
#[test]
fn test_pub055_a2_e08() {
    test_pub("Pub055", "Your Name", "08");
}
#[test]
fn test_pub055_a2_e09() {
    test_pub("Pub055", "Your Name", "09");
}
#[test]
fn test_pub055_a2_e10() {
    test_pub("Pub055", "Your Name", "10");
}
#[test]
fn test_pub055_a2_e11() {
    test_pub("Pub055", "Your Name", "11");
}
#[test]
fn test_pub055_a2_e12() {
    test_pub("Pub055", "Your Name", "12");
}
#[test]
fn test_pub055_a3_e01() {
    test_pub("Pub055", "Weathering With You", "01");
}
#[test]
fn test_pub055_a3_e02() {
    test_pub("Pub055", "Weathering With You", "02");
}
#[test]
fn test_pub055_a3_e03() {
    test_pub("Pub055", "Weathering With You", "03");
}
#[test]
fn test_pub055_a3_e04() {
    test_pub("Pub055", "Weathering With You", "04");
}
#[test]
fn test_pub055_a3_e05() {
    test_pub("Pub055", "Weathering With You", "05");
}
#[test]
fn test_pub055_a3_e06() {
    test_pub("Pub055", "Weathering With You", "06");
}
#[test]
fn test_pub055_a3_e07() {
    test_pub("Pub055", "Weathering With You", "07");
}
#[test]
fn test_pub055_a3_e08() {
    test_pub("Pub055", "Weathering With You", "08");
}
#[test]
fn test_pub055_a3_e09() {
    test_pub("Pub055", "Weathering With You", "09");
}
#[test]
fn test_pub055_a3_e10() {
    test_pub("Pub055", "Weathering With You", "10");
}
#[test]
fn test_pub055_a3_e11() {
    test_pub("Pub055", "Weathering With You", "11");
}
#[test]
fn test_pub055_a3_e12() {
    test_pub("Pub055", "Weathering With You", "12");
}

// Pub056
#[test]
fn test_pub056_a1_e01() {
    test_pub("Pub056", "Spirited Away", "01");
}
#[test]
fn test_pub056_a1_e02() {
    test_pub("Pub056", "Spirited Away", "02");
}
#[test]
fn test_pub056_a1_e03() {
    test_pub("Pub056", "Spirited Away", "03");
}
#[test]
fn test_pub056_a1_e04() {
    test_pub("Pub056", "Spirited Away", "04");
}
#[test]
fn test_pub056_a1_e05() {
    test_pub("Pub056", "Spirited Away", "05");
}
#[test]
fn test_pub056_a1_e06() {
    test_pub("Pub056", "Spirited Away", "06");
}
#[test]
fn test_pub056_a1_e07() {
    test_pub("Pub056", "Spirited Away", "07");
}
#[test]
fn test_pub056_a1_e08() {
    test_pub("Pub056", "Spirited Away", "08");
}
#[test]
fn test_pub056_a1_e09() {
    test_pub("Pub056", "Spirited Away", "09");
}
#[test]
fn test_pub056_a1_e10() {
    test_pub("Pub056", "Spirited Away", "10");
}
#[test]
fn test_pub056_a1_e11() {
    test_pub("Pub056", "Spirited Away", "11");
}
#[test]
fn test_pub056_a1_e12() {
    test_pub("Pub056", "Spirited Away", "12");
}
#[test]
fn test_pub056_a2_e01() {
    test_pub("Pub056", "Howl's Moving Castle", "01");
}
#[test]
fn test_pub056_a2_e02() {
    test_pub("Pub056", "Howl's Moving Castle", "02");
}
#[test]
fn test_pub056_a2_e03() {
    test_pub("Pub056", "Howl's Moving Castle", "03");
}
#[test]
fn test_pub056_a2_e04() {
    test_pub("Pub056", "Howl's Moving Castle", "04");
}
#[test]
fn test_pub056_a2_e05() {
    test_pub("Pub056", "Howl's Moving Castle", "05");
}
#[test]
fn test_pub056_a2_e06() {
    test_pub("Pub056", "Howl's Moving Castle", "06");
}
#[test]
fn test_pub056_a2_e07() {
    test_pub("Pub056", "Howl's Moving Castle", "07");
}
#[test]
fn test_pub056_a2_e08() {
    test_pub("Pub056", "Howl's Moving Castle", "08");
}
#[test]
fn test_pub056_a2_e09() {
    test_pub("Pub056", "Howl's Moving Castle", "09");
}
#[test]
fn test_pub056_a2_e10() {
    test_pub("Pub056", "Howl's Moving Castle", "10");
}
#[test]
fn test_pub056_a2_e11() {
    test_pub("Pub056", "Howl's Moving Castle", "11");
}
#[test]
fn test_pub056_a2_e12() {
    test_pub("Pub056", "Howl's Moving Castle", "12");
}
#[test]
fn test_pub056_a3_e01() {
    test_pub("Pub056", "Princess Mononoke", "01");
}
#[test]
fn test_pub056_a3_e02() {
    test_pub("Pub056", "Princess Mononoke", "02");
}
#[test]
fn test_pub056_a3_e03() {
    test_pub("Pub056", "Princess Mononoke", "03");
}
#[test]
fn test_pub056_a3_e04() {
    test_pub("Pub056", "Princess Mononoke", "04");
}
#[test]
fn test_pub056_a3_e05() {
    test_pub("Pub056", "Princess Mononoke", "05");
}
#[test]
fn test_pub056_a3_e06() {
    test_pub("Pub056", "Princess Mononoke", "06");
}
#[test]
fn test_pub056_a3_e07() {
    test_pub("Pub056", "Princess Mononoke", "07");
}
#[test]
fn test_pub056_a3_e08() {
    test_pub("Pub056", "Princess Mononoke", "08");
}
#[test]
fn test_pub056_a3_e09() {
    test_pub("Pub056", "Princess Mononoke", "09");
}
#[test]
fn test_pub056_a3_e10() {
    test_pub("Pub056", "Princess Mononoke", "10");
}
#[test]
fn test_pub056_a3_e11() {
    test_pub("Pub056", "Princess Mononoke", "11");
}
#[test]
fn test_pub056_a3_e12() {
    test_pub("Pub056", "Princess Mononoke", "12");
}

// Pub057
#[test]
fn test_pub057_a1_e01() {
    test_pub("Pub057", "Wolf Children", "01");
}
#[test]
fn test_pub057_a1_e02() {
    test_pub("Pub057", "Wolf Children", "02");
}
#[test]
fn test_pub057_a1_e03() {
    test_pub("Pub057", "Wolf Children", "03");
}
#[test]
fn test_pub057_a1_e04() {
    test_pub("Pub057", "Wolf Children", "04");
}
#[test]
fn test_pub057_a1_e05() {
    test_pub("Pub057", "Wolf Children", "05");
}
#[test]
fn test_pub057_a1_e06() {
    test_pub("Pub057", "Wolf Children", "06");
}
#[test]
fn test_pub057_a1_e07() {
    test_pub("Pub057", "Wolf Children", "07");
}
#[test]
fn test_pub057_a1_e08() {
    test_pub("Pub057", "Wolf Children", "08");
}
#[test]
fn test_pub057_a1_e09() {
    test_pub("Pub057", "Wolf Children", "09");
}
#[test]
fn test_pub057_a1_e10() {
    test_pub("Pub057", "Wolf Children", "10");
}
#[test]
fn test_pub057_a1_e11() {
    test_pub("Pub057", "Wolf Children", "11");
}
#[test]
fn test_pub057_a1_e12() {
    test_pub("Pub057", "Wolf Children", "12");
}
#[test]
fn test_pub057_a2_e01() {
    test_pub("Pub057", "The Garden of Words", "01");
}
#[test]
fn test_pub057_a2_e02() {
    test_pub("Pub057", "The Garden of Words", "02");
}
#[test]
fn test_pub057_a2_e03() {
    test_pub("Pub057", "The Garden of Words", "03");
}
#[test]
fn test_pub057_a2_e04() {
    test_pub("Pub057", "The Garden of Words", "04");
}
#[test]
fn test_pub057_a2_e05() {
    test_pub("Pub057", "The Garden of Words", "05");
}
#[test]
fn test_pub057_a2_e06() {
    test_pub("Pub057", "The Garden of Words", "06");
}
#[test]
fn test_pub057_a2_e07() {
    test_pub("Pub057", "The Garden of Words", "07");
}
#[test]
fn test_pub057_a2_e08() {
    test_pub("Pub057", "The Garden of Words", "08");
}
#[test]
fn test_pub057_a2_e09() {
    test_pub("Pub057", "The Garden of Words", "09");
}
#[test]
fn test_pub057_a2_e10() {
    test_pub("Pub057", "The Garden of Words", "10");
}
#[test]
fn test_pub057_a2_e11() {
    test_pub("Pub057", "The Garden of Words", "11");
}
#[test]
fn test_pub057_a2_e12() {
    test_pub("Pub057", "The Garden of Words", "12");
}
#[test]
fn test_pub057_a3_e01() {
    test_pub("Pub057", "5 Centimeters Per Second", "01");
}
#[test]
fn test_pub057_a3_e02() {
    test_pub("Pub057", "5 Centimeters Per Second", "02");
}
#[test]
fn test_pub057_a3_e03() {
    test_pub("Pub057", "5 Centimeters Per Second", "03");
}
#[test]
fn test_pub057_a3_e04() {
    test_pub("Pub057", "5 Centimeters Per Second", "04");
}
#[test]
fn test_pub057_a3_e05() {
    test_pub("Pub057", "5 Centimeters Per Second", "05");
}
#[test]
fn test_pub057_a3_e06() {
    test_pub("Pub057", "5 Centimeters Per Second", "06");
}
#[test]
fn test_pub057_a3_e07() {
    test_pub("Pub057", "5 Centimeters Per Second", "07");
}
#[test]
fn test_pub057_a3_e08() {
    test_pub("Pub057", "5 Centimeters Per Second", "08");
}
#[test]
fn test_pub057_a3_e09() {
    test_pub("Pub057", "5 Centimeters Per Second", "09");
}
#[test]
fn test_pub057_a3_e10() {
    test_pub("Pub057", "5 Centimeters Per Second", "10");
}
#[test]
fn test_pub057_a3_e11() {
    test_pub("Pub057", "5 Centimeters Per Second", "11");
}
#[test]
fn test_pub057_a3_e12() {
    test_pub("Pub057", "5 Centimeters Per Second", "12");
}

// Pub058
#[test]
fn test_pub058_a1_e01() {
    test_pub("Pub058", "Clannad After Story", "01");
}
#[test]
fn test_pub058_a1_e02() {
    test_pub("Pub058", "Clannad After Story", "02");
}
#[test]
fn test_pub058_a1_e03() {
    test_pub("Pub058", "Clannad After Story", "03");
}
#[test]
fn test_pub058_a1_e04() {
    test_pub("Pub058", "Clannad After Story", "04");
}
#[test]
fn test_pub058_a1_e05() {
    test_pub("Pub058", "Clannad After Story", "05");
}
#[test]
fn test_pub058_a1_e06() {
    test_pub("Pub058", "Clannad After Story", "06");
}
#[test]
fn test_pub058_a1_e07() {
    test_pub("Pub058", "Clannad After Story", "07");
}
#[test]
fn test_pub058_a1_e08() {
    test_pub("Pub058", "Clannad After Story", "08");
}
#[test]
fn test_pub058_a1_e09() {
    test_pub("Pub058", "Clannad After Story", "09");
}
#[test]
fn test_pub058_a1_e10() {
    test_pub("Pub058", "Clannad After Story", "10");
}
#[test]
fn test_pub058_a1_e11() {
    test_pub("Pub058", "Clannad After Story", "11");
}
#[test]
fn test_pub058_a1_e12() {
    test_pub("Pub058", "Clannad After Story", "12");
}
#[test]
fn test_pub058_a2_e01() {
    test_pub("Pub058", "Plastic Memories", "01");
}
#[test]
fn test_pub058_a2_e02() {
    test_pub("Pub058", "Plastic Memories", "02");
}
#[test]
fn test_pub058_a2_e03() {
    test_pub("Pub058", "Plastic Memories", "03");
}
#[test]
fn test_pub058_a2_e04() {
    test_pub("Pub058", "Plastic Memories", "04");
}
#[test]
fn test_pub058_a2_e05() {
    test_pub("Pub058", "Plastic Memories", "05");
}
#[test]
fn test_pub058_a2_e06() {
    test_pub("Pub058", "Plastic Memories", "06");
}
#[test]
fn test_pub058_a2_e07() {
    test_pub("Pub058", "Plastic Memories", "07");
}
#[test]
fn test_pub058_a2_e08() {
    test_pub("Pub058", "Plastic Memories", "08");
}
#[test]
fn test_pub058_a2_e09() {
    test_pub("Pub058", "Plastic Memories", "09");
}
#[test]
fn test_pub058_a2_e10() {
    test_pub("Pub058", "Plastic Memories", "10");
}
#[test]
fn test_pub058_a2_e11() {
    test_pub("Pub058", "Plastic Memories", "11");
}
#[test]
fn test_pub058_a2_e12() {
    test_pub("Pub058", "Plastic Memories", "12");
}
#[test]
fn test_pub058_a3_e01() {
    test_pub("Pub058", "Erased", "01");
}
#[test]
fn test_pub058_a3_e02() {
    test_pub("Pub058", "Erased", "02");
}
#[test]
fn test_pub058_a3_e03() {
    test_pub("Pub058", "Erased", "03");
}
#[test]
fn test_pub058_a3_e04() {
    test_pub("Pub058", "Erased", "04");
}
#[test]
fn test_pub058_a3_e05() {
    test_pub("Pub058", "Erased", "05");
}
#[test]
fn test_pub058_a3_e06() {
    test_pub("Pub058", "Erased", "06");
}
#[test]
fn test_pub058_a3_e07() {
    test_pub("Pub058", "Erased", "07");
}
#[test]
fn test_pub058_a3_e08() {
    test_pub("Pub058", "Erased", "08");
}
#[test]
fn test_pub058_a3_e09() {
    test_pub("Pub058", "Erased", "09");
}
#[test]
fn test_pub058_a3_e10() {
    test_pub("Pub058", "Erased", "10");
}
#[test]
fn test_pub058_a3_e11() {
    test_pub("Pub058", "Erased", "11");
}
#[test]
fn test_pub058_a3_e12() {
    test_pub("Pub058", "Erased", "12");
}

// Pub059
#[test]
fn test_pub059_a1_e01() {
    test_pub("Pub059", "Orange", "01");
}
#[test]
fn test_pub059_a1_e02() {
    test_pub("Pub059", "Orange", "02");
}
#[test]
fn test_pub059_a1_e03() {
    test_pub("Pub059", "Orange", "03");
}
#[test]
fn test_pub059_a1_e04() {
    test_pub("Pub059", "Orange", "04");
}
#[test]
fn test_pub059_a1_e05() {
    test_pub("Pub059", "Orange", "05");
}
#[test]
fn test_pub059_a1_e06() {
    test_pub("Pub059", "Orange", "06");
}
#[test]
fn test_pub059_a1_e07() {
    test_pub("Pub059", "Orange", "07");
}
#[test]
fn test_pub059_a1_e08() {
    test_pub("Pub059", "Orange", "08");
}
#[test]
fn test_pub059_a1_e09() {
    test_pub("Pub059", "Orange", "09");
}
#[test]
fn test_pub059_a1_e10() {
    test_pub("Pub059", "Orange", "10");
}
#[test]
fn test_pub059_a1_e11() {
    test_pub("Pub059", "Orange", "11");
}
#[test]
fn test_pub059_a1_e12() {
    test_pub("Pub059", "Orange", "12");
}
#[test]
fn test_pub059_a2_e01() {
    test_pub("Pub059", "Planet Watermelon", "01");
}
#[test]
fn test_pub059_a2_e02() {
    test_pub("Pub059", "Planet Watermelon", "02");
}
#[test]
fn test_pub059_a2_e03() {
    test_pub("Pub059", "Planet Watermelon", "03");
}
#[test]
fn test_pub059_a2_e04() {
    test_pub("Pub059", "Planet Watermelon", "04");
}
#[test]
fn test_pub059_a2_e05() {
    test_pub("Pub059", "Planet Watermelon", "05");
}
#[test]
fn test_pub059_a2_e06() {
    test_pub("Pub059", "Planet Watermelon", "06");
}
#[test]
fn test_pub059_a2_e07() {
    test_pub("Pub059", "Planet Watermelon", "07");
}
#[test]
fn test_pub059_a2_e08() {
    test_pub("Pub059", "Planet Watermelon", "08");
}
#[test]
fn test_pub059_a2_e09() {
    test_pub("Pub059", "Planet Watermelon", "09");
}
#[test]
fn test_pub059_a2_e10() {
    test_pub("Pub059", "Planet Watermelon", "10");
}
#[test]
fn test_pub059_a2_e11() {
    test_pub("Pub059", "Planet Watermelon", "11");
}
#[test]
fn test_pub059_a2_e12() {
    test_pub("Pub059", "Planet Watermelon", "12");
}
#[test]
fn test_pub059_a3_e01() {
    test_pub("Pub059", "Anthem of the Heart", "01");
}
#[test]
fn test_pub059_a3_e02() {
    test_pub("Pub059", "Anthem of the Heart", "02");
}
#[test]
fn test_pub059_a3_e03() {
    test_pub("Pub059", "Anthem of the Heart", "03");
}
#[test]
fn test_pub059_a3_e04() {
    test_pub("Pub059", "Anthem of the Heart", "04");
}
#[test]
fn test_pub059_a3_e05() {
    test_pub("Pub059", "Anthem of the Heart", "05");
}
#[test]
fn test_pub059_a3_e06() {
    test_pub("Pub059", "Anthem of the Heart", "06");
}
#[test]
fn test_pub059_a3_e07() {
    test_pub("Pub059", "Anthem of the Heart", "07");
}
#[test]
fn test_pub059_a3_e08() {
    test_pub("Pub059", "Anthem of the Heart", "08");
}
#[test]
fn test_pub059_a3_e09() {
    test_pub("Pub059", "Anthem of the Heart", "09");
}
#[test]
fn test_pub059_a3_e10() {
    test_pub("Pub059", "Anthem of the Heart", "10");
}
#[test]
fn test_pub059_a3_e11() {
    test_pub("Pub059", "Anthem of the Heart", "11");
}
#[test]
fn test_pub059_a3_e12() {
    test_pub("Pub059", "Anthem of the Heart", "12");
}

// Pub060
#[test]
fn test_pub060_a1_e01() {
    test_pub("Pub060", "I Want to Eat Your Pancreas", "01");
}
#[test]
fn test_pub060_a1_e02() {
    test_pub("Pub060", "I Want to Eat Your Pancreas", "02");
}
#[test]
fn test_pub060_a1_e03() {
    test_pub("Pub060", "I Want to Eat Your Pancreas", "03");
}
#[test]
fn test_pub060_a1_e04() {
    test_pub("Pub060", "I Want to Eat Your Pancreas", "04");
}
#[test]
fn test_pub060_a1_e05() {
    test_pub("Pub060", "I Want to Eat Your Pancreas", "05");
}
#[test]
fn test_pub060_a1_e06() {
    test_pub("Pub060", "I Want to Eat Your Pancreas", "06");
}
#[test]
fn test_pub060_a1_e07() {
    test_pub("Pub060", "I Want to Eat Your Pancreas", "07");
}
#[test]
fn test_pub060_a1_e08() {
    test_pub("Pub060", "I Want to Eat Your Pancreas", "08");
}
#[test]
fn test_pub060_a1_e09() {
    test_pub("Pub060", "I Want to Eat Your Pancreas", "09");
}
#[test]
fn test_pub060_a1_e10() {
    test_pub("Pub060", "I Want to Eat Your Pancreas", "10");
}
#[test]
fn test_pub060_a1_e11() {
    test_pub("Pub060", "I Want to Eat Your Pancreas", "11");
}
#[test]
fn test_pub060_a1_e12() {
    test_pub("Pub060", "I Want to Eat Your Pancreas", "12");
}
#[test]
fn test_pub060_a2_e01() {
    test_pub("Pub060", "Ride Your Wave", "01");
}
#[test]
fn test_pub060_a2_e02() {
    test_pub("Pub060", "Ride Your Wave", "02");
}
#[test]
fn test_pub060_a2_e03() {
    test_pub("Pub060", "Ride Your Wave", "03");
}
#[test]
fn test_pub060_a2_e04() {
    test_pub("Pub060", "Ride Your Wave", "04");
}
#[test]
fn test_pub060_a2_e05() {
    test_pub("Pub060", "Ride Your Wave", "05");
}
#[test]
fn test_pub060_a2_e06() {
    test_pub("Pub060", "Ride Your Wave", "06");
}
#[test]
fn test_pub060_a2_e07() {
    test_pub("Pub060", "Ride Your Wave", "07");
}
#[test]
fn test_pub060_a2_e08() {
    test_pub("Pub060", "Ride Your Wave", "08");
}
#[test]
fn test_pub060_a2_e09() {
    test_pub("Pub060", "Ride Your Wave", "09");
}
#[test]
fn test_pub060_a2_e10() {
    test_pub("Pub060", "Ride Your Wave", "10");
}
#[test]
fn test_pub060_a2_e11() {
    test_pub("Pub060", "Ride Your Wave", "11");
}
#[test]
fn test_pub060_a2_e12() {
    test_pub("Pub060", "Ride Your Wave", "12");
}
#[test]
fn test_pub060_a3_e01() {
    test_pub("Pub060", "Doukyuusei", "01");
}
#[test]
fn test_pub060_a3_e02() {
    test_pub("Pub060", "Doukyuusei", "02");
}
#[test]
fn test_pub060_a3_e03() {
    test_pub("Pub060", "Doukyuusei", "03");
}
#[test]
fn test_pub060_a3_e04() {
    test_pub("Pub060", "Doukyuusei", "04");
}
#[test]
fn test_pub060_a3_e05() {
    test_pub("Pub060", "Doukyuusei", "05");
}
#[test]
fn test_pub060_a3_e06() {
    test_pub("Pub060", "Doukyuusei", "06");
}
#[test]
fn test_pub060_a3_e07() {
    test_pub("Pub060", "Doukyuusei", "07");
}
#[test]
fn test_pub060_a3_e08() {
    test_pub("Pub060", "Doukyuusei", "08");
}
#[test]
fn test_pub060_a3_e09() {
    test_pub("Pub060", "Doukyuusei", "09");
}
#[test]
fn test_pub060_a3_e10() {
    test_pub("Pub060", "Doukyuusei", "10");
}
#[test]
fn test_pub060_a3_e11() {
    test_pub("Pub060", "Doukyuusei", "11");
}
#[test]
fn test_pub060_a3_e12() {
    test_pub("Pub060", "Doukyuusei", "12");
}
