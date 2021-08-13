use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use std::collections::HashMap;

use crate::metadata::domain::InnerMetadata;
use crate::user::domain::User;
use crate::app::domain::App;
use crate::directory::domain::Directory;
use crate::constants::errors::ALREADY_EXISTS;
use crate::time::unix_timestamp;

pub trait SessionRepository {
    fn find(&self, cookie: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>>;
    fn find_by_email(&self, email: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>>;
    fn insert(&self, session: Session) -> Result<Arc<RwLock<Session>>, Box<dyn Error>>;
    fn delete(&self, session: &Session) -> Result<(), Box<dyn Error>>;
    fn delete_all_by_app(&self, app: &App) -> Result<(), Box<dyn Error>>;

    // group by app methods
    fn find_all_by_app(&self, app: &App) -> Result<Vec<Arc<RwLock<Session>>>, Box<dyn Error>>;
    fn add_to_app_group(&self, app: &App, sess: &Session) -> Result<(), Box<dyn Error>>;
    fn delete_from_app_group(&self, app: &App, sess: &Session) -> Result<(), Box<dyn Error>>;
}

pub struct Session {
    pub(super) sid: String,
    pub(super) deadline: SystemTime,
    pub(super) user: User,
    pub(super) apps: HashMap<i32, Directory>,
    pub(super) meta: InnerMetadata,
    // sandbox is used for storing temporal data that must not be persisted nor
    // accessed by any other party than the Session itself
    pub(super) sandbox: HashMap<String, String>,
}

impl Session {
    pub fn new(user: User,
               timeout: Duration) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {

        let sess = Session{
            sid: "".to_string(), // will be set by the repository controller down below
            deadline: SystemTime::now() + timeout,
            user: user,
            apps: HashMap::new(),
            meta: InnerMetadata::new(),
            sandbox: HashMap::new(),
        };

        super::get_repository().insert(sess)
    }

    pub fn get_user(&self) -> &User {
        &self.user
    }

    pub fn get_user_mut(&mut self) -> &mut User {
        &mut self.user
    }

    pub fn get_deadline(&self) -> SystemTime {
        self.deadline
    }

    /// if, and only if, the session does not have any directory for the directory's app then it gets inserted
    /// into the session's directories
    pub fn set_directory(&mut self, dir: Directory) -> Result<(), Box<dyn Error>> {
        if self.apps.get(&dir.get_app()).is_some() {
            return Err(ALREADY_EXISTS.into());
        }

        self.apps.insert(dir.get_app(), dir);
        self.meta.touch();
        Ok(())
    }

    /// returns the directory of the provided application, if any
    pub fn get_directory(&mut self, app: &App) -> Option<&mut Directory> {
        self.apps.get_mut(&app.get_id())
    }

    /// deletes the directory for the given application, if any
    pub fn delete_directory(&mut self, app: &App) -> Option<Directory> {
        self.apps.remove(&app.get_id())
    }

    /// deletes the session from the repository
    pub fn delete(&mut self) -> Result<(), Box<dyn Error>> {
        super::get_repository().delete(self)?;
        Ok(())
    }

    /// stores a new value for an entry into the session's sandbox. If there it was any older value, it is returned, else
    /// return is None 
    pub fn store(&mut self, key: &str, value: &str) -> Option<String> {
        self.sandbox.insert(key.to_string(), value.to_string())
    }   

    /// removes an entry from the session's sandbox
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.sandbox.remove(key)
    }

    /// returns an entry from the session's sandbox
    pub fn get(&mut self, key: &str) -> Option<String> {
        if let Some(value) = self.sandbox.get(key) {
            return Some(value.clone());
        }

        None
    }
}

#[derive(Serialize, Deserialize)]
pub struct Token {
    pub exp: usize,     // expiration time (as UTC timestamp) - required
    pub iat: SystemTime,     // issued at: creation time
    pub iss: String,         // issuer
    pub sub: String,         // subject: the user's session
    pub app: i32,            // application id
}

impl Token {
    pub fn new(sess: &Session, app: &App, deadline: SystemTime) -> Self {
        Token {
            exp: unix_timestamp(deadline),
            iat: SystemTime::now(),
            iss: "oauth.alvidir.com".to_string(),
            sub: sess.sid.clone(),
            app: app.get_id(),
        }
    }
}