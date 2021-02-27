use std::error::Error;
use crate::models::{user, app};
use crate::regex::*;
use crate::token::Token;
use crate::models::{session, namesp};

// Proto message structs
use crate::proto::user_proto;
use user_proto::LoginResponse;

const ERR_IDENT_NOT_MATCH: &str = "The provided indentity is not of the expected type";
const ERR_PWD_NOT_MATCH: &str = "The provided password does not match";

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

    fn find_sess_by_identity(&self) -> Option<&mut Box<dyn session::Ctrl>> {
        if let Ok(_) = match_name(self.ident) {
            let sess = session::get_instance().get_by_name(self.ident)?;
            Some(sess)
        } else if let Ok(_) = match_email(self.ident) {
            let sess = session::get_instance().get_by_email(self.ident)?;
            Some(sess)
        } else {
            None
        }
    }

    fn find_user_by_identity(&self) -> Result<Box<dyn user::Ctrl>, Box<dyn Error>> {
        if let Ok(_) = match_name(self.ident) {
            let ctrl = user::find_by_name(self.ident)?;
            return Ok(ctrl);
        } else if let Ok(_) = match_email(self.ident) {
            let ctrl = user::find_by_email(self.ident)?;
            return Ok(ctrl);
        }
        
        Err(ERR_IDENT_NOT_MATCH.into())
    }

    fn session_response(&self, sess: &Box<dyn session::Ctrl>, token: &Token) -> LoginResponse {
        LoginResponse {
            cookie: format!("{}{}", sess.get_cookie(), token),
            status: sess.get_status() as i32,
        }
    }

    pub fn execute(&self) -> Result<LoginResponse, Box<dyn Error>> {
        println!("Got Login request from user {} ", self.ident);
        if let Some(sess) = self.find_sess_by_identity() {
            // user has session
            if !sess.match_pwd(self.pwd) {
                return Err(ERR_PWD_NOT_MATCH.into());
            }

            // password does match
            if let Some(token) = sess.get_token(self.app) {
                // user is currently loged in the application
                return Ok(self.session_response(sess, token));
            }

            // user is not loged in the application
            let token = sess.new_directory(self.app)?;

            if let Some(np) = namesp::get_instance().get_by_label(self.app) {
                // application is using a namespace
                let resp = self.session_response(sess, &token);
                np.set_cookie(sess.get_cookie().clone(), token)?;
                return Ok(resp);
            }

            // application has no namespace
            let app = app::find_by_label(self.app)?;
            let np = namesp::get_instance().new_namespace(app)?;
            let resp = self.session_response(sess, &token);
            np.set_cookie(sess.get_cookie().clone(), token)?;
            return Ok(resp);
        }

        // user has no session
        let user = self.find_user_by_identity()?;
        if !user.match_pwd(self.pwd) {
            return Err(ERR_PWD_NOT_MATCH.into());
        }

        let sess = session::get_instance().new_session(user)?;
        let token = sess.new_directory(self.app)?;

        if let Some(np) = namesp::get_instance().get_by_label(self.app) {
            // application is using a namespace
            let resp = self.session_response(sess, &token);
            np.set_cookie(sess.get_cookie().clone(), token)?;
            return Ok(resp);
        }

        // application has no namespace
        let app = app::find_by_label(self.app)?;
        let np = namesp::get_instance().new_namespace(app)?;
        let resp = self.session_response(sess, &token);
        np.set_cookie(sess.get_cookie().clone(), token)?;
        return Ok(resp);
    }
}