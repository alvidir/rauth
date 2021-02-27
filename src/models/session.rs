use std::error::Error;
use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use crate::token::Token;
use crate::proto::Status;
use crate::default;
use super::{user, dir};
use super::dir::Ctrl as DirCtrl;

const ERR_DEADLINE_EXCEEDED: &str = "Deadline exceeded";
const ERR_SESSION_ALREADY_EXISTS: &str = "A session already exists for client";
const ERR_COOKIE_NOT_FOUND: &str = "No session has been found for the provided cookie";
const ERR_SESSION_BUILD: &str = "Something has failed while building session for";
const ERR_TOKEN_EXISTS: &str = "Provided token already exists";
const ERR_LABEL_ALREADY_EXISTS: &str = "Application already has a directory for this user";
static mut INSTANCE: Option<Box<dyn Factory>> = None;

pub trait Ctrl {
    fn get_client_id(&self) -> i32;
    fn get_cookie(&self) -> &Token;
    fn get_created_at(&self) -> SystemTime;
    fn get_touch_at(&self) -> SystemTime;
    fn get_status(&self) -> Status;
    fn get_email(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_user_id(&self) -> i32;
    fn get_token(&self,  target: i32) -> Option<&Token>;
    fn get_open_dirs(&self) -> Vec<Token>;
    fn get_directory(&self, token: Token) -> Option<Box<&dyn dir::Ctrl>>;
    fn new_directory(&mut self, app: i32) -> Result<Token, Box<dyn Error>>;
    fn delete_directory(&mut self, token: &Token) -> Option<i32>;
    fn match_pwd(&self, pwd: &str) -> bool;
    fn is_alive(&self) -> Result<(), Box<dyn Error>>;
}

pub trait Factory {
    fn new_session(&mut self, client: Box<dyn user::Ctrl>) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>>;
    fn get_by_cookie(&mut self, token: &Token) -> Option<&mut Box<dyn Ctrl>>;
    fn get_by_email(&mut self, addr: &str) -> Option<&mut Box<dyn Ctrl>>;
    fn get_by_name(&mut self, name: &str) -> Option<&mut Box<dyn Ctrl>>;
    fn destroy_session(&mut self, token: &Token) -> Result<(), Box<dyn Error>>;
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
            let timeout = Duration::new(default::TOKEN_TIMEOUT, 0);
            let instance = Provider::new(timeout);
            
            unsafe {
                INSTANCE = Some(Box::new(instance));
            }
            
            get_instance()
        },
    }
}

struct Provider {
    _timeout: Duration,
    allsess: HashMap<Token, Box<dyn Ctrl>>,
}

impl Provider {
    fn new(timeout: Duration) -> impl Factory {
        Provider{
            _timeout: timeout,
            allsess: HashMap::new(),
        }
    }

    fn cookie_gen(&mut self) -> Token {
        Token::new(default::TOKEN_LEN)
    }

    fn _get_session_by_token(&mut self, token: &Token) -> Option<&mut Box<dyn Ctrl>> {
        self.allsess.get_mut(token)
    }
}

impl Factory for Provider {
    fn new_session(&mut self, user: Box<dyn user::Ctrl>) -> Result<&mut Box<dyn Ctrl>, Box<dyn Error>> {
        if let None = self.allsess.iter().find(|(_, sess)| sess.get_user_id() == user.get_id()) {
            let token = self.cookie_gen();
            let email = user.get_email().to_string();
            let sess = Session::new(user, token.clone());
            
            self.allsess.insert(token.clone(), Box::new(sess));
            if let Some(sess) = self.allsess.get_mut(&token) {
                Ok(sess)
            } else {
                let msg = format!("{} {}", ERR_SESSION_BUILD, email);
                Err(msg.into())
            }

        } else {
            // checking if there is already a session for the provided email
            let msg = format!("{} {}", ERR_SESSION_ALREADY_EXISTS, user.get_email());
            Err(msg.into())
        }
    }

    fn get_by_cookie(&mut self, token: &Token) ->  Option<&mut Box<dyn Ctrl>> {
        self.allsess.get_mut(token)
    }

    fn get_by_email(&mut self, email: &str) -> Option<&mut Box<dyn Ctrl>> {
        if let Some((_, sess)) = self.allsess.iter_mut().find(|(_, sess)| sess.get_email() == email) {
            Some(sess)
        } else {
            None
        }
    }

    fn get_by_name(&mut self, name: &str) ->  Option<&mut Box<dyn Ctrl>> {
        if let Some((_, sess)) = self.allsess.iter_mut().find(|(_, sess)| sess.get_name() == name) {
            Some(sess)
        } else {
            None
        }
    }

    fn destroy_session(&mut self, token: &Token) -> Result<(), Box<dyn Error>> {
        if let Some(_) = self.allsess.remove(token) {
            Ok(())
        } else {
            let msg = format!("{} {}", ERR_COOKIE_NOT_FOUND, token);
            Err(msg.into())
        }
    }
}

struct Session {
    cookie: Token,
    touch_at: SystemTime,
    status: Status,
    user: Box<dyn user::Ctrl>,
    dirs: HashMap<Token, dir::Dir>, // token & app label
}

impl Session {
    pub fn new(user: Box<dyn user::Ctrl>, cookie: Token) -> impl Ctrl {
        Session{
            cookie: cookie,
            touch_at: SystemTime::now(),
            status: Status::New,
            user: user,
            dirs: HashMap::new(),
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

    fn get_cookie(&self) -> &Token {
        &self.cookie
    }

    fn get_email(&self) -> &str {
        self.user.get_email()
    }

    fn get_name(&self) -> &str {
        self.user.get_name()
    }

    fn get_created_at(&self) -> SystemTime {
        self.cookie.get_deadline()
    }

    fn get_touch_at(&self) -> SystemTime {
        self.touch_at
    }

    fn get_status(&self) -> Status {
        self.status
    }

    fn get_open_dirs(&self) -> Vec<Token> {
        self.dirs.iter().map(|(token, _)| (token.clone())).collect()
    }

    fn get_token(&self, target: i32) -> Option<&Token> {
        if let Some((token, _)) = self.dirs.iter().find(|(_, dir)| dir.get_app_id() == target) {
            Some(token)
        } else {
            None
        }
    }

    fn delete_directory(&mut self, token: &Token) -> Option<i32> {
        if let Some(dir) = self.dirs.remove(token) {
            Some(dir.get_app_id())
        } else {
            None
        }
    }

    fn new_directory(&mut self, app: i32) -> Result<Token, Box<dyn Error>> {
        if let Some(_) = self.dirs.iter().find(|(_, dir)| dir.get_app_id() == app) {
            return Err(ERR_LABEL_ALREADY_EXISTS.into());
        }

        let token = Token::new(default::TOKEN_LEN);
        if let Some(_) = self.dirs.get(&token) {
            return Err(ERR_TOKEN_EXISTS.into());
        }

        let dir = dir::Dir::new(self.user.get_id(), app);
        self.dirs.insert(token.clone(), dir);
        Ok(token)
    }

    fn get_directory(&self, token: Token) -> Option<Box<&dyn dir::Ctrl>> {
        if let Some(dir) = self.dirs.get(&token) {
            Some(Box::new(dir))
        } else {
            None
        }
    }

    fn match_pwd(&self, pwd: &str) -> bool {
        self.user.match_pwd(pwd)
    }

    fn is_alive(&self) -> Result<(), Box<dyn Error>> {
        let timeout = Duration::new(default::TOKEN_TIMEOUT, 0);
        if self.cookie.deadline_exceed(timeout) {
            let msg = format!("{}", ERR_DEADLINE_EXCEEDED);
            Err(msg.into())
        } else {
            Ok(())
        }
    }
}