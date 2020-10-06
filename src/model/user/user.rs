use crate::model::user::traits::*;

use std::any::Any;
use std::error::Error;

use crate::model::client::traits::ClientSide;

pub struct UserImplementation<'a> {
    pub nickname: &'a str,
    pub email: &'a str,
}

impl<'a> UserImplementation<'a> {
    pub fn new(name: &'a str, email: &'a str) -> Self {
        UserImplementation{
            nickname: &name,
            email: &email,
        }
    }
}

impl<'a> User for UserImplementation<'a> {}

impl<'a> ClientSide for UserImplementation<'a> {
    fn get_name(&self) -> &str {
        self.nickname
    }

    fn get_endpoint(&self) -> &str {
        self.email
    }
}