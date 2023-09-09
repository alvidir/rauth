//! Criptography utilities for the validation and generation of JWTs as well as RSA encription and decription.

use base64::{
    alphabet,
    engine::{self, general_purpose},
    DecodeError, Engine,
};

pub const B64_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

/// Encodes an slice of u8 into a b64 string.
pub fn encode(v: &[u8]) -> String {
    B64_ENGINE.encode(v)
}

/// Decodes a b64 string into a vector of u8.
pub fn _decode<Err>(s: &str) -> Result<Vec<u8>, Err>
where
    Err: From<DecodeError>,
{
    B64_ENGINE.decode(s).map_err(Into::into)
}
