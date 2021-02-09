use std::error::Error;
use crate::regex::*;
use crate::models::session;

pub struct TxLogout<'a> {
    cookie: &'a str,
}

impl<'a> TxLogout<'a> {
    pub fn new(cookie: &'a str) -> Self {
        TxLogout{
            cookie: cookie,
        }
    }

    fn destroy_session(&self) ->  Result<(), Box<dyn Error>> {
        let provider = session::get_instance();
        provider.destroy_session(self.cookie)
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        println!("Got a Logout request for cookie {} ", self.cookie);

        match_cookie(self.cookie)?;
        //let session = find_session(self.cookie)?;
        self.destroy_session()?;
        Ok(())
    }
}