use crate::models::user;
use crate::regex::*;
use super::*;

const ERR_PWD_NOT_MATCH: &str = "The provided password does not match with user's";

pub struct TxLogin<'a> {
    cookie: &'a str,
    ident: &'a str,
    pwd: &'a str,
}

impl<'a> TxLogin<'a> {
    pub fn new(cookie: &'a str, ident: &'a str, pwd: &'a str) -> Self {
        TxLogin{
            cookie: cookie,
            ident: ident,
            pwd: pwd,
        }
    }

    fn require_password_match(&self, user: &Box<dyn user::Ctrl>) -> Result<(), Box<dyn Cause>> {
        if !user.match_pwd(self.pwd) {
            let msg = format!("{}", ERR_PWD_NOT_MATCH);
            let cause = TxCause::new(-1, msg.into());
            return Err(Box::new(cause));
        }

        Ok(())
    }

    fn precondition_cookie(&self) ->  Result<&mut Box<dyn session::Ctrl>, Box<dyn Cause>> {
        if let Err(err) = match_cookie(self.cookie) {
            let cause = TxCause::new(-1, err.to_string());
            Err(Box::new(cause))
        } else {
            find_session(self.cookie)
        }
    }

    fn require_user_by_email(&self) -> Result<Box<dyn user::Ctrl>, Box<dyn Cause>> {
        match user::find_by_email(self.ident) {
            Err(err) => {
                let cause = TxCause::new(-1, err.to_string());
                Err(Box::new(cause))
            }

            Ok(user) => {
                self.require_password_match(&user)?;
                Ok(user)
            }
        }
    }

    fn precondition_email(&self) -> Result<Box<dyn user::Ctrl>, Box<dyn Cause>> {
        if let Err(err) = match_email(self.ident) {
            let cause = TxCause::new(-1, err.to_string());
            Err(Box::new(cause))
        } else {
            self.require_user_by_email()
        }
    }

    fn require_user_by_name(&self) -> Result<Box<dyn user::Ctrl>, Box<dyn Cause>> {
        match user::find_by_name(self.ident) {
            Err(err) => {
                let cause = TxCause::new(-1, err.to_string());
                Err(Box::new(cause))
            }

            Ok(user) => {
                self.require_password_match(&user)?;
                Ok(user)
            }
        }
    }

    fn precondition_name(&self) -> Result<Box<dyn user::Ctrl>, Box<dyn Cause>> {
        if let Err(err) = match_name(self.ident) {
            let cause = TxCause::new(-1, err.to_string());
            Err(Box::new(cause))
        } else {
            let user = self.require_user_by_name()?;
            self.require_password_match(&user)?;
    
            Ok(user)
        }
    }

    fn precondition_ident(&self) -> Result<Box<dyn user::Ctrl>, Box<dyn Cause>> {
        if let Ok(user) = self.precondition_email() {
            Ok(user)
        } else {
            self.precondition_name()
        }
    }

    fn check_alive_session(&self, user: &Box<dyn user::Ctrl>) -> Result<&mut Box<dyn session::Ctrl>, Box<dyn Cause>> {
        let provider = session::get_instance();
        match provider.get_session_by_email(&user.get_email()) {
            Err(err) => {
                let cause = TxCause::new(-1, err.to_string());
                Err(Box::new(cause))
            }

            Ok(sess) => Ok(sess)
        }
    }

    pub fn execute(&self) -> Result<SessionResponse, Box<dyn Cause>> {
        println!("Got Login request from user {} ", self.ident);
        let user = self.precondition_ident()?;
        let session: &mut Box<dyn session::Ctrl>;
        if let Ok(sess) = self.precondition_cookie() {
            session = sess
        } else if let Ok(sess) = self.check_alive_session(&user) {
            session = sess;
        } else {
            session = build_session(user)?;
        }
        
        println!("Session for user {} got cookie {}", session.get_email(), session.get_cookie());
        let token = get_token(session)?;
        session_response(&session, &token)
    }
}