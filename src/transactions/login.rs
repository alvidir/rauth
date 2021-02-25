use std::error::Error;
use crate::models::{user, app};
use crate::regex::*;
use crate::time;
use crate::token::Token;
use crate::models::{secret, session, namesp};
use crate::models::app::Ctrl;

// Proto message structs
use crate::proto::user_proto;
use user_proto::LoginResponse;

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
            ctrl = user::find_by_name(self.ident)?;
            return Ok(ctrl);
        } else if let Ok(_) = match_email(self.ident) {
            ctrl = user::find_by_email(self.ident)?;
            return Ok(ctrl);
        }
        
        Err(ERR_IDENT_NOT_MATCH.into())
    }

    fn build_session(&self, user: Box<dyn user::Ctrl>) -> Result<&mut Box<dyn session::Ctrl>, Box<dyn Error>> {
        let provider = session::get_instance();
        provider.new_session(user)
    }

    //fn session_response(&self, session: &Box<dyn session::Ctrl>, token: &str) -> Result<LoginResponse, Box<dyn Error>> {
    //    match time::unix_seconds(session.get_deadline()) {
    //        Ok(deadline) => Ok(
    //            LoginResponse {
    //                token: token.to_string(),
    //                deadline: deadline,
    //                status: session.get_status() as i32,
    //            }
    //        ),
    //
    //        Err(err) => Err(err)
    //    }
    //}

    pub fn execute(&self) -> Result<LoginResponse, Box<dyn Error>> {
        println!("Got Login request from user {} ", self.ident);
        let user = self.find_user_by_identity()?;
        let client_id = user.get_client_id();
        let app = app::find_by_label(self.app)?;
        let secret = secret::find_by_client_and_name(app.get_client_id(), super::register::DEFAULT_PKEY_NAME)?;
        
        let session: &mut Box<dyn session::Ctrl>;
        let provider = session::get_instance();
        if let Ok(sess) = provider.get_by_email(&user.get_email()) {
            session = sess;
        } else {
            session = self.build_session(user)?;
        }
        
        println!("Session for user {} got cookie {}", session.get_email(), session.get_cookie());
        //let token_str = session.get_token()?;
        //self.session_response(&session, &token_str)
        Err("".into())
    }
}