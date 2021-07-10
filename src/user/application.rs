use std::error::Error;

pub trait EmailSender {
    fn send_verification_email(email: &str) -> Result<(), Box<dyn Error>>;
}

pub fn user_signup(email: &str, pwd: &str) -> Result<(), Box<dyn Error>> {
    println!("Got signup request from user {} ", email);
    

    Err("Unimplemented".into())
}

pub fn user_delete(email: &str, secret: &str) -> Result<(), Box<dyn Error>> {
    println!("Got a deletion request from user {} ", email);
    Err("Unimplemented".into())
}