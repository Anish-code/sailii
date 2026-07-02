use crate::checkers::CheckerTypes;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrackResult {
    pub success: bool,
    pub encrypted_text: String,
    pub unencrypted_text: Option<Vec<String>>,
    pub decoder: String,
    pub checker_name: String,
    pub key: Option<String>,
    pub description: String,
    pub link: String,
    pub match_ratio: f64,
}

impl CrackResult {
    pub fn new(decoder: &str, description: &str, link: &str) -> Self {
        CrackResult {
            success: false,
            encrypted_text: String::new(),
            unencrypted_text: None,
            decoder: decoder.to_string(),
            checker_name: String::new(),
            key: None,
            description: description.to_string(),
            link: link.to_string(),
            match_ratio: 0.0,
        }
    }
}

pub trait Crack: Sync + Send {
    fn crack(&self, text: &str, checker: &CheckerTypes) -> CrackResult;
    fn get_name(&self) -> &'static str;
    fn get_popularity(&self) -> f32;
    fn get_tags(&self) -> &'static [&'static str];
    fn get_description(&self) -> &'static str;
    fn get_link(&self) -> &'static str;
}

pub struct Decoder<T> {
    pub name: &'static str,
    pub description: &'static str,
    pub link: &'static str,
    pub tags: Vec<&'static str>,
    pub popularity: f32,
    pub phantom: PhantomData<T>,
}

pub struct DefaultDecoder;

pub fn check_string_success(decoded: &str, original: &str) -> bool {
    !decoded.is_empty() && decoded != original
}
