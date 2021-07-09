use std::error::Error;

use crate::regex::*;
use crate::metadata::domain::Metadata;

pub trait UserRepository {
    fn find(&self, email: &str) -> Result<User, Box<dyn Error>>;
    fn save(&self, user: &mut User) -> Result<(), Box<dyn Error>>;
    fn delete(&self, user: &User) -> Result<(), Box<dyn Error>>;
}

pub struct User {
    pub id: i32,
    pub email: String,
    pub pwd: String,
    pub meta: Metadata,
}

impl User {
    pub fn new<'a>(id: i32, email: &'a str, pwd: &'a str, meta: Metadata) -> Result<Self, Box<dyn Error>> {
        match_email(email)?;
        match_pwd(pwd)?;

        let user = User {
            id: id,
            email: email.to_string(),
            pwd: pwd.to_string(),
            meta: meta,
        };

        Ok(user)
    }
}