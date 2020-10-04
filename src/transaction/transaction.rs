extern crate context;

use crate::transaction::traits::Body;
use crate::transaction::traits::Tx;
use context::Context;
use std::error::Error;
use std::any::Any;

struct Transaction<T: Body> {
    checked: bool, // determines if the precondition has passed of not
    body: T, // transaction's body
    result: Box<dyn Any>, // transaction's result
    err: Box<dyn Error>,
}

impl<t: Body> Tx for Transaction<dyn Body> {
    fn Execute<T: ?Sized>(t: &T, ctx: Context) {

    }

	fn Result<T: ?Sized>(t: &T) -> Result<Box<dyn Any>, Box<dyn Error>> {
        return;
    }
}