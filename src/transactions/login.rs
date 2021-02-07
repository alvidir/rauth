use crate::models::{user, app};
use crate::regex::*;
use super::*;

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
        if let Ok(_) = match_name(self.ident) {
            return user::find_by_name(self.ident);
        } else if let Ok(_) = match_email(self.ident) {
            return user::find_by_email(self.ident);
        }
        
        Err(ERR_IDENT_NOT_MATCH.into())
    }

    pub fn execute(&self) -> Result<SessionResponse, Box<dyn Error>> {
        println!("Got Login request from user {} ", self.ident);
        let user = self.find_user_by_identity()?;
        //let app = app::find_by_label(self.app)?;

        let secret = secret::find_by_client_and_name(user.get_client_id(), super::DEFAULT_PKEY_NAME)?;
        let session: &mut Box<dyn session::Ctrl>;

        let provider = session::get_instance();
        if let Ok(sess) = provider.get_session_by_email(&user.get_email()) {
            session = sess;
        } else {
            session = build_session(user)?;
        }
        
        println!("Session for user {} got cookie {}", session.get_email(), session.get_cookie());
        let (token, _) = build_signed_token(secret, self.pwd)?;
        let token_str = token.to_string();
        session.set_token(token)?;
        session_response(&session, &token_str)
    }
}