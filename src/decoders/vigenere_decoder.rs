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

fn try_word_boundary_all(text: &str, checker: &CheckerTypes) -> Vec<String> {
    let total_deadline = std::time::Instant::now() + std::time::Duration::from_millis(5000);
    let cx_words: Vec<(usize, Vec<u8>)> = {
        let mut pos = 0usize;
        text.split_whitespace().filter_map(|w| {
            let letters: Vec<u8> = w.bytes().filter(|b| b.is_ascii_alphabetic()).map(|b| b.to_ascii_uppercase() - b'A').collect();
            if letters.is_empty() { return None; }
            if letters.len() < 2 { pos += 1; return None; }
            let start = pos;
            pos += letters.len();
            Some((start, letters))
        }).collect()
    };
    if cx_words.len() < 2 { eprintln!("[vigenere] wb: fewer than 2 multi-letter words"); return vec![]; }

    let total_chars: usize = cx_words.iter().map(|(_, w)| w.len()).sum();
    let max_klen = (total_chars / 2).min(20).max(2);
    let dict = crate::dictionary::wordlist();
    let mut results = Vec::new();
    let per_klen = std::time::Duration::from_millis(2500);

    for klen in 2..=max_klen {
        if std::time::Instant::now() >= total_deadline { eprintln!("[vigenere] wb: total deadline expired, breaking at klen={}", klen); break; }
        let klen_deadline = std::time::Instant::now() + per_klen;
        let mut key: Vec<Option<u8>> = vec![None; klen];
        if backtrack_fill(&cx_words, &dict.by_length, 0, &mut key, klen, klen_deadline) {
            if key.iter().all(|k| k.is_some()) {
                let key_str: String = key.iter().map(|k| (k.unwrap() + b'A') as char).collect();
                let decoded = vigenere_decode(text, &key_str);
                let cr = checker.check_text(&decoded);
                eprintln!("[vigenere] wb: klen={} key={} decoded={:?} identified={}", klen, key_str, decoded, cr.is_identified);
                if cr.is_identified {
                    results.push(key_str);
                }
            }
        }
    }
    eprintln!("[vigenere] wb: done, results={:?}", results);
    results
}

fn backtrack_fill(
    cx_words: &[(usize, Vec<u8>)],
    by_len: &[Vec<String>],
    idx: usize,
    key: &mut Vec<Option<u8>>,
    klen: usize,
    deadline: std::time::Instant,
) -> bool {
    if idx == cx_words.len() {
        let filled = key.iter().all(|k| k.is_some());
        if filled {
            let key_str: String = key.iter().map(|k| (k.unwrap() + b'A') as char).collect();
            eprintln!("[vigenere] backtrack SUCCESS idx={}/{} key={:?} final={}", idx, cx_words.len(), key, key_str);
        }
        return filled;
    }
    let (start, cx_letters) = &cx_words[idx];
    let wlen = cx_letters.len();
    if wlen > 20 || wlen >= by_len.len() { return false; }

    for pw in &by_len[wlen] {
        let pw_bytes = pw.as_bytes();
        let mut ok = true;
        let mut updates: Vec<(usize, u8)> = Vec::new();
        for i in 0..wlen {
            let kp = (start + i) % klen;
            let kv = (cx_letters[i] + 26 - (pw_bytes[i] - b'a')) % 26;
            if let Some(existing) = key[kp] {
                if existing != kv { ok = false; break; }
            } else {
                // Check if this key position already has a pending update
                if let Some(&(_, existing_kv)) = updates.iter().find(|&&(p, _)| p == kp) {
                    if existing_kv != kv { ok = false; break; }
                } else {
                    updates.push((kp, kv));
                }
            }
        }
        if !ok { continue; }
        for &(kp, kv) in &updates { key[kp] = Some(kv); }
        let prev_key_str: String = key.iter().map(|k| if let Some(v) = k { (v + b'A') as char } else { '?' }).collect();
        if std::time::Instant::now() >= deadline { return false; }
        if backtrack_fill(cx_words, by_len, idx + 1, key, klen, deadline) { 
            eprintln!("[vigenere] backtrack found at idx={} pw={:?} key after={}", idx, pw, prev_key_str);
            return true; 
        }
        for &(kp, _) in &updates { key[kp] = None; }
    }
    false
}

fn vigenere_score(text: &str, key: &str) -> f64 {
    let decoded = vigenere_decode(text, key);
    full_text_chi_squared(&decoded)
}

fn full_text_chi_squared(text: &str) -> f64 {
    let clean = filter_alpha(text);
    let total = clean.len() as f64;
    if total < 2.0 {
        return f64::MAX;
    }
    let mut counts = [0usize; 26];
    for b in clean.bytes() {
        if b >= b'A' && b <= b'Z' {
            counts[(b - b'A') as usize] += 1;
        }
    }
    let mut chi = 0.0;
    for i in 0..26 {
        let expected = total * ENGLISH_FREQ[i];
        if expected > 0.0 {
            let diff = counts[i] as f64 - expected;
            chi += diff * diff / expected;
        }
    }
    chi
}

fn hill_climb_one(text: &str, key_bytes: &mut Vec<u8>, deadline: std::time::Instant) -> f64 {
    let mut best_score = vigenere_score(text, &String::from_utf8(key_bytes.clone()).unwrap());
    for _ in 0..100 {
        if std::time::Instant::now() >= deadline {
            break;
        }
        let mut improved = false;
        for pos in 0..key_bytes.len() {
            if std::time::Instant::now() >= deadline {
                break;
            }
            let mut best_for_pos = key_bytes[pos];
            for letter in 0..26 {
                if std::time::Instant::now() >= deadline {
                    break;
                }
                let l = b'A' + letter;
                if l == best_for_pos {
                    continue;
                }
                key_bytes[pos] = l;
                let test_key = String::from_utf8(key_bytes.clone()).unwrap();
                let score = vigenere_score(text, &test_key);
                if score < best_score {
                    best_score = score;
                    best_for_pos = l;
                    improved = true;
                }
            }
            key_bytes[pos] = best_for_pos;
        }
        if !improved {
            break;
        }
    }
    best_score
}

fn hill_climb_key(text: &str, initial_key: &str, deadline: std::time::Instant, checker: &CheckerTypes) -> Option<String> {
    let klen = initial_key.len();
    let mut best_key = initial_key.to_uppercase().into_bytes();
    let mut best_score = hill_climb_one(text, &mut best_key, deadline);

    let check_key = |key: &[u8]| -> Option<String> {
        let k = String::from_utf8(key.to_vec()).unwrap();
        let decoded = vigenere_decode(text, &k);
        if check_string_success(&decoded, text) {
            let cr = checker.check_text(&decoded);
            if cr.is_identified {
                return Some(k);
            }
        }
        None
    };

    if let Some(r) = check_key(&best_key) {
        return Some(r);
    }

    let mut rng_state = deadline.elapsed().as_nanos() as u64 ^ 0xDEADBEEF;
    #[inline(always)]
    fn rng_next(state: &mut u64) -> u8 {
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*state % 26) as u8
    }

    let restarts_per = if klen >= 8 { 100 } else { 50 };
    for pos0 in 0..26 {
        if std::time::Instant::now() >= deadline { break; }
        for _ in 0..restarts_per {
            if std::time::Instant::now() >= deadline { break; }
            let mut trial = Vec::with_capacity(klen);
            trial.push(b'A' + pos0);
            for _ in 1..klen {
                trial.push(b'A' + rng_next(&mut rng_state));
            }
            let score = hill_climb_one(text, &mut trial, deadline);
            if score < best_score {
                best_score = score;
                best_key = trial;
                if let Some(r) = check_key(&best_key) {
                    return Some(r);
                }
            }
        }
    }

    check_key(&best_key)
}

fn estimate_key_lengths(text: &str) -> Vec<usize> {
    let clean = filter_alpha(text);
    let max_key_len = (clean.len() / 2).min(20).max(2);
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
    let ic_top: Vec<usize> = scores.iter().take(5).map(|(k, _)| *k).collect();
    let all: Vec<usize> = (2..=max_key_len).collect();
    let mut combined: Vec<usize> = ic_top.clone();
    for k in all {
        if !combined.contains(&k) {
            combined.push(k);
        }
    }
    combined
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
        eprintln!("[vigenere] estimated key lengths: {:?}", key_lengths);
        let mut tried = Vec::new();

        for &kl in &key_lengths {
            let key = solve_key(text, kl);
            eprintln!("[vigenere] kl={} key={}", kl, key);
            if tried.contains(&key) {
                continue;
            }
            tried.push(key.clone());
            let decoded = vigenere_decode(text, &key);
            eprintln!("[vigenere] decoded={:?} success={}", decoded, check_string_success(&decoded, text));
            if check_string_success(&decoded, text) {
                let check_result = checker.check_text(&decoded);
                eprintln!("[vigenere] check_result.is_identified={}", check_result.is_identified);
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
            "APPLE", "HOUSE", "PHONE", "WATER", "MONEY", "PAPER",
            "TABLE", "CHAIR", "DOOR", "WINDOW", "FLOOR", "WALL",
            "THERE", "WHICH", "THEIR", "WOULD", "ABOUT", "PEOPLE",
            "COULD", "FIRST", "WORLD", "STILL", "SHOULD", "NEED",
            "STATE", "NEVER", "START", "LIGHT", "SOUND", "WHITE",
            "BLACK", "GREEN", "GREAT", "SMALL", "UNDER", "LARGE",
            "AFTER", "RIGHT", "HOUSE", "PLACE", "POINT", "GROUP",
            "WOMAN", "CHILD", "HELLO", "WORLD", "ALICE", "BOB",
            "CHARLIE", "DELTA", "ECHO", "BRAVO", "ALPHA", "GAMMA",
            "BLUE", "PINK", "PURPLE", "GOLD", "SILVER", "METAL",
            "QUEEN", "KING", "JACK", "QUEEN", "ACE", "SPADE",
            "HEART", "DIAMOND", "CLUB", "RIVER", "BRIDGE", "FOREST",
            "MOUNT", "VALLEY", "OCEAN", "DESERT", "ISLAND", "STONE",
            "CLOUD", "STORM", "RAIN", "SNOW", "WIND", "FIRE", "EARTH",
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

        {
            let wb_keys = try_word_boundary_all(text, checker);
            if !wb_keys.is_empty() {
                let mut best = (String::new(), f64::MAX, String::new());
                for wb_key in &wb_keys {
                    let decoded = vigenere_decode(text, wb_key);
                    let chi = full_text_chi_squared(&decoded);
                    eprintln!("[vigenere] wb key={} decoded={:?} chi={:.1}", wb_key, decoded, chi);
                    if chi < best.1 {
                        best = (wb_key.clone(), chi, decoded);
                    }
                }
                let wb_key = best.0;
                let decoded = best.2;
                eprintln!("[vigenere] best wb key={} decoded={:?}", wb_key, decoded);
                if check_string_success(&decoded, text) {
                    let check_result = checker.check_text(&decoded);
                    if check_result.is_identified {
                        result.success = true;
                        result.unencrypted_text = Some(vec![decoded]);
                        result.key = Some(wb_key);
                        result.checker_name = check_result.checker_name;
                        return result;
                    }
                }
            } else {
                eprintln!("[vigenere] word_boundary did not find any key");
            }
        }

        let hc_budget = std::time::Duration::from_millis(15000);
        let hc_deadline = std::time::Instant::now() + hc_budget;
        let mut hc_klens: Vec<usize> = key_lengths.iter().filter(|&&kl| kl >= 4).copied().collect();
        hc_klens.sort();
        for &kl in &hc_klens {
            if std::time::Instant::now() >= hc_deadline {
                break;
            }
            let initial = solve_key(text, kl);
            if let Some(hill_key) = hill_climb_key(text, &initial, hc_deadline, checker) {
                eprintln!("[vigenere] hill climbing produced key: {}", hill_key);
                let decoded = vigenere_decode(text, &hill_key);
                if check_string_success(&decoded, text) {
                    let check_result = checker.check_text(&decoded);
                    if check_result.is_identified {
                        result.success = true;
                        result.unencrypted_text = Some(vec![decoded]);
                        result.key = Some(hill_key);
                        result.checker_name = check_result.checker_name;
                        return result;
                    }
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

        let max_brute_len = if clean_len < 12 { 4 } else if clean_len < 16 { 3 } else { 2 };
        let brute_deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
        'brute: for len in 2..=max_brute_len {
            let max_attempts = 26usize.pow(len as u32);
            for attempt in 0..max_attempts {
                if std::time::Instant::now() >= brute_deadline {
                    break 'brute;
                }
                let mut key = String::with_capacity(len);
                let mut n = attempt;
                for _ in 0..len {
                    key.push((b'A' + (n % 26) as u8) as char);
                    n /= 26;
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



// force2
