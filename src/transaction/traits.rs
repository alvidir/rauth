use std::error::Error;
use std::any::Any;

pub trait Body {
    fn precondition(&self) -> Result<(), String>;
	fn postcondition(&self) -> Option<Result<Box<dyn Any>, String>>;
	fn commit(&self) -> Result<(), Box<dyn Error>>;
	fn rollback(&self);
}

pub trait Tx {
	fn execute(&mut self);
	fn result(&self) -> &Option<Result<Box<dyn Any>, String>>;
}