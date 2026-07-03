use crate::checkers::{Check, CheckResult, Checker};
use std::marker::PhantomData;

const COMMON_ENGLISH_WORDS: &[&str] = &[
    "the", "be", "to", "of", "and", "a", "in", "that", "have", "it", "for", "not", "on", "with",
    "he", "as", "you", "do", "at", "this", "but", "his", "by", "from", "they", "we", "say", "her",
    "she", "or", "an", "will", "my", "one", "all", "would", "there", "their", "what", "so", "up",
    "out", "if", "about", "who", "get", "which", "go", "me", "when", "make", "can", "like", "time",
    "no", "just", "him", "know", "take", "people", "into", "year", "your", "good", "some", "could",
    "them", "see", "other", "than", "then", "now", "look", "only", "come", "its", "over", "think",
    "also", "back", "after", "use", "two", "how", "our", "work", "first", "well", "way", "even",
    "new", "want", "because", "any", "these", "give", "day", "most", "are", "was", "were", "been",
    "said", "more", "very", "every", "still", "between", "own", "each", "right", "great", "same",
    "old", "another", "while", "three", "place", "small", "under", "large", "long", "off", "hand",
    "high", "different", "end", "through", "turn", "should", "world", "need", "play", "must",
    "may", "set", "home", "hand", "again", "find", "many", "much", "ask", "part", "last", "put",
    "thing", "next", "keep", "head", "stand", "own", "show", "between", "should", "country",
    "house", "point", "here", "number", "group", "water", "man", "woman", "child", "life", "hand",
    "eye", "face", "place", "week", "case", "question", "program", "system", "information",
    "government", "company", "problem", "example", "service", "support", "process", "product",
    "result", "research", "development", "business", "education", "community", "security",
    "something", "everything", "nothing", "message", "password", "username", "admin", "login",
    "access", "secret", "encrypted", "decoded", "decrypted", "flag", "key", "text", "data",
    "file", "name", "user", "code", "type", "value", "level", "control", "state", "line",
    "order", "report", "member", "price", "check", "help", "form", "area", "view", "task",
    "design", "test", "list", "note", "rate", "rule", "role", "link", "flag", "mark", "step",
    "plan", "item", "cost", "fact", "idea", "move", "team", "sort", "kind", "door", "rule",
    "table", "story", "range", "field", "power", "class", "force", "base", "space", "heart",
    "light", "sound", "color", "sense", "speed", "shape", "truth", "watch", "score", "track",
    "shall", "drive", "cross", "speak", "cover", "carry", "raise", "break", "fight", "throw",
    "spend", "fall", "lead", "learn", "agree", "allow", "appear", "change", "believe",
    "decide", "expect", "happen", "include", "provide", "remember", "require", "consider",
    "continue", "determine", "develop", "establish", "identify", "indicate", "involve",
    "maintain", "measure", "prepare", "present", "produce", "represent", "understand",
    "value", "follow", "receive", "achieve", "complete", "contain", "describe", "discuss",
    "explain", "express", "improve", "increase", "reduce", "remove", "replace", "report",
    "suggest", "support", "believe", "benefit", "challenge", "commit", "communicate",
    "contribute", "defend", "define", "deliver", "depend", "protect", "provide", "publish",
    "realize", "recognize", "recommend", "record", "reform", "register", "regulate",
    "reinforce", "relate", "release", "rely", "remove", "require", "research", "resolve",
    "respond", "restore", "reveal", "secure", "select", "settle", "solve", "submit",
    "succeed", "suffer", "supply", "survey", "survive", "suspect", "teach", "transfer",
    "transform", "treat", "utilize", "verify", "hello", "world", "welcome", "please",
    "thanks", "sorry", "congratulations", "congratulation", "success", "failed",
    "attack", "attacks", "attacked", "attacking", "dawn", "morning", "night",
    "evening", "afternoon", "midnight", "noon", "today", "tomorrow", "yesterday",
    "north", "south", "east", "west", "northern", "southern", "eastern", "western",
    "left", "right", "center", "middle", "front", "back", "side",
    "stop", "start", "begin", "end", "finish", "continue", "pause",
    "forward", "backward", "upward", "downward", "inside", "outside",
    "above", "below", "under", "over", "before", "after", "during",
    "always", "never", "sometimes", "often", "rarely", "usually",
    "quick", "slow", "fast", "hard", "soft", "heavy", "light",
    "open", "close", "enter", "exit", "push", "pull", "press",
    "red", "blue", "green", "yellow", "black", "white", "brown",
    "pink", "purple", "orange", "gray", "gold", "silver",
    "mother", "father", "sister", "brother", "daughter", "son",
    "friend", "family", "parent", "child", "baby", "adult",
    "city", "town", "village", "street", "road", "building",
    "floor", "wall", "door", "window", "room", "kitchen", "bedroom",
    "summer", "winter", "spring", "autumn", "fall",
    "happy", "sad", "angry", "tired", "sleepy", "hungry", "thirsty", "sweet", "lovely",
    "healthy", "sick", "ill", "pain", "hurt", "safe", "danger",
    "rich", "poor", "young", "old", "new", "modern", "ancient",
        "special", "normal", "simple", "difficult", "easy", "hard", "is", "test",
    "common", "rare", "unique", "typical", "usual", "strange",
    "possible", "impossible", "probable", "certain", "sure",
];

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

        let matches = meaningful_words.iter().filter(|w| COMMON_ENGLISH_WORDS.contains(w)).count();
        let ratio = matches as f64 / meaningful_words.len() as f64;

        let mut is_identified = matches == meaningful_words.len()
            || (ratio >= 0.55 && matches >= 2);

        if !is_identified && meaningful_words.len() == 1 && meaningful_words[0].len() >= 8 {
            let word = meaningful_words[0];
            let wlen = word.len();
            let mut covered = vec![false; wlen];
            let mut total_covered = 0usize;
            let mut substr_count = 0usize;
            let mut dict_words: Vec<&str> = COMMON_ENGLISH_WORDS.iter()
                .filter(|w| w.len() >= 3 && w.len() <= wlen)
                .copied().collect();
            dict_words.sort_by(|a, b| b.len().cmp(&a.len()));
            for &dw in &dict_words {
                let dwlen = dw.len();
                let max_start = wlen - dwlen;
                for start in 0..=max_start {
                    let end = start + dwlen;
                    let already_covered = (start..end).any(|i| covered[i]);
                    if !already_covered && word[start..end] == *dw {
                        for i in start..end {
                            covered[i] = true;
                        }
                        total_covered += dwlen;
                        substr_count += 1;
                        break;
                    }
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
                format!("English plaintext detected ({}% word match)", (ratio * 100.0) as u32)
            } else {
                String::new()
            },
            checker_name: self.get_name().to_string(),
            checker_description: self.get_description().to_string(),
            link: self.link.to_string(),
            match_ratio: ratio,
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
