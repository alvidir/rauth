use crate::transaction::traits::Body;
use crate::transaction::traits::Tx;
use std::any::Any;

pub struct Transaction {
    body: Box<dyn Body>, // transaction's body
    result: Option<Result<Box<dyn Any>, String>>, // transaction's result
}

impl Transaction {
    pub fn new(body: Box<dyn Body>) -> Self {
        Self{
            body: body,
            result: None,
        }
    }

}

impl Tx for Transaction {
    fn execute(&mut self) {
        match self.body.precondition() {
            Ok(_) => {
                self.result = self.body.postcondition();
            }
            Err(err) => {
                self.result = Some(Err(err));
                return;
            }
        }
    }

	fn result(&self) -> &Option<Result<Box<dyn Any>, String>> {
        &self.result
    }
}