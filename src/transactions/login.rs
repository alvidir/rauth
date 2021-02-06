use crate::models::user;
use crate::regex::*;
use super::*;

const ERR_IDENT_NOT_MATCH: &str = "The provided indentity is not of the expected type";

pub struct TxLogin<'a> {
    cookie: &'a str,
    ident: &'a str,
    pwd: &'a str,
}

impl<'a> TxLogin<'a> {
    pub fn new(cookie: &'a str, ident: &'a str, pwd: &'a str) -> Self {
        TxLogin{
            cookie: cookie,
            ident: ident,
            pwd: pwd,
        }
    }

    fn find_session_by_cookie(&self) ->  Result<&mut Box<dyn session::Ctrl>, Box<dyn Error>> {
        match_cookie(self.cookie)?;
        find_session(self.cookie)
    }

    fn find_user_by_identity(&self) -> Result<Box<dyn user::Ctrl>, Box<dyn Error>> {
        if let Ok(_) = match_name(self.ident) {
            return user::find_by_name(self.ident);
        } else if let Ok(_) = match_email(self.ident) {
            return user::find_by_email(self.ident);
        }
        
        Err(ERR_IDENT_NOT_MATCH.into())
    }

    fn check_session_liveliness(&self, user: &Box<dyn user::Ctrl>) -> Result<&mut Box<dyn session::Ctrl>, Box<dyn Error>> {
        let provider = session::get_instance();
        provider.get_session_by_email(&user.get_email())
    }

    pub fn execute(&self) -> Result<SessionResponse, Box<dyn Error>> {
        println!("Got Login request from user {} ", self.ident);
        let user = self.find_user_by_identity()?;
        let secret = secret::find_by_client_and_name(user.get_client_id(), "default.pem")?;
        let session: &mut Box<dyn session::Ctrl>;

        if let Ok(sess) = self.find_session_by_cookie() {
            session = sess
        } else if let Ok(sess) = self.check_session_liveliness(&user) {
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