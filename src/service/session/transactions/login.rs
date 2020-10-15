use crate::transaction::traits::Body;
use crate::service::session::transactions::regex::{check_name, check_email, check_pwd, check_base64};

use std::error::Error;
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

impl Body for TxLogin {
    fn precondition(&self) -> Result<(), String> {
        if self.cookie.len() > 0 {
            check_base64(&self.cookie)?;
        }

        // username or email must be provided, both are unic
        if self.name.len() > 0 {
            check_name(&self.name)?;
        } else {
            check_email(&self.addr)?;
        }
        
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