use std::error::Error;
use crate::models::client::user::*;
use crate::models::client::{Client, Controller as ClientController};
use crate::models::kind::{KIND_USER, Kind, Controller as KindController};

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

    pub fn execute(&self) -> Result<Box<dyn ClientController>, Box<dyn Error>> {
        println!("Got Signup request from client {} ", self.email);

        let kind = Kind::find_by_name(KIND_USER)?;
        let mut client = Client::create(kind.get_id(), self.name, self.pwd)?;
        let user = User::create(client.get_id(), self.email)?;
        client.set_extension(Box::new(user))?;
        println!("Client {} successfully registered with id {}", self.email, client.get_id());
        Ok(client)
    }
}