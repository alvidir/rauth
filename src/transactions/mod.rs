use std::error::Error;
pub mod login;
pub mod logout;
pub mod signup;

use crate::models::session;
use crate::models::user;
use crate::proto::client_proto;
use crate::models::secret;
use crate::time;
use crate::token::Token;

// Proto message structs
use client_proto::SessionResponse;

const TOKEN_LEN: usize = 8;

fn session_response(session: &Box<dyn session::Ctrl>, token: &str) -> Result<SessionResponse, Box<dyn Error>> {
	match time::unix_seconds(session.get_deadline()) {
		Ok(deadline) => Ok(
			SessionResponse {
				deadline: deadline,
				cookie: session.get_cookie().to_string(),
				status: session.get_status() as i32,
				token: token.to_string(),
			}
		),

		Err(err) => Err(err)
	}
}

fn build_session<'a>(client: Box<dyn user::Ctrl>) -> Result<&'a mut Box<dyn session::Ctrl>, Box<dyn Error>> {
	let provider = session::get_instance();
	provider.new_session(client)
}

fn find_session(cookie: &str) ->  Result<&mut Box<dyn session::Ctrl>, Box<dyn Error>> {
	let provider = session::get_instance();
	provider.get_session_by_cookie(cookie)
}

fn destroy_session(cookie: &str) ->  Result<(), Box<dyn Error>> {
	let provider = session::get_instance();
	provider.destroy_session(cookie)
}

fn build_signed_token(secret: Box<dyn secret::Ctrl>, pwd: &str) -> Result<(Token, String), Box<dyn Error>> {
	let token = Token::new(TOKEN_LEN);
	let signed: &[u8] = &secret.sign(pwd, token.to_string().as_bytes())?;
	let sign_str = format!("{:X?}", signed);
	Ok((token, sign_str))
}