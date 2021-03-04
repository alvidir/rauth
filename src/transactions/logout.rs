use crate::token::Token;
use std::error::Error;
use crate::regex::*;
use crate::models::{session, namesp};
use crate::default;

const ERR_SESSION_NOT_FOUND: &str = "Session not found for the provided cookie";

pub struct TxLogout<'a> {
    cookie: &'a str,
}

impl<'a> TxLogout<'a> {
    pub fn new(cookie: &'a str) -> Self {
        TxLogout{
            cookie: cookie,
        }
    }

    fn split_cookie(&self, left: bool) -> Result<Token, Box<dyn Error>> {
        match_cookie(self.cookie)?;
        let token: &str = {
            if left {
                &self.cookie[..default::TOKEN_LEN]
            } else {
                &self.cookie[default::TOKEN_LEN..]
            }
        };

        Ok(Token::from_string(token))
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        println!("Got a Logout request for cookie {} ", self.cookie);
        
        let token = self.split_cookie(true)?;
        if let Some(sess) = session::get_instance().get_by_cookie(&token) {
            // user has a session
            let dir_token = self.split_cookie(false)?;
            if let Some(app_id) = sess.delete_directory(&dir_token) {
                // user was loged in the application
                if let Some(np) = namesp::get_instance().get_by_id(app_id) {
                    // application is using a namespace
                    np.delete_token(sess.get_cookie());
                }
            }
        } else {
            return Err(ERR_SESSION_NOT_FOUND.into());
        }

        Ok(())
    }
}