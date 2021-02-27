use openssl::rsa::{Rsa, Padding};
use openssl::pkey::Public;
use std::error::Error;
use diesel::NotFound;
use std::time::SystemTime;
use openssl::sign::Verifier;
use openssl::encrypt::Encrypter;
use openssl::pkey::PKey;
use crate::schema::secrets;
use crate::postgres::*;
use crate::diesel::prelude::*;
use crate::regex::*;

pub trait Ctrl {
    fn get_client_id(&self) -> i32;
    fn get_encrypter(&self) -> Result<Encrypter, Box<dyn Error>>;
    fn get_verifier(&self) -> Result<Verifier, Box<dyn Error>>;
}

pub fn _find_all_by_client(target_id: i32) -> Result<Vec<impl Ctrl + super::Gateway>, Box<dyn Error>>  {
    use crate::schema::secrets::dsl::*;
    let results = { // block is required because of connection release
        let connection = open_stream().get()?;
        secrets.filter(client_id.eq(target_id))
            .load::<Secret>(&connection)?
    };

    if results.len() > 0 {
        let mut wrappers = Vec::with_capacity(results.len());
        for (i, secret) in results.iter().enumerate() {
            wrappers[i] = secret.build()?;
        }

        Ok(wrappers)

    } else {
        Err(Box::new(NotFound))
    }
}

pub fn find_by_client_and_name(target_id: i32, target_name: &str) -> Result<Box<impl Ctrl + super::Gateway>, Box<dyn Error>>  {
    use crate::schema::secrets::dsl::*;

    let results = { // block is required because of connection release
        let connection = open_stream().get()?;
        secrets.filter(client_id.eq(target_id))
        .filter(name.eq(target_name))
        .load::<Secret>(&connection)?
    };

    if results.len() > 0 {
        let wrapper = results[0].build()?;
        Ok(Box::new(wrapper))
    } else {
        Err(Box::new(NotFound))
    }
}

#[derive(Insertable)]
#[derive(Identifiable)]
#[derive(Queryable)]
#[derive(Clone)]
#[table_name="secrets"]
pub struct Secret {
    pub id: i32,
    pub client_id: i32,
    pub name: String,
    pub document: String,
    pub created_at: SystemTime,
    pub deadline: Option<SystemTime>,
}

#[derive(Insertable)]
#[table_name="secrets"]
struct NewSecret<'a> {
    pub client_id: i32,
    pub name: &'a str,
    pub document: &'a str,
    pub deadline: Option<SystemTime>,
}

impl Secret {
    pub fn new<'a>(client_id: i32, name: &'a str, pem: &[u8]) -> Result<Box<impl Ctrl + super::Gateway>, Box<dyn Error>> {
        match_name(name)?;

        // make sure the data is a public key
        let public = Rsa::public_key_from_pem(pem)?;
        let public = public.public_key_to_pem()?;
        let public = String::from_utf8(public)?;
        
        let secret = Secret {
            id: 0,
            client_id: client_id,
            name: name.to_string(),
            document: public,
            created_at: SystemTime::now(),
            deadline: None,
        };
    
        let wrapper = secret.build()?;
        Ok(Box::new(wrapper))
    }

    fn build(&self) -> Result<Wrapper, Box<dyn Error>> {
        let public = Rsa::public_key_from_pem(self.document.as_bytes())?;
        let public = PKey::from_rsa(public)?;

        Ok(Wrapper{
            secret: self.clone(),
            pkey: public,
        })
    }
}

pub struct Wrapper {
    secret: Secret,
    pkey: PKey<Public>,
}

impl Ctrl for Wrapper {
    fn get_client_id(&self) -> i32 {
        self.secret.client_id
    }

    fn get_encrypter(&self) -> Result<Encrypter, Box<dyn Error>> {
        let mut encrypter = Encrypter::new(&self.pkey)?;
        encrypter.set_rsa_padding(Padding::PKCS1)?;
        Ok(encrypter)
    }

    fn get_verifier(&self) -> Result<Verifier, Box<dyn Error>> {
        let ver = Verifier::new_without_digest(&self.pkey)?;
        Ok(ver)
    }
}

impl super::Gateway for Wrapper {
    fn select(&mut self) -> Result<(), Box<dyn Error>> {
        Err("".into())
    }
    
    fn insert(&mut self) -> Result<(), Box<dyn Error>> {
        let new_secret = NewSecret {
            client_id: self.secret.client_id,
            name: &self.secret.name,
            document: &self.secret.document,
            deadline: self.secret.deadline,
        };

        let result = { // block is required because of connection release
            let connection = open_stream().get()?;
            diesel::insert_into(secrets::table)
            .values(&new_secret)
            .get_result::<Secret>(&connection)?
        };

        self.secret.id = result.id;
        self.secret.created_at = result.created_at;
        Ok(())
    }
    
    fn update(&mut self) -> Result<(), Box<dyn Error>> {
        { // block is required because of connection release
            let connection = open_stream().get()?;
            diesel::update(&self.secret)
            .set(secrets::deadline.eq(self.secret.deadline))
            .execute(&connection)?;
        }

        Ok(())
    }
    
    fn delete(&self) -> Result<(), Box<dyn Error>> {
        use crate::schema::secrets::dsl::*;

        { // block is required because of connection release
            let connection = open_stream().get()?;
            diesel::delete(
                secrets.filter(
                    id.eq(self.secret.id)
                )
            ).execute(&connection)?;
        }

        Ok(())
    }
}