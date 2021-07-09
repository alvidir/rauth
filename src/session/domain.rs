use std::error::Error;
use std::time::{Duration, SystemTime};
use rand::Rng;

use crate::metadata::domain::Metadata;
use crate::user::domain::User;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789)(*&^%$#@!~?][+-";


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