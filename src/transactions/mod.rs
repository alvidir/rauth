use std::fmt;
use crate::models::session::{Controller as SessionController};
use crate::models::session::provider as SessionProvider;
use crate::models::client::Controller as ClientController;
use crate::proto::client_proto;
use crate::time;

pub mod login;
//pub mod signin;
pub mod signup;
pub mod logout;

// Proto message structs
use client_proto::SessionResponse;

pub trait Cause {
	fn get_code(&self) -> i32;
	fn get_msg(&self) -> &str;
}

struct TxCause {
	code: i32,
	msg: String,
}

impl TxCause {
	fn new(code: i32, msg: String) -> impl Cause {
		TxCause {
			code: code,
			msg: msg,
		}
	}
}

impl Cause for TxCause {
	fn get_code(&self) -> i32 {
		self.code
	}

	fn get_msg(&self) -> &str {
		&self.msg
	}
}

impl fmt::Display for dyn Cause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_msg())
    }
}

fn session_response(session: &Box<dyn SessionController>, token: &str) -> Result<SessionResponse, Box<dyn Cause>> {
	match time::unix_seconds(session.get_deadline()) {
		Ok(deadline) => Ok(
			SessionResponse {
				deadline: deadline,
				cookie: session.get_cookie().to_string(),
				status: session.get_status() as i32,
				token: token.to_string(),
			}
		),

		Err(err) => {
			let cause = TxCause::new(0, err.to_string());
			Err(Box::new(cause))
		}
	}
}

fn build_session<'a>(client: Box<dyn ClientController>) -> Result<&'a Box<dyn SessionController>, Box<dyn Cause>> {
	let provider = SessionProvider::get_instance();
	match provider.new_session(client) {
		Err(err) => {
			let cause = TxCause::new(-1, err.to_string());
			Err(Box::new(cause))
		}

		Ok(sess) => Ok(sess)
	}
}

fn find_session(cookie: &str) ->  Result<&Box<dyn SessionController>, Box<dyn Cause>> {
	let provider = SessionProvider::get_instance();
	match provider.get_session_by_cookie(cookie) {
		Err(err) => {
			let cause = TxCause::new(-1, err.to_string());
			Err(Box::new(cause))
		}

		Ok(sess) => Ok(sess)
	}
}

fn destroy_session(cookie: &str) ->  Result<(), Box<dyn Cause>> {
	let provider = SessionProvider::get_instance();
	match provider.destroy_session(cookie) {
		Err(err) => {
			let cause = TxCause::new(-1, err.to_string());
			Err(Box::new(cause))
		}

		Ok(sess) => Ok(sess)
	}
}