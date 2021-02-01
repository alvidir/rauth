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

    fn create_user(&self) -> Result<Box<dyn user::Ctrl>, Box<dyn Cause>> {
        match user::User::new(self.name, self.email, self.pwd) {
            Err(err) => {
                let cause = TxCause::new(-1, err.to_string());
                Err(Box::new(cause))
            }

            Ok(user) => Ok(user)
        }
    }

    pub fn execute(&self) -> Result<SessionResponse, Box<dyn Cause>> {
        println!("Got Signup request from user {} ", self.email);
        let user = self.create_user()?;
        
        println!("User {} successfully registered with email {}", self.email, user.get_email());
        match build_session(user) {
            Ok(sess) => {
                let token = get_token(sess)?;
                session_response(&sess, &token)
            },

            Err(cause) => Err(cause)
        }
    }
}

