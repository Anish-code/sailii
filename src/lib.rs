pub mod config;
pub mod dictionary;
pub mod decoders;
pub mod checkers;
pub mod searchers;
pub mod filtration_system;
pub mod storage;
pub mod timer;

use config::{Config, set_global_config};
use decoders::CrackResult;

pub fn perform_cracking(text: &str, config: Config) -> Option<CrackResult> {
    set_global_config(config);

    if text.trim().is_empty() {
        return None;
    }

    searchers::search_for_plaintext(text)
}
