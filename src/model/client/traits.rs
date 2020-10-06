use std::any::Any;
use std::error::Error;

pub trait Client {
    fn get_id(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_status(&self) -> i8;
    fn get_endpoint(&self) -> &str;
    fn public_key(&self, id: &str) -> Result<Box<&str>, Box<dyn Error>>;
}