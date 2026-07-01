use crate::checkers::{CheckerTypes, Checker, Athena};
use crate::decoders::{Crack, CrackResult, get_all_decoders};
use crate::searchers::helper_functions::{
    generate_heuristic, calculate_string_worth, check_if_string_cant_be_decoded,
    record_decoder_success, record_decoder_attempt,
};
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use dashmap::DashSet;

#[derive(Debug, Clone)]
pub struct SearchNode {
    pub text: String,
    pub path: Vec<String>,
    pub total_cost: f32,
}

impl Eq for SearchNode {}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.total_cost == other.total_cost
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.total_cost.partial_cmp(&other.total_cost).map(|o| o.reverse())
    }
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.total_cost.partial_cmp(&other.total_cost)
            .map(|o| o.reverse())
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

pub struct AStarSearch {
    open_set: Arc<Mutex<BinaryHeap<SearchNode>>>,
    seen_strings: Arc<DashSet<String>>,
    seen_results: Arc<DashSet<String>>,
    checker: CheckerTypes,
    stop_flag: Arc<AtomicBool>,
    result_tx: std::sync::mpsc::Sender<CrackResult>,
}

impl AStarSearch {
    pub fn new(result_tx: std::sync::mpsc::Sender<CrackResult>, stop_flag: Arc<AtomicBool>) -> Self {
        AStarSearch {
            open_set: Arc::new(Mutex::new(BinaryHeap::new())),
            seen_strings: Arc::new(DashSet::new()),
            seen_results: Arc::new(DashSet::new()),
            checker: CheckerTypes::Athena(Checker::<Athena>::new()),
            stop_flag,
            result_tx,
        }
    }

    pub fn start(&self, initial_text: &str, max_depth: usize) {
        let initial_node = SearchNode {
            text: initial_text.to_string(),
            path: Vec::new(),
            total_cost: 0.0,
        };

        {
            let mut open = self.open_set.lock().unwrap();
            open.push(initial_node);
        }

        let batch_size = 8;

        loop {
            if self.stop_flag.load(Ordering::Relaxed) {
                return;
            }

            let batch = {
                let mut open = self.open_set.lock().unwrap();
                let mut batch = Vec::with_capacity(batch_size);
                for _ in 0..batch_size {
                    if let Some(node) = open.pop() {
                        batch.push(node);
                    } else {
                        break;
                    }
                }
                batch
            };

            if batch.is_empty() {
                break;
            }

            let results: Vec<Option<CrackResult>> = {
                let searcher = &self;
                batch.into_iter().map(|node| {
                    searcher.expand_node(&node, max_depth)
                }).collect()
            };

            for result in results.into_iter().flatten() {
                if result.success {
                    let key = result.unencrypted_text.as_ref()
                        .and_then(|v| v.first())
                        .cloned()
                        .unwrap_or_default();
                    if self.seen_results.insert(key) {
                        let _ = self.result_tx.send(result);
                    }
                }
            }

            if self.open_set.lock().unwrap().is_empty() {
                break;
            }
        }
    }

    fn expand_node(&self, node: &SearchNode, max_depth: usize) -> Option<CrackResult> {
        if node.path.len() >= max_depth {
            return None;
        }

        let is_first = node.path.is_empty();
        let decoders: Vec<&'static dyn Crack> = if is_first {
            let mut d = get_all_decoders();
            d.sort_by(|a, b| b.get_popularity().partial_cmp(&a.get_popularity()).unwrap_or(std::cmp::Ordering::Equal));
            d
        } else {
            let last_decoder = &node.path[node.path.len() - 1];
            let mut d: Vec<&'static dyn Crack> = get_all_decoders();
            d.retain(|dec| dec.get_name() != last_decoder);
            d
        };

        for decoder in &decoders {
            if self.stop_flag.load(Ordering::Relaxed) {
                return None;
            }

            let decoder_name = decoder.get_name();

            if node.path.len() >= 1 {
                let prev = &node.path[node.path.len() - 1];
                if decoder.get_tags().contains(&"base") && decoder_name == prev {
                    continue;
                }
            }

            record_decoder_attempt(decoder_name);

            let result = decoder.crack(&node.text, &self.checker);

            if result.success {
                record_decoder_success(decoder_name);
                return Some(result);
            }

            if result.unencrypted_text.is_some() {
                let decoded = result.unencrypted_text.as_ref().unwrap()[0].clone();
                if check_if_string_cant_be_decoded(&decoded) {
                    continue;
                }
                if !calculate_string_worth(&decoded) {
                    continue;
                }
                if self.seen_strings.contains(&decoded) {
                    continue;
                }
                self.seen_strings.insert(decoded.clone());

                let new_path = {
                    let mut p = node.path.clone();
                    p.push(decoder_name.to_string());
                    p
                };

                let heuristic = generate_heuristic(
                    &new_path,
                    decoder_name,
                    &decoded,
                    decoder.get_popularity(),
                );

                let cost = new_path.len() as f32;
                let total_cost = cost + heuristic;

                let new_node = SearchNode {
                    text: decoded,
                    path: new_path,
                    total_cost,
                };

                let mut open = self.open_set.lock().unwrap();
                open.push(new_node);
            }
        }

        None
    }
}
