use crate::decoders::{Crack, CrackResult, Decoder, check_string_success};
use crate::checkers::CheckerTypes;
use std::marker::PhantomData;

pub struct Rot13Decoder;

fn rot13(text: &str) -> String {
    text.chars().map(|c| {
        if c.is_ascii_uppercase() {
            let offset = (c as u8 - b'A' + 13) % 26;
            (b'A' + offset) as char
        } else if c.is_ascii_lowercase() {
            let offset = (c as u8 - b'a' + 13) % 26;
            (b'a' + offset) as char
        } else {
            c
        }
    }).collect()
}

impl Crack for Decoder<Rot13Decoder> {
    fn crack(&self, text: &str, checker: &CheckerTypes) -> CrackResult {
        let mut result = CrackResult::new(self.get_name(), self.get_description(), self.get_link());
        result.encrypted_text = text.to_string();

        let decoded = rot13(text);
        if check_string_success(&decoded, text) {
            let check_result = checker.check_text(&decoded);
            if check_result.is_identified {
                result.success = true;
                result.unencrypted_text = Some(vec![decoded]);
                result.key = Some("ROT13".to_string());
                result.checker_name = check_result.checker_name;
                return result;
            }
        }

        result
    }

    fn get_name(&self) -> &'static str { "ROT13" }
    fn get_popularity(&self) -> f32 { 0.5 }
    fn get_tags(&self) -> &'static [&'static str] { &["rot13", "decoder", "cipher", "classical"] }
    fn get_description(&self) -> &'static str { "ROT13 is a Caesar cipher variant that shifts letters by 13 positions." }
    fn get_link(&self) -> &'static str { "https://en.wikipedia.org/wiki/ROT13" }
}

impl Decoder<Rot13Decoder> {
    pub fn new() -> Self {
        Decoder {
            name: "ROT13",
            description: "ROT13 is a Caesar cipher variant that shifts letters by 13 positions.",
            link: "https://en.wikipedia.org/wiki/ROT13",
            tags: vec!["rot13", "decoder", "cipher", "classical"],
            popularity: 0.5,
            phantom: PhantomData,
        }
    }
}
