use crate::transactions::Tx;
//use crate::transactions::client::regex::{check_name, check_email, check_pwd, check_base64};
use std::any::Any;

pub struct TxLogin<'a> {
    cookie: &'a str,
    name: &'a str,
    addr: &'a str,
    pwd: &'a str,
}

impl<'a> TxLogin<'a> {
    pub fn new(cookie: &'a str, name: &'a str, addr: &'a str, pwd: &'a str) -> Self {
        TxLogin{
            cookie: cookie,
            name: name,
            addr: addr,
            pwd: pwd,
        }
    }

    pub fn execute(&mut self) {
        println!("Got Login request from client {} ", "test")
    }
}