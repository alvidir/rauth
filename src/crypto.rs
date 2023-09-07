//! Criptography utilities for the validation and generation of JWTs as well as RSA encription and decription.

use std::collections::hash_map::DefaultHasher;

use crate::on_error;
use argon2::{Algorithm as ArgonAlgorithm, Argon2, Params, Version};
use base64::{
    alphabet,
    engine::{self, general_purpose},
    DecodeError, Engine,
};
use libreauth::{
    hash::HashFunction::Sha256,
    oath::{TOTPBuilder, TOTP},
};
use once_cell::sync::Lazy;
use rand::prelude::*;
use std::hash::{Hash, Hasher};

const ARGON: Lazy<Argon2<'_>> =
    Lazy::new(|| Argon2::new(ArgonAlgorithm::Argon2id, Version::V0x13, Params::default()));

pub const URL_SAFE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub const B64_CUSTOM_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

/// Encodes an slice of u8 into a b64 string.
pub fn encode_b64(v: &[u8]) -> String {
    B64_CUSTOM_ENGINE.encode(v)
}

/// Decodes a b64 string into a vector of u8.
pub fn decode_b64<Err>(s: &str) -> Result<Vec<u8>, Err>
where
    Err: From<DecodeError>,
{
    B64_CUSTOM_ENGINE
        .decode(s)
        .map_err(on_error!("decoding base64 string"))
}

/// Returns the salted hash of the given value.
pub fn salt<const LEN: usize, Err>(value: &[u8], salt: &[u8]) -> Result<[u8; LEN], Err>
where
    Err: From<String>,
{
    let salt: [u8; LEN] = salt
        .try_into()
        .map_err(on_error!("converting into sized array"));

    let mut buffer = [0_u8; LEN];
    ARGON
        .hash_password_into(value, &salt, &mut buffer)
        .map(|_| buffer)
        .map_err(on_error!("salting and hashing password"))
}

/// Returns the hash of the given value.
pub fn hash<H: Hash>(value: H) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

// /// Given an elliptic curve secret in PEM format returns the resulting string of signing the provided
// /// payload in a JWT format.
// pub fn encode_jwt<S: Serialize, Err>(secret: &[u8], data: S) -> Result<String, Err>
// where
//     Err: From<String>,
// {
//     let header = Header::new(JwtAlgorithm::ES256);
//     let key =
//         EncodingKey::from_ec_pem(secret).map_err(on_error!("encoding elliptic curve keypair"))?;

//     let token =
//         jsonwebtoken::encode(&header, &data, &key).map_err(on_error!("signing json web token"))?;

//     Ok(token)
// }

// /// Given an elliptic curve secret in PEM format returns the token's claim if, and only if, the provided token
// /// is valid. Otherwise an error is returned.
// pub fn decode_jwt<T: DeserializeOwned, Err>(public: &[u8], data: &str) -> Result<T, Err>
// where
//     Err: From<String>,
// {
//     let validation = Validation::new(JwtAlgorithm::ES256);
//     let key =
//         DecodingKey::from_ec_pem(public).map_err(on_error!("decoding elliptic curve keypair"))?;

//     let token = jsonwebtoken::decode::<T>(data, &key, &validation)
//         .map_err(on_error!("checking token's signature"))?;

//     Ok(token.claims)
// }

/// Fills the given buffer with random data.
pub fn randomize(buff: &mut [u8]) {
    for index in 0..buff.len() {
        let mut rand = rand::thread_rng();
        let idx = rand.gen_range(0..URL_SAFE.len());
        buff[index] = URL_SAFE[idx]
    }
}

/// Given an array of bytes to use as secret, generates a TOTP instance.
pub fn generate_totp<Err>(secret: &[u8]) -> Result<TOTP, Err>
where
    Err: From<String>,
{
    TOTPBuilder::new()
        .key(secret)
        .hash_function(Sha256)
        .finalize()
        .map_err(on_error!("genereting time-based one time password"))
}

/// Given an array of bytes to use as TOTP's secret and a candidate of pwd, returns true if, and only if, pwd
/// has the same value as the TOTP.  
pub fn totp_matches<Err>(secret: &[u8], pwd: &str) -> Result<bool, Err>
where
    Err: From<String>,
{
    let totp = generate_totp(secret)?;
    Ok(totp.is_valid(pwd))
}

#[cfg(test)]
pub mod tests {
    use super::{generate_totp, totp_matches};

    #[test]
    fn verify_totp_ok_should_not_fail() {
        const SECRET: &[u8] = "hello world".as_bytes();

        let code = generate_totp::<String>(SECRET).unwrap().generate();

        assert_eq!(code.len(), 6);
        assert!(totp_matches::<String>(SECRET, &code).is_ok());
    }

    #[test]
    fn verify_totp_ko_should_not_fail() {
        const SECRET: &[u8] = "hello world".as_bytes();
        assert!(!totp_matches::<String>(SECRET, "tester").unwrap());
    }
}
