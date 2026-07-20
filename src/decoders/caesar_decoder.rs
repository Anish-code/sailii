use crate::decoders::{Crack, CrackResult, Decoder, check_string_success};use crate::checkers::CheckerTypes;use std::marker::PhantomData;pub struct CaesarDecoder;fn caesar_shift(text: &str, shift: u8) -> String {    text.chars().map(|c| {        if c.is_ascii_uppercase() {            let offset = (c as u8 - b'A' + shift) % 26;            (b'A' + offset) as char        } else if c.is_ascii_lowercase() {            let offset = (c as u8 - b'a' + shift) % 26;            (b'a' + offset) as char        } else {            c        }    }).collect()}impl Crack for Decoder<CaesarDecoder> {    fn crack(&self, text: &str, checker: &CheckerTypes) -> CrackResult {        let mut result = CrackResult::new(self.get_name(), self.get_description(), self.get_link());        result.encrypted_text = text.to_string();                for key_str in &crate::config::get_config().keys {
            if let Ok(shift) = key_str.parse::<u8>() {
                if (1..=25).contains(&shift) {
                    let decoded = caesar_shift(text, shift);
                    if check_string_success(&decoded, text) {
                        let check_result = checker.check_text(&decoded);
                        if check_result.is_identified {
                            result.success = true;
                            result.unencrypted_text = Some(vec![decoded]);
                            result.key = Some(shift.to_string());
                            result.checker_name = check_result.checker_name;
                            return result;
                        }
                    }
                }
            }
        }        if !crate::config::get_config().keys.is_empty() { return result; }        for shift in 1..26 {            let decoded = caesar_shift(text, shift);            if check_string_success(&decoded, text) {                let check_result = checker.check_text(&decoded);                if check_result.is_identified {                    result.success = true;                    result.unencrypted_text = Some(vec![decoded]);                    result.key = Some(shift.to_string());                    result.checker_name = check_result.checker_name;                    return result;                }            }        }        result    }    fn get_name(&self) -> &'static str { "Caesar" }    fn get_popularity(&self) -> f32 { 0.8 }    fn get_tags(&self) -> &'static [&'static str] { &["caesar", "decoder", "cipher", "classical"] }    fn get_description(&self) -> &'static str { "Caesar cipher shifts each letter by a fixed number of positions in the alphabet." }    fn get_link(&self) -> &'static str { "https://en.wikipedia.org/wiki/Caesar_cipher" }}impl Decoder<CaesarDecoder> {    pub fn new() -> Self {        Decoder {            name: "Caesar",            description: "Caesar cipher shifts each letter by a fixed number of positions in the alphabet.",            link: "https://en.wikipedia.org/wiki/Caesar_cipher",            tags: vec!["caesar", "decoder", "cipher", "classical"],            popularity: 0.8,            phantom: PhantomData,        }    }}



