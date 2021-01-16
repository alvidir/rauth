use crate::models::client::Controller as ClientController;
use crate::models::client::User;
use crate::transactions::*;
use crate::regex::*;

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

    fn require_password_match(&self, client: &Box<dyn ClientController>) -> Result<(), Box<dyn Cause>> {
        if !client.match_pwd(self.pwd.to_string()) {
            let msg = format!("{}", ERR_PWD_NOT_MATCH);
            let cause = TxCause::new(-1, msg.into());
            return Err(Box::new(cause));
        }

        Ok(())
    }

    fn precondition_cookie(&self) ->  Result<&Box<dyn SessionController>, Box<dyn Cause>> {
        match match_cookie(self.cookie) {
            Err(err) => {
                let cause = TxCause::new(-1, err.to_string());
                Err(Box::new(cause))
            }

            Ok(_) => find_session(self.cookie)
        }
    }

    fn require_client_by_email(&self) -> Result<Box<dyn ClientController>, Box<dyn Cause>> {
        match User::find_by_email(self.ident) {
            Err(err) => {
                let cause = TxCause::new(-1, err.to_string());
                Err(Box::new(cause))
            }

            Ok(client) => {
                self.require_password_match(&client)?;
                Ok(client)
            }
        }
    }

    fn precondition_email(&self) -> Result<Box<dyn ClientController>, Box<dyn Cause>> {
        match match_email(self.ident) {
            Err(err) => {
                let cause = TxCause::new(-1, err.to_string());
                Err(Box::new(cause))
            }

            Ok(_) => self.require_client_by_email()
        }
    }

    fn require_client_by_name(&self) -> Result<Box<dyn ClientController>, Box<dyn Cause>> {
        match User::find_by_name(self.ident) {
            Err(err) => {
                let cause = TxCause::new(-1, err.to_string());
                Err(Box::new(cause))
            }

            Ok(client) => {
                self.require_password_match(&client)?;
                Ok(client)
            }
        }
    }

    fn precondition_name(&self) -> Result<Box<dyn ClientController>, Box<dyn Cause>> {
        match match_name(self.ident) {
            Ok(_) => {
                let client = self.require_client_by_name()?;
                self.require_password_match(&client)?;
        
                Ok(client)
            }

            Err(err) => {
                let cause = TxCause::new(-1, err.to_string());
                Err(Box::new(cause))
            }
        }
    }

    fn precondition_ident(&self) -> Result<Box<dyn ClientController>, Box<dyn Cause>> {
        match self.precondition_email() {
            Err(_) => {
                self.precondition_name()
            }

            Ok(client) => Ok(client)
        }
    }

    fn check_alive_session(&self, client: &Box<dyn ClientController>) -> Result<&Box<dyn SessionController>, Box<dyn Cause>> {
        let provider = SessionProvider::get_instance();
        match provider.get_session_by_email(&client.get_addr()) {
            Err(err) => {
                let cause = TxCause::new(-1, err.to_string());
                Err(Box::new(cause))
            }

            Ok(sess) => Ok(sess)
        }
    }

    pub fn execute(&self) -> Result<SessionResponse, Box<dyn Cause>> {
        println!("Got Login request from client {} ", self.ident);
        
        match self.precondition_cookie() {
            Err(_) => {
                let client = self.precondition_ident()?;
                let session: &Box<dyn SessionController>;
                match self.check_alive_session(&client) {
                    Err(_) => {
                        session = build_session(client)?;
                    }

                    Ok(sess) => {
                        session = sess;
                    }
                }

                println!("Session for client {} got cookie {}", session.get_client().get_addr(), session.get_cookie());
                session_response(&session, "")
            }

            Ok(session) => {
                println!("Session for client {} already exists", session.get_client().get_addr());
                session_response(&session, "")
            }
        }        
    }
}