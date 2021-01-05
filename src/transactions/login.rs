use crate::models::client::Controller as ClientController;
use crate::models::client::user::User;
use crate::transactions::*;
use crate::regex::*;

const ERR_PWD_NOT_MATCH: &str = "The provided password does not match with user's";

pub struct TxLogin<'a> {
    cookie: &'a str,
    email: &'a str,
    pwd: &'a str,
}

impl<'a> TxLogin<'a> {
    pub fn new(cookie: &'a str, email: &'a str, pwd: &'a str) -> Self {
        TxLogin{
            cookie: cookie,
            email: email,
            pwd: pwd,
        }
    }

    fn require_client(&self) -> Result<Box<dyn ClientController>, Status> {
        match User::find_by_email(self.email) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                Err(status)
            }

            Ok(user) => Ok(user)
        }
    }

    fn require_session(&self) ->  Result<&Box<dyn SessionController>, Status> {
        let provider = SessionProvider::get_instance();
        match provider.get_session_by_cookie(self.cookie) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                Err(status)
            }

            Ok(sess) => Ok(sess)
        }
    }

    fn precondition_cookie(&self) ->  Result<&Box<dyn SessionController>, Status> {
        match match_cookie(self.cookie) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                Err(status)
            }

            Ok(_) => {
                self.require_session()
            }
        }
    }

    fn precondition_email(&self) -> Result<Box<dyn ClientController>, Status> {
        match match_email(self.email) {
            Err(err) => {
                println!("GOT ERROR FOR EMAIL {}", self.email);
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                return Err(status);
            }

            Ok(_) => {
                let client = self.require_client()?;
                if !client.match_pwd(self.pwd.to_string()) {
                    let msg = format!("{}", ERR_PWD_NOT_MATCH);
                    let status = Status::failed_precondition(msg);
                    return Err(status);
                }
        
                Ok(client)
            }
        }
    }

    fn check_alive_session(&self, client: &Box<dyn ClientController>) -> Result<&Box<dyn SessionController>, Status> {
        let provider = SessionProvider::get_instance();
        match provider.get_session_by_email(&client.get_addr()) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                Err(status)
            }

            Ok(sess) => Ok(sess)
        }
    }

    pub fn execute(&self) -> Result<Response<SessionResponse>, Status> {
        println!("Got Login request from client {} ", self.email);
        
        match self.precondition_cookie() {
            Err(_) => {
                let client = self.precondition_email()?;
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
                println!("Session for client {} got cookie {}", session.get_client().get_addr(), session.get_cookie());
                session_response(&session, "")
            }
        }        
    }
}