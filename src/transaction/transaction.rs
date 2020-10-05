extern crate context;

use crate::transaction::traits::Body;
use crate::transaction::traits::Tx;
use std::any::Any;

pub struct Transaction<'a> {
    body: &'a dyn Body, // transaction's body
    result: Option<Result<Box<dyn Any>, String>>, // transaction's result
}

impl <'a> Transaction<'a> {
    pub fn new(body: &'a (dyn Body + 'static)) -> Self {
        Self{
            body: body,
            result: None,
        }
    }

}

impl Tx for Transaction<'_> {
    fn execute(&mut self) {
        match self.body.precondition() {
            Ok(_) => {
                self.result = Some(self.body.postcondition());
            }
            Err(err) => {
                self.result = Some(Err(err));
                return;
            }
        }
    }

	fn result(&self) -> Result<Box<dyn Any>, String> {
        match &self.result {
            Some(_) => {
                Ok(Box::new(""))
            }
            None => {
                Err(String::from(""))
            }
        }
    }
}