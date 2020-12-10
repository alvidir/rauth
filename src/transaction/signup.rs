use crate::transaction::Tx;
//use crate::transactions::client::regex::{check_name, check_email, check_pwd};

use std::any::Any;

pub struct TxSignup {
    name: String,
    addr: String,
    pwd: String,
}

impl TxSignup {
    pub fn new(name: String, addr: String, pwd: String) -> Box<dyn Tx> {
        let signup = TxSignup{
            name: name,
            addr: addr,
            pwd: pwd,
        };

        Box::new(signup)
    }
}

impl Tx for TxSignup {
    fn execute(&mut self) {
        println!("Got Signup request from client {} ", "test")
    }

	fn result(&self) -> Option<Result<Box<dyn Any>, String>> {
        Some(Ok(Box::new(())))
    }
}