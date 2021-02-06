use std::error::Error;
use diesel::NotFound;
use std::time::SystemTime;
use openssl::base64;
use openssl::sign::{Signer, Verifier};
use openssl::ec::{EcKey,EcGroup, EcPoint};
use openssl::nid::Nid;
use openssl::symm::Cipher;
use openssl::pkey::PKey;
use openssl::hash::MessageDigest;

use crate::schema::secrets;
use crate::postgres::*;
use crate::diesel::prelude::*;
use crate::regex::*;

static DEFAULT_NID: Nid = Nid::X9_62_PRIME256V1;

const ERR_NAME_FORMAT: &str = "The secret's name mismatches the expected format";
const ERR_SIGN_DATA: &str = "The secret is not valid for signing any data";
const ERR_VERIFY: &str = "Verification has failed";

pub trait Ctrl {
    fn sign(&self, pwd: &str, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
    fn verify(&self, data: &[u8], signature: &[u8], pwd: &str) -> Result<(), Box<dyn Error>>;
}

//pub fn find_all_by_client(target_id: i32) -> Result<Vec<impl Ctrl>, Box<dyn Error>>  {
//    use crate::schema::secrets::dsl::*;
//
//    let connection = open_stream();
//    let results = secrets.filter(client_id.eq(target_id))
//        .load::<Secret>(connection)?;
//
//    if results.len() > 0 {
//        Ok(results)
//    } else {
//        Err(Box::new(NotFound))
//    }
//}

pub fn find_by_client_and_name(target_id: i32, target_name: &str) -> Result<Box<dyn Ctrl>, Box<dyn Error>>  {
    use crate::schema::secrets::dsl::*;

    let connection = open_stream();
    let results = secrets.filter(client_id.eq(target_id))
        .filter(name.eq(target_name))
        .load::<Secret>(connection)?;

    if results.len() > 0 {
        Ok(Box::new(results[0].clone()))
    } else {
        Err(Box::new(NotFound))
    }
}

custom_derive! {
    #[derive(EnumFromStr)]
    #[derive(Eq, PartialEq, Copy, Clone)]
    #[derive(Debug)]
    enum Format {
        PEM,
        PUB,
    }
}

impl Format {
    pub fn derive(name: &str) -> Result<Format, Box<dyn Error>> {
        let upper = name.to_uppercase();
        let form: Format = upper.parse()?;
        Ok(form)
    }
}

#[derive(Insertable)]
#[derive(Queryable)]
#[derive(Clone)]
#[table_name="secrets"]
pub struct Secret {
    pub id: i32,
    pub client_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub document: String,
    pub created_at: SystemTime,
    pub deadline: Option<SystemTime>,
}

#[derive(Insertable)]
#[table_name="secrets"]
struct NewSecret<'a> {
    pub client_id: i32,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub document: &'a str,
    pub created_at: SystemTime,
    pub deadline: Option<SystemTime>,
}

impl Secret {
    pub fn new<'a>(client_id: i32, secret_name: &'a str, pwd: &'a str) -> Result<Self, Box<dyn Error>> {
        match_pwd(pwd)?;

        let group = EcGroup::from_curve_name(DEFAULT_NID)?;
        let key = EcKey::generate(&group)?;
        let pem = key.private_key_to_pem_passphrase(Cipher::aes_128_cbc(), pwd.as_bytes())?;

        let new_secret = NewSecret {
            client_id: client_id,
            name: secret_name,
            description: None,
            document: &base64::encode_block(&pem),
            created_at: SystemTime::now(),
            deadline: None,
        };

        let connection = open_stream();
        let result = diesel::insert_into(secrets::table)
            .values(&new_secret)
            .get_result::<Secret>(connection)?;

        Ok(result)
    }

    fn as_format(&self) -> Result<Format, Box<dyn Error>> {
        if let Some(pos) = self.name.rfind('.') {
            let substr = &self.name[pos..];
            if substr.len() > 1 {
                return Format::derive(&substr[1..])
            }
        }
        
        Err(ERR_NAME_FORMAT.into())
    }
}

impl Ctrl for Secret {
    fn sign(&self, pwd: &str, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if let Format::PUB = self.as_format()? {
            return Err(ERR_SIGN_DATA.into());
        }

        let doc = &base64::decode_block(&self.document)?;
        let key = EcKey::private_key_from_pem_passphrase(doc, pwd.as_bytes())?;
        let keypair = PKey::from_ec_key(key)?;
        let mut signer = Signer::new(MessageDigest::sha256(), &keypair)?;
        signer.update(data)?;
        
        let signature = signer.sign_to_vec()?;
        Ok(signature)
    }

    fn verify(&self, signature: &[u8], data: &[u8], pwd: &str) -> Result<(), Box<dyn Error>> {
        match self.as_format()? {
            Format::PEM => {
                let doc = &base64::decode_block(&self.document)?;
                let key = EcKey::private_key_from_pem_passphrase(doc, pwd.as_bytes())?;
                let keypair = PKey::from_ec_key(key)?;
                let mut verifier = Verifier::new(MessageDigest::sha256(), &keypair)?;
                if !verifier.verify_oneshot(&signature, data)? {
                    return Err(ERR_VERIFY.into());
                }

                Ok(())
            },

            Format::PUB => {
                let mut ctx = openssl::bn::BigNumContext::new()?;
                let doc = &base64::decode_block(&self.document)?;
                let group = EcGroup::from_curve_name(DEFAULT_NID)?;
                let point = EcPoint::from_bytes(&group, doc, &mut ctx)?;
                let key = EcKey::from_public_key(&group, &point)?;
                let keypair = PKey::from_ec_key(key)?;
                let mut verifier = Verifier::new(MessageDigest::sha256(), &keypair)?;
                if !verifier.verify_oneshot(&signature, data)? {
                    return Err(ERR_VERIFY.into());
                }

                Ok(())
            }
        }
    }
}