use std::error::Error;

pub fn session_login(email: &str, pwd: &str) -> Result<String, Box<dyn Error>> {
    println!("Got login request from user {} ", email);
    Err("Unimplemented".into())
}

pub fn session_logout(cookie: &str) -> Result<(), Box<dyn Error>> {
    println!("Got a logout request for cookie {} ", cookie);
    Err("Unimplemented".into())
}