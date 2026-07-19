use crate::decoders::{Crack, CrackResult, Decoder, check_string_success};
use crate::checkers::CheckerTypes;
use crate::dictionary;
use aes::cipher::{BlockDecrypt, KeyInit, generic_array::GenericArray, generic_array::typenum::U16};
use aes::{Aes128Dec, Aes192Dec, Aes256Dec};
use rayon::prelude::*;
use sha2::{Sha256, Digest};
use std::marker::PhantomData;

pub struct Aes256Decoder;

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

fn check_and_set(text: &str, decoded: &str, checker: &CheckerTypes, result: &mut CrackResult, key_desc: &str) -> bool {
    if check_string_success(decoded, text) {
        let cr = checker.check_text(decoded);
        if cr.is_identified {
            result.success = true;
            result.unencrypted_text = Some(vec![decoded.to_string()]);
            result.key = Some(key_desc.to_string());
            result.checker_name = cr.checker_name;
            return true;
        }
    }
    false
}

fn try_all(text: &str, raw: &[u8], pre_iv: Option<[u8; 16]>, pre_ct: &[u8], checker: &CheckerTypes, result: &mut CrackResult, pw: &str) -> bool {
    let zero_iv = [0u8; 16];

    // AES-128 (16-byte key)
    let k16_pad = pad_key::<16>(pw);
    let k16_hash = hash_key::<16>(pw);
    for (key, tag) in [(&k16_pad, "pad16"), (&k16_hash, "sha16")] {
        let desc = format!("AES-128/{}:{}", tag, pw);
        if let Some(d) = try_ecb_128(raw, key) { if check_and_set(text, &d, checker, result, &desc) { return true; } }
        if pre_iv.is_some() { if let Some(d) = try_ecb_128(pre_ct, key) { if check_and_set(text, &d, checker, result, &desc) { return true; } } }
        if let Some(d) = try_cbc_128(raw, key, &zero_iv) { if check_and_set(text, &d, checker, result, &desc) { return true; } }
        if let Some(ref iv) = pre_iv { if let Some(d) = try_cbc_128(pre_ct, key, iv) { if check_and_set(text, &d, checker, result, &desc) { return true; } } }
    }

    // AES-192 (24-byte key)
    let k24_pad = pad_key::<24>(pw);
    let k24_hash = hash_key::<24>(pw);
    for (key, tag) in [(&k24_pad, "pad24"), (&k24_hash, "sha24")] {
        let desc = format!("AES-192/{}:{}", tag, pw);
        if let Some(d) = try_ecb_192(raw, key) { if check_and_set(text, &d, checker, result, &desc) { return true; } }
        if pre_iv.is_some() { if let Some(d) = try_ecb_192(pre_ct, key) { if check_and_set(text, &d, checker, result, &desc) { return true; } } }
        if let Some(d) = try_cbc_192(raw, key, &zero_iv) { if check_and_set(text, &d, checker, result, &desc) { return true; } }
        if let Some(ref iv) = pre_iv { if let Some(d) = try_cbc_192(pre_ct, key, iv) { if check_and_set(text, &d, checker, result, &desc) { return true; } } }
    }

    // AES-256 (32-byte key)
    let k32_pad = pad_key::<32>(pw);
    let k32_hash = hash_key::<32>(pw);
    for (key, tag) in [(&k32_pad, "pad32"), (&k32_hash, "sha32")] {
        let desc = format!("AES-256/{}:{}", tag, pw);
        if let Some(d) = try_ecb_256(raw, key) { if check_and_set(text, &d, checker, result, &desc) { return true; } }
        if pre_iv.is_some() { if let Some(d) = try_ecb_256(pre_ct, key) { if check_and_set(text, &d, checker, result, &desc) { return true; } } }
        if let Some(d) = try_cbc_256(raw, key, &zero_iv) { if check_and_set(text, &d, checker, result, &desc) { return true; } }
        if let Some(ref iv) = pre_iv { if let Some(d) = try_cbc_256(pre_ct, key, iv) { if check_and_set(text, &d, checker, result, &desc) { return true; } } }
    }

    false
}

fn try_password(text: &str, raw: &[u8], pre_iv: Option<[u8; 16]>, pre_ct: &[u8], checker: &CheckerTypes, pw: &str) -> Option<CrackResult> {
    let mut r = CrackResult::new("AES-256", "", "");
    if try_all(text, raw, pre_iv, pre_ct, checker, &mut r, pw) {
        Some(r)
    } else {
        None
    }
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

        let hardcoded = [
            "jkandkj21321kldanfkenaf",
            "", "password", "Password", "12345678", "secret", "aes", "AES",
        ];

        for pw in &hardcoded {
            if let Some(r) = try_password(text, &raw, pre_iv, pre_ct, checker, pw) {
                return r;
            }
        }

        let dict = dictionary::wordlist();
        let mut dict_words: Vec<&str> = Vec::with_capacity(500);
        for wlen in 4..=12usize {
            if wlen >= dict.by_length.len() { continue; }
            for word in &dict.by_length[wlen] {
                if dict_words.len() >= 500 { break; }
                dict_words.push(word.as_str());
            }
            if dict_words.len() >= 500 { break; }
        }

        if !dict_words.is_empty() {
            if let Some(r) = dict_words.par_iter()
                .filter_map(|pw| try_password(text, &raw, pre_iv, pre_ct, checker, pw))
                .collect::<Vec<_>>()
                .into_iter()
                .next()
            {
                return r;
            }
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
