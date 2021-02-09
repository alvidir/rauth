use std::error::Error;
use crate::models::{user, app};
use crate::regex::*;
use crate::time;
use crate::token::Token;
use crate::models::session;
use crate::models::secret;

// Proto message structs
use crate::proto::client_proto;
use client_proto::LoginResponse;

const TOKEN_LEN: usize = 8;
const ERR_IDENT_NOT_MATCH: &str = "The provided indentity is not of the expected type";

pub struct TxLogin<'a> {
    ident: &'a str,
    pwd: &'a str,
    app: &'a str,
}

impl<'a> TxLogin<'a> {
    pub fn new(ident: &'a str, pwd: &'a str, app: &'a str) -> Self {
        TxLogin{
            ident: ident,
            pwd: pwd,
            app: app,
        }
    }

    fn find_user_by_identity(&self) -> Result<Box<dyn user::Ctrl>, Box<dyn Error>> {
        let ctrl: Box<dyn user::Ctrl>;
        if let Ok(_) = match_name(self.ident) {
            ctrl = user::find_by_name(self.ident, false)?;
            return Ok(ctrl);
        } else if let Ok(_) = match_email(self.ident) {
            ctrl = user::find_by_email(self.ident, false)?;
            return Ok(ctrl);
        }
        
        Err(ERR_IDENT_NOT_MATCH.into())
    }

    fn build_signed_token(&self, client_id: i32) -> Result<(Token, String), Box<dyn Error>> {
        let secret: Box<dyn secret::Ctrl> = secret::find_by_client_and_name(client_id, super::DEFAULT_PKEY_NAME)?;
        let token = Token::new(TOKEN_LEN);
        let signed: &[u8] = &secret.sign(self.pwd, token.to_string().as_bytes())?;
        let sign_str = format!("{:X?}", signed);
        Ok((token, sign_str))
    }

    fn build_session(&self, user: Box<dyn user::Ctrl>) -> Result<&mut Box<dyn session::Ctrl>, Box<dyn Error>> {
        let provider = session::get_instance();
        provider.new_session(user)
    }

    fn session_response(&self, session: &Box<dyn session::Ctrl>, token: &str) -> Result<LoginResponse, Box<dyn Error>> {
        match time::unix_seconds(session.get_deadline()) {
            Ok(deadline) => Ok(
                LoginResponse {
                    token: token.to_string(),
                    deadline: deadline,
                    status: session.get_status() as i32,
                }
            ),
    
            Err(err) => Err(err)
        }
    }

    pub fn execute(&self) -> Result<LoginResponse, Box<dyn Error>> {
        println!("Got Login request from user {} ", self.ident);
        let user = self.find_user_by_identity()?;
        let client_id = user.get_client_id();
        let _app = app::find_by_label(self.app, false)?;

        
        let session: &mut Box<dyn session::Ctrl>;
        let provider = session::get_instance();
        if let Ok(sess) = provider.get_session_by_email(&user.get_email()) {
            session = sess;
        } else {
            session = self.build_session(user)?;
        }
        
        println!("Session for user {} got cookie {}", session.get_email(), session.get_cookie());
        let (token, _) = self.build_signed_token(client_id)?;
        let token_str = token.to_string();
        session.set_token(token)?;
        self.session_response(&session, &token_str)
    }
}