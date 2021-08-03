use std::error::Error;
use crate::metadata::domain::{Metadata, MetadataRepository};
use crate::security;
use crate::constants::errors;
use super::domain::{User, UserRepository};

pub fn user_signup(user_repo: &dyn UserRepository,
                    meta_repo: &dyn MetadataRepository,
                    email: &str,
                    password: &str) -> Result<(), Box<dyn Error>> {
    
    info!("got signup request from user {} ", email);
    
    let meta = Metadata::new(meta_repo)?;
    User::new(user_repo, meta, email, password)?;
    Ok(())
}

pub fn user_delete(user_repo: &dyn UserRepository,
                    email: &str,
                    pwd: &str,
                    totp: &str) -> Result<(), Box<dyn Error>> {
    
    info!("got a deletion request from user {} ", email);

    let user = user_repo.find(email)?;
    if !user.match_password(pwd) {
        return Err(errors::NOT_FOUND.into());
    }

    // if, and only if, the user has activated the 2fa
    if let Some(secret) = &user.secret {
        let data = secret.get_data();
        security::verify_totp(data, totp)?;
    }

    user_repo.delete(&user)?;
    Ok(())
}