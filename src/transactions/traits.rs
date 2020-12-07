use std::any::Any;

pub trait Tx {
	fn execute(&mut self);
	fn result(&self) -> Option<Result<Box<dyn Any>, String>>;
}