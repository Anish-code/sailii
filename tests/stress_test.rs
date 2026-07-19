use std::time::Instant;
use sailii::decoders::{
    vigenere_decoder::VigenereDecoder, xor_decoder::XorDecoder,
    aes256_decoder::Aes256Decoder, substitution_decoder::SubstitutionDecoder,
    caesar_decoder::CaesarDecoder, rot13_decoder::Rot13Decoder,
    Decoder, Crack, CrackResult,
};
use sailii::checkers::{CheckerTypes, Checker, english::EnglishChecker};
use sailii::config::{Config, set_global_config};

fn vigenere_encrypt(plain: &str, key: &str) -> String {
    let key = key.to_uppercase();
    let key_bytes: Vec<u8> = key.bytes().collect();
    let mut key_idx = 0;
    plain.chars()
        .map(|c| {
            if c.is_ascii_alphabetic() {
                let shift = key_bytes[key_idx % key_bytes.len()] - b'A';
                key_idx += 1;
                if c.is_ascii_uppercase() {
                    (((c as u8 - b'A') + shift) % 26 + b'A') as char
                } else {
                    (((c as u8 - b'a') + shift) % 26 + b'a') as char
                }
            } else {
                c
            }
        })
        .collect()
}

fn xor_encrypt(plain: &str, key: &[u8]) -> Vec<u8> {
    plain.bytes().zip(key.iter().cycle()).map(|(p, &k)| p ^ k).collect()
}

fn random_key(rng: &mut u64, len: usize) -> String {
    let mut key = String::with_capacity(len);
    for _ in 0..len {
        *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        key.push((b'A' + (*rng % 26) as u8) as char);
    }
    key
}

fn random_bytes(rng: &mut u64, len: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(len);
    for _ in 0..len {
        *rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        bytes.push((*rng & 0xFF) as u8);
    }
    bytes
}

fn run_crack<D: Crack>(decoder: &D, text: &str, checker: &CheckerTypes, timeout_secs: u64) -> CrackResult {
    set_global_config(Config { timeout_secs, verbose: false, max_depth: 20, ..Default::default() });
    decoder.crack(text, checker)
}

fn main() {
    let checker = CheckerTypes::English(Checker::<EnglishChecker>::new());
    let plaintext = "my name is anish and there is a bomb that will blast today";
    let plaintext_short = "hello world this is a secret message";
    let plaintext_aes = "my allrgic reaction kills me and more words here for length";

    let mut rng: u64 = 0x123456789ABCDEF;
    let timeout = 60u64;

    println!("=== STRESS TEST: Random Keys 20-30 chars ===\n");

    // === VIGENERE TESTS ===
    println!("--- Vigenere Tests ---");
    let vigenere = Decoder::<VigenereDecoder>::new();
    let mut vigenere_success = 0u32;
    let vigenere_trials = 10;

    for trial in 0..vigenere_trials {
        let klen = 20 + (trial % 11);
        let key = random_key(&mut rng, klen);
        let ciphertext = vigenere_encrypt(plaintext, &key);
        let start = Instant::now();
        let result = run_crack(&vigenere, &ciphertext, &checker, timeout);
        let elapsed = start.elapsed_secs_f64();
        if result.success {
            let decoded = result.unencrypted_text.as_ref().and_then(|v| v.first()).cloned().unwrap_or_default();
            let match_pct = if decoded.eq_ignore_ascii_case(plaintext) { 100 } else {
                let matches = decoded.chars().zip(plaintext.chars()).filter(|(a, b)| a == b).count();
                matches * 100 / plaintext.len().max(1)
            };
            println!("  Vigenere klen={} key={}: SUCCESS ({:.1}s, match={}%)", klen, key, elapsed, match_pct);
            vigenere_success += 1;
        } else {
            println!("  Vigenere klen={} key={}: FAIL ({:.1}s)", klen, key, elapsed);
        }
    }

    // === XOR TESTS ===
    println!("\n--- XOR Tests ---");
    let xor = Decoder::<XorDecoder>::new();
    let mut xor_success = 0u32;
    let xor_trials = 10;

    for trial in 0..xor_trials {
        let klen = 20 + (trial % 11);
        let key = random_bytes(&mut rng, klen);
        let encrypted = xor_encrypt(plaintext_short, &key);
        let ciphertext = hex::encode(&encrypted);
        let start = Instant::now();
        let result = run_crack(&xor, &ciphertext, &checker, timeout);
        let elapsed = start.elapsed_secs_f64();
        if result.success {
            let decoded = result.unencrypted_text.as_ref().and_then(|v| v.first()).cloned().unwrap_or_default();
            let matches = decoded.chars().zip(plaintext_short.chars()).filter(|(a, b)| a == b).count();
            let match_pct = matches * 100 / plaintext_short.len().max(1);
            println!("  XOR klen={}: SUCCESS ({:.1}s, match={}%)", klen, elapsed, match_pct);
            xor_success += 1;
        } else {
            println!("  XOR klen={}: FAIL ({:.1}s)", klen, elapsed);
        }
    }

    // === AES TESTS ===
    println!("\n--- AES Dictionary Tests ---");
    let aes = Decoder::<Aes256Decoder>::new();
    let mut aes_success = 0u32;

    // Test with a word from the dictionary as passphrase
    let dict = sailii::dictionary::wordlist();
    for &klen in &[20, 24, 30] {
        // Pick a random word from dictionary of appropriate length
        let dict_words: Vec<&String> = dict.by_length.get(klen).map(|v| v.iter().collect()).unwrap_or_default();
        if dict_words.is_empty() { continue; }
        let idx = (rng as usize) % dict_words.len();
        let key = dict_words[idx];
        let ciphertext = aes_encrypt_test(plaintext_aes, key);

        let start = Instant::now();
        let result = run_crack(&aes, &ciphertext, &checker, timeout);
        let elapsed = start.elapsed_secs_f64();
        let success = result.success;
        if success {
            println!("  AES klen={} key={}: SUCCESS ({:.1}s)", klen, key, elapsed);
            aes_success += 1;
        } else {
            println!("  AES klen={} key={}: FAIL ({:.1}s)", klen, key, elapsed);
        }
    }

    // === SUBSTITUTION TESTS ===
    println!("\n--- Substitution Tests ---");
    let substitution = Decoder::<SubstitutionDecoder>::new();
    let sub_plain = "the quick brown fox jumps over the lazy dog and the cat sat on the mat";
    let sub_key_str = "QWERTYUIOPASDFGHJKLZXCVBNM";
    let sub_cipher: String = sub_plain.chars().map(|c| {
        if c.is_ascii_alphabetic() {
            let idx = if c.is_ascii_uppercase() { (c as u8 - b'A') as usize } else { (c as u8 - b'a') as usize };
            let sub_char = sub_key_str.as_bytes()[idx] as char;
            if c.is_ascii_lowercase() { sub_char.to_ascii_lowercase() } else { sub_char }
        } else { c }
    }).collect();
    let start = Instant::now();
    let result = run_crack(&substitution, &sub_cipher, &checker, timeout);
    let elapsed = start.elapsed_secs_f64();
    if result.success {
        let decoded = result.unencrypted_text.as_ref().and_then(|v| v.first()).cloned().unwrap_or_default();
        println!("  Substitution: SUCCESS ({:.1}s)", elapsed);
        println!("    Decoded: {}", &decoded[..decoded.len().min(60)]);
    } else {
        println!("  Substitution: FAIL ({:.1}s)", elapsed);
    }

    // Summary
    println!("\n=== SUMMARY ===");
    println!("Vigenere (20-30 char keys): {}/{} success", vigenere_success, vigenere_trials);
    println!("XOR (20-30 byte keys): {}/{} success", xor_success, xor_trials);
    println!("AES (20-30 char passphrases): {}/3 success", aes_success);
    println!("Substitution: {}", if substitution.decoder.crack(&sub_cipher, &checker).success { "SUCCESS" } else { "FAIL" });
}

// Minimal AES encryption for testing
fn aes_encrypt_test(plaintext: &str, passphrase: &str) -> String {
    use aes::cipher::{KeyInit, BlockEncrypt, BlockSizeUser};
    use sha2::{Sha256, Digest};

    // Derive 32-byte key via SHA-256
    let key = Sha256::digest(passphrase.as_bytes());
    let key_arr = key.into();

    // Pad plaintext to block boundary
    let block_size = 16;
    let pad_len = block_size - (plaintext.len() % block_size);
    let mut padded = plaintext.as_bytes().to_vec();
    for _ in 0..pad_len {
        padded.push(pad_len as u8);
    }

    // Simple ECB mode (no IV for testing)
    let cipher = aes::Aes256Enc::new(&key_arr);
    let mut result = Vec::with_capacity(padded.len());
    for chunk in padded.chunks(block_size) {
        let mut block = *<aes::Aes256Enc as BlockSizeUser>::BlockSize::from_slice(chunk);
        cipher.encrypt_block(&mut block);
        result.extend_from_slice(&block);
    }

    base64_encode(&result)
}

fn base64_encode(bytes: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { out.push(CHARS[((triple >> 6) & 0x3F) as usize] as char); } else { out.push('='); }
        if chunk.len() > 2 { out.push(CHARS[(triple & 0x3F) as usize] as char); } else { out.push('='); }
    }
    out
}
