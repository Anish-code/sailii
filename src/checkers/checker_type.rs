use std::marker::PhantomData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub is_identified: bool,
    pub text: String,
    pub description: String,
    pub checker_name: String,
    pub checker_description: String,
    pub link: String,
    pub match_ratio: f64,
}

pub trait Check: Sync + Send {
    fn check(&self, text: &str) -> CheckResult;
    fn get_name(&self) -> &str;
    fn get_description(&self) -> &str;
}

pub struct Checker<T> {
    pub name: &'static str,
    pub description: &'static str,
    pub link: &'static str,
    pub phantom: PhantomData<T>,
}
