use std::error::Error;

use crate::regex;
use crate::secret::domain::Secret;
use crate::metadata::domain::Metadata;

pub trait UserRepository {
    fn find(&self, email: &str) -> Result<User, Box<dyn Error>>;
    fn save(&self, user: &mut User) -> Result<(), Box<dyn Error>>;
    fn delete(&self, user: &User) -> Result<(), Box<dyn Error>>;
}

pub struct User {
    pub id: i32,
    pub email: String, // hash of the email
    pub password: String,
    pub secret: Option<Secret>,
    pub meta: Metadata,
}

impl User {
    pub fn new<'a>(user_repo: &dyn UserRepository,
                   meta: Metadata,
                   email: &'a str,
                   password: &'a str) -> Result<Self, Box<dyn Error>> {
        
        regex::match_regex(regex::EMAIL, email)?;
        regex::match_regex(regex::BASE64, password)?;
        
        let mut user = User {
            id: 0,
            email: email.to_string(),
            password: password.to_string(),
            secret: None,
            meta: meta,
        };

        user_repo.save(&mut user)?;
        Ok(user)
    }

    pub fn match_password(&self, password: &str) -> Result<(), Box<dyn Error>> {
        if password != self.password {
            return Err("wrong password".into());
        }

        Ok(())
    }
}