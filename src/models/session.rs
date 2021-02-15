use std::error::Error;
use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use std::collections::HashSet;
use crate::token::Token;
use crate::proto::Status;
use super::user;

const TOKEN_TIMEOUT: u64 = 3600; // 60s * 60min
const DUST_LEN: usize = 8;
const COOKIE_LEN: usize = 32;
const COOKIE_TIMEOUT: u64 = 86400; // 3600s * 24h
const COOKIE_SEPARATOR: &str = "=";
const ERR_DEADLINE_EXCEEDED: &str = "Deadline exceeded";
const ERR_NO_TID: &str = "The provided cookie has no token ID";
const ERR_SESSION_ALREADY_EXISTS: &str = "A session already exists for client";
const ERR_BROKEN_COOKIE: &str = "No session has been found for cookie";
const ERR_NO_LOGED_EMAIL: &str = "No session has been logged with email";
const ERR_SESSION_BUILD: &str = "Something has failed while building session for";
const ERR_TOKEN_EXISTS: &str = "Provided token already exists";

static mut INSTANCE: Option<Box<dyn Factory>> = None;

pub trait Ctrl {
    fn get_client_id(&self) -> i32;
    fn get_cookie(&self) -> &str;
    fn get_created_at(&self) -> SystemTime;
    fn get_touch_at(&self) -> SystemTime;
    fn get_deadline(&self) -> SystemTime;
    fn get_status(&self) -> Status;
    fn get_email(&self) -> &str;
    fn get_token(&mut self) -> Result<String, Box<dyn Error>>;
    fn get_user_id(&self) -> i32;
}

pub trait Factory {
    fn new_session(&mut self, client: Box<dyn user::Ctrl>) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>>;
    fn get_session_by_cookie(&mut self, cookie: &str) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>>;
    fn get_session_by_email(&mut self, addr: &str) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>>;
    fn destroy_session(&mut self, cookie: &str) -> Result<(), Box<dyn Error>>;
}

pub fn get_instance<'a>() -> &'a mut Box<dyn Factory> {
    let provider: &mut Option<Box<dyn Factory>>;
    unsafe {
        provider = &mut INSTANCE
    }

    match provider {
        Some(ctrl) => {
            ctrl
        },
        None => {
            let timeout = Duration::new(COOKIE_TIMEOUT, 0);
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
    allsess: HashMap<Token, Box<dyn Ctrl>>,
    byemail: HashMap<String, String>,
}

impl Provider {
    fn new(timeout: Duration) -> impl Factory {
        Provider{
            timeout: timeout,
            allsess: HashMap::new(),
            byemail: HashMap::new(),
        }
    }

    fn cookie_gen(&mut self) -> Token {
        Token::new(COOKIE_LEN)
    }

    fn split_cookie(cookie: &str) -> Vec<&str> {
        let split = cookie.split(COOKIE_SEPARATOR);
        split.collect()
    }

    fn split_token(cookie: &str) -> Result<&str, Box<dyn Error>> {
        let split = Provider::split_cookie(cookie);
        if split.len() < 1 {
            Err(ERR_NO_TID.into())
        } else {
            Ok(split[0])
        }
    }

    fn is_alive(&mut self, token: &Token) -> Result<(), Box<dyn Error>> {
        if let Some(pair) = self.allsess.get_key_value(token) {
            let timeout = Duration::new(TOKEN_TIMEOUT, 0);
            if pair.0.deadline_exceed(timeout) {
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

    fn get_session_by_token(&mut self, token: &Token) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>> {
        self.is_alive(token)?;
        if let Some(sess) = self.allsess.get_mut(token) {
            Ok(sess)
        } else {
            let msg = format!("{} {}", ERR_BROKEN_COOKIE, token);
            Err(msg.into())
        }
    }

    fn destroy_session_by_token(&mut self, token: &Token) -> Result<(), Box<dyn Error>> {
        if let Some(sess) = self.allsess.remove(&token) {
            let email = sess.get_email();
            self.byemail.remove(email);
        } else {
            let msg = format!("{} {}", ERR_BROKEN_COOKIE, token);
            return Err(msg.into());
        }

        Ok(())
    }
}

impl Factory for Provider {
    fn new_session(&mut self, user: Box<dyn user::Ctrl>) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>> {
        let timeout = self.timeout;
        let email = user.get_email().to_string();

        if let None = self.byemail.get(&email) {
            let token = self.cookie_gen();
            let sess = Session::new(user, token.to_string(), timeout);
            
            self.byemail.insert(email.to_string(), token.to_string());
            self.allsess.insert(token.clone(), Box::new(sess));

            if let Some(sess) = self.allsess.get_mut(&token) {
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

    fn get_session_by_cookie(&mut self, cookie: &str) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>> {
        let tid = Provider::split_token(cookie)?;
        let token = Token::from_string(tid);
        self.get_session_by_token(&token)
    }

    fn get_session_by_email(&mut self, email: &str) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>> {
        if let Some(tid) = self.byemail.get(email) {
            let token = Token::from_string(tid);
            self.get_session_by_token(&token)
        } else {
            let msg = format!("{} {}", ERR_NO_LOGED_EMAIL, email);
            Err(msg.into())
        }
    }

    fn destroy_session(&mut self, cookie: &str) -> Result<(), Box<dyn Error>> {
        let tid = Provider::split_token(cookie)?;
        let token = Token::from_string(tid);
        self.destroy_session_by_token(&token)
    }
}

struct Session {
    pub cookie: String,
    pub created_at: SystemTime,
    pub touch_at: SystemTime,
    pub timeout: Duration,
    pub status: Status,
    user: Box<dyn user::Ctrl>,
    tokens: HashSet<Token>,
}

impl Session {
    pub fn new(user: Box<dyn user::Ctrl>, cookie: String, timeout: Duration) -> Self {
        Session{
            cookie: cookie,
            created_at: SystemTime::now(),
            touch_at: SystemTime::now(),
            timeout: timeout,
            status: Status::New,
            user: user,
            tokens: HashSet::new(),
        }
    }
}

impl Ctrl for Session {
    fn get_client_id(&self) -> i32 {
        self.user.get_client_id()
    }

    fn get_user_id(&self) -> i32 {
        self.user.get_id()
    }

    fn get_cookie(&self) -> &str {
        &self.cookie
    }

    fn get_email(&self) -> &str {
        self.user.get_email()
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

    fn get_token(&mut self) -> Result<String, Box<dyn Error>> {
        let token = Token::new(DUST_LEN);
        let dir = format!("{}{}", self.cookie, token);
        if self.tokens.insert(token) {
            Ok(dir)
        } else {
            Err(ERR_TOKEN_EXISTS.into())
        }
    }
}