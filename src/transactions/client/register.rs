use crate::models::client::User;
use crate::models::client::Controller as ClientController;
use crate::transactions::client::*;

pub struct TxRegister<'a> {
    name: &'a str,
    url: &'a str,
    descr: &'a str,
    key: &'a str,
    email: &'a str,
    pwd: &'a str,
}

impl<'a> TxRegister<'a> {
    pub fn new(name: &'a str, url: &'a str, descr: &'a str, key: &'a str, email: &'a str, pwd: &'a str) -> Self {
        let signup = TxRegister{
            name: name,
            url: url,
            descr: descr,
            key: key,
            email: email,
            pwd: pwd,
        };

        signup
    }

    fn create_user_client(&self) -> Result<Box<dyn ClientController>, Box<dyn Cause>> {
        match User::create(self.name, self.email, self.pwd) {
            Err(err) => {
                let cause = TxCause::new(-1, err.to_string());
                Err(Box::new(cause))
            }

            Ok(client) => Ok(client)
        }
    }

    pub fn execute(&self) -> Result<SessionResponse, Box<dyn Cause>> {
        println!("Got Signup request from client {} ", self.email);
        let client = self.create_user_client()?;
        
        println!("Client {} successfully registered with id {}", self.email, client.get_id());
        match build_session(client) {
            Ok(sess) => {
                let token = ephimeral_token(sess)?;
                session_response(&sess, &token)
            },

            Err(cause) => Err(cause)
        }
    }
}

