use std::error::Error;
use std::time::{Duration, SystemTime};
use crate::models::session::token::Token;

pub trait Controller {
    fn get_token(&self) -> String;
    fn get_deadline(&self) -> SystemTime;
}

pub struct Descriptor {
    token: Token,
    deadline: SystemTime,
}

impl Descriptor {
    pub fn new(token: Token, deadline: SystemTime) -> impl Controller {
        Descriptor {
            token: token,
            deadline: deadline,
        }  
    }
}

impl Controller for Descriptor {
    fn get_token(&self) -> String {
        self.token.to_string()
    }

    fn get_deadline(&self) -> SystemTime {
        self.deadline
    }

}