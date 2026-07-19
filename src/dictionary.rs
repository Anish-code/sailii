use std::collections::HashSet;
use std::sync::OnceLock;

pub struct WordList {
    pub by_length: Vec<Vec<String>>,
    pub by_length_freq: Vec<Vec<String>>,
    pub set: HashSet<String>,
}

const EN_LETTER_FREQ: [f64; 26] = [
    0.08167, 0.01492, 0.02782, 0.04253, 0.12702, 0.02228, 0.02015,
    0.06094, 0.06966, 0.00153, 0.00772, 0.04025, 0.02406, 0.06749,
    0.07507, 0.01929, 0.00095, 0.05987, 0.06327, 0.09056, 0.02758,
    0.00978, 0.02360, 0.00150, 0.01974, 0.00074,
];

fn word_freq_score(word: &str) -> f64 {
    word.bytes().map(|b| {
        if b >= b'a' && b <= b'z' {
            EN_LETTER_FREQ[(b - b'a') as usize]
        } else {
            0.0
        }
    }).sum()
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

    let mut by_length_freq: Vec<Vec<String>> = by_length.iter().map(|list| {
        let mut sorted = list.clone();
        sorted.sort_by(|a, b| word_freq_score(b).partial_cmp(&word_freq_score(a)).unwrap_or(std::cmp::Ordering::Equal));
        sorted
    }).collect();
    for list in &mut by_length_freq {
        list.dedup();
    }

    WordList { by_length, by_length_freq, set }
}

pub fn wordlist() -> &'static WordList {
    static WL: OnceLock<WordList> = OnceLock::new();
    WL.get_or_init(load_csv)
}
