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
                self.result = Some(self.body.postcondition());
            }
            Err(err) => {
                self.result = Some(Err(err));
                return;
            }
        }
    }

	fn result(&self) -> Box<Result<Box<dyn Any>, String>> {
        let err : Result<Box<dyn Any>, String> = Err("".to_string());
        let res = self.result.ok_or_else(|| err);
        match res {
            Ok(res) => {
                Box::new(res)
            }

            Err(err) => {
                Box::new(err)
            }
        }
    }
}