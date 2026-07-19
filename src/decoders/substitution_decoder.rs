use crate::decoders::{Crack, CrackResult, Decoder};
use crate::checkers::CheckerTypes;
use std::marker::PhantomData;

pub struct SubstitutionDecoder;

const EN_FREQ_SORTED: [u8; 26] = [
    4, 19, 0, 14, 8, 13, 18, 3, 11, 1, 20, 5, 6,
    7, 15, 2, 16, 17, 12, 9, 10, 21, 22, 23, 24, 25,
];

fn letter_frequencies(text: &str) -> [usize; 26] {
    let mut counts = [0usize; 26];
    for b in text.bytes() {
        if b >= b'A' && b <= b'Z' {
            counts[(b - b'A') as usize] += 1;
        } else if b >= b'a' && b <= b'z' {
            counts[(b - b'a') as usize] += 1;
        }
    }
    counts
}

fn build_initial_key() -> [u8; 26] {
    let mut key = [0u8; 26];
    for i in 0..26 {
        key[i] = b'A' + i as u8;
    }
    key
}

fn apply_key(text: &str, key: &[u8; 26]) -> String {
    text.chars().map(|c| {
        if c >= 'A' && c <= 'Z' {
            key[(c as u8 - b'A') as usize] as char
        } else if c >= 'a' && c <= 'z' {
            (key[(c as u8 - b'a') as usize] - b'A' + b'a') as char
        } else {
            c
        }
    }).collect()
}

const EN_QUAD: [(&str, f64); 30] = [
    ("THAT", 0.0025), ("THER", 0.0018), ("WITH", 0.0017), ("TION", 0.0016),
    ("HERE", 0.0014), ("IGHT", 0.0013), ("HAVE", 0.0013), ("THIS", 0.0012),
    ("THEC", 0.0012), ("OFTH", 0.0012), ("ANDT", 0.0011), ("FROM", 0.0011),
    ("MENT", 0.0011), ("THEI", 0.0010), ("THER", 0.0010), ("OFTE", 0.0009),
    ("ATIO", 0.0009), ("ALLE", 0.0009), ("WHIC", 0.0009), ("TION", 0.0009),
    ("WICH", 0.0009), ("IGHT", 0.0009), ("ETHE", 0.0008), ("FORT", 0.0008),
    ("THES", 0.0008), ("TEDT", 0.0008), ("TING", 0.0008), ("WERE", 0.0008),
    ("SHOU", 0.0007), ("DTHA", 0.0007),
];

const EN_TRIGRAMS: [(&str, f64); 30] = [
    ("THE", 0.0356), ("AND", 0.0159), ("ING", 0.0115), ("HER", 0.0082),
    ("THA", 0.0075), ("NTH", 0.0063), ("INT", 0.0058), ("ETH", 0.0056),
    ("FOR", 0.0055), ("DTH", 0.0054), ("HIS", 0.0052), ("TER", 0.0049),
    ("WAS", 0.0048), ("ITH", 0.0047), ("ENT", 0.0046), ("ION", 0.0045),
    ("TIO", 0.0044), ("ERS", 0.0043), ("ATI", 0.0042), ("HAT", 0.0041),
    ("ALL", 0.0040), ("SHE", 0.0039), ("HEC", 0.0038), ("OTH", 0.0037),
    ("VER", 0.0036), ("HIN", 0.0035), ("ARE", 0.0034), ("STH", 0.0033),
    ("TTH", 0.0032), ("YOU", 0.0031),
];

fn quadgram_score(text: &str) -> f64 {
    let upper: Vec<u8> = text.bytes().filter(|&b| b.is_ascii_alphabetic()).map(|b| b.to_ascii_uppercase()).collect();
    if upper.len() < 4 { return f64::MAX; }
    let mut score = 0.0;
    for w in upper.windows(4) {
        let quad = std::str::from_utf8(w).unwrap_or("");
        let mut found = false;
        for &(pat, prob) in EN_QUAD.iter() {
            if quad == pat {
                score -= prob;
                found = true;
                break;
            }
        }
        if !found { score += 0.001; }
    }
    for w in upper.windows(3) {
        let tri = std::str::from_utf8(w).unwrap_or("");
        let mut found = false;
        for &(pat, prob) in EN_TRIGRAMS.iter() {
            if tri == pat {
                score -= prob;
                found = true;
                break;
            }
        }
        if !found { score += 0.0001; }
    }
    score
}

fn hill_climb_substitution(text: &str, key: &mut [u8; 26], deadline: std::time::Instant) {
    let mut best_score = quadgram_score(&apply_key(text, key));
    let mut improved = true;
    while improved {
        if std::time::Instant::now() >= deadline { break; }
        improved = false;
        for i in 0..26 {
            if std::time::Instant::now() >= deadline { break; }
            for j in (i + 1)..26 {
                if std::time::Instant::now() >= deadline { break; }
                key.swap(i, j);
                let score = quadgram_score(&apply_key(text, key));
                if score < best_score {
                    best_score = score;
                    improved = true;
                } else {
                    key.swap(i, j);
                }
            }
        }
    }
}

fn find_dict_words(text: &str) -> usize {
    let dict = crate::dictionary::wordlist();
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut count = 0;
    for w in &words {
        let clean: String = w.chars().filter(|c| c.is_ascii_alphabetic()).map(|c| c.to_ascii_lowercase()).collect();
        if clean.len() >= 2 && dict.set.contains(&clean) {
            count += 1;
        }
    }
    if words.is_empty() { 0 } else { count * 100 / words.len() }
}

fn generate_substitution_keys(text: &str) -> Vec<[u8; 26]> {
    let freqs = letter_frequencies(text);
    let mut freq_pairs: Vec<(usize, u8)> = freqs.iter().enumerate().map(|(i, &c)| (c, i as u8)).collect();
    freq_pairs.sort_by(|a, b| b.0.cmp(&a.0));

    let mut keys = Vec::with_capacity(4);

    let mut k1 = build_initial_key();
    for (i, &(_, cipher_letter)) in freq_pairs.iter().enumerate() {
        if i < 26 {
            let eng_idx = EN_FREQ_SORTED[i];
            k1[cipher_letter as usize] = b'A' + eng_idx;
        }
    }
    keys.push(k1);

    if freq_pairs.len() >= 2 {
        let mut k2 = k1;
        k2.swap(freq_pairs[0].1 as usize, freq_pairs[1].1 as usize);
        keys.push(k2);
    }

    for _ in 0..2 {
        let mut k3 = build_initial_key();
        let mut rng_state = 12345u64;
        for _ in 0..6 {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let a = (rng_state % 26) as usize;
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let b = (rng_state % 26) as usize;
            if a != b { k3.swap(a, b); }
        }
        keys.push(k3);
    }

    keys
}

impl Crack for Decoder<SubstitutionDecoder> {
    fn crack(&self, text: &str, checker: &CheckerTypes) -> CrackResult {
        let mut result = CrackResult::new(self.get_name(), self.get_description(), self.get_link());
        result.encrypted_text = text.to_string();

        let alpha_count = text.chars().filter(|c| c.is_ascii_alphabetic()).count();
        if alpha_count < 8 {
            return result;
        }

        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(5000);
        let mut best_decoded = String::new();
        let mut best_pct = 0usize;

        let initial_keys = generate_substitution_keys(text);
        for mut key in initial_keys {
            if std::time::Instant::now() >= deadline { break; }
            hill_climb_substitution(text, &mut key, deadline);
            let decoded = apply_key(text, &key);
            let cr = checker.check_text(&decoded);
            if cr.is_identified {
                let pct = find_dict_words(&decoded);
                if pct > best_pct {
                    best_pct = pct;
                    best_decoded = decoded;
                }
                if pct >= 60 {
                    result.success = true;
                    result.unencrypted_text = Some(vec![best_decoded]);
                    let key_str: String = key.iter().map(|&k| k as char).collect();
                    result.key = Some(key_str);
                    result.checker_name = cr.checker_name;
                    return result;
                }
            }
        }

        if !best_decoded.is_empty() {
            result.success = true;
            result.unencrypted_text = Some(vec![best_decoded]);
            result.checker_name = "English".to_string();
        }

        result
    }

    fn get_name(&self) -> &'static str {
        "Substitution"
    }
    fn get_popularity(&self) -> f32 {
        0.7
    }
    fn get_tags(&self) -> &'static [&'static str] {
        &["substitution", "decoder", "cipher", "classical"]
    }
    fn get_description(&self) -> &'static str {
        "Monoalphabetic substitution cipher using frequency analysis and hill climbing."
    }
    fn get_link(&self) -> &'static str {
        "https://en.wikipedia.org/wiki/Substitution_cipher"
    }
}

impl Decoder<SubstitutionDecoder> {
    pub fn new() -> Self {
        Decoder {
            name: "Substitution",
            description: "Monoalphabetic substitution cipher using frequency analysis and hill climbing.",
            link: "https://en.wikipedia.org/wiki/Substitution_cipher",
            tags: vec!["substitution", "decoder", "cipher", "classical"],
            popularity: 0.7,
            phantom: PhantomData,
        }
    }
}
