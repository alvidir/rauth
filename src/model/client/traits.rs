use std::any::Any;
use std::error::Error;

pub trait ClientSide {
    fn get_endpoint(&self) -> &str;
    fn get_name(&self) -> &str;
}

pub trait Client {
    fn get_id(&self) -> &str;
    fn get_status(&self) -> i8;
}