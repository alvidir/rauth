use crate::app::domain::App;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use std::collections::HashMap;

use crate::metadata::domain::Metadata;
use crate::user::domain::User;
use crate::directory::domain::Directory;

pub trait SessionRepository {
    fn find(&self, cookie: &str) -> Result<Arc<Mutex<Session>>, Box<dyn Error>>;
    fn find_by_email(&self, email: &str) -> Result<Arc<Mutex<Session>>, Box<dyn Error>>;
    fn save(&self, session: Session) -> Result<Arc<Mutex<Session>>, Box<dyn Error>>;
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
    pub fn new(sess_repo: &Box<dyn SessionRepository>,
               user: User,
               timeout: Duration) -> Result<Arc<Mutex<Session>>, Box<dyn Error>> {

        let sess = Session{
            sid: "".to_string(),
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

    pub fn _get_directory(&mut self, token: &str) -> Option<&mut Directory> {
        self.apps.get_mut(token)
    }

    pub fn get_directory_by_app(&mut self, app: &App) -> Option<&mut Directory> {
        self.apps.get_mut(&app.url)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Token {
    exp: SystemTime,     // expiration time (as UTC timestamp) - required
    iat: SystemTime,     // issued at: creation time
    url: String,         // application url
    sub: String,         // subject: the user's session
}

impl Token {
    pub fn new(sess: &Session, app: &App) -> Self {
        Token {
            exp: sess.deadline,
            iat: SystemTime::now(),
            url: app.url.clone(),
            sub: sess.sid.clone(),
        }
    }
}