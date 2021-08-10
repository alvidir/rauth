use serde::Serialize;
use serde::de::DeserializeOwned;
use std::error::Error;
use std::env;
use openssl::sign::Verifier;
use openssl::pkey::{PKey};
use openssl::ec::EcKey;
use openssl::hash::MessageDigest;
use libreauth::oath::{TOTPBuilder};
use libreauth::hash::HashFunction::Sha256;
use jsonwebtoken::{Header, EncodingKey, DecodingKey, Validation, Algorithm};
use rand::Rng;

use crate::constants::{environment, errors};

lazy_static! {
    static ref JWT_SECRET: EncodingKey = {
        let pem = env::var(environment::SECRET_PEM).unwrap();
        EncodingKey::from_ec_pem(pem.as_bytes()).unwrap()
    };

    static ref PUBLIC_KEY: Vec<u8> = {
        let pem = env::var(environment::PUBLIC_PEM).unwrap();
        pem.as_bytes().to_vec()
    };
}

const SECURE_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789";

pub fn encode_jwt(payload: impl Serialize) -> Result<String, Box<dyn Error>> {
    let header = Header::new(Algorithm::ES256);
    let token = jsonwebtoken::encode(&header, &payload, &JWT_SECRET)?;
    Ok(token)
}

pub fn decode_jwt<T: DeserializeOwned>(token: &str) -> Result<T, Box<dyn Error>> {
    let key = DecodingKey::from_ec_pem(&PUBLIC_KEY)?;
    let validation = Validation::new(Algorithm::ES256);
    let token = jsonwebtoken::decode::<T>(token, &key, &validation)?;
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

pub fn verify_totp(secret: &[u8], pwd: &str) -> Result<(), Box<dyn Error>> {
    let totp_result = TOTPBuilder::new()
        .key(secret)
        //.output_len(6)
        .period(30)
        .hash_function(Sha256)
        .finalize();

    if let Err(err) = totp_result {
        let msg = format!("{:?}", err);
        return Err(msg.into());
    }


    let totp = totp_result.unwrap(); // this line will not fail due to the previous check of err
    if !totp.is_valid(pwd) {
        return Err(errors::UNAUTHORIZED.into());
    }
    Ok(())
}

pub fn verify_ec_signature(pem: &[u8], signature: &[u8], data: &[&[u8]]) -> Result<(), Box<dyn Error>> {
    let eckey = EcKey::public_key_from_pem(pem)?;
    let keypair = PKey::from_ec_key(eckey)?;

    let mut verifier = Verifier::new(MessageDigest::sha256(), &keypair)?;
    for item in data {
        verifier.update(item)?;
    }
    
    if !verifier.verify(&signature)? {
        Err(errors::UNAUTHORIZED.into())
    } else {
        Ok(())
    }
}