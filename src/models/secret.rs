use std::error::Error;
use crate::schema::secrets;
use crate::postgres::*;
use crate::diesel::prelude::*;
use crate::regex::*;

use ring::{
    rand,
    signature::{self, KeyPair},
    pkcs8,
};

pub trait Ctrl {
    fn sign_data(&self, data: &str) -> signature::Signature;
    fn verify(&self, data: &str, sign: &str) -> Result<(), Box<dyn Error>>;
}

fn generate_random_document() -> Result<pkcs8::Document, Box<dyn Error>> {
    // Generate a key pair in PKCS#8 (v2) format.
    let rng = rand::SystemRandom::new();
    match signature::Ed25519KeyPair::generate_pkcs8(&rng) {
        Ok(document) => Ok(document),
        Err(err) => Err(err.to_string().into()),
    }
}

fn from_pkcs8(data: &[u8]) -> Result<signature::Ed25519KeyPair, Box<dyn Error>> {
    match signature::Ed25519KeyPair::from_pkcs8(data) {
        Ok(key) => Ok(key),
        Err(err) => Err(err.to_string().into()),
    }
}

#[derive(Insertable)]
#[derive(Queryable)]
#[derive(Clone)]
#[table_name="secrets"]
pub struct Secret {
    pub id: i32,
    pub document: String,
}

#[derive(Insertable)]
#[table_name="secrets"]
struct NewSecret<'a> {
    pub document: &'a str,
}

impl Secret {
    pub fn new<'a>(email: &'a str, pwd: &'a str) -> Result<Box<dyn Ctrl>, Box<dyn Error>> {
        match_email(email)?;
        match_pwd(pwd)?;

        let document = generate_random_document()?;
        match String::from_utf8(document.as_ref().to_vec()) {
            Ok(doc) => {
                let new_secret = NewSecret{
                    document: &doc,
                };

                let connection = open_stream();
                let result = diesel::insert_into(secrets::table)
                    .values(&new_secret)
                    .get_result::<Secret>(connection)?;

                let wrapper = result.build()?;
                Ok(Box::new(wrapper))
            }

            Err(err) => Err(err.to_string().into()),
        }
    }

    fn build(&self) -> Result<Wrapper, Box<dyn Error>> {
        Ok(Wrapper{
            key: from_pkcs8(self.document.as_bytes())?,
        })
    }
}

struct Wrapper {
    key: signature::Ed25519KeyPair,
}

impl Ctrl for Wrapper {
    fn sign_data(&self, data: &str) -> signature::Signature {
        self.key.sign(data.as_bytes())
    }

    fn verify(&self, data: &str, sign: &str) -> Result<(), Box<dyn Error>> {
        let public_key = self.key.public_key().as_ref();
        let peer = signature::UnparsedPublicKey::new(&signature::ED25519, public_key);
        if let Err(err) = peer.verify(data.as_bytes(), sign.as_bytes()) {
            return Err(err.to_string().into());
        }

        Ok(())
    }
}