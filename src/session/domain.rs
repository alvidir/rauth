use crate::app::domain::App;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use std::collections::HashMap;

use crate::metadata::domain::InnerMetadata;
use crate::user::domain::User;
use crate::directory::domain::Directory;

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
    pub sid: String,
    pub deadline: SystemTime,
    pub user: User,
    pub apps: HashMap<String, Directory>,
    // the updated_at field from metadata works as a touch_at field, being updated for each
    // read/write action done by the user (owner) over the sessions data
    pub meta: InnerMetadata,
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
        };

        super::get_repository().insert(sess)
    }

    pub fn set_directory(&mut self, app: &App, dir: Directory) -> Result<(), Box<dyn Error>> {
        if self.apps.get(app.get_url()).is_some() {
            return Err("already exists".into());
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

    pub fn save(&mut self) -> Result<(), Box<dyn Error>> {
        for (_, dir) in self.apps.iter_mut() {
            dir.save()?;
        }

        Ok(())
    }

    pub fn delete(&mut self, save: bool) -> Result<(), Box<dyn Error>> {
        if save {
            // deleting the session does not always means removing directories from the db. Instead,
            // all directories must be saved for future sessions
            self.save()?;
        }

        super::get_repository().delete(self)?;
        Ok(())
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
            url: app.get_url().to_string(),
            sub: sess.sid.clone(),
        }
    }
}