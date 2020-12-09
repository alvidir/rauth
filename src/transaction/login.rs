use crate::transaction::Tx;
//use crate::transactions::client::regex::{check_name, check_email, check_pwd, check_base64};
use std::any::Any;

pub struct TxLogin {
    cookie: String,
    name: String,
    addr: String,
    pwd: String,
}

impl TxLogin {
    pub fn new(cookie: String, name: String, addr: String, pwd: String) -> Self {
        TxLogin{
            cookie: cookie,
            name: name,
            addr: addr,
            pwd: pwd,
        }
    }
}

impl Tx for TxLogin {
    fn execute(&mut self) {
        println!("Got Login request from client {} ", "test")
    }
    
	fn result(&self) -> Option<Result<Box<dyn Any>, String>> {
        Some(Ok(Box::new(())))
    }
}