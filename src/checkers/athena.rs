use crate::checkers::{Check, CheckResult, Checker};
use crate::checkers::english::EnglishChecker;
use std::marker::PhantomData;

pub struct Athena;

impl Check for Checker<Athena> {
    fn check(&self, text: &str) -> CheckResult {
        let english_checker = Checker::<EnglishChecker>::new();

        let result = english_checker.check(text);
        if result.is_identified {
            return result;
        }

        CheckResult {
            is_identified: false,
            text: text.to_string(),
            description: String::new(),
            checker_name: self.get_name().to_string(),
            checker_description: self.get_description().to_string(),
            link: self.link.to_string(),
        }
    }

    fn get_name(&self) -> &str { "Athena" }
    fn get_description(&self) -> &str { "Athena orchestrator that runs all checkers to identify plaintext." }
}

impl Checker<Athena> {
    pub fn new() -> Self {
        Checker {
            name: "Athena",
            description: "Athena orchestrator that runs all checkers to identify plaintext.",
            link: "",
            phantom: PhantomData,
        }
    }
}
