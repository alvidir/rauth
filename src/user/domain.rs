use std::error::Error;
use std::time::{Duration, SystemTime};
use rand::Rng;

use crate::regex::*;
use crate::meta::domain::Metadata;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789)(*&^%$#@!~?][+-";

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

pub trait SessionRepository {
    fn find(&self, cookie: &str) -> Result<Session, Box<dyn Error>>;
    fn save(&self, session: &mut Session) -> Result<(), Box<dyn Error>>;
    fn delete(&self, session: &Session) -> Result<(), Box<dyn Error>>;
}


pub struct Session {
    cookie: String,
    deadline: SystemTime,
    user: User,
    meta: Metadata,
}

impl Session {
    pub fn new(user: User, cookie: String, timeout: Duration, meta: Metadata) -> Self {
        Session{
            cookie: cookie,
            deadline: SystemTime::now() + timeout,
            user: user,
            meta: meta,
        }
    }

    pub fn generate_token(size: usize) -> String {
        let token: String = (0..size)
        .map(|_| {
            let mut rand = rand::thread_rng();
            let idx = rand.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    
        token
    }
}