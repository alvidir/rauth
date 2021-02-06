use std::time::SystemTime;
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

    pub fn execute(&self) -> Result<SessionResponse, Box<dyn Error>> {
        println!("Got a Logout request for cookie {} ", self.cookie);

        match_cookie(self.cookie)?;
        let session = find_session(self.cookie)?;
        destroy_session(self.cookie)?;

        println!("Session for client {} has cookie {}", session.get_email(), session.get_cookie());
        match time::unix_seconds(SystemTime::now()) {
            Ok(deadline) => Ok(
                SessionResponse {
                    deadline: deadline,
                    cookie: session.get_cookie().to_string(),
                    status: session.get_status() as i32,
                    token: "".to_string(),
                }
            ),

            Err(err) => Err(err)
        }
    }
}