use crate::model::app::traits::*;

use std::any::Any;
use std::error::Error;

use crate::model::client::traits::ClientSide;

pub struct AppImplementation<'a> {
    name: &'a str,
    addr: &'a str,
}

impl<'a> AppImplementation<'a> {
    pub fn new(name: &'a str, addr: &'a str) -> Self {
        AppImplementation{
            name: &name,
            addr: &addr,
        }
    }
}

impl<'a> App for AppImplementation<'a> {
    fn get_name(&self) -> &str {
        self.get_name()
    }

    fn get_endpoint(&self) -> &str {
        self.get_endpoint()
    }
}