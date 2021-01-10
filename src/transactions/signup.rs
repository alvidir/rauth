use tonic::{Response, Status};
use crate::models::client::User;
use crate::models::client::Controller as ClientController;
use crate::transactions::*;

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

    fn create_user_client(&self) -> Result<Box<dyn ClientController>, Status> {
        match User::create(self.name, self.email, self.pwd) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::internal(msg);
                Err(status)
            }

            Ok(client) => Ok(client)
        }
    }

    pub fn execute(&self) -> Result<Response<SessionResponse>, Status> {
        println!("Got Signup request from client {} ", self.email);
        let client = self.create_user_client()?;
        
        println!("Client {} successfully registered with id {}", self.email, client.get_id());
        let session = build_session(client)?;
        session_response(&session, "")
    }
}