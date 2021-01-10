use std::error::Error;

pub mod session;
pub mod client;
mod kind;

pub trait Gateway {
    fn insert(&self) -> Result<(), Box<dyn Error>>;
    fn update(&self) -> Result<(), Box<dyn Error>>;
    fn delete(&self) -> Result<(), Box<dyn Error>>;
}