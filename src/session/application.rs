use std::error::Error;
use std::time::Duration;
use std::sync::{Arc, RwLock, RwLockWriteGuard, RwLockReadGuard};
use std::collections::HashSet;

use crate::user::domain::UserRepository;
use crate::app::domain::AppRepository;
use crate::directory::domain::{Directory, DirectoryRepository};
use crate::constants::{errors, settings};
use crate::security;

use super::domain::{Session, SessionRepository, Token};

pub trait GroupByAppRepository {
    fn get(&self, url: &str) -> Result<Arc<RwLock<HashSet<String>>>, Box<dyn Error>>;
    fn store(&self, url: &str, sid: &str) -> Result<(), Box<dyn Error>>;
    fn remove(&self, url: &str, sid: &str) -> Result<(), Box<dyn Error>>;
}

pub trait SuperSessionRepository: GroupByAppRepository + SessionRepository {}

impl<T> SuperSessionRepository for T
where T: SessionRepository + GroupByAppRepository {}

pub fn _get_readable_session(sess_arc: &Arc<RwLock<Session>>) -> Result<RwLockReadGuard<Session>, Box<dyn Error>> {
    let sess_wr = sess_arc.read();
    if let Err(err) = sess_wr {
        error!("read-only lock for session got poisoned: {}", err);
        return Err(errors::POISONED.into());
    }

    Ok(sess_wr.unwrap()) // this line will not panic due the previous check of Err
}

pub fn get_writable_session(sess_arc: &Arc<RwLock<Session>>) -> Result<RwLockWriteGuard<Session>, Box<dyn Error>> {
    let sess_wr = sess_arc.write();
    if let Err(err) = sess_wr {
        error!("read-write lock for session got poisoned: {}", err);
        return Err(errors::POISONED.into());
    }

    Ok(sess_wr.unwrap()) // this line will not panic due the previous check of Err
}

pub fn session_login(sess_repo: &dyn SuperSessionRepository,
                     user_repo: &dyn UserRepository,
                     app_repo: &dyn AppRepository,
                     dir_repo: &dyn DirectoryRepository,
                     email: &str,
                     pwd: &str,
                     totp: &str,
                     app: &str) -> Result<String, Box<dyn Error>> {
    
    info!("got login request from user {} ", email);

    // make sure the user exists and its credentials are alright
    let user = user_repo.find(email)?;
    if !user.match_password(pwd) {
        return Err(errors::NOT_FOUND.into());
    } else if !user.is_verified() {
        return Err(errors::NOT_VERIFIED.into());
    }

    // if, and only if, the user has activated the 2fa
    if let Some(secret) = &user.secret {
        let data = secret.get_data();
        security::verify_totp(data, totp)?;
    }

    // get the existing session or create a new one
    let sess_arc = match sess_repo.find_by_email(email) {
        Ok(sess_arc) => sess_arc,
        Err(_) => {
            let timeout =  Duration::from_secs(settings::TOKEN_TIMEOUT);
            Session::new(sess_repo, user, timeout)?
        }
    };

    // generate a token for the gotten session and the given app
    let token = {
        let app = app_repo.find(app)?;
        let token: String;
        
        let mut sess = get_writable_session(&sess_arc)?;
        let claim = Token::new(&sess, &app, sess.deadline);
        token = security::encode_jwt(claim)?;

        if let None = sess.get_directory(&app.url) {
            if let Ok(dir) = dir_repo.find_by_user_and_app(sess.user.id, app.id) {
                sess.set_directory(&app.url, dir)?;
            } else {
                let dir = Directory::new(dir_repo, &sess, &app)?;
                sess.set_directory(&app.url, dir)?;
            }

            // register the session's sid into the app's url group 
            sess_repo.store(&app.url, &sess.sid)?;
        }

        token
    };

    Ok(token)
}

pub fn session_logout(sess_repo: &dyn SuperSessionRepository,
                      dir_repo: &dyn DirectoryRepository,
                      token: &str) -> Result<(), Box<dyn Error>> {
    info!("got a logout request for cookie {} ", token);
    let claim = security::decode_jwt::<Token>(token)?;
    
    let sess_arc = sess_repo.find(&claim.sub)?;
    let mut sess = get_writable_session(&sess_arc)?;
    let sid = &sess.sid.clone(); // required because of mutability issues

    if let Some(mut dir) = sess.delete_directory(&claim.url) {
        sess_repo.remove(&claim.url, sid)?;
        dir_repo.save(&mut dir)?;
    }

    Ok(())
}