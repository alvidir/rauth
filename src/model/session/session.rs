use crate::model::session::traits::*;
use crate::model::client::traits::Client;

use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::any::Any;
use std::error::Error;

pub struct SessionImplementation<'a> {
    id: &'a str,
    deadline: Duration,
    creation: Duration,
    client: &'a dyn Client,
}

impl<'a> SessionImplementation<'a> {
    pub fn new(client: &'a (dyn Client + 'static), timeout: Duration) -> Self {
        let now = SystemTime::now();
        let now_unix = now
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        SessionImplementation{
            id: "hello world",
            deadline: now_unix + timeout,
            creation: now_unix,
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

    fn deadline(&self) -> Duration {
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