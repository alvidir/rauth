use std::any::Any;
use std::error::Error;

pub trait Session {
    fn cookie(&self) -> &str;
    fn userID(&self) -> &str;
    fn deadline(&self) -> u64;
    fn set(&self, key: &str, value: Box<dyn Any>) -> Result<(), Box<dyn Error>>;
    fn get(&self, key: &str) -> Result<Box<dyn Any>, Box<dyn Error>>;
    fn delete(&self, key: &str) -> Result<(), Box<dyn Error>>;
}