use crate::transaction::traits::{Body};
use crate::service::session::transactions::regex::{check_name, check_email, check_pwd};

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
        check_name(&self.name)?;
        check_email(&self.addr)?;
        check_pwd(&self.pwd)?;
        
        Ok(())
    }

	fn postcondition(&self) -> Option<Result<Box<dyn Any>, String>> {
        None
    }

	fn commit(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

	fn rollback(&self) {

    }

}