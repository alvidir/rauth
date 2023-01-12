//! Base64 related utilities like custom engines for specific encodings/decodings.

use base64::{
    alphabet,
    engine::{self, general_purpose},
};

/// An url safe implementation of [`base64::engine::Engine`]
pub const B64_CUSTOM_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);
