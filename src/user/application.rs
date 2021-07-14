use std::error::Error;
use super::domain;

pub trait EmailSender {
    fn send_verification_email(&self, email: &str) -> Result<(), Box<dyn Error>>;
}

pub fn user_signup<'a>(sender: impl EmailSender, repo: impl domain::UserRepository, email: &'a str) -> Result<(), Box<dyn Error>> {
    println!("Got signup request from user {} ", email);
    
    // the email is required in order to verify the identity of the user, so if no email
    // can be sent, the user shall not be created
    sender.send_verification_email(email)?;
    domain::User::new(repo, email)?;
    Ok(())
}

pub fn user_delete<'a>(email: &'a str, pwd: &'a str) -> Result<(), Box<dyn Error>> {
    println!("Got a deletion request from user {} ", email);
    Err("Unimplemented".into())
}