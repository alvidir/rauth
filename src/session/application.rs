use std::error::Error;

pub fn _session_login(email: &str, _pwd: &str) -> Result<String, Box<dyn Error>> {
    println!("Got login request from user {} ", email);
    Err("Unimplemented".into())
}

pub fn _session_logout(cookie: &str) -> Result<(), Box<dyn Error>> {
    println!("Got a logout request for cookie {} ", cookie);
    Err("Unimplemented".into())
}