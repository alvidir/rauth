use std::error::Error;
use std::time::Duration;
use std::sync::{Arc, RwLock};

use crate::user::domain::UserRepository;
use crate::app::domain::AppRepository;
use crate::directory::domain::{Directory, DirectoryRepository};
use crate::constants;
use crate::security;

use super::domain::{Session, SessionRepository, Token};

pub trait GroupByAppRepository {
    fn get(&self, url: &str) -> Result<Arc<RwLock<Vec<String>>>, Box<dyn Error>>;
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
                     app: &str) -> Result<String, Box<dyn Error>> {
    
    println!("Got login request from user {} ", email);

    let sess_arc = match sess_repo.find_by_email(email) {
        Ok(sess_arc) => sess_arc,
        Err(_) => {
            let user = user_repo.find(email)?;
            let timeout =  Duration::from_secs(constants::TOKEN_TIMEOUT);
            Session::new(sess_repo, user, timeout)?
        }
    };

    let token = {
        let app = app_repo.find(app)?;
        let token: String;
        
        let mut sess = sess_arc.write().unwrap();
        let claim = Token::new(&sess, &app, sess.deadline);
        token = security::generate_jwt(claim)?;

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

pub fn _session_logout(cookie: &str) -> Result<(), Box<dyn Error>> {
    println!("Got a logout request for cookie {} ", cookie);
    Err("Unimplemented".into())
}