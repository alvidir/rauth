use std::any::Any;

pub mod login;
//pub mod signin;
pub mod signup;
//pub mod logout;

mod regex;

pub trait Tx {
	fn execute(&mut self);
	fn result(&self) -> Option<Result<Box<dyn Any>, String>>;
}