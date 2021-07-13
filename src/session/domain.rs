use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use rand::Rng;

use crate::metadata::domain::Metadata;
use crate::user::domain::User;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789)(*&^%$#@!~?][+-";


pub trait SessionRepository {
    fn find(&self, cookie: &str) -> Result<Arc<Mutex<Session>>, Box<dyn Error>>;
    fn find_by_email(&self, email: &str) -> Result<Arc<Mutex<Session>>, Box<dyn Error>>;
    fn save(&self, session: Session) -> Result<(), Box<dyn Error>>;
    fn delete(&self, session: &Session) -> Result<(), Box<dyn Error>>;
}

pub struct Session {
    pub token: String,
    pub deadline: SystemTime,
    pub user: User,
    // the updated_at field from metadata works as a touch_at field, being updated for each
    // read/write action done by the user (owner) over the sessions data
    pub meta: Metadata,
}

impl Session {
    pub fn new(user: User, token: String, timeout: Duration) -> Self {
        Session{
            token: token,
            deadline: SystemTime::now() + timeout,
            user: user,
            meta: Metadata::new(),
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