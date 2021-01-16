pub mod provider;
mod descriptor;
mod token;

use std::error::Error;
use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use rand::prelude::ThreadRng;
use self::token::Token;
use self::descriptor::{Controller as DescriptorController};
use crate::proto::Status;
use crate::models::client::Controller as ClientController;

const TOKEN_LEN: usize = 8;

pub trait Controller {
    fn get_created_at(&self) -> SystemTime;
    fn get_touch_at(&self) -> SystemTime;
    fn get_deadline(&self) -> SystemTime;
    fn get_status(&self) -> Status;
    fn get_cookie(&self) -> &str;
    fn get_client(&self) -> &Box<dyn ClientController>;
    fn get_addr(&self) -> String;
    fn match_cookie(&self, cookie: String) -> bool;
    fn build_token(&mut self) -> Result<String, Box<dyn Error>>;
}

pub struct Session {
    pub cookie: String,
    pub created_at: SystemTime,
    pub touch_at: SystemTime,
    pub timeout: Duration,
    pub status: Status,
    rand_gen: ThreadRng,
    client: Box<dyn ClientController>,
    tokens: HashMap<String, Option<Box<dyn DescriptorController>>>,
}

impl Session {
    pub fn new(client: Box<dyn ClientController>, cookie: String, timeout: Duration) -> impl Controller {
        Session{
            cookie: cookie,
            created_at: SystemTime::now(),
            touch_at: SystemTime::now(),
            timeout: timeout,
            status: Status::New,
            rand_gen: rand::thread_rng(),
            client: client,
            tokens: HashMap::new(),
        }
    }
}

impl Controller for Session {
    fn get_addr(&self) -> String {
        self.client.get_addr()
    }

    fn get_created_at(&self) -> SystemTime {
        self.created_at
    }

    fn get_touch_at(&self) -> SystemTime {
        self.touch_at
    }

    fn get_deadline(&self) -> SystemTime {
        self.created_at + self.timeout
    }

    fn get_status(&self) -> Status {
        self.status
    }

    fn get_cookie(&self) -> &str {
        &self.cookie
    }

    fn get_client(&self) -> &Box<dyn ClientController> {
        &self.client
    }

    fn match_cookie(&self, cookie: String) -> bool {
        self.cookie == cookie
    }

    fn build_token(&mut self) -> Result<String, Box<dyn Error>> {
        //let deadline = SystemTime::now();
        let token = Token::new(&mut self.rand_gen, TOKEN_LEN);
        self.tokens.insert(token.to_string(), None);
        Ok(token.to_string())
    }
}