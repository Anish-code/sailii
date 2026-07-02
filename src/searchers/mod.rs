mod astar;
mod helper_functions;

use crate::checkers::{CheckerTypes, Checker, Athena};
use crate::checkers::english::EnglishChecker;
use crate::config::get_config;
use crate::decoders::CrackResult;
use crate::storage::{read_cache, insert_cache};
use astar::AStarSearch;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub fn search_for_plaintext(input: &str) -> Option<CrackResult> {
    if let Some(cached) = read_cache(input) {
        return Some(cached);
    }

    let config = get_config();
    let timeout = config.timeout_secs;

    let checker = CheckerTypes::Athena(Checker::<Athena>::new());
    let initial_check = checker.check_text(input);

    if initial_check.is_identified {
        let result = CrackResult {
            success: true,
            encrypted_text: input.to_string(),
            unencrypted_text: Some(vec![input.to_string()]),
            decoder: "Plaintext".to_string(),
            checker_name: initial_check.checker_name,
            key: None,
            description: "Input is already plaintext".to_string(),
            link: String::new(),
            match_ratio: initial_check.match_ratio,
        };
        insert_cache(input, &result);
        return Some(result);
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let (tx, rx) = std::sync::mpsc::channel::<CrackResult>();

    let search = AStarSearch::new(tx.clone(), stop_flag.clone());
    let input_clone = input.to_string();
    let max_depth = config.max_depth;

    let _search_thread = thread::spawn(move || {
        search.start(&input_clone, max_depth);
    });

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout);
    let mut best_result: Option<CrackResult> = None;
    let english_checker = CheckerTypes::English(Checker::<EnglishChecker>::new());

    let result = loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.as_secs() == 0 && remaining.subsec_millis() == 0 {
            stop_flag.store(true, Ordering::Relaxed);
            break best_result;
        }

        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(mut result) => {
                if result.success {
                    let plaintext = result.unencrypted_text
                        .as_ref()
                        .and_then(|v| v.first())
                        .map(|s| s.as_str())
                        .unwrap_or("");
                    let check_result = english_checker.check_text(plaintext);
                    result.match_ratio = check_result.match_ratio;
                    if result.match_ratio >= 0.90 {
                        stop_flag.store(true, Ordering::Relaxed);
                        insert_cache(input, &result);
                        break Some(result);
                    }

                    let is_better = match &best_result {
                        Some(current) => result.match_ratio > current.match_ratio
                            || (result.match_ratio == current.match_ratio
                                && result.unencrypted_text.as_ref().and_then(|v| v.first()).map(|s| s.len()).unwrap_or(0)
                                > current.unencrypted_text.as_ref().and_then(|v| v.first()).map(|s| s.len()).unwrap_or(0)),
                        None => true,
                    };
                    if is_better {
                        best_result = Some(result);
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                continue;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break best_result;
            }
        }
    };

    if let Some(ref best) = result {
        if best.success {
            insert_cache(input, best);
        }
    }
    result
    }
