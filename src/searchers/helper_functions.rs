use std::sync::LazyLock;
use std::sync::Mutex;
use std::collections::HashMap;

pub static DECODER_SUCCESS_RATES: LazyLock<Mutex<HashMap<String, (u32, u32)>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

pub fn calculate_string_quality(text: &str) -> f32 {
    if text.len() < 3 || text.len() > 10000 {
        return 0.0;
    }

    let printable = text.chars().filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()).count();
    let printable_ratio = printable as f32 / text.len() as f32;
    if printable_ratio < 0.7 {
        return 0.0;
    }

    let non_printable = text.chars().filter(|c| !c.is_ascii() || c.is_ascii_control()).count();
    let non_printable_ratio = non_printable as f32 / text.len() as f32;

    1.0 - non_printable_ratio
}

pub fn calculate_string_worth(text: &str) -> bool {
    let quality = calculate_string_quality(text);
    quality >= 0.2
}

pub fn check_if_string_cant_be_decoded(text: &str) -> bool {
    text.len() <= 2 || {
        let non_printable = text.chars().filter(|c| !c.is_ascii_graphic() && !c.is_ascii_whitespace()).count();
        non_printable as f32 / text.len() as f32 > 0.3
    } || calculate_string_quality(text) < 0.2
}

pub fn is_common_sequence(prev: &str, current: &str) -> bool {
    matches!(
        (prev, current),
        ("Base64", "Base64")
            | ("Base64", "Base32")
            | ("Base32", "Base64")
            | ("Base64", "Hex")
            | ("Hex", "Base64")
            | ("Base64", "Base58")
            | ("Base58", "Base64")
            | ("Hex", "Hex")
    )
}

pub fn generate_heuristic(
    path: &[String],
    next_decoder_name: &str,
    text: &str,
    decoder_popularity: f32,
) -> f32 {
    let popularity_penalty = 1.0 - decoder_popularity;

    let success_rate_penalty = {
        let rates = DECODER_SUCCESS_RATES.lock().unwrap();
        if let Some(&(successes, total)) = rates.get(next_decoder_name) {
            if total > 0 {
                (1.0 - successes as f32 / total as f32) * 0.25
            } else {
                0.0
            }
        } else {
            0.0
        }
    };

    let depth = path.len() as f32;
    let depth_penalty = (0.05 * (1.0 + depth / 20.0) * depth).powi(2);

    let string_quality = calculate_string_quality(text);
    let quality_penalty = (1.0 - string_quality) * 0.5;

    let sequence_penalty = if path.len() >= 1 {
        let prev = &path[path.len() - 1];
        if is_common_sequence(prev, next_decoder_name) {
            0.0
        } else {
            0.25
        }
    } else {
        0.0
    };

    popularity_penalty + success_rate_penalty + depth_penalty + quality_penalty + sequence_penalty
}

pub fn record_decoder_success(name: &str) {
    let mut rates = DECODER_SUCCESS_RATES.lock().unwrap();
    let entry = rates.entry(name.to_string()).or_insert((0, 0));
    entry.0 += 1;
    entry.1 += 1;
}

pub fn record_decoder_attempt(name: &str) {
    let mut rates = DECODER_SUCCESS_RATES.lock().unwrap();
    let entry = rates.entry(name.to_string()).or_insert((0, 0));
    entry.1 += 1;
}
