//! Base64 related utilities like custom engines for specific encodings/decodings.

use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine,
};

/// An url safe implementation of [`base64::engine::Engine`]
pub const B64_CUSTOM_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

/// Decodes a b64 string
pub fn decode_str(s: &str) -> Result<String, String> {
    B64_CUSTOM_ENGINE
        .decode(s)
        .map_err(|err| err.to_string())
        .and_then(|value| String::from_utf8(value).map_err(|err| err.to_string()))
}
