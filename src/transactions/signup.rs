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

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        println!("Got Signup request from user {} ", self.email);
        let user = user::User::new(self.name, self.email)?;
        println!("User {} successfully registered with email {}", self.name, self.email);
        secret::Secret::new(user.get_client_id(), super::DEFAULT_PKEY_NAME, self.pwd)?;
        Ok(())
    }
}

