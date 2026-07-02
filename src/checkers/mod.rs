mod checker_type;
pub use checker_type::{CheckResult, Checker, Check as CheckTrait};
pub(crate) use checker_type::Check;
pub(crate) use athena::Athena;

pub(crate) mod english;
mod athena;

use std::sync::LazyLock;
use std::collections::HashMap;

pub type CheckerBox = Box<dyn Check + Sync + Send>;

impl CheckerTypes {
    pub fn check_text(&self, text: &str) -> CheckResult {
        Check::check(self, text)
    }
}

pub enum CheckerTypes {
    English(Checker<english::EnglishChecker>),
    Athena(Checker<athena::Athena>),
}

impl Check for CheckerTypes {
    fn check(&self, text: &str) -> CheckResult {
        match self {
            CheckerTypes::English(c) => c.check(text),
            CheckerTypes::Athena(c) => c.check(text),
        }
    }

    fn get_name(&self) -> &str {
        match self {
            CheckerTypes::English(c) => c.get_name(),
            CheckerTypes::Athena(c) => c.get_name(),
        }
    }

    fn get_description(&self) -> &str {
        match self {
            CheckerTypes::English(c) => c.get_description(),
            CheckerTypes::Athena(c) => c.get_description(),
        }
    }
}

pub static CHECKER_MAP: LazyLock<HashMap<&'static str, CheckerBox>> = LazyLock::new(|| {
    let mut m: HashMap<&str, CheckerBox> = HashMap::new();
    m.insert("English", Box::new(CheckerTypes::English(Checker::<english::EnglishChecker>::new())));
    m.insert("Athena", Box::new(CheckerTypes::Athena(Checker::<athena::Athena>::new())));
    m
});

pub fn get_checker_by_name(name: &str) -> Option<&'static dyn Check> {
    CHECKER_MAP.get(name).map(|b| b.as_ref() as &dyn Check)
}
