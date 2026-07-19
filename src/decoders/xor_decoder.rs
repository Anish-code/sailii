use crate::decoders::{Crack, CrackResult, Decoder, check_string_success};
use crate::checkers::CheckerTypes;
use rayon::prelude::*;
use std::marker::PhantomData;

pub struct XorDecoder;

const EN_FREQ: [f64; 256] = {
    let mut f = [0.0f64; 256];
    let letters = b" etaoinsrhldcumfpgwybvkxjqzETAOINSRHLDCUMFPGWYBVKXJQZ";
    let weights = [
        0.12702, 0.09056, 0.08167, 0.07507, 0.06966, 0.06749, 0.06327, 0.06094,
        0.05987, 0.04253, 0.04025, 0.02782, 0.02758, 0.02406, 0.02360, 0.02228,
        0.02015, 0.01974, 0.01929, 0.01812, 0.01523, 0.00978, 0.00772, 0.00153,
        0.00150, 0.00095, 0.00074,
        0.12702, 0.09056, 0.08167, 0.07507, 0.06966, 0.06749, 0.06327, 0.06094,
        0.05987, 0.04253, 0.04025, 0.02782, 0.02758, 0.02406, 0.02360, 0.02228,
        0.02015, 0.01974, 0.01929, 0.01812, 0.01523, 0.00978, 0.00772, 0.00153,
        0.00150, 0.00095, 0.00074,
    ];
    let mut i = 0;
    while i < letters.len() && i < weights.len() {
        f[letters[i] as usize] = weights[i];
        i += 1;
    }
    f[b' ' as usize] = 0.15;
    f
};

fn score_text(bytes: &[u8]) -> f64 {
    if bytes.is_empty() {
        return f64::MAX;
    }
    let printable = bytes.iter().filter(|&&b| b.is_ascii_graphic() || b == b' ').count();
    if (printable as f64) / (bytes.len() as f64) < 0.90 {
        return f64::MAX;
    }
    let mut score = 0.0;
    for &b in bytes {
        score += EN_FREQ[b as usize];
    }
    -score
}

fn xor_single_byte(data: &[u8], key: u8) -> Vec<u8> {
    data.iter().map(|&b| b ^ key).collect()
}

fn xor_multi_byte(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter().enumerate().map(|(i, &b)| b ^ key[i % key.len()]).collect()
}

fn solve_multi_byte_xor(data: &[u8], key_len: usize) -> Vec<u8> {
    let mut key = Vec::with_capacity(key_len);
    for pos in 0..key_len {
        let col: Vec<u8> = data.iter().skip(pos).step_by(key_len).copied().collect();
        let mut best_key_byte = 0u8;
        let mut best_score = f64::MAX;
        for k in 0..=255 {
            let decrypted: Vec<u8> = col.iter().map(|&b| b ^ k).collect();
            let s = score_text(&decrypted);
            if s < best_score {
                best_score = s;
                best_key_byte = k;
            }
        }
        key.push(best_key_byte);
    }
    key
}

impl Crack for Decoder<XorDecoder> {
    fn crack(&self, text: &str, checker: &CheckerTypes) -> CrackResult {
        let mut result = CrackResult::new(self.get_name(), self.get_description(), self.get_link());
        result.encrypted_text = text.to_string();

        let trimmed = text.trim();
        let mut was_decoded = false;
        let bytes = if trimmed.len() > 2 && trimmed.as_bytes().iter().all(|b| b.is_ascii_hexdigit()) && trimmed.len() % 2 == 0 {
            was_decoded = true;
            if let Ok(b) = hex::decode(trimmed) { b }
            else { trimmed.as_bytes().to_vec() }
        } else if let Ok(b) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, trimmed) {
            was_decoded = true;
            b
        } else {
            trimmed.as_bytes().to_vec()
        };

        if bytes.len() < 2 { return result; }

        if !was_decoded { return result; }

        let single_byte_keys: Vec<u8> = (0..=255u8).collect();
        let single_results: Vec<(u8, Vec<u8>, f64)> = single_byte_keys.par_iter().map(|&k| {
            let dec = xor_single_byte(&bytes, k);
            let sc = score_text(&dec);
            (k, dec, sc)
        }).collect();

        let mut best_single: Option<(u8, Vec<u8>, f64)> = None;
        for (k, dec, sc) in &single_results {
            if *sc == f64::MAX { continue; }
            if best_single.as_ref().map_or(true, |b| sc < &b.2) {
                best_single = Some((*k, dec.clone(), *sc));
            }
        }

        if let Some((key_byte, decrypted, _)) = &best_single {
            if let Ok(s) = String::from_utf8(decrypted.clone()) {
                if check_string_success(&s, text) {
                    let cr = checker.check_text(&s);
                    if cr.is_identified {
                        result.success = true;
                        result.unencrypted_text = Some(vec![s]);
                        result.key = Some(format!("XOR-single:0x{:02x}", key_byte));
                        result.checker_name = cr.checker_name;
                        return result;
                    }
                }
            }
        }

        let max_klen = (bytes.len() / 2).min(20).max(2);
        let multi_key_lens: Vec<usize> = (2..=max_klen).collect();
        let multi_results: Vec<(Vec<u8>, Vec<u8>)> = multi_key_lens.par_iter().map(|&klen| {
            let key = solve_multi_byte_xor(&bytes, klen);
            let dec = xor_multi_byte(&bytes, &key);
            (key, dec)
        }).collect();

        for (key, decrypted) in &multi_results {
            if let Ok(s) = String::from_utf8(decrypted.clone()) {
                if check_string_success(&s, text) {
                    let cr = checker.check_text(&s);
                    if cr.is_identified {
                        let key_str: String = key.iter().map(|&b| format!("{:02x}", b)).collect();
                        result.success = true;
                        result.unencrypted_text = Some(vec![s]);
                        result.key = Some(format!("XOR-multi:{}", key_str));
                        result.checker_name = cr.checker_name;
                        return result;
                    }
                }
            }
        }

        if let Some((_, decrypted, _)) = &best_single {
            if let Ok(s) = String::from_utf8(decrypted.clone()) {
                if check_string_success(&s, text) && s.len() > 2 {
                    let cr = checker.check_text(&s);
                    if cr.match_ratio > 0.3 {
                        result.unencrypted_text = Some(vec![s]);
                    }
                }
            }
        }

        result
    }

    fn get_name(&self) -> &'static str { "XOR" }
    fn get_popularity(&self) -> f32 { 0.75 }
    fn get_tags(&self) -> &'static [&'static str] { &["xor", "decoder", "cipher", "classical"] }
    fn get_description(&self) -> &'static str { "XOR cipher brute-forces all possible single-byte keys and multi-byte keys using frequency analysis." }
    fn get_link(&self) -> &'static str { "https://en.wikipedia.org/wiki/XOR_cipher" }
}

impl Decoder<XorDecoder> {
    pub fn new() -> Self {
        Decoder {
            name: "XOR",
            description: "XOR cipher brute-forces all possible single-byte keys and multi-byte keys using frequency analysis.",
            link: "https://en.wikipedia.org/wiki/XOR_cipher",
            tags: vec!["xor", "decoder", "cipher", "classical"],
            popularity: 0.75,
            phantom: PhantomData,
        }
    }
}
