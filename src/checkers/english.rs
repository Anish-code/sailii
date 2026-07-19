use crate::checkers::{Check, CheckResult, Checker};
use crate::dictionary;
use std::marker::PhantomData;

pub struct EnglishChecker;

impl Check for Checker<EnglishChecker> {
    fn check(&self, text: &str) -> CheckResult {
        let text = text.trim();

        let early_result = |is_identified: bool| CheckResult {
            is_identified,
            text: text.to_string(),
            description: String::new(),
            checker_name: self.get_name().to_string(),
            checker_description: self.get_description().to_string(),
            link: self.link.to_string(),
            match_ratio: 0.0,
        };

        if text.len() < 3 {
            return early_result(false);
        }

        let printable_ratio = text.chars().filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()).count() as f64 / text.len() as f64;
        if printable_ratio < 0.8 {
            return early_result(false);
        }

        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return early_result(false);
        }

        let lower_words: Vec<String> = words.iter().map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphabetic()).to_string()).collect();
        let meaningful_words: Vec<&str> = lower_words.iter().map(|s| s.as_str()).filter(|w| w.len() >= 2).collect();

        if meaningful_words.is_empty() {
            return early_result(false);
        }

        let dict = dictionary::wordlist();

        let long_matches = meaningful_words.iter().filter(|w| w.len() >= 3 && dict.set.contains::<str>(w)).count();
        let short_matches = meaningful_words.iter().filter(|w| w.len() == 2 && dict.set.contains::<str>(w)).count();
        let total_ratio = (long_matches + short_matches) as f64 / meaningful_words.len() as f64;

        let mut is_identified = (long_matches == meaningful_words.iter().filter(|w| w.len() >= 3).count() && long_matches >= 1)
            || (long_matches >= 1 && total_ratio >= 0.85);

        if !is_identified && meaningful_words.len() == 1 && meaningful_words[0].len() >= 8 {
            let word = meaningful_words[0];
            let wlen = word.len();
            let mut covered = vec![false; wlen];
            let mut total_covered = 0usize;
            let mut substr_count = 0usize;

            let mut found: Vec<(usize, usize, &str)> = Vec::new();
            for start in 0..wlen {
                for end in (start + 3)..=wlen.min(start + 20) {
                    let sub = &word[start..end];
                    if dict.set.contains(sub) {
                        found.push((start, end - start, sub));
                    }
                }
            }
            found.sort_by(|a, b| b.1.cmp(&a.1));

            for &(start, len, _) in &found {
                let already_covered = (start..start + len).any(|i| covered[i]);
                if !already_covered {
                    for i in start..start + len {
                        covered[i] = true;
                    }
                    total_covered += len;
                    substr_count += 1;
                }
            }

            let coverage_ratio = total_covered as f64 / wlen as f64;
            if substr_count >= 2 && coverage_ratio >= 0.6 {
                is_identified = true;
            }
        }

        CheckResult {
            is_identified,
            text: text.to_string(),
            description: if is_identified {
                format!("English plaintext detected ({}% word match)", (total_ratio * 100.0) as u32)
            } else {
                String::new()
            },
            checker_name: self.get_name().to_string(),
            checker_description: self.get_description().to_string(),
            link: self.link.to_string(),
            match_ratio: total_ratio,
        }
    }

    fn get_name(&self) -> &str { "English Checker" }
    fn get_description(&self) -> &str { "Checks if text is English plaintext using dictionary word matching." }
}

impl Checker<EnglishChecker> {
    pub fn new() -> Self {
        Checker {
            name: "English Checker",
            description: "Checks if text is English plaintext using dictionary word matching.",
            link: "",
            phantom: PhantomData,
        }
    }
}
