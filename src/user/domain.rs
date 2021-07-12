use std::error::Error;

use crate::regex;
use crate::secret::domain::Secret;
use crate::metadata::domain::Metadata;

pub trait UserRepository {
    fn find(email: &str) -> Result<User, Box<dyn Error>>;
    fn save(user: &mut User) -> Result<(), Box<dyn Error>>;
    fn delete(user: &User) -> Result<(), Box<dyn Error>>;
}

pub struct User {
    pub id: i32,
    pub email: String, // hash of the email
    pub secret: Option<Secret>,
    pub meta: Metadata,
}

impl User {
    pub fn new<'a>(email: &'a str) -> Result<Self, Box<dyn Error>> {
        regex::match_regex(regex::EMAIL, email)?;

        let user = User {
            id: 0,
            email: email.to_string(),
            secret: None,
            meta: Metadata::new(),
        };

        Ok(user)
    }
}