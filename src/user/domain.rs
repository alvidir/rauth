use std::error::Error;
use std::time::{SystemTime, Duration};

use crate::regex;
use crate::secret::domain::Secret;
use crate::metadata::domain::Metadata;

pub trait UserRepository {
    fn find(&self, email: &str) -> Result<User, Box<dyn Error>>;
    fn create(&self, user: &mut User) -> Result<(), Box<dyn Error>>;
    fn save(&self, user: &User) -> Result<(), Box<dyn Error>>;
    fn delete(&self, user: &User) -> Result<(), Box<dyn Error>>;
}

pub struct User {
    pub id: i32,
    pub email: String, // hash of the email
    pub password: String,
    pub verified: bool,
    pub secret: Option<Secret>,
    pub meta: Metadata,
}

impl User {
    pub fn new(meta: Metadata,
               email: &str,
               password: &str) -> Result<Self, Box<dyn Error>> {
        
        regex::match_regex(regex::EMAIL, email)?;
        regex::match_regex(regex::BASE64, password)?;
        
        let mut user = User {
            id: 0,
            email: email.to_string(),
            password: password.to_string(),
            verified: false,
            secret: None,
            meta: meta,
        };

        super::get_repository().create(&mut user)?;
        Ok(user)
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn is_verified(&self) -> bool {
        self.verified
    }

    pub fn match_password(&self, password: &str) -> bool {
        password != self.password
    }

    pub fn _save(&self) -> Result<(), Box<dyn Error>> {
        self.meta._save()?;
        super::get_repository().save(self)?;
        Ok(())
    }

    pub fn delete(&self) -> Result<(), Box<dyn Error>> {
        if let Some(secret) = &self.secret {
            secret.delete()?;
        }

        super::get_repository().delete(self)?;
        self.meta.delete()?;
        Ok(())
    }
}

// token for email-verification
#[derive(Serialize, Deserialize)]
pub struct Token {
    pub exp: SystemTime,     // expiration time (as UTC timestamp) - required
    pub iat: SystemTime,     // issued at: creation time
    pub iss: String,         // issuer
    pub sub: i32,            // subject: the user id
}

impl Token {
    pub fn new(user: &User, timeout: Duration) -> Self {
        Token {
            exp: SystemTime::now() + timeout,
            iat: SystemTime::now(),
            iss: "oauth.alvidir.com".to_string(),
            sub: user.id,
        }
    }
}