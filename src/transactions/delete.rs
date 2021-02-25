use std::error::Error;
use crate::models::{user, secret, session, Gateway};

const ERR_IDENT_NOT_MATCH: &str = "The provided indentity is not of the expected type";

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

    fn check_pwd(&self, user: &Box<&dyn user::Ctrl>) -> Result<(), Box<dyn Error>> {
        if !user.match_pwd(self.pwd) {
            return Err("Password does not match".into());
        }

        Ok(())
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        println!("Got a Delete request from user {} ", self.ident);

        let email: String; // required by the session locator
        let client_id: i32; // required by the secret locator
        let user_gw: Box<dyn Gateway>; // required in order to delete the user from de DDBB

        if let Ok(user) = user::find_by_name(self.ident) {
            let ctrl: Box<&dyn user::Ctrl> = Box::new(user.as_ref());
            self.check_pwd(&ctrl)?;

            email = ctrl.get_email().to_string();
            client_id = ctrl.get_client_id();
            user_gw = user;
        } else if let Ok(user) = user::find_by_email(self.ident) {
            email = self.ident.to_string();

            let ctrl: Box<&dyn user::Ctrl> = Box::new(user.as_ref());
            self.check_pwd(&ctrl)?;
            
            client_id = ctrl.get_client_id();
            user_gw = user;
        } else {
            return Err(ERR_IDENT_NOT_MATCH.into())
        }
        
        if let Ok(sess) = session::get_instance().get_by_email(&email) {
            // user has a session
            session::get_instance().destroy_session(sess.get_cookie())?;
        }
        
        if let Ok(secrets) = secret::find_all_by_client(client_id) {
            // user has secrets
            for secret in secrets.iter() {
                secret.delete()?;
            }
        }

        user_gw.delete()
    }
}