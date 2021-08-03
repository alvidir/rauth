use crate::app::domain::App;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use std::collections::HashMap;

use crate::metadata::domain::Metadata;
use crate::user::domain::User;
use crate::directory::domain::Directory;

pub trait SessionRepository {
    fn find(&self, cookie: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>>;
    fn find_by_email(&self, email: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>>;
    fn save(&self, session: Session) -> Result<Arc<RwLock<Session>>, Box<dyn Error>>;
    fn delete(&self, session: &Session) -> Result<(), Box<dyn Error>>;
}

pub struct Session {
    pub sid: String,
    pub deadline: SystemTime,
    pub user: User,
    pub apps: HashMap<String, Directory>,
    // the updated_at field from metadata works as a touch_at field, being updated for each
    // read/write action done by the user (owner) over the sessions data
    pub meta: Metadata,
}

impl Session {
    pub fn new<T: SessionRepository + ?Sized>(sess_repo: &T,
               user: User,
               timeout: Duration) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {

        let sess = Session{
            sid: "".to_string(), // will be set by the repository controller down below
            deadline: SystemTime::now() + timeout,
            user: user,
            apps: HashMap::new(),
            meta: Metadata::now(),
        };

        let sess = sess_repo.save(sess)?;
        Ok(sess)
    }

    pub fn set_directory(&mut self, key: &str, dir: Directory) -> Result<(), Box<dyn Error>> {
        if self.apps.get(key).is_some() {
            return Err("already exists".into());
        }

        self.apps.insert(key.to_string(), dir);
        Ok(())
    }

    pub fn get_directory(&mut self, url: &str) -> Option<&mut Directory> {
        self.apps.get_mut(url)
    }

    pub fn delete_directory(&mut self, url: &str) -> Option<Directory> {
        self.apps.remove(url)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Token {
    pub exp: SystemTime,     // expiration time (as UTC timestamp) - required
    pub iat: SystemTime,     // issued at: creation time
    pub iss: String,         // issuer
    pub url: String,         // application url
    pub sub: String,         // subject: the user's session
}

impl Token {
    pub fn new(sess: &Session, app: &App, deadline: SystemTime) -> Self {
        Token {
            exp: deadline,
            iat: SystemTime::now(),
            iss: "oauth.alvidir.com".to_string(),
            url: app.url.clone(),
            sub: sess.sid.clone(),
        }
    }
}