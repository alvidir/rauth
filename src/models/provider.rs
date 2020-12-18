use crate::models::session::{Session, Controller as SessionController};
use crate::models::client::{Controller as ClientController};
use std::time::Duration;
use std::collections::HashMap;
//use std::sync::Mutex;

use rand::Rng;
use rand::prelude::ThreadRng;
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789)(*&^%$#@!~";

const COOKIE_LEN: usize = 32;
const ERR_SESSION_ALREADY_EXISTS: &str = "A session already exists for client";
const ERR_BROKEN_COOKIE: &str = "No session has been found for cookie";
const ERR_SESSION_BUILD: &str = "Something has failed while building session for";

pub static mut DEFAULT: Option<Box<dyn Controller>> = None;

pub trait Controller {
    fn new_session(&mut self, client: Box<dyn ClientController>) -> Result<&Box<dyn SessionController>, String>;
    fn get_session(&self, cookie: &str) -> Result<&Box<dyn SessionController>, String>;
    fn destroy_session(&mut self, cookie: &str) -> Result<(), String>;
}

pub struct Provider {
    name: String,
    timeout: Duration,
    instances: HashMap<String, Box<dyn SessionController>>,
    byemail: HashMap<String, String>,
    rand_gen: ThreadRng,
}

impl Provider {
    fn new(name: String, timeout: Duration) -> Self {
        Provider{
            name: name,
            timeout: timeout,
            instances: HashMap::new(),
            byemail: HashMap::new(),
            rand_gen: rand::thread_rng(),
        }
    }

    fn cookie_gen(&mut self) -> String {
        let cookie: String = (0..COOKIE_LEN)
            .map(|_| {
                let idx = self.rand_gen.gen_range(0, CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        
        cookie
    }

    pub fn get_instance() -> &'static Box<dyn Controller> {
        let provider: &Option<Box<dyn Controller>>;
        unsafe {
            provider = &DEFAULT
        }

        match &provider {
            Some(ctrl) => {
                ctrl
            },
            None => {
                let name = "tp-auth-default".to_string();
                let timeout = Duration::new(3600, 0);
                let instance = Provider::new(name, timeout);
                
                unsafe {
                    DEFAULT = Some(Box::new(instance));
                }
                
                Provider::get_instance()
            },
        }
    }
}

impl Controller for Provider {
    fn new_session(&mut self, client: Box<dyn ClientController>) -> Result<&Box<dyn SessionController>, String> {
        let timeout = self.timeout;
        let email = client.get_addr();

        match self.byemail.get(&email) {
            // checking if there is already a session for the provided email
            Some(_) => {
                let msg = format!("{} {}", ERR_SESSION_ALREADY_EXISTS, email);
                Err(msg)
            },

            None => {
                let hash = self.cookie_gen();
                let cookie = format!("{}={}", hash, email);
                let sess = Session::new(client, &cookie, timeout);
               
                self.byemail.insert(email.to_string(), cookie.to_string());
                self.instances.insert(cookie.to_string(), Box::new(sess));

                match self.instances.get(&cookie) {
                    Some(s) => {
                        Ok(s)
                    }

                    None => {
                        let msg = format!("{} {}", ERR_SESSION_BUILD, email);
                        Err(msg)
                    }
                }
                
            }
        }
    }

    fn get_session(&self, cookie: &str) -> Result<&Box<dyn SessionController>, String> {
        match self.instances.get(cookie) {
            Some(sess) => {
                Ok(sess)
            },

            None => {
                let msg = format!("{} {}", ERR_BROKEN_COOKIE, cookie);
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
                let msg = format!("{} {}", ERR_BROKEN_COOKIE, cookie);
                Err(msg)
            }
        }
    }
}