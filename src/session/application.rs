use std::error::Error;
use std::time::Duration;
use std::sync::{Arc, RwLock};
use std::collections::HashSet;

use crate::user::domain::UserRepository;
use crate::app::domain::AppRepository;
use crate::directory::domain::{Directory, DirectoryRepository};
use crate::constants;
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

pub fn session_login(sess_repo: &dyn SuperSessionRepository,
                     user_repo: &dyn UserRepository,
                     app_repo: &dyn AppRepository,
                     dir_repo: &dyn DirectoryRepository,
                     email: &str,
                     pwd: &str,
                     app: &str) -> Result<String, Box<dyn Error>> {
    
    info!("got login request from user {} ", email);

    let sess_arc = match sess_repo.find_by_email(email) {
        Ok(sess_arc) => sess_arc,
        Err(_) => {
            let user = user_repo.find(email)?;
            if !user.match_password(pwd) {
                return Err("wrong password".into());
            } else if !user.is_verified() {
                return Err("user not verified".into());
            }

            let timeout =  Duration::from_secs(constants::TOKEN_TIMEOUT);
            Session::new(sess_repo, user, timeout)?
        }
    };

    let token = {
        let app = app_repo.find(app)?;
        let token: String;
        
        let sess_wr = sess_arc.write();
        if let Err(err) = &sess_wr {
            error!("write locking for session of {} has failed: {}", email, err);
        }

        let mut sess = sess_wr.unwrap(); // this line would panic if the lock was poisoned
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

        Ok(token)
    };

    token
}

pub fn session_logout(sess_repo: &dyn SuperSessionRepository,
                      dir_repo: &dyn DirectoryRepository,
                      token: &str) -> Result<(), Box<dyn Error>> {
    info!("got a logout request for cookie {} ", token);
    let claim = security::decode_jwt::<Token>(token)?;
    
    let sess_arc = sess_repo.find(&claim.sub)?;
    let sess_wr = sess_arc.write();

    if let Err(err) = &sess_wr {
        error!("write locking for session {} has failed: {}", claim.sub, err);
    }

    let mut sess = sess_wr.unwrap(); // this line would panic if the lock was poisoned
    let sid = &sess.sid.clone(); // required because of mutability issues in the loop

     // foreach application the session was logged in
    for (url, dir) in sess.apps.iter_mut() {
        if let Err(err) = dir_repo.save(dir) {
            error!("got error while saving directory {} : {}", dir.id, err);
        } else if let Err(err) = sess_repo.remove(&url, sid) {
            error!("got error while removing session {} from group {}: {}", sid, &url, err);
        }
    };

    sess_repo.delete(&sess)
}