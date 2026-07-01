use crate::decoders::{Crack, CrackResult, Decoder, check_string_success};
use crate::checkers::CheckerTypes;
use base64::Engine;
use std::marker::PhantomData;

pub struct Base64Decoder;

impl Crack for Decoder<Base64Decoder> {
    fn crack(&self, text: &str, checker: &CheckerTypes) -> CrackResult {
        let mut result = CrackResult::new(self.get_name(), self.get_description(), self.get_link());
        result.encrypted_text = text.to_string();

        for engine in [
            &base64::engine::general_purpose::STANDARD,
            &base64::engine::general_purpose::URL_SAFE,
        ] {
            if let Ok(decoded_bytes) = engine.decode(text) {
                if let Ok(decoded_str) = String::from_utf8(decoded_bytes) {
                    if check_string_success(&decoded_str, text) {
                        let check_result = checker.check_text(&decoded_str);
                        if check_result.is_identified {
                            result.success = true;
                            result.unencrypted_text = Some(vec![decoded_str]);
                            result.checker_name = check_result.checker_name;
                            return result;
                        }
                    }
                }
            }
        }
        result
    }

    fn get_name(&self) -> &'static str { "Base64" }
    fn get_popularity(&self) -> f32 { 1.0 }
    fn get_tags(&self) -> &'static [&'static str] { &["base64", "decoder", "base"] }
    fn get_description(&self) -> &'static str { "Base64 is a binary-to-text encoding scheme that represents binary data in an ASCII string format by translating it into a radix-64 representation." }
    fn get_link(&self) -> &'static str { "https://en.wikipedia.org/wiki/Base64" }
}

impl Decoder<Base64Decoder> {
    pub fn new() -> Self {
        Decoder {
            name: "Base64",
            description: "Base64 is a binary-to-text encoding scheme that represents binary data in an ASCII string format by translating it into a radix-64 representation.",
            link: "https://en.wikipedia.org/wiki/Base64",
            tags: vec!["base64", "decoder", "base"],
            popularity: 1.0,
            phantom: PhantomData,
        }
    }
}


