use serde::Serialize;
use std::error::Error;
use std::env;
use openssl::sign::{Signer, Verifier};
use openssl::pkey::PKey;
use openssl::ec::{EcKey,EcGroup};
use openssl::nid::Nid;
use openssl::symm::Cipher;
use openssl::hash::MessageDigest;
use libreauth::oath::{TOTPBuilder};
use libreauth::hash::HashFunction::Sha256;
use jsonwebtoken::{Header, EncodingKey};
use rand::Rng;

use crate::constants;

lazy_static! {
    static ref PRIVATE_KEY: Vec<u8> = {
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
        let key = EcKey::generate(&group).unwrap();
        
        if let Ok(password) = env::var(constants::ENV_SECRET_PWD) {
            key.private_key_to_pem_passphrase(Cipher::aes_128_cbc(), password.as_bytes()).unwrap()
        } else {
            key.private_key_to_pem().unwrap()
        }
    };

    pub static ref PUBLIC_KEY: Vec<u8> = {
        let mut ctx = openssl::bn::BigNumContext::new().unwrap();
        let pkey = EcKey::private_key_from_pem(&PRIVATE_KEY).unwrap();
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
        
        pkey.public_key().to_bytes(&group,
            openssl::ec::PointConversionForm::COMPRESSED, &mut ctx).unwrap()
    };
}

const SECURE_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789";

pub fn generate_jwt(payload: impl Serialize) -> Result<String, Box<dyn Error>> {
    let key = EncodingKey::from_ec_pem(&PRIVATE_KEY)?;
    let token = jsonwebtoken::encode(&Header::default(), &payload, &key)?;
    Ok(token)
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