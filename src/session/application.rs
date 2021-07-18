use std::error::Error;
use std::time::Duration;

use crate::user::domain::UserRepository;
use crate::app::domain::AppRepository;
use crate::metadata::domain::{Metadata, MetadataRepository};
use crate::constants;
use super::domain::{Session, SessionRepository};

pub fn session_login(sess_repo: Box<dyn SessionRepository>,
                      user_repo: Box<dyn UserRepository>,
                      app_repo: Box<dyn AppRepository>,
                      meta_repo: Box<dyn MetadataRepository>,
                      email: &str, app: &str) -> Result<String, Box<dyn Error>> {
    
    println!("Got login request from user {} ", email);

    let user = user_repo.find(email)?;
    let _app = app_repo.find(app)?;
    let meta = Metadata::new(meta_repo)?;

    let timeout =  Duration::from_secs(constants::TOKEN_TIMEOUT);
    let token = Session::new(sess_repo, user, meta, timeout)?;
    Ok(token)
}

pub fn _session_logout(cookie: &str) -> Result<(), Box<dyn Error>> {
    println!("Got a logout request for cookie {} ", cookie);
    Err("Unimplemented".into())
}