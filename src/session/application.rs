use std::error::Error;
use std::time::Duration;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

use crate::user::get_repository as get_user_repository;
use crate::app::get_repository as get_app_repository;
use crate::directory::{
    get_repository as get_dir_repository,
    domain::Directory,
};

use crate::constants::{errors, settings};
use crate::security;

use super::domain::{Session, Token};

pub fn get_writable_session(sess_arc: &Arc<RwLock<Session>>) -> Result<RwLockWriteGuard<Session>, Box<dyn Error>> {
    let sess_wr = sess_arc.write();
    if let Err(err) = sess_wr {
        error!("read-write lock for session got poisoned: {}", err);
        return Err(errors::POISONED.into());
    }

    Ok(sess_wr.unwrap()) // this line will not panic due the previous check of Err
}

pub fn session_login(email: &str,
                     pwd: &str,
                     totp: &str,
                     app: &str) -> Result<String, Box<dyn Error>> {
    
    info!("got a login request from user {} ", email);

    // make sure the user exists and its credentials are alright
    let user = get_user_repository().find_by_email(email)?;
    if !user.match_password(pwd) {
        return Err(errors::NOT_FOUND.into());
    } else if !user.is_verified() {
        return Err(errors::NOT_VERIFIED.into());
    }

    // if, and only if, the user has activated the 2fa
    if let Some(secret) = &user.get_secret() {
        let data = secret.get_data();
        security::verify_totp(data, totp)?;
    }

    // get the existing session or create a new one
    let sess_arc = match super::get_repository().find_by_email(email) {
        Ok(sess_arc) => sess_arc,
        Err(_) => {
            let timeout =  Duration::from_secs(settings::TOKEN_TIMEOUT);
            Session::new(user, timeout)?
        }
    };

    // generate a token for the gotten session and the given app
    let token = {
        let app = get_app_repository().find_by_url(app)?;
        let token: String;
        
        let mut sess = get_writable_session(&sess_arc)?;
        let claim = Token::new(&sess, &app, sess.deadline);
        token = security::encode_jwt(claim)?;

        if let None = sess.get_directory(&app) {
            if let Ok(dir) = get_dir_repository().find_by_user_and_app(sess.user.get_id(), app.get_id()) {
                sess.set_directory(&app, dir)?;
            } else {
                let dir = Directory::new(&sess, &app)?;
                sess.set_directory(&app, dir)?;
            }

            // subscribe the session's sid into the app's group 
            super::get_repository().add_to_app_group(&app, &sess)?;
        }

        token
    };

    Ok(token)
}

pub fn session_logout(token: &str) -> Result<(), Box<dyn Error>> {
    info!("got a logout request for cookie {} ", token);
    let claim = security::decode_jwt::<Token>(token)?;
    
    let sess_arc = super::get_repository().find(&claim.sub)?;
    let mut sess = get_writable_session(&sess_arc)?;

    let app = get_app_repository().find(claim.app)?;
    if let Some(dir) = sess.delete_directory(&app) {
        // unsubscribe the session's from the app's group 
        super::get_repository().delete_from_app_group(&app, &sess)?;
        dir.save()?;
    }

    Ok(())
}