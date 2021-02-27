use std::error::Error;
use crate::models::{user, session, namesp, Gateway};
use crate::regex::*;
use crate::mongo;

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

    fn clear_user_data(&self, user: Box<&dyn user::Ctrl>) -> Result<i32, Box<dyn Error>> {
        if !user.match_pwd(self.pwd) {
            return Err(ERR_PWD_NOT_MATCH.into());
        }

        if let Some(sess) = session::get_instance().get_by_email(user.get_email()) {
            // user has session            
            for token in sess.get_open_dirs() {
                // foreach loged-in application
                if let Some(app_id) = sess.delete_directory(&token) {
                    if let Some(np) = namesp::get_instance().get_by_id(app_id) {
                        // application is using a namespace
                        np.delete_cookie(sess.get_cookie());
                    }
                }
            }

            session::get_instance().destroy_session(&sess.get_cookie())?;
        }

        Ok(user.get_id())
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        println!("Got a Account deletion request from user {} ", self.ident);

        let user_gw: Box<dyn Gateway>;
        let user_id: i32;
        if let Ok(_) = match_name(self.ident) {
            let user = user::find_by_name(self.ident)?;
            user_id = self.clear_user_data(Box::new(user.as_ref()))?;
            user_gw = user;
        } else if let Ok(_) = match_email(self.ident) {
            let user = user::find_by_email(self.ident)?;
            user_id = self.clear_user_data(Box::new(user.as_ref()))?;
            user_gw = user;
        } else {
            return Err(ERR_IDENT_NOT_MATCH.into());
        }

        let coll_name = mongo::get_collection_name()?;
        let delete_result = mongo::open_stream(&coll_name).delete_many(
            doc! {
               "user_id": user_id,
            },
            None,
        )?;

        println!("Deleted {} documents of user {}", delete_result.deleted_count, self.ident);
        user_gw.delete()?;
        Ok(())
    }
}