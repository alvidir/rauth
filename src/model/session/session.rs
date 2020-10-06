use crate::model::session::traits::*;
use crate::model::client::traits::Client;

use std::any::Any;
use std::error::Error;

pub struct SessionImplementation<'a> {
    pub id: &'a str,
    pub deadline: u64,
    pub creation: u64,

    client: &'a dyn Client,
}

impl SessionImplementation<'_> {
    pub fn new(client: &'static (dyn Client + 'static)) -> Self {
        SessionImplementation{
            id: "hello world",
            deadline: 32,
            creation: 0,
            client: client,
        }
    }
}

impl<'a> Session for SessionImplementation<'a> {
    fn cookie(&self) -> &str {
        self.id
    }

    fn user_id(&self) -> &str {
        self.id
    }

    fn deadline(&self) -> u64 {
        self.deadline
    }

    fn set(&self, key: &str, value: Box<dyn Any>) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Box<dyn Any>, Box<dyn Error>> {
        Ok(Box::new(String::new()))
    }

    fn delete(&self, key: &str) -> Result<(), Box<dyn Error>> {
        Ok(())
    }


}