use std::error::Error;
use crate::models::user;
use super::*;

pub struct TxSignup<'a> {
    name: &'a str,
    email: &'a str,
    pwd: &'a str,
}

impl<'a> TxSignup<'a> {
    pub fn new(name: &'a str, email: &'a str, pwd: &'a str) -> Self {
        let signup = TxSignup{
            name: name,
            email: email,
            pwd: pwd,
        };

        signup
    }

    pub fn execute(&self) -> Result<SessionResponse, Box<dyn Error>> {
        println!("Got Signup request from user {} ", self.email);
        let user = user::User::new(self.name, self.email)?;
        let key = secret::Secret::new(user.get_client_id(), "default.pem", self.pwd)?;
        
        match build_session(user) {
            Ok(sess) => {
                let (token, _) = build_signed_token(Box::new(key), self.pwd)?;
                let token_str = token.to_string();
                sess.set_token(token)?;
                println!("User {} successfully registered with token {}", self.email, token_str);
                session_response(&sess, &token_str)
            },

            Err(cause) => Err(cause)
        }
    }
}

