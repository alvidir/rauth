//! Criptography utilities for the validation and generation of JWTs as well as RSA encription and decription.

use crate::result::{Error, Result};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use libreauth::{
    hash::HashFunction::Sha256,
    oath::{TOTPBuilder, TOTP},
};
use openssl::{
    encrypt::{Decrypter, Encrypter},
    pkey::PKey,
    rsa::Padding,
};
use rand::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

const SECURE_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789";

/// Given an elliptic curve secret in PEM format returns the resulting string of signing the provided
/// payload in a JWT format.
pub fn sign_jwt<S: Serialize>(secret: &[u8], payload: S) -> Result<String> {
    let header = Header::new(Algorithm::ES256);
    let key = EncodingKey::from_ec_pem(secret).map_err(|err| {
        error!(error = err.to_string(), "encoding elliptic curve keypair",);
        Error::Unknown
    })?;

    let token = jsonwebtoken::encode(&header, &payload, &key).map_err(|err| {
        error!(error = err.to_string(), "signing json web token");
        Error::Unknown
    })?;

    Ok(token)
}

/// Given an elliptic curve secret in PEM format returns the token's claim if, and only if, the provided token
/// is valid. Otherwise an error is returned.
pub fn decode_jwt<T: DeserializeOwned>(public: &[u8], token: &str) -> Result<T> {
    let validation = Validation::new(Algorithm::ES256);
    let key = DecodingKey::from_ec_pem(public).map_err(|err| {
        error!(error = err.to_string(), "decoding elliptic curve keypair",);
        Error::Unknown
    })?;

    let token = jsonwebtoken::decode::<T>(token, &key, &validation).map_err(|err| {
        error!(error = err.to_string(), "checking token's signature",);
        Error::InvalidToken
    })?;

    Ok(token.claims)
}

/// Returns an url safe random string.
pub fn get_random_string(size: usize) -> String {
    let token: String = (0..size)
        .map(|_| {
            let mut rand = rand::thread_rng();
            let idx = rand.gen_range(0..SECURE_CHARSET.len());
            SECURE_CHARSET[idx] as char
        })
        .collect();

    token
}

/// Given an array of bytes to use as secret, generates a TOTP instance.
pub fn generate_totp(secret: &[u8]) -> Result<TOTP> {
    TOTPBuilder::new()
        .key(secret)
        .hash_function(Sha256)
        .finalize()
        .map_err(|err| {
            error!(
                error = err.to_string(),
                "genereting time-based one time password",
            );
            Error::Unknown
        })
}

/// Given an array of bytes to use as TOTP's secret and a candidate of pwd, returns true if, and only if, pwd
/// has the same value as the TOTP.  
pub fn verify_totp(secret: &[u8], pwd: &str) -> Result<bool> {
    let totp = generate_totp(secret)?;
    Ok(totp.is_valid(pwd))
}

/// Given a subject str and a sufix returns the sha256 digest of apending them both.
pub fn obfuscate(subject: &str, sufix: &str) -> String {
    let format_pwd = format!("{}{}", subject, sufix);
    return sha256::digest(format_pwd.as_bytes());
}

/// Given a RSA public key in PEM format returns the value of data encrypted by that key,
pub fn _encrypt(public: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let pkey = PKey::public_key_from_pem(public).map_err(|err| {
        error!(error = err.to_string(), "parsing public key from pem");
        Error::Unknown
    })?;

    let mut encrypter = Encrypter::new(&pkey).map_err(|err| {
        error!(error = err.to_string(), "building encrypter");
        Error::Unknown
    })?;

    encrypter.set_rsa_padding(Padding::PKCS1).map_err(|err| {
        error!(error = err.to_string(), "setting up rsa padding");
        Error::Unknown
    })?;

    let buffer_len = encrypter.encrypt_len(data).map_err(|err| {
        error!(error = err.to_string(), "computing encription length");
        Error::Unknown
    })?;

    let mut encrypted = vec![0; buffer_len];

    let encrypted_len = encrypter.encrypt(data, &mut encrypted).map_err(|err| {
        error!(error = err.to_string(), "encripting data");
        Error::Unknown
    })?;

    encrypted.truncate(encrypted_len);
    Ok(encrypted)
}

/// Given a RSA private key in PEM format returns the value of data decrypted by that key.
pub fn _decrypt(private: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    let key = PKey::private_key_from_pem(private).map_err(|err| {
        error!(error = err.to_string(), "parsing private key from pem");
        Error::Unknown
    })?;

    let mut decrypter = Decrypter::new(&key).map_err(|err| {
        error!(error = err.to_string(), "building decrypter");
        Error::Unknown
    })?;

    decrypter.set_rsa_padding(Padding::PKCS1).map_err(|err| {
        error!(error = err.to_string(), "setting up rsa padding");
        Error::Unknown
    })?;

    let buffer_len = decrypter.decrypt_len(data).map_err(|err| {
        error!(error = err.to_string(), "computing decryption length");
        Error::Unknown
    })?;

    let mut decrypted = vec![0; buffer_len];

    let decrypted_len = decrypter.decrypt(data, &mut decrypted).map_err(|err| {
        error!(error = err.to_string(), "decrypting data");
        Error::Unknown
    })?;

    decrypted.truncate(decrypted_len);
    Ok((*decrypted).to_vec())
}

#[cfg(test)]
pub mod tests {
    use super::{generate_totp, verify_totp};

    #[test_log::test]
    fn verify_totp_ok_should_not_fail() {
        const SECRET: &[u8] = "hello world".as_bytes();

        let code = generate_totp(SECRET).unwrap().generate();

        assert_eq!(code.len(), 6);
        assert!(verify_totp(SECRET, &code).is_ok());
    }

    #[test_log::test]
    fn verify_totp_ko_should_not_fail() {
        const SECRET: &[u8] = "hello world".as_bytes();
        assert!(!verify_totp(SECRET, "tester").unwrap());
    }
}
