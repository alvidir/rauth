use crate::regex::*;
use super::*;

pub struct TxLogout<'a> {
    cookie: &'a str,
}

impl<'a> TxLogout<'a> {
    pub fn new(cookie: &'a str) -> Self {
        TxLogout{
            cookie: cookie,
        }
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        println!("Got a Logout request for cookie {} ", self.cookie);

        match_cookie(self.cookie)?;
        //let session = find_session(self.cookie)?;
        destroy_session(self.cookie)?;
        Ok(())
    }
}