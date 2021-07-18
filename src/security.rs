use std::error::Error;
use std::env;
use openssl::sign::{Signer, Verifier};
use openssl::pkey::PKey;
use openssl::ec::EcKey;
use openssl::hash::MessageDigest;
use libreauth::oath::{TOTPBuilder};
use libreauth::hash::HashFunction::Sha256;
use rand::Rng;

use crate::constants;

pub fn generate_token(size: usize) -> String {
    let token: String = (0..size)
    .map(|_| {
        let mut rand = rand::thread_rng();
        let idx = rand.gen_range(0..constants::TOKEN_CHARSET.len());
        constants::TOKEN_CHARSET[idx] as char
    })
    .collect();

    token
}

pub fn verify_totp_password(secret: &[u8], pwd: &str) -> Result<(), Box<dyn Error>> {
    let totp = TOTPBuilder::new()
        .key(secret)
        //.output_len(6)
        .period(30)
        .hash_function(Sha256)
        .finalize();

    if let Ok(code) = totp {
        if !code.is_valid(pwd) {
            return Err("password not match".into());
        }
    } else {
        return Err("failed to generate code".into());
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
        Err("signature verification has failed".into())
    } else {
        Ok(())
    }
}

pub fn _apply_server_signature(data: &[&[u8]]) -> Result<Vec<u8>, Box<dyn Error>> {
    let secret = env::var(constants::ENV_SMTP_USERNAME)?;
    let eckey = EcKey::private_key_from_pem(secret.as_bytes())?;
    let keypair = PKey::from_ec_key(eckey)?;

    let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
    for item in data {
        signer.update(item)?;
    }

    let signature = signer.sign_to_vec()?;
    Ok(signature)
}