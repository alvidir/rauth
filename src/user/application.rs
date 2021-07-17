use std::error::Error;
use crate::session::domain::SessionRepository;
use crate::metadata::domain::{Metadata, MetadataRepository};
use super::domain::{User, UserRepository};

pub trait EmailManager {
    fn send_verification_email(&self, email: &str) -> Result<(), Box<dyn Error>>;
}

pub fn user_signup<'a>(user_repo: Box<dyn UserRepository>,
                       meta_repo: Box<dyn MetadataRepository>,
                       email_manager: Box<dyn EmailManager>,
                       email: &'a str) -> Result<(), Box<dyn Error>> {
    
    println!("got signup request from user {} ", email);
    
    // the email is required in order to verify the identity of the user, so if no email
    // can be sent, the user shall not be created
    email_manager.send_verification_email(email)?;

    let meta = Metadata::new(meta_repo)?;
    User::new(user_repo, meta, email)?;
    Ok(())
}

pub fn user_delete<'a>(user_repo: Box<dyn UserRepository>,
                       sess_repo: Box<dyn SessionRepository>,
                       email: &'a str) -> Result<(), Box<dyn Error>> {
    
    println!("got a deletion request from user {} ", email);
    
    let user = user_repo.find(email)?;
    
    if let Ok(sess_arc) = sess_repo.find_by_email(email) {
        if let Ok(sess) = sess_arc.lock() {
            sess_repo.delete(&sess)?;
        }
    }

    user_repo.delete(&user)?;
    Ok(())
}