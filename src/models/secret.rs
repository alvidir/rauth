use std::error::Error;
use diesel::NotFound;
use std::time::{Duration, SystemTime};
use rand::rngs::OsRng;
use rand::thread_rng;
use pwbox::{Eraser, ErasedPwBox, Suite, sodium::Sodium};
use ed25519_dalek::{
    Keypair,
    Signature,
    Signer,
    PublicKey,
    SecretKey,
    Verifier,
    KEYPAIR_LENGTH,
    SECRET_KEY_LENGTH,
    PUBLIC_KEY_LENGTH,
};

use crate::schema::secrets;
use crate::postgres::*;
use crate::diesel::prelude::*;
use crate::regex::*;

pub trait Ctrl {
    fn encrypt(&self, data: &[u8]) -> Result<&[u8], Box<dyn Error>>;
    fn decrypt(&self, data: &[u8]) -> Result<&[u8], Box<dyn Error>>;
    fn sign(&self, data: &[u8]) -> Result<&[u8], Box<dyn Error>>;
    fn verify(&self, data: &[u8]) -> bool;
}

pub fn find_by_client_id(target: i32) -> Result<Box<dyn Ctrl>, Box<dyn Error>>  {
    use crate::schema::secrets::dsl::*;

    let connection = open_stream();
    let results = secrets.filter(client_id.eq(target))
        .load::<Secret>(connection)?;

    if results.len() > 0 {
        Ok(Box::new(results[0].clone()))
    } else {
        Err(Box::new(NotFound))
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
    pub secret: Option<String>,
    pub public: String,
    pub created_at: SystemTime,
    pub deadline: Option<SystemTime>,
}

#[derive(Insertable)]
#[table_name="secrets"]
struct NewSecret<'a> {
    pub client_id: i32,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub secret: Option<&'a str>,
    pub public: &'a str,
    pub created_at: SystemTime,
    pub deadline: Option<SystemTime>,
}

impl Secret {
    pub fn new<'a>(email: &'a str, pwd: &'a str) -> Result<Self, Box<dyn Error>> {
        match_email(email)?;
        match_pwd(pwd)?;

        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);
        keypair.to_bytes();
        let secret_key: SecretKey = keypair.secret;
        
        Err("".into())
    }

    pub fn store<'a>(client_id: i32, name: &'a str, public: &'a [u8]) -> Result<Self, Box<dyn Error>> {
        if public.len() != PUBLIC_KEY_LENGTH {
            let msg = format!("Got public key of length {}, want {}", public.len(), PUBLIC_KEY_LENGTH);
            return Err(msg.into());
        }

        let new_secret = NewSecret {
            client_id: client_id,
            name: name,
            description: None,
            secret: None,
            public: &String::from_utf8(public.to_vec())?,
            created_at: SystemTime::now(),
            deadline: None,
        };

        let connection = open_stream();
        let result = diesel::insert_into(secrets::table)
            .values(&new_secret)
            .get_result::<Secret>(connection)?;

        Ok(result)
    }

    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }

    pub fn set_deadline(&mut self, deadline: Option<SystemTime>) {
        self.deadline = deadline;
    }
}

impl Ctrl for Secret {
    fn encrypt(&self, data: &[u8]) -> Result<&[u8], Box<dyn Error>> {
        Err("".into())
    }

    fn decrypt(&self, data: &[u8]) -> Result<&[u8], Box<dyn Error>> {
        Err("".into())
    }

    fn sign(&self, data: &[u8]) -> Result<&[u8], Box<dyn Error>> {
        Err("".into())
    }

    fn verify(&self, data: &[u8]) -> bool {
        false
    }

}