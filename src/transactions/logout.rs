use std::time::SystemTime;
use crate::regex::*;
use crate::models::session;
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

    fn precondition_cookie(&self) ->  Result<&mut Box<dyn session::Ctrl>, Box<dyn Cause>> {
        if let Err(err) = match_cookie(self.cookie) {
            let cause = TxCause::new(-1, err.to_string());
            Err(Box::new(cause))
        } else {
            find_session(self.cookie)
        }
    }

    pub fn execute(&self) -> Result<SessionResponse, Box<dyn Cause>> {
        println!("Got a Logout request for cookie {} ", self.cookie);

        self.precondition_cookie()?;
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

            Err(err) => {
                let cause = TxCause::new(-1, err.to_string());
                Err(Box::new(cause))
            }
        }
    }
}