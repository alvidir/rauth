use tonic::{Response, Status};
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::models::session::{Controller as SessionController};
use crate::models::session::provider as SessionProvider;
use crate::models::client::Controller as ClientController;
use crate::proto::client_proto;

pub mod login;
//pub mod signin;
pub mod signup;
pub mod logout;

// Proto message structs
use client_proto::SessionResponse;

fn unix_seconds(current: SystemTime) -> Result<u64, Box<dyn Error>> {
	match current.duration_since(UNIX_EPOCH) {
		Err(err) => {
			let msg = format!("Time went backwards: {}", err);
			return Err(msg.into());
		}

		Ok(unix) => Ok(unix.as_secs())
	}
}

fn session_response(session: &Box<dyn SessionController>, token: &str) -> Result<Response<SessionResponse>, Status> {
	match unix_seconds(session.get_deadline()) {
		Err(err) => {
			let status = Status::invalid_argument(err.to_string());
			return Err(status);
		}

		Ok(deadline) => Ok(Response::new(
			SessionResponse {
				deadline: deadline,
				cookie: session.get_cookie().to_string(),
				status: session.get_status() as i32,
				token: token.to_string(),
			}
		))
	}
}

fn build_session<'a>(client: Box<dyn ClientController>) -> Result<&'a Box<dyn SessionController>, Status> {
	let provider = SessionProvider::get_instance();
	match provider.new_session(client) {
		Err(err) => {
			let msg = format!("{}", err);
			let status = Status::internal(msg);
			Err(status)
		}

		Ok(session) => Ok(session)
	}
}