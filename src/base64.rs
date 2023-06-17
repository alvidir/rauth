//! Base64 related utilities like custom engines for specific encodings/decodings.

use crate::result::{Error, Result};
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine,
};

/// An url safe implementation of [`base64::engine::Engine`]
pub const B64_CUSTOM_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

/// Decodes a b64 string
pub fn decode_str(s: &str) -> Result<String> {
    B64_CUSTOM_ENGINE
        .decode(s)
        .map_err(|err| {
            warn!(error = err.to_string(), "decoding base64 string");
            Error::Unknown
        })
        .and_then(|value| {
            String::from_utf8(value).map_err(|err| {
                warn!(
                    error = err.to_string(),
                    "getting string from base64 decoded data",
                );

                Error::Unknown
            })
        })
}
