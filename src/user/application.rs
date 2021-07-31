use std::error::Error;
use crate::session::domain::SessionRepository;
use crate::metadata::domain::{Metadata, MetadataRepository};
use super::domain::{User, UserRepository};

pub fn user_signup<'a>(user_repo: &dyn UserRepository,
                       meta_repo: &dyn MetadataRepository,
                       email: &'a str,
                       password: &'a str) -> Result<(), Box<dyn Error>> {
    
    info!("got signup request from user {} ", email);
    
    let meta = Metadata::new(meta_repo)?;
    User::new(user_repo, meta, email, password)?;
    Ok(())
}

pub fn user_delete<'a>(user_repo: Box<dyn UserRepository>,
                       sess_repo: Box<dyn SessionRepository>,
                       email: &'a str) -> Result<(), Box<dyn Error>> {
    
    info!("got a deletion request from user {} ", email);
    
    let user = user_repo.find(email)?;
    
    if let Ok(sess_arc) = sess_repo.find_by_email(email) {
        if let Ok(sess) = sess_arc.read() {
            sess_repo.delete(&sess)?;
        }
    }

    user_repo.delete(&user)?;
    Ok(())
}