use std::error::Error;
use crate::models::client::{Client, Controller as ClientController};
use crate::models::client::user::User;
use crate::models::session::{Controller as SessionController};
use crate::models::session::provider as SessionProvider;
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

    pub fn execute(&self) -> Result<&Box<dyn SessionController>, Box<dyn Error>> {
        println!("Got Login request from client {} ", self.ident);

        let client: Box<dyn ClientController>;
        if match_email(self.ident).is_ok() {
            let user = User::find_by_email(self.ident)?;
            client = Client::from_extension(Box::new(user))?;
        } else if match_name(self.ident).is_ok() {
            client = Client::find_by_name(self.ident)?;
        } else {
            return Err(ERR_IDENTIFIER_FORMAT.into());
        }

        if !client.match_pwd(self.pwd.to_string()) {
            return Err(ERR_PWD_NOT_MATCH.into());
        }

        let provider = SessionProvider::get_instance();
        let session = provider.new_session(client)?;
        println!("Session for client {} has cookie {}", session.get_client().get_addr(), session.get_cookie());
        Ok(session)
    }
}