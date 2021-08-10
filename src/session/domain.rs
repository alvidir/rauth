use std::error::Error;
use std::any::Any;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use std::collections::HashMap;

use crate::metadata::domain::InnerMetadata;
use crate::user::domain::User;
use crate::app::domain::App;
use crate::directory::domain::Directory;
use crate::constants::errors::ALREADY_EXISTS;

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
    pub(super) apps: HashMap<String, Directory>,
    pub(super) meta: InnerMetadata,
    // sandbox is used for storing temporal data that must not be persisted nor
    // accessed by any other party than the Session itself
    pub(super) sandbox: HashMap<String, Box<dyn Any>>,
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

    pub fn get_deadline(&self) -> SystemTime {
        self.deadline
    }

    pub fn set_directory(&mut self, app: &App, dir: Directory) -> Result<(), Box<dyn Error>> {
        if self.apps.get(app.get_url()).is_some() {
            return Err(ALREADY_EXISTS.into());
        }

        self.apps.insert(app.get_url().into(), dir);
        Ok(())
    }

    pub fn get_directory(&mut self, app: &App) -> Option<&mut Directory> {
        self.apps.get_mut(app.get_url())
    }

    pub fn delete_directory(&mut self, app: &App) -> Option<Directory> {
        self.apps.remove(app.get_url())
    }

    pub fn delete(&mut self) -> Result<(), Box<dyn Error>> {
        super::get_repository().delete(self)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Token {
    pub(super) exp: SystemTime,     // expiration time (as UTC timestamp) - required
    pub(super) iat: SystemTime,     // issued at: creation time
    pub(super) iss: String,         // issuer
    pub(super) url: String,         // application url
    pub(super) sub: String,         // subject: the user's session
}

impl Token {
    pub fn new(sess: &Session, app: &App, deadline: SystemTime) -> Self {
        Token {
            exp: deadline,
            iat: SystemTime::now(),
            iss: "oauth.alvidir.com".to_string(),
            url: app.get_url().to_string(),
            sub: sess.sid.clone(),
        }
    }
}