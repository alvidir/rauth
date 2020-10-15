use crate::transaction::traits::Tx;
use crate::transaction::factory as TxFactory;
use crate::service::session::transactions::{login::TxLogin,
                                            signup::TxSignup};

pub fn new_tx_signup(name: String, addr: String, pwd: String) -> Box<dyn Tx> {
    let body = TxSignup::new(name, addr, pwd);
    TxFactory::new_transaction(Box::new(body))
}

pub fn new_tx_login(cookie: String, name: String, addr: String, pwd: String) -> Box<dyn Tx> {
    let body = TxLogin::new(cookie, name, addr, pwd);
    TxFactory::new_transaction(Box::new(body))
}