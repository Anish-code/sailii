use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub fn start_timer(seconds: u64, stop_flag: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        thread::sleep(std::time::Duration::from_secs(seconds));
        stop_flag.store(true, Ordering::Relaxed);
    })
}
