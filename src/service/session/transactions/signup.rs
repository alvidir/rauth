use crate::transaction::traits::{Body, Tx};
use crate::transaction::factory as TxFactory;

use std::error::Error;
use std::any::Any;

pub struct TxSignup {
    name: String,
    addr: String,
    pwd: String,
}

impl TxSignup {
    pub fn new(name: String, addr: String, pwd: String) -> Self {
        TxSignup{
            name: name,
            addr: addr,
            pwd: pwd,
        }
    }
}

impl Body for TxSignup {
    fn precondition(&self) -> Result<(), String> {
        Ok(())
    }

	fn postcondition(&self) -> Result<Box<dyn Any>, String> {
        Ok(Box::new(String::new()))
    }

	fn commit(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

	fn rollback(&self) {

    }

}