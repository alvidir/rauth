use crate::token::Token;
use std::error::Error;
use crate::models::{user, session, namesp, Gateway};
use crate::models::namesp::Ctrl as NpCtrl;
use crate::regex::*;

const ERR_IDENT_NOT_MATCH: &str = "The provided indentity is not of the expected type";
const ERR_PWD_NOT_MATCH: &str = "The provided password does not match";

pub struct TxDelete<'a> {
    ident: &'a str,
    pwd: &'a str,
}

impl<'a> TxDelete<'a> {
    pub fn new(ident: &'a str, pwd: &'a str) -> Self {
        TxDelete{
            ident: ident,
            pwd: pwd,
        }
    }

    fn clear_user_data(&self, user: Box<&dyn user::Ctrl>) -> Result<(), Box<dyn Error>> {
        if !user.match_pwd(self.pwd) {
            return Err(ERR_PWD_NOT_MATCH.into());
        }

        if let Some(sess) = session::get_instance().get_by_email(user.get_email()) {
            // user has session
            for (token, label) in sess.get_tokens_iter() {
                // foreach loged-in application
                if let Some(np) = namesp::get_instance().get_by_label(label) {
                    // application is using a namespace
                    if let Ok(gw) = np.delete_directory(token) {
                        // namespace had a dir for the provided token
                        gw.delete()?;
                    }
                }
            }

            let token = sess.get_cookie();
            session::get_instance().destroy_session(&token)?;
        }

        Ok(())
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        println!("Got a Delete request from user {} ", self.ident);

        let user_gw: Box<dyn Gateway>;
        if let Ok(_) = match_name(self.ident) {
            let user = user::find_by_name(self.ident)?;
            self.clear_user_data(Box::new(user.as_ref()))?;
            user_gw = user;
        } else if let Ok(_) = match_email(self.ident) {
            let user = user::find_by_email(self.ident)?;
            self.clear_user_data(Box::new(user.as_ref()))?;
            user_gw = user;
        } else {
            return Err(ERR_IDENT_NOT_MATCH.into());
        }

        user_gw.delete()?;
        Ok(())
    }
}