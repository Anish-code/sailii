mod astar;
mod helper_functions;

use crate::checkers::{CheckerTypes, Checker, Athena};
use crate::config::get_config;
use crate::decoders::CrackResult;
use astar::AStarSearch;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub fn search_for_plaintext(input: &str) -> Option<CrackResult> {
    let config = get_config();
    let timeout = config.timeout_secs;

    let checker = CheckerTypes::Athena(Checker::<Athena>::new());
    let initial_check = checker.check_text(input);

    if initial_check.is_identified {
        return Some(CrackResult {
            success: true,
            encrypted_text: input.to_string(),
            unencrypted_text: Some(vec![input.to_string()]),
            decoder: "Plaintext".to_string(),
            checker_name: initial_check.checker_name,
            key: None,
            description: "Input is already plaintext".to_string(),
            link: String::new(),
        });
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let (tx, rx) = std::sync::mpsc::channel::<CrackResult>();

    let search = AStarSearch::new(tx.clone(), stop_flag.clone());
    let input_clone = input.to_string();
    let max_depth = config.max_depth;

    let search_thread = thread::spawn(move || {
        search.start(&input_clone, max_depth);
    });

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout);

    let result = loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.as_secs() == 0 && remaining.subsec_millis() == 0 {
            stop_flag.store(true, Ordering::Relaxed);
            let _ = search_thread.join();
            break None;
        }

        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(result) => {
                if result.success {
                    stop_flag.store(true, Ordering::Relaxed);
                    let _ = search_thread.join();
                    break Some(result);
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                continue;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break None;
            }
        }
    };

    result
}
