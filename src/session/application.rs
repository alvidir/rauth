use std::error::Error;
use std::time::Duration;

use crate::user::domain::UserRepository;
use crate::app::domain::AppRepository;
use crate::directory::domain::{Directory, DirectoryRepository};
use crate::constants;

use super::domain::{Session, SessionRepository};

//pub trait SessionManager {}

pub fn session_login(sess_repo: Box<dyn SessionRepository>,
                     user_repo: Box<dyn UserRepository>,
                     app_repo: Box<dyn AppRepository>,
                     dir_repo: Box<dyn DirectoryRepository>,
                     email: &str, app: &str) -> Result<String, Box<dyn Error>> {
    
    println!("Got login request from user {} ", email);

    let sess_arc = match sess_repo.find_by_email(email) {
        Ok(sess_arc) => sess_arc,
        Err(_) => {
            let user = user_repo.find(email)?;
            let timeout =  Duration::from_secs(constants::TOKEN_TIMEOUT);
            Session::new(&sess_repo, user, timeout)?
        }
    };

    match sess_arc.lock() {
        Err(err) => Err(format!("{}", err)),
        Ok(sess) => {
            let app = app_repo.find(app)?;
            let mut tokens = vec!(sess.token.clone());

            if let Some((token, _)) = sess.get_directory_by_app(&app) {
                tokens.push(token.to_string());
            } else {
                let dir = Directory::new(dir_repo, &sess.user, &app)?;
                let token = sess.set_directory(dir)?;
                tokens.push(token);
            }

            Ok(tokens.join(""))
        }
    }
}

pub fn _session_logout(cookie: &str) -> Result<(), Box<dyn Error>> {
    println!("Got a logout request for cookie {} ", cookie);
    Err("Unimplemented".into())
}