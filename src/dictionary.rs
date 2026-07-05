use std::collections::HashSet;
use std::sync::OnceLock;

pub struct WordList {
    pub by_length: Vec<Vec<String>>,
    pub set: HashSet<String>,
}

fn load_csv() -> WordList {
    let content = include_str!("../words/words.csv");
    let mut by_length: Vec<Vec<String>> = (0..=20).map(|_| Vec::new()).collect();
    let mut set = HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim().trim_matches('"');
        if trimmed.len() < 2 { continue; }
        if !trimmed.chars().all(|c| c.is_ascii_alphabetic()) { continue; }
        let lower = trimmed.to_lowercase();
        if lower.len() <= 20 {
            by_length[lower.len()].push(lower.clone());
        }
        set.insert(lower);
    }

    for list in &mut by_length {
        list.sort();
        list.dedup();
    }

    WordList { by_length, set }
}

pub fn wordlist() -> &'static WordList {
    static WL: OnceLock<WordList> = OnceLock::new();
    WL.get_or_init(load_csv)
}
