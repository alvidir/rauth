use std::error::Error;
use crate::session::domain::SessionRepository;
use crate::metadata::domain::MetadataRepository;
use super::domain::{UserRepository, User};

pub trait EmailSender {
    fn send_verification_email(&self, email: &str) -> Result<(), Box<dyn Error>>;
}

pub fn user_signup<'a>(user_repo: Box<dyn UserRepository>,
                       meta_repo: Box<dyn MetadataRepository>,
                       sender: Box<dyn EmailSender>,
                       email: &'a str) -> Result<(), Box<dyn Error>> {
    
    println!("Got signup request from user {} ", email);
    
    // the email is required in order to verify the identity of the user, so if no email
    // can be sent, the user shall not be created
    sender.send_verification_email(email)?;
    User::new(user_repo, meta_repo, email)?;
    Ok(())
}

pub fn user_delete<'a>(user_repo: Box<dyn UserRepository>,
                       sess_repo: Box<dyn SessionRepository>,
                       email: &'a str,
                       pwd: &'a str) -> Result<(), Box<dyn Error>> {
    
    println!("got a deletion request from user {} ", email);
    
    let user = user_repo.find(email)?;
    if let Some(secret) = &user.secret {
        // TODO: check provided TOTP does match for the user
    }

    if let Ok(sess_arc) = sess_repo.find_by_email(email) {
        if let Ok(sess) = sess_arc.lock() {
            sess_repo.delete(&sess)?;
        } else {
            println!("session's mutex for user {} got poisoned", email);
            return Err("mutex poisoned".into());
        }
    } else {
        println!("user {} has no session", email);
    }

    user_repo.delete(&user)?;
    Err("Unimplemented".into())
}