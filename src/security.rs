use std::error::Error;

use serde::{de::DeserializeOwned, Serialize};

use openssl::{
    encrypt::{Decrypter, Encrypter},
    pkey::PKey,
    rsa::Padding,
};

use libreauth::{
    hash::HashFunction::Sha256,
    oath::{TOTPBuilder, TOTP},
};

use crate::errors;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::prelude::*;

const SECURE_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789";

pub fn sign_jwt<S: Serialize>(secret: &[u8], payload: S) -> Result<String, Box<dyn Error>> {
    let header = Header::new(Algorithm::ES256);
    let key = EncodingKey::from_ec_pem(secret).map_err(|err| {
        error!(
            "{} encoding elliptic curve keypair: {}",
            errors::ERR_UNKNOWN,
            err
        );
        errors::ERR_UNKNOWN
    })?;

    let token = jsonwebtoken::encode(&header, &payload, &key).map_err(|err| {
        error!("{} signing json web token: {}", errors::ERR_UNKNOWN, err);
        errors::ERR_UNKNOWN
    })?;

    Ok(token)
}

pub fn verify_jwt<T: DeserializeOwned>(public: &[u8], token: &str) -> Result<T, Box<dyn Error>> {
    let validation = Validation::new(Algorithm::ES256);
    let key = DecodingKey::from_ec_pem(public).map_err(|err| {
        error!(
            "{} decoding elliptic curve keypair: {}",
            errors::ERR_UNKNOWN,
            err
        );
        errors::ERR_UNKNOWN
    })?;

    let token = jsonwebtoken::decode::<T>(token, &key, &validation).map_err(|err| {
        error!(
            "{} checking token's signature: {}",
            errors::ERR_INVALID_TOKEN,
            err
        );
        errors::ERR_INVALID_TOKEN
    })?;

    Ok(token.claims)
}

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

pub fn generate_totp(secret: &[u8]) -> Result<TOTP, Box<dyn Error>> {
    TOTPBuilder::new()
        .key(secret)
        .hash_function(Sha256)
        .finalize()
        .map_err(|err| {
            error!(
                "{} genereting time-based one time password: {:?}",
                errors::ERR_UNKNOWN,
                err
            );
            errors::ERR_UNKNOWN.into()
        })
}

pub fn verify_totp(secret: &[u8], pwd: &str) -> Result<bool, Box<dyn Error>> {
    let totp = generate_totp(secret)?;
    Ok(totp.is_valid(pwd))
}

pub fn shadow(subject: &str, sufix: &str) -> String {
    let format_pwd = format!("{}{}", subject, sufix);
    return sha256::digest_bytes(format_pwd.as_bytes());
}

pub fn _encrypt(public: &[u8], data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let pkey = PKey::public_key_from_pem(public)?;
    let mut encrypter = Encrypter::new(&pkey)?;
    encrypter.set_rsa_padding(Padding::PKCS1)?;

    let buffer_len = encrypter.encrypt_len(data)?;
    let mut encrypted = vec![0; buffer_len];

    let encrypted_len = encrypter.encrypt(data, &mut encrypted)?;
    encrypted.truncate(encrypted_len);
    Ok(encrypted)
}

pub fn _decrypt(private: &[u8], data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let key = PKey::private_key_from_pem(private)?;
    let mut decrypter = Decrypter::new(&key)?;
    decrypter.set_rsa_padding(Padding::PKCS1)?;

    let buffer_len = decrypter.decrypt_len(data)?;
    let mut decrypted = vec![0; buffer_len];

    let decrypted_len = decrypter.decrypt(data, &mut decrypted)?;
    decrypted.truncate(decrypted_len);
    Ok((*decrypted).to_vec())
}

#[cfg(test)]
pub mod tests {
    use super::{generate_totp, verify_totp};

    #[test]
    fn verify_totp_ok_should_not_fail() {
        const SECRET: &[u8] = "hello world".as_bytes();

        let code = generate_totp(SECRET).unwrap().generate();

        assert_eq!(code.len(), 6);
        assert!(verify_totp(SECRET, &code).is_ok());
    }

    #[test]
    fn verify_totp_ko_should_not_fail() {
        const SECRET: &[u8] = "hello world".as_bytes();
        assert!(!verify_totp(SECRET, "tester").unwrap());
    }
}
