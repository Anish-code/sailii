use crate::decoders::{Crack, CrackResult, Decoder};
use crate::checkers::CheckerTypes;
use crate::dictionary;
use crate::config::get_config;
use aes::cipher::{BlockDecrypt, KeyInit, generic_array::GenericArray, generic_array::typenum::U16};
use aes::{Aes128Dec, Aes192Dec, Aes256Dec};
use rayon::prelude::*;
use sha2::{Sha256, Digest};
use std::marker::PhantomData;
use std::sync::Mutex;

pub struct Aes256Decoder;

/// Try all AES-256 modes (ECB, CBC with zero/extracted IV) using the given password,
/// testing both pad32 and sha32 key derivation. Returns the first successful plaintext.
pub fn decrypt_with_key(ciphertext_b64: &str, password: &str) -> Option<String> {
    let raw = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, ciphertext_b64.trim()).ok()?;
    if raw.is_empty() || raw.len() % 16 != 0 { return None; }

    let zero_iv = [0u8; 16];
    let k32_pad = pad_key::<32>(password);
    let k32_hash = hash_key::<32>(password);

    let (pre_iv, pre_ct) = if raw.len() > 16 {
        let mut iv = [0u8; 16];
        iv.copy_from_slice(&raw[..16]);
        (Some(iv), &raw[16..])
    } else {
        (None, &raw[..])
    };

    for (key, _tag) in [(&k32_pad, "pad32"), (&k32_hash, "sha32")] {
        if let Some(pt) = try_ecb_256(&raw, key) { return Some(pt); }
        if pre_iv.is_some() { if let Some(pt) = try_ecb_256(pre_ct, key) { return Some(pt); } }
        if let Some(pt) = try_cbc_256(&raw, key, &zero_iv) { return Some(pt); }
        if let Some(ref iv) = pre_iv { if let Some(pt) = try_cbc_256(pre_ct, key, iv) { return Some(pt); } }
    }
    None
}

fn unpad_pkcs7(plaintext: &mut Vec<u8>) -> Option<()> {
    let pad_len = *plaintext.last()? as usize;
    if pad_len == 0 || pad_len > 16 { return None; }
    let start = plaintext.len() - pad_len;
    if plaintext[start..].iter().any(|&b| b != pad_len as u8) { return None; }
    plaintext.truncate(start);
    Some(())
}

fn try_ecb_128(ciphertext: &[u8], key: &[u8; 16]) -> Option<String> {
    let cipher = Aes128Dec::new(GenericArray::from_slice(key));
    let mut plaintext = Vec::with_capacity(ciphertext.len());
    for block in ciphertext.chunks(16) {
        let mut arr = GenericArray::<u8, U16>::clone_from_slice(block);
        cipher.decrypt_block(&mut arr);
        plaintext.extend_from_slice(&arr);
    }
    unpad_pkcs7(&mut plaintext)?;
    String::from_utf8(plaintext).ok()
}

fn try_ecb_192(ciphertext: &[u8], key: &[u8; 24]) -> Option<String> {
    let cipher = Aes192Dec::new(GenericArray::from_slice(key));
    let mut plaintext = Vec::with_capacity(ciphertext.len());
    for block in ciphertext.chunks(16) {
        let mut arr = GenericArray::<u8, U16>::clone_from_slice(block);
        cipher.decrypt_block(&mut arr);
        plaintext.extend_from_slice(&arr);
    }
    unpad_pkcs7(&mut plaintext)?;
    String::from_utf8(plaintext).ok()
}

fn try_ecb_256(ciphertext: &[u8], key: &[u8; 32]) -> Option<String> {
    let cipher = Aes256Dec::new(GenericArray::from_slice(key));
    let mut plaintext = Vec::with_capacity(ciphertext.len());
    for block in ciphertext.chunks(16) {
        let mut arr = GenericArray::<u8, U16>::clone_from_slice(block);
        cipher.decrypt_block(&mut arr);
        plaintext.extend_from_slice(&arr);
    }
    unpad_pkcs7(&mut plaintext)?;
    String::from_utf8(plaintext).ok()
}

fn try_cbc_128(ciphertext: &[u8], key: &[u8; 16], iv: &[u8; 16]) -> Option<String> {
    let cipher = Aes128Dec::new(GenericArray::from_slice(key));
    let mut plaintext = Vec::with_capacity(ciphertext.len());
    let mut prev = *iv;
    for block in ciphertext.chunks(16) {
        let mut arr = GenericArray::<u8, U16>::clone_from_slice(block);
        cipher.decrypt_block(&mut arr);
        for i in 0..16 { arr[i] ^= prev[i]; }
        prev.copy_from_slice(block);
        plaintext.extend_from_slice(&arr);
    }
    unpad_pkcs7(&mut plaintext)?;
    String::from_utf8(plaintext).ok()
}

fn try_cbc_192(ciphertext: &[u8], key: &[u8; 24], iv: &[u8; 16]) -> Option<String> {
    let cipher = Aes192Dec::new(GenericArray::from_slice(key));
    let mut plaintext = Vec::with_capacity(ciphertext.len());
    let mut prev = *iv;
    for block in ciphertext.chunks(16) {
        let mut arr = GenericArray::<u8, U16>::clone_from_slice(block);
        cipher.decrypt_block(&mut arr);
        for i in 0..16 { arr[i] ^= prev[i]; }
        prev.copy_from_slice(block);
        plaintext.extend_from_slice(&arr);
    }
    unpad_pkcs7(&mut plaintext)?;
    String::from_utf8(plaintext).ok()
}

fn try_cbc_256(ciphertext: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Option<String> {
    let cipher = Aes256Dec::new(GenericArray::from_slice(key));
    let mut plaintext = Vec::with_capacity(ciphertext.len());
    let mut prev = *iv;
    for block in ciphertext.chunks(16) {
        let mut arr = GenericArray::<u8, U16>::clone_from_slice(block);
        cipher.decrypt_block(&mut arr);
        for i in 0..16 { arr[i] ^= prev[i]; }
        prev.copy_from_slice(block);
        plaintext.extend_from_slice(&arr);
    }
    unpad_pkcs7(&mut plaintext)?;
    String::from_utf8(plaintext).ok()
}

fn pad_key<const N: usize>(password: &str) -> [u8; N] {
    let mut key = [0u8; N];
    for (i, &b) in password.as_bytes().iter().enumerate().take(N) {
        key[i] = b;
    }
    key
}

// Derive key bytes from password: SHA-256 then take first N bytes
fn hash_key<const N: usize>(password: &str) -> [u8; N] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; N];
    let n = N.min(32);
    key[..n].copy_from_slice(&result[..n]);
    key
}

struct ScoredResult {
    plaintext: String,
    key_desc: String,
    match_ratio: f64,
}

/// Try a single AES mode combination and return (plaintext, blocks_decrypted, key_desc) on success.
fn try_one(raw: &[u8], key: &[u8], is_cbc: bool, iv: &[u8; 16]) -> Option<(String, usize)> {
    let pt = match key.len() {
        16 => {
            if is_cbc { try_cbc_128(raw, unsafe { &*(key.as_ptr() as *const [u8; 16]) }, iv) }
            else { try_ecb_128(raw, unsafe { &*(key.as_ptr() as *const [u8; 16]) }) }
        }
        24 => {
            if is_cbc { try_cbc_192(raw, unsafe { &*(key.as_ptr() as *const [u8; 24]) }, iv) }
            else { try_ecb_192(raw, unsafe { &*(key.as_ptr() as *const [u8; 24]) }) }
        }
        32 => {
            if is_cbc { try_cbc_256(raw, unsafe { &*(key.as_ptr() as *const [u8; 32]) }, iv) }
            else { try_ecb_256(raw, unsafe { &*(key.as_ptr() as *const [u8; 32]) }) }
        }
        _ => return None,
    }?;
    let blocks = raw.len() / 16;
    Some((pt, blocks))
}

/// Try all AES key sizes × derivations × modes in parallel for one password.
/// Returns all candidate (plaintext, blocks, key_desc) triples that pass PKCS7 unpadding.
fn try_all_par(raw: &[u8], pre_iv: Option<[u8; 16]>, pre_ct: &[u8], pw: &str) -> Vec<(String, usize, String)> {
    let zero_iv = [0u8; 16];
    let has_iv = pre_iv.is_some();
    let extracted = pre_iv.unwrap_or(zero_iv);

    let raw_total_blocks = raw.len() / 16;
    let pre_ct_blocks = pre_ct.len() / 16;

    struct Attempt {
        key: Vec<u8>,
        data: Vec<u8>,
        iv: [u8; 16],
        is_cbc: bool,
        desc: String,
        blocks: usize,
    }

    let mut attempts = Vec::new();

    for (key_bytes, tag) in [
        (pad_key::<16>(pw).to_vec(), "pad16"),
        (hash_key::<16>(pw).to_vec(), "sha16"),
        (pad_key::<24>(pw).to_vec(), "pad24"),
        (hash_key::<24>(pw).to_vec(), "sha24"),
        (pad_key::<32>(pw).to_vec(), "pad32"),
        (hash_key::<32>(pw).to_vec(), "sha32"),
    ] {
        let key_size = key_bytes.len();
        let prefix = match key_size {
            16 => "AES-128",
            24 => "AES-192",
            _ => "AES-256",
        };

        // ECB on full raw
        attempts.push(Attempt {
            key: key_bytes.clone(),
            data: raw.to_vec(),
            iv: zero_iv,
            is_cbc: false,
            desc: format!("{}/{}/ECB:{}", prefix, tag, pw),
            blocks: raw_total_blocks,
        });
        // CBC with zero IV on full raw
        attempts.push(Attempt {
            key: key_bytes.clone(),
            data: raw.to_vec(),
            iv: zero_iv,
            is_cbc: true,
            desc: format!("{}/{}/CBC-zero:{}", prefix, tag, pw),
            blocks: raw_total_blocks,
        });

        if has_iv {
            // ECB on pre_ct (first 16 bytes extracted as IV)
            attempts.push(Attempt {
                key: key_bytes.clone(),
                data: pre_ct.to_vec(),
                iv: zero_iv,
                is_cbc: false,
                desc: format!("{}/{}/ECB-extracted:{}", prefix, tag, pw),
                blocks: pre_ct_blocks,
            });
            // CBC with extracted IV on pre_ct
            attempts.push(Attempt {
                key: key_bytes.clone(),
                data: pre_ct.to_vec(),
                iv: extracted,
                is_cbc: true,
                desc: format!("{}/{}/CBC-extracted:{}", prefix, tag, pw),
                blocks: pre_ct_blocks,
            });
        }
    }

    // Parallel execution
    let results = Mutex::new(Vec::new());
    attempts.par_iter().for_each(|a| {
        if let Some((pt, _)) = try_one(&a.data, &a.key, a.is_cbc, &a.iv) {
            let mut r = results.lock().unwrap();
            r.push((pt, a.blocks, a.desc.clone()));
        }
    });

    results.into_inner().unwrap()
}

/// Score a candidate: prefer longer plaintext with more blocks decrypted.
fn score_candidate(pt: &str, blocks: usize, max_blocks: usize, checker: &CheckerTypes) -> Option<ScoredResult> {
    let cr = checker.check_text(pt);
    if !cr.is_identified { return None; }
    // Strong preference for decrypting the full ciphertext (all blocks).
    // Extracted-IV modes decrypt half the data and can produce artificially
    // high match ratios on shorter text.
    let block_penalty = 1.0 - (max_blocks.saturating_sub(blocks) as f64 * 0.20);
    let effective_ratio = cr.match_ratio * block_penalty;
    Some(ScoredResult {
        plaintext: pt.to_string(),
        key_desc: String::new(),
        match_ratio: effective_ratio,
    })
}

/// Test a single password: try all modes in parallel, score with checker, return best result.
fn try_password_par(text: &str, raw: &[u8], pre_iv: Option<[u8; 16]>, pre_ct: &[u8], checker: &CheckerTypes, pw: &str) -> Option<CrackResult> {
    let candidates = try_all_par(raw, pre_iv, pre_ct, pw);
    if candidates.is_empty() { return None; }

    let max_blocks = raw.len() / 16;

    // Score all candidates with the checker and pick the best
    let best = candidates.par_iter()
        .filter_map(|(pt, blocks, desc)| {
            let mut s = score_candidate(pt, *blocks, max_blocks, checker)?;
            s.key_desc = desc.clone();
            Some(s)
        })
        .max_by(|a, b| a.match_ratio.partial_cmp(&b.match_ratio).unwrap_or(std::cmp::Ordering::Equal));

    best.map(|s| {
        let mut r = CrackResult::new("AES-256", "", "");
        r.success = true;
        r.encrypted_text = text.to_string();
        r.unencrypted_text = Some(vec![s.plaintext]);
        r.key = Some(s.key_desc);
        r.match_ratio = s.match_ratio;
        r
    })
}

fn collect_passwords() -> Vec<String> {
    let config = get_config();
    let mut all = config.keys.clone();

    let hardcoded = [
        "jkandkj21321kldanfkenaf",
        "", "password", "Password", "12345678", "secret", "aes", "AES",
    ];
    for pw in &hardcoded {
        if !all.contains(&pw.to_string()) {
            all.push(pw.to_string());
        }
    }

    let dict = dictionary::wordlist();
    for wlen in 4..=12usize {
        if wlen >= dict.by_length.len() { continue; }
        for word in &dict.by_length[wlen] {
            if all.len() >= 500 + config.keys.len() { break; }
            if !all.contains(word) {
                all.push(word.clone());
            }
        }
        if all.len() >= 500 + config.keys.len() { break; }
    }

    all
}

impl Crack for Decoder<Aes256Decoder> {
    fn crack(&self, text: &str, checker: &CheckerTypes) -> CrackResult {
        let mut result = CrackResult::new(self.get_name(), self.get_description(), self.get_link());
        result.encrypted_text = text.to_string();

        let trimmed = text.trim();
        let raw = if let Ok(b) = hex::decode(trimmed) { b }
        else if let Ok(b) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, trimmed) { b }
        else { return result; };

        if raw.is_empty() || raw.len() % 16 != 0 { return result; }

        let (pre_iv, pre_ct) = if raw.len() > 16 {
            let mut iv = [0u8; 16];
            iv.copy_from_slice(&raw[..16]);
            (Some(iv), &raw[16..])
        } else {
            (None, &raw[..])
        };

        let passwords = collect_passwords();

        // Test all passwords in parallel, collect scored results, return best
        if let Some(best) = passwords.par_iter()
            .filter_map(|pw| try_password_par(text, &raw, pre_iv, pre_ct, checker, pw))
            .max_by(|a, b| a.match_ratio.partial_cmp(&b.match_ratio).unwrap_or(std::cmp::Ordering::Equal))
        {
            return best;
        }

        result
    }

    fn get_name(&self) -> &'static str { "AES-256" }
    fn get_popularity(&self) -> f32 { 0.5 }
    fn get_tags(&self) -> &'static [&'static str] { &["aes", "decoder", "cipher", "symmetric"] }
    fn get_description(&self) -> &'static str { "AES is a symmetric block cipher with support for 128, 192, and 256-bit keys." }
    fn get_link(&self) -> &'static str { "https://en.wikipedia.org/wiki/Advanced_Encryption_Standard" }
}

impl Decoder<Aes256Decoder> {
    pub fn new() -> Self {
        Decoder {
            name: "AES-256",
            description: "AES is a symmetric block cipher with support for 128, 192, and 256-bit keys.",
            link: "https://en.wikipedia.org/wiki/Advanced_Encryption_Standard",
            tags: vec!["aes", "decoder", "cipher", "symmetric"],
            popularity: 0.5,
            phantom: PhantomData,
        }
    }
}
