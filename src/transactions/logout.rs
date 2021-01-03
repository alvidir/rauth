use tonic::{Response, Status};
use crate::models::session::{Controller as SessionController};
use crate::models::session::provider as SessionProvider;
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

    fn precondition(&self) -> Result<(), Status> {
        match match_cookie(self.cookie) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                Err(status)
            }

            Ok(_) => Ok(())
        }
    }

    fn require_session(&self) ->  Result<&Box<dyn SessionController>, Status> {
        let provider = SessionProvider::get_instance();
        match provider.get_session(self.cookie) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                Err(status)
            }

            Ok(sess) => Ok(sess)
        }
    }

    fn destroy_session(&self) ->  Result<(), Status> {
        let provider = SessionProvider::get_instance();
        match provider.destroy_session(self.cookie) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::internal(msg);
                Err(status)
            }

            Ok(_) => Ok(())
        }
    }

    pub fn execute(&self) -> Result<Response<SessionResponse>, Status> {
        println!("Got a Logout request for cookie {} ", self.cookie);

        self.precondition()?;
        let session = self.require_session()?;
        self.destroy_session()?;

        println!("Session for client {} has cookie {}", session.get_client().get_addr(), session.get_cookie());
        match unix_seconds(SystemTime::now()) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::internal(msg);
                Err(status)
            }

            Ok(deadline) => Ok(Response::new(
                SessionResponse {
                    deadline: deadline,
                    cookie: session.get_cookie().to_string(),
                    status: session.get_status() as i32,
                    token: "".to_string(),
                }
            ))
        }
    }
}