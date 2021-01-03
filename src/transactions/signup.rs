use tonic::{Response, Status};
use crate::models::client::user::*;
use crate::models::client::{Client, Controller as ClientController, Extension};
use crate::models::kind::{KIND_USER, Kind, Controller as KindController};
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

    fn require_user_kind(&self) -> Result<Box<dyn KindController>, Status> {
        match Kind::find_by_name(KIND_USER) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::internal(msg);
                Err(status)
            }

            Ok(kind) => Ok(Box::new(kind))
        }
    }

    fn create_client(&self, kind: Box<dyn KindController>) -> Result<Box<dyn ClientController>, Status> {
        match Client::create(kind.get_id(), self.name, self.pwd) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::internal(msg);
                Err(status)
            }

            Ok(client) => Ok(client)
        }
    }

    fn create_user_client(&self, client: &Box<dyn ClientController>) -> Result<Box<dyn Extension>, Status> {
        match User::create(client.get_id(), self.email) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::internal(msg);
                Err(status)
            }

            Ok(ext) => Ok(Box::new(ext))
        }
    }

    fn extends_from(&self, client: &mut Box<dyn ClientController>, user: Box<dyn Extension>) -> Result<(), Status> {
        match client.set_extension(user) {
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

        let kind = self.require_user_kind()?;
        let mut client = self.create_client(kind)?;
        let user = self.create_user_client(&client)?;
        self.extends_from(&mut client, user)?;
        
        println!("Client {} successfully registered with id {}", self.email, client.get_id());
        let session = build_session(client)?;
        session_response(&session, "")
    }
}