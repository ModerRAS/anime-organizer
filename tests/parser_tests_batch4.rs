// Auto-generated test file - Batch 4
// Publisher: Pub041-Pub050 (10 publishers × 3 anime × 12 episodes = 360 tests)
use anime_organizer::FilenameParser;
use std::path::PathBuf;

fn test_pub(name: &str, anime: &str, episode: &str) {
    let path = PathBuf::from(format!("[{}] {} - {} [1080P].mkv", name, anime, episode));
    let result = FilenameParser::parse(&path).unwrap();
    assert_eq!(result.publisher, name);
    assert_eq!(result.anime_name, anime);
    assert_eq!(result.episode, episode);
}

// Pub041
#[test]
fn test_pub041_a1_e01() {
    test_pub("Pub041", "Sword Art Online", "01");
}
#[test]
fn test_pub041_a1_e02() {
    test_pub("Pub041", "Sword Art Online", "02");
}
#[test]
fn test_pub041_a1_e03() {
    test_pub("Pub041", "Sword Art Online", "03");
}
#[test]
fn test_pub041_a1_e04() {
    test_pub("Pub041", "Sword Art Online", "04");
}
#[test]
fn test_pub041_a1_e05() {
    test_pub("Pub041", "Sword Art Online", "05");
}
#[test]
fn test_pub041_a1_e06() {
    test_pub("Pub041", "Sword Art Online", "06");
}
#[test]
fn test_pub041_a1_e07() {
    test_pub("Pub041", "Sword Art Online", "07");
}
#[test]
fn test_pub041_a1_e08() {
    test_pub("Pub041", "Sword Art Online", "08");
}
#[test]
fn test_pub041_a1_e09() {
    test_pub("Pub041", "Sword Art Online", "09");
}
#[test]
fn test_pub041_a1_e10() {
    test_pub("Pub041", "Sword Art Online", "10");
}
#[test]
fn test_pub041_a1_e11() {
    test_pub("Pub041", "Sword Art Online", "11");
}
#[test]
fn test_pub041_a1_e12() {
    test_pub("Pub041", "Sword Art Online", "12");
}
#[test]
fn test_pub041_a2_e01() {
    test_pub("Pub041", "Attack on Titan", "01");
}
#[test]
fn test_pub041_a2_e02() {
    test_pub("Pub041", "Attack on Titan", "02");
}
#[test]
fn test_pub041_a2_e03() {
    test_pub("Pub041", "Attack on Titan", "03");
}
#[test]
fn test_pub041_a2_e04() {
    test_pub("Pub041", "Attack on Titan", "04");
}
#[test]
fn test_pub041_a2_e05() {
    test_pub("Pub041", "Attack on Titan", "05");
}
#[test]
fn test_pub041_a2_e06() {
    test_pub("Pub041", "Attack on Titan", "06");
}
#[test]
fn test_pub041_a2_e07() {
    test_pub("Pub041", "Attack on Titan", "07");
}
#[test]
fn test_pub041_a2_e08() {
    test_pub("Pub041", "Attack on Titan", "08");
}
#[test]
fn test_pub041_a2_e09() {
    test_pub("Pub041", "Attack on Titan", "09");
}
#[test]
fn test_pub041_a2_e10() {
    test_pub("Pub041", "Attack on Titan", "10");
}
#[test]
fn test_pub041_a2_e11() {
    test_pub("Pub041", "Attack on Titan", "11");
}
#[test]
fn test_pub041_a2_e12() {
    test_pub("Pub041", "Attack on Titan", "12");
}
#[test]
fn test_pub041_a3_e01() {
    test_pub("Pub041", "My Hero Academia", "01");
}
#[test]
fn test_pub041_a3_e02() {
    test_pub("Pub041", "My Hero Academia", "02");
}
#[test]
fn test_pub041_a3_e03() {
    test_pub("Pub041", "My Hero Academia", "03");
}
#[test]
fn test_pub041_a3_e04() {
    test_pub("Pub041", "My Hero Academia", "04");
}
#[test]
fn test_pub041_a3_e05() {
    test_pub("Pub041", "My Hero Academia", "05");
}
#[test]
fn test_pub041_a3_e06() {
    test_pub("Pub041", "My Hero Academia", "06");
}
#[test]
fn test_pub041_a3_e07() {
    test_pub("Pub041", "My Hero Academia", "07");
}
#[test]
fn test_pub041_a3_e08() {
    test_pub("Pub041", "My Hero Academia", "08");
}
#[test]
fn test_pub041_a3_e09() {
    test_pub("Pub041", "My Hero Academia", "09");
}
#[test]
fn test_pub041_a3_e10() {
    test_pub("Pub041", "My Hero Academia", "10");
}
#[test]
fn test_pub041_a3_e11() {
    test_pub("Pub041", "My Hero Academia", "11");
}
#[test]
fn test_pub041_a3_e12() {
    test_pub("Pub041", "My Hero Academia", "12");
}

// Pub042
#[test]
fn test_pub042_a1_e01() {
    test_pub("Pub042", "Re:Zero", "01");
}
#[test]
fn test_pub042_a1_e02() {
    test_pub("Pub042", "Re:Zero", "02");
}
#[test]
fn test_pub042_a1_e03() {
    test_pub("Pub042", "Re:Zero", "03");
}
#[test]
fn test_pub042_a1_e04() {
    test_pub("Pub042", "Re:Zero", "04");
}
#[test]
fn test_pub042_a1_e05() {
    test_pub("Pub042", "Re:Zero", "05");
}
#[test]
fn test_pub042_a1_e06() {
    test_pub("Pub042", "Re:Zero", "06");
}
#[test]
fn test_pub042_a1_e07() {
    test_pub("Pub042", "Re:Zero", "07");
}
#[test]
fn test_pub042_a1_e08() {
    test_pub("Pub042", "Re:Zero", "08");
}
#[test]
fn test_pub042_a1_e09() {
    test_pub("Pub042", "Re:Zero", "09");
}
#[test]
fn test_pub042_a1_e10() {
    test_pub("Pub042", "Re:Zero", "10");
}
#[test]
fn test_pub042_a1_e11() {
    test_pub("Pub042", "Re:Zero", "11");
}
#[test]
fn test_pub042_a1_e12() {
    test_pub("Pub042", "Re:Zero", "12");
}
#[test]
fn test_pub042_a2_e01() {
    test_pub("Pub042", "Demon Slayer", "01");
}
#[test]
fn test_pub042_a2_e02() {
    test_pub("Pub042", "Demon Slayer", "02");
}
#[test]
fn test_pub042_a2_e03() {
    test_pub("Pub042", "Demon Slayer", "03");
}
#[test]
fn test_pub042_a2_e04() {
    test_pub("Pub042", "Demon Slayer", "04");
}
#[test]
fn test_pub042_a2_e05() {
    test_pub("Pub042", "Demon Slayer", "05");
}
#[test]
fn test_pub042_a2_e06() {
    test_pub("Pub042", "Demon Slayer", "06");
}
#[test]
fn test_pub042_a2_e07() {
    test_pub("Pub042", "Demon Slayer", "07");
}
#[test]
fn test_pub042_a2_e08() {
    test_pub("Pub042", "Demon Slayer", "08");
}
#[test]
fn test_pub042_a2_e09() {
    test_pub("Pub042", "Demon Slayer", "09");
}
#[test]
fn test_pub042_a2_e10() {
    test_pub("Pub042", "Demon Slayer", "10");
}
#[test]
fn test_pub042_a2_e11() {
    test_pub("Pub042", "Demon Slayer", "11");
}
#[test]
fn test_pub042_a2_e12() {
    test_pub("Pub042", "Demon Slayer", "12");
}
#[test]
fn test_pub042_a3_e01() {
    test_pub("Pub042", "One Punch Man", "01");
}
#[test]
fn test_pub042_a3_e02() {
    test_pub("Pub042", "One Punch Man", "02");
}
#[test]
fn test_pub042_a3_e03() {
    test_pub("Pub042", "One Punch Man", "03");
}
#[test]
fn test_pub042_a3_e04() {
    test_pub("Pub042", "One Punch Man", "04");
}
#[test]
fn test_pub042_a3_e05() {
    test_pub("Pub042", "One Punch Man", "05");
}
#[test]
fn test_pub042_a3_e06() {
    test_pub("Pub042", "One Punch Man", "06");
}
#[test]
fn test_pub042_a3_e07() {
    test_pub("Pub042", "One Punch Man", "07");
}
#[test]
fn test_pub042_a3_e08() {
    test_pub("Pub042", "One Punch Man", "08");
}
#[test]
fn test_pub042_a3_e09() {
    test_pub("Pub042", "One Punch Man", "09");
}
#[test]
fn test_pub042_a3_e10() {
    test_pub("Pub042", "One Punch Man", "10");
}
#[test]
fn test_pub042_a3_e11() {
    test_pub("Pub042", "One Punch Man", "11");
}
#[test]
fn test_pub042_a3_e12() {
    test_pub("Pub042", "One Punch Man", "12");
}

// Pub043
#[test]
fn test_pub043_a1_e01() {
    test_pub("Pub043", "Jujutsu Kaisen", "01");
}
#[test]
fn test_pub043_a1_e02() {
    test_pub("Pub043", "Jujutsu Kaisen", "02");
}
#[test]
fn test_pub043_a1_e03() {
    test_pub("Pub043", "Jujutsu Kaisen", "03");
}
#[test]
fn test_pub043_a1_e04() {
    test_pub("Pub043", "Jujutsu Kaisen", "04");
}
#[test]
fn test_pub043_a1_e05() {
    test_pub("Pub043", "Jujutsu Kaisen", "05");
}
#[test]
fn test_pub043_a1_e06() {
    test_pub("Pub043", "Jujutsu Kaisen", "06");
}
#[test]
fn test_pub043_a1_e07() {
    test_pub("Pub043", "Jujutsu Kaisen", "07");
}
#[test]
fn test_pub043_a1_e08() {
    test_pub("Pub043", "Jujutsu Kaisen", "08");
}
#[test]
fn test_pub043_a1_e09() {
    test_pub("Pub043", "Jujutsu Kaisen", "09");
}
#[test]
fn test_pub043_a1_e10() {
    test_pub("Pub043", "Jujutsu Kaisen", "10");
}
#[test]
fn test_pub043_a1_e11() {
    test_pub("Pub043", "Jujutsu Kaisen", "11");
}
#[test]
fn test_pub043_a1_e12() {
    test_pub("Pub043", "Jujutsu Kaisen", "12");
}
#[test]
fn test_pub043_a2_e01() {
    test_pub("Pub043", "Spy x Family", "01");
}
#[test]
fn test_pub043_a2_e02() {
    test_pub("Pub043", "Spy x Family", "02");
}
#[test]
fn test_pub043_a2_e03() {
    test_pub("Pub043", "Spy x Family", "03");
}
#[test]
fn test_pub043_a2_e04() {
    test_pub("Pub043", "Spy x Family", "04");
}
#[test]
fn test_pub043_a2_e05() {
    test_pub("Pub043", "Spy x Family", "05");
}
#[test]
fn test_pub043_a2_e06() {
    test_pub("Pub043", "Spy x Family", "06");
}
#[test]
fn test_pub043_a2_e07() {
    test_pub("Pub043", "Spy x Family", "07");
}
#[test]
fn test_pub043_a2_e08() {
    test_pub("Pub043", "Spy x Family", "08");
}
#[test]
fn test_pub043_a2_e09() {
    test_pub("Pub043", "Spy x Family", "09");
}
#[test]
fn test_pub043_a2_e10() {
    test_pub("Pub043", "Spy x Family", "10");
}
#[test]
fn test_pub043_a2_e11() {
    test_pub("Pub043", "Spy x Family", "11");
}
#[test]
fn test_pub043_a2_e12() {
    test_pub("Pub043", "Spy x Family", "12");
}
#[test]
fn test_pub043_a3_e01() {
    test_pub("Pub043", "Chainsaw Man", "01");
}
#[test]
fn test_pub043_a3_e02() {
    test_pub("Pub043", "Chainsaw Man", "02");
}
#[test]
fn test_pub043_a3_e03() {
    test_pub("Pub043", "Chainsaw Man", "03");
}
#[test]
fn test_pub043_a3_e04() {
    test_pub("Pub043", "Chainsaw Man", "04");
}
#[test]
fn test_pub043_a3_e05() {
    test_pub("Pub043", "Chainsaw Man", "05");
}
#[test]
fn test_pub043_a3_e06() {
    test_pub("Pub043", "Chainsaw Man", "06");
}
#[test]
fn test_pub043_a3_e07() {
    test_pub("Pub043", "Chainsaw Man", "07");
}
#[test]
fn test_pub043_a3_e08() {
    test_pub("Pub043", "Chainsaw Man", "08");
}
#[test]
fn test_pub043_a3_e09() {
    test_pub("Pub043", "Chainsaw Man", "09");
}
#[test]
fn test_pub043_a3_e10() {
    test_pub("Pub043", "Chainsaw Man", "10");
}
#[test]
fn test_pub043_a3_e11() {
    test_pub("Pub043", "Chainsaw Man", "11");
}
#[test]
fn test_pub043_a3_e12() {
    test_pub("Pub043", "Chainsaw Man", "12");
}

// Pub044
#[test]
fn test_pub044_a1_e01() {
    test_pub("Pub044", "Vinland Saga", "01");
}
#[test]
fn test_pub044_a1_e02() {
    test_pub("Pub044", "Vinland Saga", "02");
}
#[test]
fn test_pub044_a1_e03() {
    test_pub("Pub044", "Vinland Saga", "03");
}
#[test]
fn test_pub044_a1_e04() {
    test_pub("Pub044", "Vinland Saga", "04");
}
#[test]
fn test_pub044_a1_e05() {
    test_pub("Pub044", "Vinland Saga", "05");
}
#[test]
fn test_pub044_a1_e06() {
    test_pub("Pub044", "Vinland Saga", "06");
}
#[test]
fn test_pub044_a1_e07() {
    test_pub("Pub044", "Vinland Saga", "07");
}
#[test]
fn test_pub044_a1_e08() {
    test_pub("Pub044", "Vinland Saga", "08");
}
#[test]
fn test_pub044_a1_e09() {
    test_pub("Pub044", "Vinland Saga", "09");
}
#[test]
fn test_pub044_a1_e10() {
    test_pub("Pub044", "Vinland Saga", "10");
}
#[test]
fn test_pub044_a1_e11() {
    test_pub("Pub044", "Vinland Saga", "11");
}
#[test]
fn test_pub044_a1_e12() {
    test_pub("Pub044", "Vinland Saga", "12");
}
#[test]
fn test_pub044_a2_e01() {
    test_pub("Pub044", "Mob Psycho 100", "01");
}
#[test]
fn test_pub044_a2_e02() {
    test_pub("Pub044", "Mob Psycho 100", "02");
}
#[test]
fn test_pub044_a2_e03() {
    test_pub("Pub044", "Mob Psycho 100", "03");
}
#[test]
fn test_pub044_a2_e04() {
    test_pub("Pub044", "Mob Psycho 100", "04");
}
#[test]
fn test_pub044_a2_e05() {
    test_pub("Pub044", "Mob Psycho 100", "05");
}
#[test]
fn test_pub044_a2_e06() {
    test_pub("Pub044", "Mob Psycho 100", "06");
}
#[test]
fn test_pub044_a2_e07() {
    test_pub("Pub044", "Mob Psycho 100", "07");
}
#[test]
fn test_pub044_a2_e08() {
    test_pub("Pub044", "Mob Psycho 100", "08");
}
#[test]
fn test_pub044_a2_e09() {
    test_pub("Pub044", "Mob Psycho 100", "09");
}
#[test]
fn test_pub044_a2_e10() {
    test_pub("Pub044", "Mob Psycho 100", "10");
}
#[test]
fn test_pub044_a2_e11() {
    test_pub("Pub044", "Mob Psycho 100", "11");
}
#[test]
fn test_pub044_a2_e12() {
    test_pub("Pub044", "Mob Psycho 100", "12");
}
#[test]
fn test_pub044_a3_e01() {
    test_pub("Pub044", "Dr. Stone", "01");
}
#[test]
fn test_pub044_a3_e02() {
    test_pub("Pub044", "Dr. Stone", "02");
}
#[test]
fn test_pub044_a3_e03() {
    test_pub("Pub044", "Dr. Stone", "03");
}
#[test]
fn test_pub044_a3_e04() {
    test_pub("Pub044", "Dr. Stone", "04");
}
#[test]
fn test_pub044_a3_e05() {
    test_pub("Pub044", "Dr. Stone", "05");
}
#[test]
fn test_pub044_a3_e06() {
    test_pub("Pub044", "Dr. Stone", "06");
}
#[test]
fn test_pub044_a3_e07() {
    test_pub("Pub044", "Dr. Stone", "07");
}
#[test]
fn test_pub044_a3_e08() {
    test_pub("Pub044", "Dr. Stone", "08");
}
#[test]
fn test_pub044_a3_e09() {
    test_pub("Pub044", "Dr. Stone", "09");
}
#[test]
fn test_pub044_a3_e10() {
    test_pub("Pub044", "Dr. Stone", "10");
}
#[test]
fn test_pub044_a3_e11() {
    test_pub("Pub044", "Dr. Stone", "11");
}
#[test]
fn test_pub044_a3_e12() {
    test_pub("Pub044", "Dr. Stone", "12");
}

// Pub045
#[test]
fn test_pub045_a1_e01() {
    test_pub("Pub045", "Tokyo Revengers", "01");
}
#[test]
fn test_pub045_a1_e02() {
    test_pub("Pub045", "Tokyo Revengers", "02");
}
#[test]
fn test_pub045_a1_e03() {
    test_pub("Pub045", "Tokyo Revengers", "03");
}
#[test]
fn test_pub045_a1_e04() {
    test_pub("Pub045", "Tokyo Revengers", "04");
}
#[test]
fn test_pub045_a1_e05() {
    test_pub("Pub045", "Tokyo Revengers", "05");
}
#[test]
fn test_pub045_a1_e06() {
    test_pub("Pub045", "Tokyo Revengers", "06");
}
#[test]
fn test_pub045_a1_e07() {
    test_pub("Pub045", "Tokyo Revengers", "07");
}
#[test]
fn test_pub045_a1_e08() {
    test_pub("Pub045", "Tokyo Revengers", "08");
}
#[test]
fn test_pub045_a1_e09() {
    test_pub("Pub045", "Tokyo Revengers", "09");
}
#[test]
fn test_pub045_a1_e10() {
    test_pub("Pub045", "Tokyo Revengers", "10");
}
#[test]
fn test_pub045_a1_e11() {
    test_pub("Pub045", "Tokyo Revengers", "11");
}
#[test]
fn test_pub045_a1_e12() {
    test_pub("Pub045", "Tokyo Revengers", "12");
}
#[test]
fn test_pub045_a2_e01() {
    test_pub("Pub045", "Fire Force", "01");
}
#[test]
fn test_pub045_a2_e02() {
    test_pub("Pub045", "Fire Force", "02");
}
#[test]
fn test_pub045_a2_e03() {
    test_pub("Pub045", "Fire Force", "03");
}
#[test]
fn test_pub045_a2_e04() {
    test_pub("Pub045", "Fire Force", "04");
}
#[test]
fn test_pub045_a2_e05() {
    test_pub("Pub045", "Fire Force", "05");
}
#[test]
fn test_pub045_a2_e06() {
    test_pub("Pub045", "Fire Force", "06");
}
#[test]
fn test_pub045_a2_e07() {
    test_pub("Pub045", "Fire Force", "07");
}
#[test]
fn test_pub045_a2_e08() {
    test_pub("Pub045", "Fire Force", "08");
}
#[test]
fn test_pub045_a2_e09() {
    test_pub("Pub045", "Fire Force", "09");
}
#[test]
fn test_pub045_a2_e10() {
    test_pub("Pub045", "Fire Force", "10");
}
#[test]
fn test_pub045_a2_e11() {
    test_pub("Pub045", "Fire Force", "11");
}
#[test]
fn test_pub045_a2_e12() {
    test_pub("Pub045", "Fire Force", "12");
}
#[test]
fn test_pub045_a3_e01() {
    test_pub("Pub045", "The Promised Neverland", "01");
}
#[test]
fn test_pub045_a3_e02() {
    test_pub("Pub045", "The Promised Neverland", "02");
}
#[test]
fn test_pub045_a3_e03() {
    test_pub("Pub045", "The Promised Neverland", "03");
}
#[test]
fn test_pub045_a3_e04() {
    test_pub("Pub045", "The Promised Neverland", "04");
}
#[test]
fn test_pub045_a3_e05() {
    test_pub("Pub045", "The Promised Neverland", "05");
}
#[test]
fn test_pub045_a3_e06() {
    test_pub("Pub045", "The Promised Neverland", "06");
}
#[test]
fn test_pub045_a3_e07() {
    test_pub("Pub045", "The Promised Neverland", "07");
}
#[test]
fn test_pub045_a3_e08() {
    test_pub("Pub045", "The Promised Neverland", "08");
}
#[test]
fn test_pub045_a3_e09() {
    test_pub("Pub045", "The Promised Neverland", "09");
}
#[test]
fn test_pub045_a3_e10() {
    test_pub("Pub045", "The Promised Neverland", "10");
}
#[test]
fn test_pub045_a3_e11() {
    test_pub("Pub045", "The Promised Neverland", "11");
}
#[test]
fn test_pub045_a3_e12() {
    test_pub("Pub045", "The Promised Neverland", "12");
}

// Pub046
#[test]
fn test_pub046_a1_e01() {
    test_pub("Pub046", "Black Clover", "01");
}
#[test]
fn test_pub046_a1_e02() {
    test_pub("Pub046", "Black Clover", "02");
}
#[test]
fn test_pub046_a1_e03() {
    test_pub("Pub046", "Black Clover", "03");
}
#[test]
fn test_pub046_a1_e04() {
    test_pub("Pub046", "Black Clover", "04");
}
#[test]
fn test_pub046_a1_e05() {
    test_pub("Pub046", "Black Clover", "05");
}
#[test]
fn test_pub046_a1_e06() {
    test_pub("Pub046", "Black Clover", "06");
}
#[test]
fn test_pub046_a1_e07() {
    test_pub("Pub046", "Black Clover", "07");
}
#[test]
fn test_pub046_a1_e08() {
    test_pub("Pub046", "Black Clover", "08");
}
#[test]
fn test_pub046_a1_e09() {
    test_pub("Pub046", "Black Clover", "09");
}
#[test]
fn test_pub046_a1_e10() {
    test_pub("Pub046", "Black Clover", "10");
}
#[test]
fn test_pub046_a1_e11() {
    test_pub("Pub046", "Black Clover", "11");
}
#[test]
fn test_pub046_a1_e12() {
    test_pub("Pub046", "Black Clover", "12");
}
#[test]
fn test_pub046_a2_e01() {
    test_pub("Pub046", "Haikyu!!", "01");
}
#[test]
fn test_pub046_a2_e02() {
    test_pub("Pub046", "Haikyu!!", "02");
}
#[test]
fn test_pub046_a2_e03() {
    test_pub("Pub046", "Haikyu!!", "03");
}
#[test]
fn test_pub046_a2_e04() {
    test_pub("Pub046", "Haikyu!!", "04");
}
#[test]
fn test_pub046_a2_e05() {
    test_pub("Pub046", "Haikyu!!", "05");
}
#[test]
fn test_pub046_a2_e06() {
    test_pub("Pub046", "Haikyu!!", "06");
}
#[test]
fn test_pub046_a2_e07() {
    test_pub("Pub046", "Haikyu!!", "07");
}
#[test]
fn test_pub046_a2_e08() {
    test_pub("Pub046", "Haikyu!!", "08");
}
#[test]
fn test_pub046_a2_e09() {
    test_pub("Pub046", "Haikyu!!", "09");
}
#[test]
fn test_pub046_a2_e10() {
    test_pub("Pub046", "Haikyu!!", "10");
}
#[test]
fn test_pub046_a2_e11() {
    test_pub("Pub046", "Haikyu!!", "11");
}
#[test]
fn test_pub046_a2_e12() {
    test_pub("Pub046", "Haikyu!!", "12");
}
#[test]
fn test_pub046_a3_e01() {
    test_pub("Pub046", "Hunter x Hunter", "01");
}
#[test]
fn test_pub046_a3_e02() {
    test_pub("Pub046", "Hunter x Hunter", "02");
}
#[test]
fn test_pub046_a3_e03() {
    test_pub("Pub046", "Hunter x Hunter", "03");
}
#[test]
fn test_pub046_a3_e04() {
    test_pub("Pub046", "Hunter x Hunter", "04");
}
#[test]
fn test_pub046_a3_e05() {
    test_pub("Pub046", "Hunter x Hunter", "05");
}
#[test]
fn test_pub046_a3_e06() {
    test_pub("Pub046", "Hunter x Hunter", "06");
}
#[test]
fn test_pub046_a3_e07() {
    test_pub("Pub046", "Hunter x Hunter", "07");
}
#[test]
fn test_pub046_a3_e08() {
    test_pub("Pub046", "Hunter x Hunter", "08");
}
#[test]
fn test_pub046_a3_e09() {
    test_pub("Pub046", "Hunter x Hunter", "09");
}
#[test]
fn test_pub046_a3_e10() {
    test_pub("Pub046", "Hunter x Hunter", "10");
}
#[test]
fn test_pub046_a3_e11() {
    test_pub("Pub046", "Hunter x Hunter", "11");
}
#[test]
fn test_pub046_a3_e12() {
    test_pub("Pub046", "Hunter x Hunter", "12");
}

// Pub047
#[test]
fn test_pub047_a1_e01() {
    test_pub("Pub047", "Assassination Classroom", "01");
}
#[test]
fn test_pub047_a1_e02() {
    test_pub("Pub047", "Assassination Classroom", "02");
}
#[test]
fn test_pub047_a1_e03() {
    test_pub("Pub047", "Assassination Classroom", "03");
}
#[test]
fn test_pub047_a1_e04() {
    test_pub("Pub047", "Assassination Classroom", "04");
}
#[test]
fn test_pub047_a1_e05() {
    test_pub("Pub047", "Assassination Classroom", "05");
}
#[test]
fn test_pub047_a1_e06() {
    test_pub("Pub047", "Assassination Classroom", "06");
}
#[test]
fn test_pub047_a1_e07() {
    test_pub("Pub047", "Assassination Classroom", "07");
}
#[test]
fn test_pub047_a1_e08() {
    test_pub("Pub047", "Assassination Classroom", "08");
}
#[test]
fn test_pub047_a1_e09() {
    test_pub("Pub047", "Assassination Classroom", "09");
}
#[test]
fn test_pub047_a1_e10() {
    test_pub("Pub047", "Assassination Classroom", "10");
}
#[test]
fn test_pub047_a1_e11() {
    test_pub("Pub047", "Assassination Classroom", "11");
}
#[test]
fn test_pub047_a1_e12() {
    test_pub("Pub047", "Assassination Classroom", "12");
}
#[test]
fn test_pub047_a2_e01() {
    test_pub("Pub047", "Noragami", "01");
}
#[test]
fn test_pub047_a2_e02() {
    test_pub("Pub047", "Noragami", "02");
}
#[test]
fn test_pub047_a2_e03() {
    test_pub("Pub047", "Noragami", "03");
}
#[test]
fn test_pub047_a2_e04() {
    test_pub("Pub047", "Noragami", "04");
}
#[test]
fn test_pub047_a2_e05() {
    test_pub("Pub047", "Noragami", "05");
}
#[test]
fn test_pub047_a2_e06() {
    test_pub("Pub047", "Noragami", "06");
}
#[test]
fn test_pub047_a2_e07() {
    test_pub("Pub047", "Noragami", "07");
}
#[test]
fn test_pub047_a2_e08() {
    test_pub("Pub047", "Noragami", "08");
}
#[test]
fn test_pub047_a2_e09() {
    test_pub("Pub047", "Noragami", "09");
}
#[test]
fn test_pub047_a2_e10() {
    test_pub("Pub047", "Noragami", "10");
}
#[test]
fn test_pub047_a2_e11() {
    test_pub("Pub047", "Noragami", "11");
}
#[test]
fn test_pub047_a2_e12() {
    test_pub("Pub047", "Noragami", "12");
}
#[test]
fn test_pub047_a3_e01() {
    test_pub("Pub047", "Blue Exorcist", "01");
}
#[test]
fn test_pub047_a3_e02() {
    test_pub("Pub047", "Blue Exorcist", "02");
}
#[test]
fn test_pub047_a3_e03() {
    test_pub("Pub047", "Blue Exorcist", "03");
}
#[test]
fn test_pub047_a3_e04() {
    test_pub("Pub047", "Blue Exorcist", "04");
}
#[test]
fn test_pub047_a3_e05() {
    test_pub("Pub047", "Blue Exorcist", "05");
}
#[test]
fn test_pub047_a3_e06() {
    test_pub("Pub047", "Blue Exorcist", "06");
}
#[test]
fn test_pub047_a3_e07() {
    test_pub("Pub047", "Blue Exorcist", "07");
}
#[test]
fn test_pub047_a3_e08() {
    test_pub("Pub047", "Blue Exorcist", "08");
}
#[test]
fn test_pub047_a3_e09() {
    test_pub("Pub047", "Blue Exorcist", "09");
}
#[test]
fn test_pub047_a3_e10() {
    test_pub("Pub047", "Blue Exorcist", "10");
}
#[test]
fn test_pub047_a3_e11() {
    test_pub("Pub047", "Blue Exorcist", "11");
}
#[test]
fn test_pub047_a3_e12() {
    test_pub("Pub047", "Blue Exorcist", "12");
}

// Pub048
#[test]
fn test_pub048_a1_e01() {
    test_pub("Pub048", "Fate/Stay Night", "01");
}
#[test]
fn test_pub048_a1_e02() {
    test_pub("Pub048", "Fate/Stay Night", "02");
}
#[test]
fn test_pub048_a1_e03() {
    test_pub("Pub048", "Fate/Stay Night", "03");
}
#[test]
fn test_pub048_a1_e04() {
    test_pub("Pub048", "Fate/Stay Night", "04");
}
#[test]
fn test_pub048_a1_e05() {
    test_pub("Pub048", "Fate/Stay Night", "05");
}
#[test]
fn test_pub048_a1_e06() {
    test_pub("Pub048", "Fate/Stay Night", "06");
}
#[test]
fn test_pub048_a1_e07() {
    test_pub("Pub048", "Fate/Stay Night", "07");
}
#[test]
fn test_pub048_a1_e08() {
    test_pub("Pub048", "Fate/Stay Night", "08");
}
#[test]
fn test_pub048_a1_e09() {
    test_pub("Pub048", "Fate/Stay Night", "09");
}
#[test]
fn test_pub048_a1_e10() {
    test_pub("Pub048", "Fate/Stay Night", "10");
}
#[test]
fn test_pub048_a1_e11() {
    test_pub("Pub048", "Fate/Stay Night", "11");
}
#[test]
fn test_pub048_a1_e12() {
    test_pub("Pub048", "Fate/Stay Night", "12");
}
#[test]
fn test_pub048_a2_e01() {
    test_pub("Pub048", "Fate/Zero", "01");
}
#[test]
fn test_pub048_a2_e02() {
    test_pub("Pub048", "Fate/Zero", "02");
}
#[test]
fn test_pub048_a2_e03() {
    test_pub("Pub048", "Fate/Zero", "03");
}
#[test]
fn test_pub048_a2_e04() {
    test_pub("Pub048", "Fate/Zero", "04");
}
#[test]
fn test_pub048_a2_e05() {
    test_pub("Pub048", "Fate/Zero", "05");
}
#[test]
fn test_pub048_a2_e06() {
    test_pub("Pub048", "Fate/Zero", "06");
}
#[test]
fn test_pub048_a2_e07() {
    test_pub("Pub048", "Fate/Zero", "07");
}
#[test]
fn test_pub048_a2_e08() {
    test_pub("Pub048", "Fate/Zero", "08");
}
#[test]
fn test_pub048_a2_e09() {
    test_pub("Pub048", "Fate/Zero", "09");
}
#[test]
fn test_pub048_a2_e10() {
    test_pub("Pub048", "Fate/Zero", "10");
}
#[test]
fn test_pub048_a2_e11() {
    test_pub("Pub048", "Fate/Zero", "11");
}
#[test]
fn test_pub048_a2_e12() {
    test_pub("Pub048", "Fate/Zero", "12");
}
#[test]
fn test_pub048_a3_e01() {
    test_pub("Pub048", "Fate/Apocrypha", "01");
}
#[test]
fn test_pub048_a3_e02() {
    test_pub("Pub048", "Fate/Apocrypha", "02");
}
#[test]
fn test_pub048_a3_e03() {
    test_pub("Pub048", "Fate/Apocrypha", "03");
}
#[test]
fn test_pub048_a3_e04() {
    test_pub("Pub048", "Fate/Apocrypha", "04");
}
#[test]
fn test_pub048_a3_e05() {
    test_pub("Pub048", "Fate/Apocrypha", "05");
}
#[test]
fn test_pub048_a3_e06() {
    test_pub("Pub048", "Fate/Apocrypha", "06");
}
#[test]
fn test_pub048_a3_e07() {
    test_pub("Pub048", "Fate/Apocrypha", "07");
}
#[test]
fn test_pub048_a3_e08() {
    test_pub("Pub048", "Fate/Apocrypha", "08");
}
#[test]
fn test_pub048_a3_e09() {
    test_pub("Pub048", "Fate/Apocrypha", "09");
}
#[test]
fn test_pub048_a3_e10() {
    test_pub("Pub048", "Fate/Apocrypha", "10");
}
#[test]
fn test_pub048_a3_e11() {
    test_pub("Pub048", "Fate/Apocrypha", "11");
}
#[test]
fn test_pub048_a3_e12() {
    test_pub("Pub048", "Fate/Apocrypha", "12");
}

// Pub049
#[test]
fn test_pub049_a1_e01() {
    test_pub("Pub049", "Bleach", "01");
}
#[test]
fn test_pub049_a1_e02() {
    test_pub("Pub049", "Bleach", "02");
}
#[test]
fn test_pub049_a1_e03() {
    test_pub("Pub049", "Bleach", "03");
}
#[test]
fn test_pub049_a1_e04() {
    test_pub("Pub049", "Bleach", "04");
}
#[test]
fn test_pub049_a1_e05() {
    test_pub("Pub049", "Bleach", "05");
}
#[test]
fn test_pub049_a1_e06() {
    test_pub("Pub049", "Bleach", "06");
}
#[test]
fn test_pub049_a1_e07() {
    test_pub("Pub049", "Bleach", "07");
}
#[test]
fn test_pub049_a1_e08() {
    test_pub("Pub049", "Bleach", "08");
}
#[test]
fn test_pub049_a1_e09() {
    test_pub("Pub049", "Bleach", "09");
}
#[test]
fn test_pub049_a1_e10() {
    test_pub("Pub049", "Bleach", "10");
}
#[test]
fn test_pub049_a1_e11() {
    test_pub("Pub049", "Bleach", "11");
}
#[test]
fn test_pub049_a1_e12() {
    test_pub("Pub049", "Bleach", "12");
}
#[test]
fn test_pub049_a2_e01() {
    test_pub("Pub049", "Inuyasha", "01");
}
#[test]
fn test_pub049_a2_e02() {
    test_pub("Pub049", "Inuyasha", "02");
}
#[test]
fn test_pub049_a2_e03() {
    test_pub("Pub049", "Inuyasha", "03");
}
#[test]
fn test_pub049_a2_e04() {
    test_pub("Pub049", "Inuyasha", "04");
}
#[test]
fn test_pub049_a2_e05() {
    test_pub("Pub049", "Inuyasha", "05");
}
#[test]
fn test_pub049_a2_e06() {
    test_pub("Pub049", "Inuyasha", "06");
}
#[test]
fn test_pub049_a2_e07() {
    test_pub("Pub049", "Inuyasha", "07");
}
#[test]
fn test_pub049_a2_e08() {
    test_pub("Pub049", "Inuyasha", "08");
}
#[test]
fn test_pub049_a2_e09() {
    test_pub("Pub049", "Inuyasha", "09");
}
#[test]
fn test_pub049_a2_e10() {
    test_pub("Pub049", "Inuyasha", "10");
}
#[test]
fn test_pub049_a2_e11() {
    test_pub("Pub049", "Inuyasha", "11");
}
#[test]
fn test_pub049_a2_e12() {
    test_pub("Pub049", "Inuyasha", "12");
}
#[test]
fn test_pub049_a3_e01() {
    test_pub("Pub049", "Yu Yu Hakusho", "01");
}
#[test]
fn test_pub049_a3_e02() {
    test_pub("Pub049", "Yu Yu Hakusho", "02");
}
#[test]
fn test_pub049_a3_e03() {
    test_pub("Pub049", "Yu Yu Hakusho", "03");
}
#[test]
fn test_pub049_a3_e04() {
    test_pub("Pub049", "Yu Yu Hakusho", "04");
}
#[test]
fn test_pub049_a3_e05() {
    test_pub("Pub049", "Yu Yu Hakusho", "05");
}
#[test]
fn test_pub049_a3_e06() {
    test_pub("Pub049", "Yu Yu Hakusho", "06");
}
#[test]
fn test_pub049_a3_e07() {
    test_pub("Pub049", "Yu Yu Hakusho", "07");
}
#[test]
fn test_pub049_a3_e08() {
    test_pub("Pub049", "Yu Yu Hakusho", "08");
}
#[test]
fn test_pub049_a3_e09() {
    test_pub("Pub049", "Yu Yu Hakusho", "09");
}
#[test]
fn test_pub049_a3_e10() {
    test_pub("Pub049", "Yu Yu Hakusho", "10");
}
#[test]
fn test_pub049_a3_e11() {
    test_pub("Pub049", "Yu Yu Hakusho", "11");
}
#[test]
fn test_pub049_a3_e12() {
    test_pub("Pub049", "Yu Yu Hakusho", "12");
}

// Pub050
#[test]
fn test_pub050_a1_e01() {
    test_pub("Pub050", "Death Note", "01");
}
#[test]
fn test_pub050_a1_e02() {
    test_pub("Pub050", "Death Note", "02");
}
#[test]
fn test_pub050_a1_e03() {
    test_pub("Pub050", "Death Note", "03");
}
#[test]
fn test_pub050_a1_e04() {
    test_pub("Pub050", "Death Note", "04");
}
#[test]
fn test_pub050_a1_e05() {
    test_pub("Pub050", "Death Note", "05");
}
#[test]
fn test_pub050_a1_e06() {
    test_pub("Pub050", "Death Note", "06");
}
#[test]
fn test_pub050_a1_e07() {
    test_pub("Pub050", "Death Note", "07");
}
#[test]
fn test_pub050_a1_e08() {
    test_pub("Pub050", "Death Note", "08");
}
#[test]
fn test_pub050_a1_e09() {
    test_pub("Pub050", "Death Note", "09");
}
#[test]
fn test_pub050_a1_e10() {
    test_pub("Pub050", "Death Note", "10");
}
#[test]
fn test_pub050_a1_e11() {
    test_pub("Pub050", "Death Note", "11");
}
#[test]
fn test_pub050_a1_e12() {
    test_pub("Pub050", "Death Note", "12");
}
#[test]
fn test_pub050_a2_e01() {
    test_pub("Pub050", "Fullmetal Alchemist", "01");
}
#[test]
fn test_pub050_a2_e02() {
    test_pub("Pub050", "Fullmetal Alchemist", "02");
}
#[test]
fn test_pub050_a2_e03() {
    test_pub("Pub050", "Fullmetal Alchemist", "03");
}
#[test]
fn test_pub050_a2_e04() {
    test_pub("Pub050", "Fullmetal Alchemist", "04");
}
#[test]
fn test_pub050_a2_e05() {
    test_pub("Pub050", "Fullmetal Alchemist", "05");
}
#[test]
fn test_pub050_a2_e06() {
    test_pub("Pub050", "Fullmetal Alchemist", "06");
}
#[test]
fn test_pub050_a2_e07() {
    test_pub("Pub050", "Fullmetal Alchemist", "07");
}
#[test]
fn test_pub050_a2_e08() {
    test_pub("Pub050", "Fullmetal Alchemist", "08");
}
#[test]
fn test_pub050_a2_e09() {
    test_pub("Pub050", "Fullmetal Alchemist", "09");
}
#[test]
fn test_pub050_a2_e10() {
    test_pub("Pub050", "Fullmetal Alchemist", "10");
}
#[test]
fn test_pub050_a2_e11() {
    test_pub("Pub050", "Fullmetal Alchemist", "11");
}
#[test]
fn test_pub050_a2_e12() {
    test_pub("Pub050", "Fullmetal Alchemist", "12");
}
#[test]
fn test_pub050_a3_e01() {
    test_pub("Pub050", "Code Geass", "01");
}
#[test]
fn test_pub050_a3_e02() {
    test_pub("Pub050", "Code Geass", "02");
}
#[test]
fn test_pub050_a3_e03() {
    test_pub("Pub050", "Code Geass", "03");
}
#[test]
fn test_pub050_a3_e04() {
    test_pub("Pub050", "Code Geass", "04");
}
#[test]
fn test_pub050_a3_e05() {
    test_pub("Pub050", "Code Geass", "05");
}
#[test]
fn test_pub050_a3_e06() {
    test_pub("Pub050", "Code Geass", "06");
}
#[test]
fn test_pub050_a3_e07() {
    test_pub("Pub050", "Code Geass", "07");
}
#[test]
fn test_pub050_a3_e08() {
    test_pub("Pub050", "Code Geass", "08");
}
#[test]
fn test_pub050_a3_e09() {
    test_pub("Pub050", "Code Geass", "09");
}
#[test]
fn test_pub050_a3_e10() {
    test_pub("Pub050", "Code Geass", "10");
}
#[test]
fn test_pub050_a3_e11() {
    test_pub("Pub050", "Code Geass", "11");
}
#[test]
fn test_pub050_a3_e12() {
    test_pub("Pub050", "Code Geass", "12");
}
