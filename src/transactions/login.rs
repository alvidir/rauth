use crate::models::client::{Client, Controller as ClientController, Extension};
use crate::models::client::user::User;
use crate::transactions::*;
use crate::regex::*;

const ERR_IDENTIFIER_FORMAT: &str = "The identifier is of the wrong format";
const ERR_PWD_NOT_MATCH: &str = "The provided password does not match with user's";

pub struct TxLogin<'a> {
    ident: &'a str,
    pwd: &'a str,
}

impl<'a> TxLogin<'a> {
    pub fn new(ident: &'a str, pwd: &'a str) -> Self {
        TxLogin{
            ident: ident,
            pwd: pwd,
        }
    }

    fn require_client_by_name(&self) -> Result<Box<dyn ClientController>, Status> {
        match Client::find_by_name(self.ident) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                Err(status)
            }

            Ok(client) => Ok(client)
        }
    }

    fn require_client_from_extension(&self, extension: Box<dyn Extension>) -> Result<Box<dyn ClientController>, Status> {
        match Client::from_extension(extension) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                Err(status)
            }

            Ok(client) => Ok(client)
        }
    }

    fn require_client_by_email(&self) -> Result<Box<dyn ClientController>, Status> {
        match User::find_by_email(self.ident) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                Err(status)
            }

            Ok(user) => self.require_client_from_extension(Box::new(user))
        }
    }

    fn precondition(&self) -> Result<Box<dyn ClientController>, Status> {
        let client: Box<dyn ClientController>;
        if match_email(self.ident).is_ok() {
           client = self.require_client_by_email()?;
        } else if match_name(self.ident).is_ok() {
            client = self.require_client_by_name()?;
        } else {
            let msg = format!("{}", ERR_IDENTIFIER_FORMAT);
            let status = Status::failed_precondition(msg);
            return Err(status);
        }

        if !client.match_pwd(self.pwd.to_string()) {
            let msg = format!("{}", ERR_PWD_NOT_MATCH);
            let status = Status::failed_precondition(msg);
            return Err(status);
        }

        Ok(client)
    }

    pub fn execute(&self) -> Result<Response<SessionResponse>, Status> {
        println!("Got Login request from client {} ", self.ident);

        let client = self.precondition()?;
        let session = build_session(client)?;
        println!("Session for client {} has cookie {}", session.get_client().get_addr(), session.get_cookie());
        session_response(&session, "")
    }
}