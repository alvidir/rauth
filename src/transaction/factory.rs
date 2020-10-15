use crate::transaction::traits::*;
use crate::transaction::transaction::Transaction;

pub fn new_transaction(body: Box<dyn Body>) -> Box<dyn Tx> {
    Box::new(Transaction::new(body))
}