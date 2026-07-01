use crate::decoders::{Crack, get_all_decoders};

pub struct Decoders {
    pub decoders: Vec<&'static dyn Crack>,
}

impl Decoders {
    pub fn new() -> Self {
        Decoders {
            decoders: get_all_decoders(),
        }
    }

    pub fn filter_by_tag(&mut self, tag: &str) {
        self.decoders.retain(|d| d.get_tags().contains(&tag));
    }

    pub fn exclude_tag(&mut self, tag: &str) {
        self.decoders.retain(|d| !d.get_tags().contains(&tag));
    }
}
