use std::error::Error;
use std::time::Duration;
use std::collections::HashMap;
use rand::prelude::ThreadRng;
use crate::models::session::{Session, Controller as SessionController};
use crate::models::session::token::Token;
use crate::models::client::Controller as ClientController;
use std::time::SystemTime;

const COOKIE_LEN: usize = 32;
const COOKIE_SEPARATOR: &str = "=";
const ERR_DEADLINE_EXCEEDED: &str = "Deadline exceeded";
const ERR_NO_TID: &str = "The provided cookie has no token ID";
const ERR_SESSION_ALREADY_EXISTS: &str = "A session already exists for client";
const ERR_BROKEN_COOKIE: &str = "No session has been found for cookie";
const ERR_NO_LOGED_EMAIL: &str = "No session has been logged with email";
const ERR_SESSION_BUILD: &str = "Something has failed while building session for";

static mut INSTANCE: Option<Box<dyn Controller>> = None;

pub trait Controller {
    fn new_session(&mut self, client: Box<dyn ClientController>) -> Result<&mut Box<dyn SessionController>, Box<dyn Error>>;
    fn get_session_by_cookie(&mut self, cookie: &str) -> Result<&mut Box<dyn SessionController>, Box<dyn Error>>;
    fn get_session_by_email(&mut self, addr: &str) -> Result<&mut Box<dyn SessionController>, Box<dyn Error>>;
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
    bycookie: HashMap<Token, Box<dyn SessionController>>,
    byemail: HashMap<String, String>,
    rand_gen: ThreadRng,
}

impl Provider {
    fn new(timeout: Duration) -> impl Controller {
        Provider{
            timeout: timeout,
            bycookie: HashMap::new(),
            byemail: HashMap::new(),
            rand_gen: rand::thread_rng(),
        }
    }

    fn cookie_gen(&mut self) -> Token {
        let deadline = SystemTime::now() + self.timeout;
        Token::new(&mut self.rand_gen, deadline, COOKIE_LEN)
    }

    fn split_cookie(cookie: &str) -> Vec<&str> {
        let split = cookie.split(COOKIE_SEPARATOR);
        split.collect()
    }

    //fn split_email(cookie: &str) -> Result<&str, Box<dyn Error>> {
    //    let split = Provider::split_cookie(cookie);
    //    if split.len() < 2 {
    //        Err(ERR_NO_EMAIL.into())
    //    } else {
    //        Ok(split[1])
    //    }
    //}

    fn split_tid(cookie: &str) -> Result<&str, Box<dyn Error>> {
        let split = Provider::split_cookie(cookie);
        if split.len() < 1 {
            Err(ERR_NO_TID.into())
        } else {
            Ok(split[0])
        }
    }

    fn is_session_alive(&mut self, token: &Token) -> Result<(), Box<dyn Error>> {
        if let Some(pair) = self.bycookie.get_key_value(token) {
            if pair.0.is_alive() {
                self.destroy_session_by_token(token)?;
                let msg = format!("{}", ERR_DEADLINE_EXCEEDED);
                Err(msg.into())
            } else {
                Ok(())
            }

        } else {
            let msg = format!("{}", ERR_BROKEN_COOKIE);
            Err(msg.into())
        }
    }

    fn get_session_by_token(&mut self, token: &Token) -> Result<&mut Box<dyn SessionController>, Box<dyn Error>> {
        self.is_session_alive(token)?;
        if let Some(sess) = self.bycookie.get_mut(token) {
            Ok(sess)
        } else {
            let msg = format!("{} {}", ERR_BROKEN_COOKIE, token);
            Err(msg.into())
        }
    }

    fn destroy_session_by_token(&mut self, token: &Token) -> Result<(), Box<dyn Error>> {
        if let Some(sess) = self.bycookie.get(token) {
            let email = sess.get_addr();
            self.bycookie.remove(&token);
            self.byemail.remove(&email);
            Ok(())
        } else {
            let msg = format!("{} {}", ERR_BROKEN_COOKIE, token);
            Err(msg.into())
        }
    }
}

impl Controller for Provider {
    fn new_session(&mut self, client: Box<dyn ClientController>) -> Result<&mut Box<dyn SessionController>, Box<dyn Error>> {
        let timeout = self.timeout;
        let email = client.get_addr();

        if let None = self.byemail.get(&email) {
            let token = self.cookie_gen();
            let cookie = format!("{}{}{}", token, COOKIE_SEPARATOR, email);
            let sess = Session::new(client, cookie.to_string(), timeout);
            
            self.byemail.insert(email.to_string(), token.to_string());
            self.bycookie.insert(token.clone(), Box::new(sess));

            if let Some(sess) = self.bycookie.get_mut(&token) {
                Ok(sess)
            } else {
                let msg = format!("{} {}", ERR_SESSION_BUILD, email);
                Err(msg.into())
            }

        } else {
            // checking if there is already a session for the provided email
            let msg = format!("{} {}", ERR_SESSION_ALREADY_EXISTS, email);
            Err(msg.into())
        }
    }

    fn get_session_by_cookie(&mut self, cookie: &str) -> Result<&mut Box<dyn SessionController>, Box<dyn Error>> {
        let tid = Provider::split_tid(cookie)?;
        let token = Token::from_string(tid);
        self.get_session_by_token(&token)
    }

    fn get_session_by_email(&mut self, email: &str) -> Result<&mut Box<dyn SessionController>, Box<dyn Error>> {
        if let Some(tid) = self.byemail.get(email) {
            let token = Token::from_string(tid);
            self.get_session_by_token(&token)
        } else {
            let msg = format!("{} {}", ERR_NO_LOGED_EMAIL, email);
            Err(msg.into())
        }
    }

    fn destroy_session(&mut self, cookie: &str) -> Result<(), Box<dyn Error>> {
        let tid = Provider::split_tid(cookie)?;
        let token = Token::from_string(tid);
        self.destroy_session_by_token(&token)
    }
}