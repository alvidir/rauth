use crate::model::app::traits::*;

use std::any::Any;
use std::error::Error;

use crate::model::client::traits::ClientSide;

pub struct AppImplementation<'a> {
    pub name: &'a str,
    pub addr: &'a str,
}

impl<'a> AppImplementation<'a> {
    pub fn new(name: &'a str, addr: &'a str) -> Self {
        AppImplementation{
            name: &name,
            addr: &addr,
        }
    }
}

impl<'a> App for AppImplementation<'a> {}

impl<'a> ClientSide for AppImplementation<'a> {
    fn get_name(&self) -> &str {
        self.name
    }

    fn get_endpoint(&self) -> &str {
        self.addr
    }
}