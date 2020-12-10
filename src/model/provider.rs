use crate::model::session::{Status, Session, Controller as SessionController};
use crate::model::client::{Controller as ClientController};
use std::time::Duration;
use std::collections::HashMap;
use std::time::Instant;
//use std::sync::Mutex;

use rand::Rng;
use rand::prelude::ThreadRng;
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789)(*&^%$#@!~";

const PASSWORD_LEN: usize = 32;

pub static mut master: Option<Provider> = None;

pub trait Controller {
    fn new_session(&mut self, client: Box<dyn ClientController>) -> Result<Box<dyn SessionController>, String>;
    fn get_session(&self, cookie: &str) -> Result<Box<dyn SessionController>, String>;
    fn destroy_session(&mut self, cookie: &str) -> Result<(), String>;
    fn purge(&mut self, deadline: Instant) -> i32;
}

pub struct Provider {
    name: String,
    timeout: Duration,
    instances: HashMap<String, Box<dyn SessionController>>,
    byemail: HashMap<String, String>,
    rand_gen: ThreadRng,
}

const errSessionAlreadyExists: &str = "A session already exists for client {}";
const errBrokenCookie: &str = "Cookie {} has no session associated";

impl Provider {
    fn new(name: String, timeout: Duration) -> Box<dyn Controller> {
        match master {
            None => {
                let instance = Provider{
                    name: name,
                    timeout: timeout,
                    instances: HashMap::new(),
                    byemail: HashMap::new(),
                    rand_gen: rand::thread_rng(),
                };

                master = Some(instance);
                Box::new(instance)
            },

            Some(ctrl) => {
                Box::new(ctrl)
            },
        }
    }

    fn cookie_gen(&self) -> String {
        let cookie: String = (0..PASSWORD_LEN)
            .map(|_| {
                let idx = self.rand_gen.gen_range(0, CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        
        cookie
    }

    pub fn get_instance() -> Box<dyn Controller> {
        match master {
            None => {
                let name = "tp-auth-default".to_string();
                let timeout = Duration::new(3600, 0);
                Provider::new(name, timeout)
            },

            Some(ctrl) => {
                Box::new(ctrl)
            },
        }
    }
}

impl Controller for Provider {
    fn new_session(&mut self, client: Box<dyn ClientController>) -> Result<Box<dyn SessionController>, String> {
        let cookie = self.cookie_gen();
        let timeout = self.timeout;
        let email = client.get_addr();

        match self.byemail.get(&email) {
            Some(_) => {
                let msg = format!("{} {}", errSessionAlreadyExists, email);
                Err(msg)
            },

            None => {
                self.byemail.insert(email.to_string(), cookie);
                let sess = Session::new(client, cookie, timeout);
                self.instances.insert(cookie, Box::new(sess));
                Ok(Box::new(sess))
            }
        }
    }

    fn get_session(&self, cookie: &str) -> Result<Box<dyn SessionController>, String> {
        match self.instances.get(cookie) {
            Some(sess) => {
                Ok(*sess)
            },

            None => {
                let msg = format!("{} {}", errBrokenCookie, cookie);
                Err(msg)
            }
        }
    }

    fn destroy_session(&mut self, cookie: &str) -> Result<(), String> {
        match self.instances.get(cookie) {
            Some(sess) => {
                let email = sess.get_addr();
                self.instances.remove(cookie);
                self.byemail.remove(&email);
                Ok(())
            },

            None => {
                let msg = format!("{} {}", errBrokenCookie, cookie);
                Err(msg)
            }
        }
    }
    
    fn purge(&mut self, deadline: Instant) -> i32 {
        0
    }
}