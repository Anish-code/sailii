use crate::decoders::{Crack, CrackResult, Decoder, check_string_success};
use crate::checkers::CheckerTypes;
use std::marker::PhantomData;

pub struct Base91Decoder;

impl Crack for Decoder<Base91Decoder> {
    fn crack(&self, text: &str, checker: &CheckerTypes) -> CrackResult {
        let mut result = CrackResult::new(self.get_name(), self.get_description(), self.get_link());
        result.encrypted_text = text.to_string();

        let decoded_bytes = base91::slice_decode(text.as_bytes());
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
        result
    }

    fn get_name(&self) -> &'static str { "Base91" }
    fn get_popularity(&self) -> f32 { 0.4 }
    fn get_tags(&self) -> &'static [&'static str] { &["base91", "decoder", "base"] }
    fn get_description(&self) -> &'static str { "Base91 is a binary-to-text encoding scheme that encodes binary data in base-91, being more efficient than Base64." }
    fn get_link(&self) -> &'static str { "https://en.wikipedia.org/wiki/Base91" }
}

impl Decoder<Base91Decoder> {
    pub fn new() -> Self {
        Decoder {
            name: "Base91",
            description: "Base91 is a binary-to-text encoding scheme that encodes binary data in base-91.",
            link: "https://en.wikipedia.org/wiki/Base91",
            tags: vec!["base91", "decoder", "base"],
            popularity: 0.4,
            phantom: PhantomData,
        }
    }
}


