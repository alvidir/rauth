use std::error::Error;
use std::time::{SystemTime, Duration};

use crate::regex;
use crate::secret::domain::Secret;
use crate::metadata::domain::Metadata;

pub trait UserRepository {
    fn find(&self, id: i32) -> Result<User, Box<dyn Error>>;
    fn find_by_email(&self, email: &str) -> Result<User, Box<dyn Error>>;
    fn create(&self, user: &mut User) -> Result<(), Box<dyn Error>>;
    fn save(&self, user: &User) -> Result<(), Box<dyn Error>>;
    fn delete(&self, user: &User) -> Result<(), Box<dyn Error>>;
}

pub struct User {
    pub(super) id: i32,
    pub(super) email: String, // hash of the email
    pub(super) password: String,
    pub(super) verified_at: Option<SystemTime>,
    pub(super) secret: Option<Secret>,
    pub(super) meta: Metadata,
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
            verified_at: None,
            secret: None,
            meta: meta,
        };

        super::get_repository().create(&mut user)?;
        Ok(user)
    }

    pub(super) fn verify(&mut self) -> Result<(), Box<dyn Error>> {
        if self.verified_at.is_some() {
            return Err("already verified".into());
        }

        self.verified_at = Some(SystemTime::now());
        self.meta.touch();
        Ok(())
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }

    pub fn get_secret(&self) -> &Option<Secret> {
        &self.secret
    }

    pub fn is_verified(&self) -> bool {
        self.verified_at.is_some()
    }

    pub fn match_password(&self, password: &str) -> bool {
        password == self.password
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        self.meta.save()?;
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
    pub(super) exp: SystemTime,     // expiration time (as UTC timestamp) - required
    pub(super) iat: SystemTime,     // issued at: creation time
    pub(super) iss: String,         // issuer
    pub(super) sub: i32,            // subject: the user id
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