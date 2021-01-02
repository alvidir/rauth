use std::error::Error;
use crate::models::client::user::*;
use crate::models::client::{Client, Controller as ClientController};
use crate::models::session::{Session, Controller as SessionController};
use crate::models::session::provider as SessionProvider;
//use crate::transactions::client::regex::{check_name, check_email, check_pwd};

const ERR_USER_ALREADY_EXISTS: &str = "Already exists an user with email:";

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

    pub fn execute(&self) -> Result<&Box<dyn SessionController>, Box<dyn Error>> {
        println!("Got Signup request from client {} ", self.email);

        match User::find_by_email(self.email) {
            Ok(_) => {
                let msg = format!("{} {}", ERR_USER_ALREADY_EXISTS, self.email);
                Err(msg.into())
            }

            Err(_) => {
                let mut client = Client::create(self.name, self.pwd)?;
                let user = User::create(client.get_id(), self.email)?.build();
                client.set_extension(Box::new(user))?;
                println!("Client {} successfully registered with id {}", self.email, client.get_id());
                //Ok(client)

                //let client = self.create_user_client()?;
                let provider = SessionProvider::get_instance();
                let session = provider.new_session(client)?;
                println!("Session for client {} has cookie {}", self.email, session.get_cookie());
                Ok(session)
            }
        }
    }
}