use std::error::Error;
use std::time::Duration;
use std::collections::HashMap;
use crate::models::session::{Session, Controller as SessionController};
use crate::models::client::Controller as ClientController;
//use std::sync::Mutex;

use rand::Rng;
use rand::prelude::ThreadRng;
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789)(*&^%$#@!~?][+-";

const COOKIE_LEN: usize = 32;
const ERR_SESSION_ALREADY_EXISTS: &str = "A session already exists for client";
const ERR_BROKEN_COOKIE: &str = "No session has been found for cookie";
const ERR_NO_LOGED_EMAIL: &str = "No session has been logged with email";
const ERR_SESSION_BUILD: &str = "Something has failed while building session for";

static mut INSTANCE: Option<Box<dyn Controller>> = None;

pub trait Controller {
    fn new_session(&mut self, client: Box<dyn ClientController>) -> Result<&Box<dyn SessionController>, Box<dyn Error>>;
    fn get_session_by_cookie(&self, cookie: &str) -> Result<&Box<dyn SessionController>, Box<dyn Error>>;
    fn get_session_by_email(&self, addr: &str) -> Result<&Box<dyn SessionController>, Box<dyn Error>>;
    fn destroy_session(&mut self, cookie: &str) -> Result<(), Box<dyn Error>>;
}

pub fn get_instance<'a>() -> &'a mut Box<dyn Controller> {
    let provider: &mut Option<Box<dyn Controller>>;
    unsafe {
        provider = &mut INSTANCE
    }

    match provider {
        Some(ctrl) => {
            ctrl
        },
        None => {
            let timeout = Duration::new(3600, 0);
            let instance = Provider::new(timeout);
            
            unsafe {
                INSTANCE = Some(Box::new(instance));
            }
            
            get_instance()
        },
    }
}

struct Provider {
    timeout: Duration,
    instances: HashMap<String, Box<dyn SessionController>>,
    byemail: HashMap<String, String>,
    rand_gen: ThreadRng,
}

impl Provider {
    fn new(timeout: Duration) -> impl Controller {
        Provider{
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
}

impl Controller for Provider {
    fn new_session(&mut self, client: Box<dyn ClientController>) -> Result<&Box<dyn SessionController>, Box<dyn Error>> {
        let timeout = self.timeout;
        let email = client.get_addr();

        match self.byemail.get(&email) {
            // checking if there is already a session for the provided email
            Some(_) => {
                let msg = format!("{} {}", ERR_SESSION_ALREADY_EXISTS, email);
                Err(msg.into())
            },

            None => {
                let hash = self.cookie_gen();
                let cookie = format!("{}={}", hash, email);
                let sess = Session::new(client, &cookie, timeout);
               
                self.byemail.insert(email.to_string(), cookie.to_string());
                self.instances.insert(cookie.to_string(), Box::new(sess));

                match self.instances.get(&cookie) {
                    None => {
                        let msg = format!("{} {}", ERR_SESSION_BUILD, email);
                        Err(msg.into())
                    }

                    Some(session) => Ok(session)
                }
                
            }
        }
    }

    fn get_session_by_cookie(&self, cookie: &str) -> Result<&Box<dyn SessionController>, Box<dyn Error>> {
        match self.instances.get(cookie) {
            None => {
                let msg = format!("{} {}", ERR_BROKEN_COOKIE, cookie);
                Err(msg.into())
            }

            Some(sess) => Ok(sess)
        }
    }

    fn get_session_by_email(&self, email: &str) -> Result<&Box<dyn SessionController>, Box<dyn Error>> {
        match self.byemail.get(email) {
            None => {
                let msg = format!("{} {}", ERR_NO_LOGED_EMAIL, email);
                Err(msg.into())
            }

            Some(cookie) => self.get_session_by_cookie(cookie)
        }
    }

    fn destroy_session(&mut self, cookie: &str) -> Result<(), Box<dyn Error>> {
        match self.instances.get(cookie) {
            Some(sess) => {
                let email = sess.get_addr();
                self.instances.remove(cookie);
                self.byemail.remove(&email);
                Ok(())
            },

            None => {
                let msg = format!("{} {}", ERR_BROKEN_COOKIE, cookie);
                Err(msg.into())
            }
        }
    }
}