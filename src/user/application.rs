use std::error::Error;
use crate::session::domain::SessionRepository;
use crate::metadata::domain::{Metadata, MetadataRepository};
use super::domain::{User, UserRepository};

pub fn user_signup<'a>(user_repo: Box<dyn UserRepository>,
                       meta_repo: Box<dyn MetadataRepository>,
                       email: &'a str) -> Result<(), Box<dyn Error>> {
    
    println!("got signup request from user {} ", email);
    
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