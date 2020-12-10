use std::collections::HashMap;
use crate::model::client::Controller as ClientController;
use std::time::{Duration, Instant};
use rand::Rng;
use rand::prelude::ThreadRng;

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789)(*&^%$#@!~";

const COOKIE_LEN: usize = 32;

const errSessionAlreadyExists: &str = "A session already exists for client {}";
const errBrokenCookie: &str = "Cookie {} has no session associated";

static mut all_instances: Option<HashMap<String, Session>> = None;
static mut index_by_email: Option<HashMap<String, String>> = None;

pub enum Status {
    ALIVE,
    DEAD,
    NEW
}

pub trait Controller {
    fn get_created_at(&self) -> Instant;
    fn get_touch_at(&self) -> Instant;
    fn get_deadline(&self) -> Instant;
    fn get_status(&self) -> &Status;
    fn get_cookie(&self) -> &str;
    fn match_cookie(&self, cookie: String) -> bool;
    fn get_addr(&self) -> String;
}

pub struct Session {
    pub cookie: String,
    pub created_at: Instant,
    pub touch_at: Instant,
    pub timeout: Duration,
    pub status: Status,
    client: Box<dyn ClientController>,
}

impl Session {
    pub fn new(client: Box<dyn ClientController>, cookie: String, timeout: Duration) -> Self {
        Session{
            cookie: cookie.clone(),
            created_at: Instant::now(),
            touch_at: Instant::now(),
            timeout: timeout,
            status: Status::NEW,
            client: client,
        }
    }

    fn cookie_gen() -> String {
        let mut rand_gen = rand::thread_rng();
        let cookie: String = (0..COOKIE_LEN)
            .map(|_| {
                let idx = rand_gen.gen_range(0, CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        
        cookie
    }
}

impl Controller for Session {
    fn get_addr(&self) -> String {
        self.client.get_addr()
    }

    fn get_created_at(&self) -> Instant {
        self.created_at
    }

    fn get_touch_at(&self) -> Instant {
        self.touch_at
    }

    fn get_deadline(&self) -> Instant {
        self.created_at + self.timeout
    }

    fn get_status(&self) -> &Status {
        &self.status
    }

    fn get_cookie(&self) -> &str {
        &self.cookie
    }

    fn match_cookie(&self, cookie: String) -> bool {
        self.cookie == cookie
    }
}