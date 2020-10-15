use crate::model::client::traits::*;

use std::any::Any;
use std::error::Error;

pub struct ClientImplementation<'a> {
    id: &'a str,
    password: &'a str,
    status: u8,
}

impl<'a> ClientImplementation<'a> {
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

    fn get_status(&self) -> i8 {
        0
    }
}