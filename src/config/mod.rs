use serde::{Deserialize, Serialize};
use once_cell::sync::OnceCell;
use std::sync::Mutex;

static GLOBAL_CONFIG: OnceCell<Mutex<Config>> = OnceCell::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub timeout_secs: u64,
    pub verbose: bool,
    pub top_results: bool,
    pub human_checker_on: bool,
    pub min_word_length: usize,
    pub max_depth: usize,
    pub keys: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            timeout_secs: 10,
            verbose: false,
            top_results: false,
            human_checker_on: false,
            min_word_length: 2,
            max_depth: 20,
            keys: Vec::new(),
        }
    }
}

pub fn set_global_config(config: Config) {
    let _ = GLOBAL_CONFIG.set(Mutex::new(config));
}

pub fn get_config() -> Config {
    GLOBAL_CONFIG
        .get()
        .map(|m| m.lock().unwrap().clone())
        .unwrap_or_default()
}
