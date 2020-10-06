use crate::model::client::traits::*;

use std::any::Any;
use std::error::Error;

pub struct ClientImplementation<'a> {
    pub id: &'a str,
    pub password: &'a str,
    pub status: u8,
}

impl ClientImplementation<'_> {
    pub fn new() -> Self {
        ClientImplementation{
            id: "hello world",
            password: "password",
            status: 0,
        }
    }
}

impl<'a> Client for ClientImplementation<'a> {
    fn get_id(&self) -> &str {
        "hello world"
    }

    fn get_name(&self) -> &str {
        "hello world"
    }

    fn get_status(&self) -> i8 {
        0
    }

    fn get_endpoint(&self) -> &str {
        "hello world"
    }

    fn public_key(&self, id: &str) -> Result<Box<&str>, Box<dyn Error>> {
        Ok(Box::new(""))
    }

}