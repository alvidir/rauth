use serde::Serialize;
use serde::de::DeserializeOwned;
use std::error::Error;
use std::env;
use openssl::sign::{Signer, Verifier};
use openssl::pkey::{PKey};
use openssl::ec::{EcKey,EcGroup};
use openssl::nid::Nid;
use openssl::hash::MessageDigest;
use libreauth::oath::{TOTPBuilder};
use libreauth::hash::HashFunction::Sha256;
use jsonwebtoken::{Header, EncodingKey, DecodingKey, Validation};
use rand::Rng;

use crate::constants::{environment, errors};

lazy_static! {
    static ref PRIVATE_KEY: Vec<u8> = {
        if let Ok(pem) = env::var(environment::SECRET_PEM) {
            let private = match env::var(environment::SECRET_PWD) {
                Ok(password) => EcKey::private_key_from_pem_passphrase(pem.as_bytes(), password.as_bytes()).unwrap(),
                Err(_) => EcKey::private_key_from_pem(pem.as_bytes()).unwrap(),
            };

            info!("got a PEM-formatted private EcKey from environment configuration");
            return private.private_key_to_pem().unwrap();
        }

        warn!("no PEM-formatted private EcKey has been provided, generating one");
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
        let private = EcKey::generate(&group).unwrap();
        private.private_key_to_pem().unwrap()
    };

    pub static ref PUBLIC_KEY: Vec<u8> = {
        let mut ctx = openssl::bn::BigNumContext::new().unwrap();
        let pkey = EcKey::private_key_from_pem(&PRIVATE_KEY).unwrap();
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
        
        let public = pkey.public_key()
            .to_bytes(&group,
                openssl::ec::PointConversionForm::COMPRESSED,
                &mut ctx)
            .unwrap();

        info!("derived public EcKey: {}", base64::encode(&public));
        public
    };
}

const SECURE_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789";

pub fn encode_jwt(payload: impl Serialize) -> Result<String, Box<dyn Error>> {
    let key = EncodingKey::from_ec_pem(&PRIVATE_KEY)?;
    let token = jsonwebtoken::encode(&Header::default(), &payload, &key)?;
    Ok(token)
}

pub fn decode_jwt<T: DeserializeOwned>(token: &str) -> Result<T, Box<dyn Error>> {
    let key = DecodingKey::from_ec_pem(&PRIVATE_KEY)?;
    let validation = Validation::default();
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

pub fn _apply_server_signature(data: &[&[u8]]) -> Result<Vec<u8>, Box<dyn Error>> {
    let secret = env::var(environment::SMTP_USERNAME)?;
    let eckey = EcKey::private_key_from_pem(secret.as_bytes())?;
    let keypair = PKey::from_ec_key(eckey)?;

    let mut signer = Signer::new(MessageDigest::sha256(), &keypair).unwrap();
    for item in data {
        signer.update(item)?;
    }

    let signature = signer.sign_to_vec()?;
    Ok(signature)
}