use std::error::Error;
use super::domain::User;
use super::domain::Session;

pub fn user_signup(email: &str, pwd: &str) -> Result<(), Box<dyn Error>> {
    println!("Got signup request from user {} ", email);
    Err("Unimplemented".into())
}

pub fn user_login(email: &str, secret: &str) -> Result<String, Box<dyn Error>> {
    println!("Got login request from user {} ", email);
    Err("Unimplemented".into())
}

pub fn user_logout(cookie: &str) -> Result<(), Box<dyn Error>> {
    println!("Got a logout request for cookie {} ", cookie);
    Err("Unimplemented".into())
}

pub fn user_delete(email: &str, secret: &str) -> Result<(), Box<dyn Error>> {
    println!("Got a deletion request from user {} ", email);
    Err("Unimplemented".into())
}