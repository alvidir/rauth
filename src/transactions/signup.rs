use crate::models::session::{Session, Controller as SessionController};
//use crate::transactions::client::regex::{check_name, check_email, check_pwd};

pub struct TxSignup<'a> {
    name: String,
    addr: String,
    pwd: String,
    result: Option<Result<&'a Session, String>>,
}

impl<'a> TxSignup<'a> {
    pub fn new(name: String, addr: String, pwd: String) -> Self {
        let signup = TxSignup{
            name: name,
            addr: addr,
            pwd: pwd,
            result: None,
        };

        signup
    }

    pub fn execute(&mut self) {
        println!("Got Signup request from client {} ", "test")
    }

	pub fn result(&self) -> Option<Result<&'a dyn SessionController, String>> {
        match self.result.as_ref()? {
            Ok(sess) => {
                Some(Ok(*sess))
            }

            Err(msg) => {
                Some(Err(msg.to_string()))
            }
        }
    }
}