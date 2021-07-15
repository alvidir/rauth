use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::metadata::domain::{Metadata, MetadataRepository};
use crate::user::domain::User;

pub trait SessionRepository {
    fn find(&self, cookie: &str) -> Result<Arc<Mutex<Session>>, Box<dyn Error>>;
    fn find_by_email(&self, email: &str) -> Result<Arc<Mutex<Session>>, Box<dyn Error>>;
    fn save(&self, session: Session) -> Result<String, Box<dyn Error>>;
    fn delete(&self, session: &Session) -> Result<(), Box<dyn Error>>;
}

pub struct Session {
    pub token: String,
    pub deadline: SystemTime,
    pub user: User,
    // the updated_at field from metadata works as a touch_at field, being updated for each
    // read/write action done by the user (owner) over the sessions data
    pub meta: Metadata,
}

impl Session {
    pub fn new(sess_repo: Box<dyn SessionRepository>,
               meta_repo: Box<dyn MetadataRepository>,
               user: User,
               timeout: Duration) -> Result<String, Box<dyn Error>> {

        let sess = Session{
            token: "".to_string(),
            deadline: SystemTime::now() + timeout,
            user: user,
            meta: Metadata::new(meta_repo)?,
        };

        let token = sess_repo.save(sess)?;
        Ok(token)
    }
}