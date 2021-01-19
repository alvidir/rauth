pub mod provider;
mod gateway;
mod token;

use std::error::Error;
use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use std::collections::HashSet;
use rand::prelude::ThreadRng;
use self::token::Token;
use self::gateway::Gateway;
use crate::proto::Status;
use crate::models::client::Controller as ClientController;

const TOKEN_LEN: usize = 8;
const EPH_TOKEN_TIMEOUT: Duration = Duration::from_secs(20);

pub trait Controller {
    fn get_created_at(&self) -> SystemTime;
    fn get_touch_at(&self) -> SystemTime;
    fn get_deadline(&self) -> SystemTime;
    fn get_status(&self) -> Status;
    fn get_cookie(&self) -> &str;
    fn get_client(&self) -> &Box<dyn ClientController>;
    fn get_addr(&self) -> String;
    fn match_cookie(&self, cookie: String) -> bool;
    fn new_eph_token(&mut self) -> Result<String, Box<dyn Error>>;
}

pub struct Session {
    pub cookie: String,
    pub created_at: SystemTime,
    pub touch_at: SystemTime,
    pub timeout: Duration,
    pub status: Status,
    rand_gen: ThreadRng,
    client: Box<dyn ClientController>,
    tokens: HashSet<Token>,
    gateways: HashMap<Token, Gateway>,
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
            tokens: HashSet::new(),
            gateways: HashMap::new(),
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

    fn new_eph_token(&mut self) -> Result<String, Box<dyn Error>> {
        let deadline = SystemTime::now() + EPH_TOKEN_TIMEOUT;
        let token = Token::new(&mut self.rand_gen, deadline, TOKEN_LEN);
        let tid = token.to_string();
        self.tokens.insert(token);
        Ok(tid)
    }
}