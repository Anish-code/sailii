use crate::decoders::{Crack, CrackResult, Decoder, check_string_success};
use crate::checkers::CheckerTypes;
use std::marker::PhantomData;

pub struct Rot47Decoder;

impl Crack for Decoder<Rot47Decoder> {
    fn crack(&self, text: &str, checker: &CheckerTypes) -> CrackResult {
        let mut result = CrackResult::new(self.get_name(), self.get_description(), self.get_link());
        result.encrypted_text = text.to_string();

        let decoded: String = text.chars().map(|c| {
            if c as u32 >= 33 && c as u32 <= 126 {
                std::char::from_u32(33 + ((c as u32 - 33 + 47) % 94)).unwrap_or(c)
            } else {
                c
            }
        }).collect();

        if check_string_success(&decoded, text) {
            let check_result = checker.check_text(&decoded);
            if check_result.is_identified {
                result.success = true;
                result.unencrypted_text = Some(vec![decoded]);
                result.checker_name = check_result.checker_name;
            }
        }
        result
    }

    fn get_name(&self) -> &'static str { "ROT47" }
    fn get_popularity(&self) -> f32 { 0.5 }
    fn get_tags(&self) -> &'static [&'static str] { &["rot47", "decoder", "cipher"] }
    fn get_description(&self) -> &'static str { "ROT47 is a variant of Caesar cipher that shifts printable ASCII characters by 47 positions." }
    fn get_link(&self) -> &'static str { "https://en.wikipedia.org/wiki/ROT13#ROT47" }
}

impl Decoder<Rot47Decoder> {
    pub fn new() -> Self {
        Decoder {
            name: "ROT47",
            description: "ROT47 is a variant of Caesar cipher that shifts printable ASCII characters by 47 positions.",
            link: "https://en.wikipedia.org/wiki/ROT13#ROT47",
            tags: vec!["rot47", "decoder", "cipher"],
            popularity: 0.5,
            phantom: PhantomData,
        }
    }
}
