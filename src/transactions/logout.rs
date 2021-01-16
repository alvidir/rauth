use std::time::SystemTime;
use crate::transactions::*;
use crate::regex::*;

pub struct TxLogout<'a> {
    cookie: &'a str,
}

impl<'a> TxLogout<'a> {
    pub fn new(cookie: &'a str) -> Self {
        TxLogout{
            cookie: cookie,
        }
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

    pub fn execute(&self) -> Result<SessionResponse, Box<dyn Cause>> {
        println!("Got a Logout request for cookie {} ", self.cookie);

        self.precondition_cookie()?;
        let session = find_session(self.cookie)?;
        destroy_session(self.cookie)?;

        println!("Session for client {} has cookie {}", session.get_client().get_addr(), session.get_cookie());
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