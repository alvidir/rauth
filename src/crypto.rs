//! Criptography utilities for the validation and generation of JWTs as well as RSA encription and decription.

use crate::on_error;
use argon2::{Algorithm as ArgonAlgorithm, Argon2, Params, Version};
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine,
};
use once_cell::sync::Lazy;
use rand::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Base64(#[from] base64::DecodeError),
    #[error("{0}")]
    Argon(String),
}

impl From<argon2::Error> for Error {
    fn from(value: argon2::Error) -> Self {
        Self::Argon(value.to_string())
    }
}

const ARGON: Lazy<Argon2<'_>> =
    Lazy::new(|| Argon2::new(ArgonAlgorithm::Argon2id, Version::V0x13, Params::default()));

pub const URL_SAFE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub const B64_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

/// Encodes an slice of u8 into a b64 string.
pub fn encode_b64(v: &[u8]) -> String {
    B64_ENGINE.encode(v)
}

/// Decodes a b64 string into a vector of u8.
pub fn decode_b64<Err>(s: &str) -> Result<Vec<u8>> {
    B64_ENGINE
        .decode(s)
        .map_err(on_error!(Error, "decoding base64 string"))
}

/// Returns the salted hash of the given value.
pub fn salt(value: &[u8], salt: &[u8]) -> Result<Vec<u8>> {
    let mut buffer = vec![0_u8; salt.len()];
    ARGON
        .hash_password_into(value, &salt, &mut buffer)
        .map(|_| buffer)
        .map_err(on_error!(Error, "salting and hashing password"))
}

/// Returns the hash of the given value.
pub fn hash<H: Hash>(value: H) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Fills the given buffer with random data.
pub fn randomize(buff: &mut [u8]) {
    for index in 0..buff.len() {
        let mut rand = rand::thread_rng();
        let idx = rand.gen_range(0..URL_SAFE.len());
        buff[index] = URL_SAFE[idx]
    }
}
