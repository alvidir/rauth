use crate::transaction::traits::Body;
use std::error::Error;
use std::any::Any;

pub struct TxLogin {}

impl Body for TxLogin {
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