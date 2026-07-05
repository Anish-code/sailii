mod interface;
pub use interface::*;

mod base64_decoder;
mod base32_decoder;
mod hex_decoder;
mod binary_decoder;
mod url_decoder;
mod reverse_decoder;
mod morse_decoder;
mod atbash_decoder;
mod caesar_decoder;
mod rot47_decoder;
mod base58_decoder;
mod base91_decoder;
mod railfence_decoder;
mod vigenere_decoder;
mod a1z26_decoder;
mod braille_decoder;
mod z85_decoder;

use std::sync::LazyLock;
use std::collections::HashMap;
use crate::checkers::CheckerTypes;

pub type DecoderBox = Box<dyn Crack + Sync + Send>;

pub enum DecoderType {
    Base64(Decoder<base64_decoder::Base64Decoder>),
    Base32(Decoder<base32_decoder::Base32Decoder>),
    Hex(Decoder<hex_decoder::HexDecoder>),
    Binary(Decoder<binary_decoder::BinaryDecoder>),
    Url(Decoder<url_decoder::UrlDecoder>),
    Reverse(Decoder<reverse_decoder::ReverseDecoder>),
    Morse(Decoder<morse_decoder::MorseDecoder>),
    Atbash(Decoder<atbash_decoder::AtbashDecoder>),
    Caesar(Decoder<caesar_decoder::CaesarDecoder>),
    Rot47(Decoder<rot47_decoder::Rot47Decoder>),
    Base58(Decoder<base58_decoder::Base58Decoder>),
    Base91(Decoder<base91_decoder::Base91Decoder>),
    Railfence(Decoder<railfence_decoder::RailfenceDecoder>),
    Vigenere(Decoder<vigenere_decoder::VigenereDecoder>),
    A1Z26(Decoder<a1z26_decoder::A1Z26Decoder>),
    Braille(Decoder<braille_decoder::BrailleDecoder>),
    Z85(Decoder<z85_decoder::Z85Decoder>),
}

impl DecoderType {
    pub fn all() -> Vec<Self> {
        vec![
            DecoderType::Base64(Decoder::<base64_decoder::Base64Decoder>::new()),
            DecoderType::Base32(Decoder::<base32_decoder::Base32Decoder>::new()),
            DecoderType::Hex(Decoder::<hex_decoder::HexDecoder>::new()),
            DecoderType::Binary(Decoder::<binary_decoder::BinaryDecoder>::new()),
            DecoderType::Url(Decoder::<url_decoder::UrlDecoder>::new()),
            DecoderType::Reverse(Decoder::<reverse_decoder::ReverseDecoder>::new()),
            DecoderType::Morse(Decoder::<morse_decoder::MorseDecoder>::new()),
            DecoderType::Atbash(Decoder::<atbash_decoder::AtbashDecoder>::new()),
            DecoderType::Caesar(Decoder::<caesar_decoder::CaesarDecoder>::new()),
            DecoderType::Rot47(Decoder::<rot47_decoder::Rot47Decoder>::new()),
            DecoderType::Base58(Decoder::<base58_decoder::Base58Decoder>::new()),
            DecoderType::Base91(Decoder::<base91_decoder::Base91Decoder>::new()),
            DecoderType::Railfence(Decoder::<railfence_decoder::RailfenceDecoder>::new()),
            DecoderType::Vigenere(Decoder::<vigenere_decoder::VigenereDecoder>::new()),
            DecoderType::A1Z26(Decoder::<a1z26_decoder::A1Z26Decoder>::new()),
            DecoderType::Braille(Decoder::<braille_decoder::BrailleDecoder>::new()),
            DecoderType::Z85(Decoder::<z85_decoder::Z85Decoder>::new()),
        ]
    }
}

macro_rules! delegate {
    ($self:expr, $method:ident, $($args:expr),*) => {
        match $self {
            DecoderType::Base64(d) => d.$method($($args),*),
            DecoderType::Base32(d) => d.$method($($args),*),
            DecoderType::Hex(d) => d.$method($($args),*),
            DecoderType::Binary(d) => d.$method($($args),*),
            DecoderType::Url(d) => d.$method($($args),*),
            DecoderType::Reverse(d) => d.$method($($args),*),
            DecoderType::Morse(d) => d.$method($($args),*),
            DecoderType::Atbash(d) => d.$method($($args),*),
            DecoderType::Caesar(d) => d.$method($($args),*),
            DecoderType::Rot47(d) => d.$method($($args),*),
            DecoderType::Base58(d) => d.$method($($args),*),
            DecoderType::Base91(d) => d.$method($($args),*),
            DecoderType::Railfence(d) => d.$method($($args),*),
            DecoderType::Vigenere(d) => d.$method($($args),*),
            DecoderType::A1Z26(d) => d.$method($($args),*),
            DecoderType::Braille(d) => d.$method($($args),*),
            DecoderType::Z85(d) => d.$method($($args),*),
        }
    };
}

impl Crack for DecoderType {
    fn crack(&self, text: &str, checker: &CheckerTypes) -> CrackResult {
        let _ = std::fs::write("C:\\Users\\anish\\Desktop\\crypto\\sailii\\debug_deleg.txt", format!("DecoderType::crack called! text={} len={}", text, text.len()));
        delegate!(self, crack, text, checker)
    }

    fn get_name(&self) -> &'static str { delegate!(self, get_name,) }
    fn get_popularity(&self) -> f32 { delegate!(self, get_popularity,) }
    fn get_tags(&self) -> &'static [&'static str] { delegate!(self, get_tags,) }
    fn get_description(&self) -> &'static str { delegate!(self, get_description,) }
    fn get_link(&self) -> &'static str { delegate!(self, get_link,) }
}

pub static DECODER_MAP: LazyLock<HashMap<&'static str, DecoderBox>> = LazyLock::new(|| {
    let mut m: HashMap<&'static str, DecoderBox> = HashMap::new();
    for dt in DecoderType::all() {
        let name: &'static str = dt.get_name();
        m.insert(name, Box::new(dt));
    }
    m
});

pub fn get_decoder_by_name(name: &str) -> Option<&'static dyn Crack> {
    DECODER_MAP.get(name).map(|b| b.as_ref() as &dyn Crack)
}

pub fn get_all_decoders() -> Vec<&'static dyn Crack> {
    let _ = std::fs::write("C:\\Users\\anish\\Desktop\\crypto\\sailii\\debug_decoders.txt", format!("get_all_decoders called! count={}", DECODER_MAP.len()));
    DECODER_MAP.values().map(|b| b.as_ref() as &dyn Crack).collect()
}

pub fn get_decoders_by_tag(tag: &str) -> Vec<&'static dyn Crack> {
    DECODER_MAP.values()
        .filter(|d| d.get_tags().contains(&tag))
        .map(|b| b.as_ref() as &dyn Crack)
        .collect()
}


