use crate::transaction::traits::*;
use crate::transaction::transaction::Transaction;

pub fn new_transaction(body: &'static (dyn Body + 'static)) -> Box<dyn Tx> {
    Box::new(Transaction::new(body))
}