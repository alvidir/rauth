//#![allow(unused)]
use std::time::{Duration, SystemTime};
use crate::models::session::token::Token;

pub trait Controller {}

pub struct Gateway {
    deadline: SystemTime,
}

impl Gateway {
    pub fn new(deadline: SystemTime) -> impl Controller {
        Gateway {
            deadline: deadline,
        }  
    }
}

impl Controller for Gateway {
    
}