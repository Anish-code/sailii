use crate::decoders::{Crack, CrackResult, Decoder, check_string_success};
use crate::checkers::CheckerTypes;
use std::marker::PhantomData;

pub struct VigenereDecoder;

const ENGLISH_FREQ: [f64; 26] = [
    0.08167, 0.01492, 0.02782, 0.04253, 0.12702, 0.02228, 0.02015,
    0.06094, 0.06966, 0.00153, 0.00772, 0.04025, 0.02406, 0.06749,
    0.07507, 0.01929, 0.00095, 0.05987, 0.06327, 0.09056, 0.02758,
    0.00978, 0.02360, 0.00150, 0.01974, 0.00074,
];

fn vigenere_decode(text: &str, key: &str) -> String {
    let key = key.to_uppercase();
    let key_bytes: Vec<u8> = key.bytes().collect();
    if key_bytes.is_empty() {
        return text.to_string();
    }
    let mut key_idx = 0;
    text.chars()
        .map(|c| {
            if c.is_ascii_uppercase() {
                let shift = key_bytes[key_idx % key_bytes.len()] - b'A';
                key_idx += 1;
                let decoded = ((c as u8 - b'A' + 26 - shift) % 26) + b'A';
                decoded as char
            } else if c.is_ascii_lowercase() {
                let shift = key_bytes[key_idx % key_bytes.len()] - b'A';
                key_idx += 1;
                let decoded = ((c as u8 - b'a' + 26 - shift) % 26) + b'a';
                decoded as char
            } else {
                c
            }
        })
        .collect()
}

fn filter_alpha(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

fn index_of_coincidence(text: &str) -> f64 {
    let len = text.len();
    if len < 2 {
        return 0.0;
    }
    let mut counts = [0usize; 26];
    for b in text.bytes() {
        if b >= b'A' && b <= b'Z' {
            counts[(b - b'A') as usize] += 1;
        }
    }
    let total = len as f64;
    let sum: f64 = counts.iter().map(|&c| c as f64 * (c as f64 - 1.0)).sum();
    sum / (total * (total - 1.0))
}

fn estimate_key_lengths(text: &str) -> Vec<usize> {
    let clean = filter_alpha(text);
    let max_key_len = (clean.len() / 3).min(20).max(2);
    let mut scores: Vec<(usize, f64)> = (2..=max_key_len)
        .map(|k| {
            let mut ic_sum = 0.0;
            let mut count = 0;
            for offset in 0..k {
                let col: String = clean.chars().skip(offset).step_by(k).collect();
                if col.len() >= 2 {
                    ic_sum += index_of_coincidence(&col);
                    count += 1;
                }
            }
            let avg_ic = if count > 0 { ic_sum / count as f64 } else { 0.0 };
            (k, (avg_ic - 0.065).abs())
        })
        .collect();
    scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    scores.into_iter().take(5).map(|(k, _)| k).collect()
}

fn chi_squared(observed: &[usize; 26], total: f64) -> f64 {
    let mut chi = 0.0;
    for i in 0..26 {
        let expected = total * ENGLISH_FREQ[i];
        if expected > 0.0 {
            let diff = observed[i] as f64 - expected;
            chi += diff * diff / expected;
        }
    }
    chi
}

fn solve_key_char(text: &str, key_len: usize, offset: usize) -> char {
    let col: String = text
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .skip(offset)
        .step_by(key_len)
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if col.is_empty() {
        return 'A';
    }
    let total = col.len() as f64;
    let mut best_shift = 0u8;
    let mut best_chi = f64::MAX;
    for shift in 0..26 {
        let mut counts = [0usize; 26];
        for b in col.bytes() {
            let dec = ((b - b'A' + 26 - shift) % 26) as usize;
            counts[dec] += 1;
        }
        let chi = chi_squared(&counts, total);
        if chi < best_chi {
            best_chi = chi;
            best_shift = shift;
        }
    }
    (b'A' + best_shift) as char
}

fn solve_key(text: &str, key_len: usize) -> String {
    (0..key_len)
        .map(|offset| solve_key_char(text, key_len, offset))
        .collect()
}

impl Crack for Decoder<VigenereDecoder> {
    fn crack(&self, text: &str, checker: &CheckerTypes) -> CrackResult {
        let mut result = CrackResult::new(self.get_name(), self.get_description(), self.get_link());
        result.encrypted_text = text.to_string();

        let clean_len = filter_alpha(text).len();
        if clean_len < 4 {
            return result;
        }

        let has_lower = text.chars().any(|c| c.is_ascii_alphabetic());
        if !has_lower {
            return result;
        }

        let key_lengths = estimate_key_lengths(text);
        let mut tried = Vec::new();

        for &kl in &key_lengths {
            let key = solve_key(text, kl);
            if tried.contains(&key) {
                continue;
            }
            tried.push(key.clone());
            let decoded = vigenere_decode(text, &key);
            if check_string_success(&decoded, text) {
                let check_result = checker.check_text(&decoded);
                if check_result.is_identified {
                    result.success = true;
                    result.unencrypted_text = Some(vec![decoded]);
                    result.key = Some(key);
                    result.checker_name = check_result.checker_name;
                    return result;
                }
            }
        }

        let fallback_keys = [
            "KEY", "SECRET", "CIPHER", "CODE", "PASSWORD", "LEMON",
            "CLOCK", "ROYAL", "QUEEN", "ALPHA",
        ];
        for key in &fallback_keys {
            if tried.contains(&key.to_string()) {
                continue;
            }
            let decoded = vigenere_decode(text, key);
            if check_string_success(&decoded, text) {
                let check_result = checker.check_text(&decoded);
                if check_result.is_identified {
                    result.success = true;
                    result.unencrypted_text = Some(vec![decoded]);
                    result.key = Some(key.to_string());
                    result.checker_name = check_result.checker_name;
                    return result;
                }
            }
        }

        for shift in 1..26 {
            let key = String::from_utf8(vec![b'A' + shift]).unwrap();
            if tried.contains(&key) {
                continue;
            }
            let decoded = vigenere_decode(text, &key);
            if check_string_success(&decoded, text) {
                let check_result = checker.check_text(&decoded);
                if check_result.is_identified {
                    result.success = true;
                    result.unencrypted_text = Some(vec![decoded]);
                    result.key = Some(key);
                    result.checker_name = check_result.checker_name;
                    return result;
                }
            }
        }

        result
    }

    fn get_name(&self) -> &'static str {
        "Vigenere"
    }
    fn get_popularity(&self) -> f32 {
        0.7
    }
    fn get_tags(&self) -> &'static [&'static str] {
        &["vigenere", "decoder", "cipher", "classical"]
    }
    fn get_description(&self) -> &'static str {
        "Vigenere cipher using polyalphabetic substitution with a keyword."
    }
    fn get_link(&self) -> &'static str {
        "https://en.wikipedia.org/wiki/Vigen%C3%A8re_cipher"
    }
}

impl Decoder<VigenereDecoder> {
    pub fn new() -> Self {
        Decoder {
            name: "Vigenere",
            description: "Vigenere cipher using polyalphabetic substitution with a keyword.",
            link: "https://en.wikipedia.org/wiki/Vigen%C3%A8re_cipher",
            tags: vec!["vigenere", "decoder", "cipher", "classical"],
            popularity: 0.7,
            phantom: PhantomData,
        }
    }
}
