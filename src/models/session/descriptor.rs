use std::error::Error;
use std::time::{Duration, SystemTime};
use crate::models::session::token::Token;

pub trait Controller {
    fn get_deadline(&self) -> SystemTime;
}

pub struct Descriptor {
    deadline: SystemTime,
}

impl Descriptor {
    pub fn new(deadline: SystemTime) -> impl Controller {
        Descriptor {
            deadline: deadline,
        }  
    }
}

impl Controller for Descriptor {
    fn get_deadline(&self) -> SystemTime {
        self.deadline
    }

}