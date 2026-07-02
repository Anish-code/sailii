use crate::decoders::CrackResult;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Mutex;

static CACHE_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

fn cache_dir() -> PathBuf {
    let mut dir = CACHE_DIR.lock().unwrap();
    if let Some(ref d) = *dir {
        return d.clone();
    }
    let d = if let Some(base) = dirs::cache_dir() {
        base.join("sailii")
    } else {
        PathBuf::from(".sailii-cache")
    };
    let _ = std::fs::create_dir_all(&d);
    *dir = Some(d.clone());
    d
}

fn hash_input(input: &str) -> String {
    let mut h = DefaultHasher::new();
    input.hash(&mut h);
    format!("{:x}", h.finish())
}

pub fn setup_database() {
    let _ = cache_dir();
}

pub fn read_cache(input: &str) -> Option<CrackResult> {
    let path = cache_dir().join(format!("{}.json", hash_input(input)));
    if path.exists() {
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(result) = serde_json::from_str::<CrackResult>(&data) {
                return Some(result);
            }
        }
    }
    None
}

pub fn insert_cache(input: &str, result: &CrackResult) {
    let path = cache_dir().join(format!("{}.json", hash_input(input)));
    if let Ok(data) = serde_json::to_string(result) {
        let _ = std::fs::write(&path, &data);
    }
}
