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

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        println!("Got a DeleteAccount request from user {} ", self.ident);

        let email: String; // required by the session locator
        let client_id: i32; // required by the secret locator
        let gw: Box<dyn Gateway>; // required in order to delete the user from de DDBB

        if let Ok(stream) = user::find_by_name(self.ident, false) {
            let ctrl: Box<&dyn user::Ctrl> = Box::new(stream.as_ref());
            email = ctrl.get_email().to_string();
            client_id = ctrl.get_client_id();
            gw = stream;
        } else if let Ok(stream) = user::find_by_email(self.ident, false) {
            email = self.ident.to_string();
            let ctrl: Box<&dyn user::Ctrl> = Box::new(stream.as_ref());
            client_id = ctrl.get_client_id();
            gw = stream;
        } else {
            return Err(ERR_IDENT_NOT_MATCH.into())
        }

        let secret_stream = secret::find_by_client_and_name(client_id, super::DEFAULT_PKEY_NAME)?;
        let secret_ctrl: Box<&dyn secret::Ctrl> = Box::new(secret_stream.as_ref());
        secret_ctrl.sign(self.pwd, b"")?; // by the moment, is not required to sign anything, just verfies the password

        // From here, the deletion is aproved
        // Deleting the secret in order to avoid sql-exceptions when deleting the client
        let secret_gw: Box<&dyn Gateway> = Box::new(secret_stream.as_ref());
        secret_gw.delete()?;
        
        if let Ok(sess) = session::get_instance().get_session_by_email(&email) {
            // user has a session
            session::get_instance().destroy_session(sess.get_cookie())?;
        }
        
        if let Ok(secrets) = secret::find_all_by_client(client_id) {
            // user has more secrets
            for secret in secrets.iter() {
                let gw: Box<&dyn Gateway> = Box::new(secret);
                gw.delete()?;
            }
        }

        gw.delete()
    }
}