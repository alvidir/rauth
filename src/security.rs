use serde::Serialize;
use serde::de::DeserializeOwned;
use std::error::Error;
use std::env;
use openssl::sign::{Verifier, Signer};
use openssl::pkey::{PKey};
use openssl::ec::EcKey;
use openssl::hash::MessageDigest;
use libreauth::oath::TOTPBuilder;
use libreauth::hash::HashFunction::Sha256;
use jsonwebtoken::{Header, EncodingKey, DecodingKey, Validation, Algorithm};
use rand::Rng;
use base64;
use sha256;

use crate::constants::{environment, errors};

lazy_static! {
    static ref JWT_SECRET: EncodingKey = {
        let pem_b64 = env::var(environment::JWT_SECRET).unwrap();
        let pem = base64::decode(pem_b64).unwrap();
        EncodingKey::from_ec_pem(&pem).unwrap()
    };

    static ref JWT_PUBLIC: Vec<u8> = {
        let pem_b64 = env::var(environment::JWT_PUBLIC).unwrap();
        base64::decode(pem_b64).unwrap()
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
    let key = DecodingKey::from_ec_pem(&JWT_PUBLIC)?;
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
    let public = base64::decode(pem)?;
    let eckey = EcKey::public_key_from_pem(&public)?;
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

pub fn _get_ec_signature(pem: &[u8], data: &[&[u8]]) -> Result<Vec<u8>, Box<dyn Error>> {
    let private = base64::decode(pem)?;
    let eckey = EcKey::private_key_from_pem(&private)?;
    let keypair = PKey::from_ec_key(eckey)?;

    let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
    for item in data {
        signer.update(item)?;
    }

    let signature = signer.sign_to_vec()?;
    Ok(signature)
}

pub fn format_password(password: &str) -> String {
    if let Ok(prefix) = env::var(environment::PWD_SUFIX) {
        let format_pwd = format!("{}{}", password, prefix);
        return sha256::digest_bytes(format_pwd.as_bytes());
    }
    
    warn!("password sufix should be set");
    password.to_string()
}

#[cfg(test)]
pub mod tests {
    use libreauth::oath::TOTPBuilder;
    use libreauth::hash::HashFunction::Sha256;
    use super::{_get_ec_signature, verify_ec_signature, verify_totp};

    const EC_SECRET: &[u8] = b"LS0tLS1CRUdJTiBFQyBQUklWQVRFIEtFWS0tLS0tCk1IY0NBUUVFSUlPejlFem04Ri9oSnluNTBrM3BVcW5Dc08wRVdGSjAxbmJjWFE1MFpyV0pvQW9HQ0NxR1NNNDkKQXdFSG9VUURRZ0FFNmlIZUZrSHRBajd1TENZOUlTdGk1TUZoaTkvaDYrbkVLbzFUOWdlcHd0UFR3MnpYNTRabgpkZTZ0NnJlM3VxUjAvcWhXcGF5TVhxb25HSEltTmsyZ3dRPT0KLS0tLS1FTkQgRUMgUFJJVkFURSBLRVktLS0tLQo";
    const EC_PUBLIC: &[u8] = b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFNmlIZUZrSHRBajd1TENZOUlTdGk1TUZoaTkvaAo2K25FS28xVDlnZXB3dFBUdzJ6WDU0Wm5kZTZ0NnJlM3VxUjAvcWhXcGF5TVhxb25HSEltTmsyZ3dRPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg";

    #[test]
    fn ec_signature_ok() {
        let mut data: Vec<&[u8]> = Vec::new();
        data.push("hello world".as_bytes());

        let sign = _get_ec_signature(EC_SECRET, &data).unwrap();
        verify_ec_signature(EC_PUBLIC, &sign, &data).unwrap();
    }

    #[test]
    fn ec_signature_ko() {
        let mut data: Vec<&[u8]> = Vec::new();
        data.push("hello world".as_bytes());

        let fake_sign = "ABCDEF1234567890".as_bytes();
        assert!(verify_ec_signature(EC_SECRET, &fake_sign, &data).is_err());
    }

    #[test]
    fn verify_totp_ok() {
        const SECRET: &[u8] = "hello world".as_bytes();

        let code = TOTPBuilder::new()
            .key(SECRET)
            .period(30)
            .hash_function(Sha256)
            .finalize()
            .unwrap()
            .generate();

        assert_eq!(code.len(), 6);
        assert!(verify_totp(&SECRET, &code).is_ok());
    }

    #[test]
    fn verify_totp_ko() {
        const SECRET: &[u8] = "hello world".as_bytes();
        assert!(verify_totp(&SECRET, "tester").is_err());
    }
}