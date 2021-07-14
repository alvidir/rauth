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
    pub secret: Option<Secret>,
    pub meta: Metadata,
}

impl User {
    pub fn new<'a>(repo: impl UserRepository, email: &'a str) -> Result<Self, Box<dyn Error>> {
        regex::match_regex(regex::EMAIL, email)?;

        let mut user = User {
            id: 0,
            email: email.to_string(),
            secret: None,
            meta: Metadata::new(),
        };

        repo.save(&mut user)?;
        Ok(user)
    }
}