use rand::distr::{Distribution, Uniform};

pub const WORKER_NAMES: [&str; 129] = [
    "Paff", "Neko", "Robo", "Ivy", "Vanessa", "Ilka", "Xenon", "Conner", "Cherry", "Joe", "Sagar", "Rin", "Aroma", "Nora", // cytus ii
    "Alice", "Hans", "Celia", "Mirai", // deemo
    "Axel", "Judy", "Johnny", "Casey", "Pandora", "Izzy", "Lars", "Eddie", "Clive", "Midori", "Xavier", // guitar hero
    "Hikari", "Tairitsu", "Kou", "Kethe", "Stella", "Ilith", "Eto", "Luna", "Shirabe", "Ayu", "Seine", "Saya", "Kanae", "Sia", "Tenniel",
    "Mir", "Lagrange", /* "Nami", */ "Shirahime", "Vita", "Linka", "Amane", "Lethe", "Maya", "Insight", "Compassion", "Nonoka",
    "Yuno", "Helena", // arcaea
    "Sapphire", "Fisica", "Trin", "Yume", "Chuni", "Haruna", "Nono", "Regulus", "Mia", "Areus", "Seele", "Isabelle", "Lily", "Marija",
    "Saki", "Setsuna", "Shama", "Milk", "Shikoku", "Mika", "Mithra", "Toa", "Luin", "Ilot", "Hoppe", "Chinatsu", "Tsumugi", "Nai",
    "Selene", "Salt", "Acid", "Sui", // arcaea collab
    "Nia", "Mei", "Ayame", "Iris", "Story", // in falsus
    "Yoso", "Nami", "Moutlush", "Bayees", "Carmen", "Tonebell", // rizline
    "Beat", "Quaver", "Clef", "Treble", "Penny", "Poco", "Apoco", "Sforzando", // unbeatable
    "Ian", "Hugh", "Ada", "Logan", "Hailey", "Cole", "Nicole", "Richard", "Lucia", "Gabe", "Lucky", // rhythm doctor
    "Saturday", "Tsuki", "Dawn", "Allison", "Eri", "Kotomi", "Chiyo", // vivid/stasis
    "Kizuna", "Miku", "Teto", // various
];

pub fn random_name() -> &'static str {
    let a = Uniform::new(0, WORKER_NAMES.len()).expect("WORKER_NAMES.len() should always be greater than 0");
    let index = a.sample(&mut rand::rng());
    WORKER_NAMES.get(index).expect("index should always be in range")
}

#[test]
fn unique_names() {
    use regex::Regex;
    use std::collections::HashSet;
    use std::sync::LazyLock;

    pub static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9]+$").expect("could not compile regex"));

    let mut set = HashSet::new();
    let mut duplicates = Vec::new();
    let mut invalid = Vec::new();
    for name in WORKER_NAMES {
        if !set.insert(name.to_lowercase()) {
            duplicates.push(name);
        }
        if !REGEX.is_match(name) {
            invalid.push(name);
        }
    }
    if !duplicates.is_empty() {
        panic!("duplicate names found: {duplicates:?}")
    }
    if !invalid.is_empty() {
        panic!("invalid names: {invalid:?}")
    }
}
