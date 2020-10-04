extern crate context;

use context::Context;
use std::error::Error;
use std::any::Any;

pub trait Body {
    fn Precondition() -> Result<(), Box<dyn Error>>;
	fn Postcondition(ctx: Context) -> Result<Box<dyn Any>, Box<dyn Error>>;
	fn Commit() -> Result<(), Box<dyn Error>>;
	fn Rollback(&self);
}

pub trait Tx {
	fn Execute(ctx: Context);
	fn Result() -> Result<Box<dyn Any>, Box<dyn Error>>;
}