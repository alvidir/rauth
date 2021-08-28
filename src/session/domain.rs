use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use std::collections::{HashMap, HashSet};

use crate::metadata::domain::InnerMetadata;
use crate::user::domain::User;
use crate::app::domain::App;
use crate::directory::domain::Directory;
use crate::constants::errors::ALREADY_EXISTS;
use crate::time::unix_timestamp;

pub trait SessionRepository {
    fn find(&self, cookie: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>>;
    fn find_by_email(&self, email: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>>;
    fn insert(&self, session: Session) -> Result<String, Box<dyn Error>>;
    fn delete(&self, session: &Session) -> Result<(), Box<dyn Error>>;
}

pub trait GroupByAppRepository {
    fn find(&self, app: &App) -> Result<Arc<RwLock<HashSet<String>>>, Box<dyn Error>>;
    fn insert(&self, app: &App) -> Result<(), Box<dyn Error>>;
    fn delete(&self, app: &App) -> Result<(), Box<dyn Error>>;
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
               timeout: Duration) -> Self {

        Session{
            sid: "".to_string(), // will be set by the repository controller down below
            deadline: SystemTime::now() + timeout,
            user: user,
            apps: HashMap::new(),
            meta: InnerMetadata::new(),
            sandbox: HashMap::new(),
        }
    }

    pub fn get_id(&self) -> &str {
        &self.sid
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
    pub fn get(&self, key: &str) -> Option<String> {
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
            iss: "tpauth.alvidir.com".to_string(),
            sub: sess.sid.clone(),
            app: app.get_id(),
        }
    }
}


#[cfg(test)]
pub mod tests {
    use std::time::{SystemTime, Duration};
    use std::collections::HashMap;
    use crate::user::domain::tests::new_user;
    use crate::metadata::domain::InnerMetadata;
    use crate::directory::domain::tests::new_directory;
    use crate::app::domain::tests::new_app;
    use crate::time::unix_timestamp;
    use super::{Session, Token};

    pub fn new_session() -> Session {
        Session{
            sid: "testing".to_string(),
            deadline: SystemTime::now(),
            user: new_user(),
            apps: HashMap::new(),
            meta: InnerMetadata::new(),
            sandbox: HashMap::new(),
        }
    }

    #[test]
    fn session_new_should_not_fail() {
        const TIMEOUT: Duration = Duration::from_secs(10);

        let user = new_user();
        let user_id = user.get_id();

        let before = SystemTime::now();
        let sess = Session::new(user, TIMEOUT);
        let after = SystemTime::now();
        
        assert!(sess.deadline < after + TIMEOUT);
        assert!(sess.deadline > before + TIMEOUT);

        assert_eq!(sess.user.get_id(), user_id);
        assert_eq!(0, sess.apps.len());
        assert_eq!(0, sess.sandbox.len());
    }

    #[test]
    fn session_set_directory_should_not_fail() {
        let dir = new_directory();
        let app_id = dir.get_app();

        let mut sess = new_session();
        let before = SystemTime::now();
        sess.set_directory(dir).unwrap();
        let after = SystemTime::now();

        assert_eq!(1, sess.apps.len());
        assert!(sess.apps.get(&app_id).is_some());
        assert!(sess.meta.touch_at >= before && sess.meta.touch_at <= after);
    }

    #[test]
    fn session_set_directory_repeated_should_fail() {
        let dir = new_directory();

        let mut sess = new_session();
        sess.set_directory(dir).unwrap();

        let dir = new_directory();
        assert!(sess.set_directory(dir).is_err());
    }

    #[test]
    fn session_token_should_not_fail() {
        let app = new_app();
        let sess = new_session();
        let deadline = SystemTime::now() + Duration::from_secs(60);

        let before = SystemTime::now();
        let claim = Token::new(&sess, &app, deadline);
        let after = SystemTime::now();

        assert!(claim.iat >= before && claim.iat <= after);        
        assert_eq!(claim.exp, unix_timestamp(deadline));
        assert_eq!("tpauth.alvidir.com", claim.iss);
        assert_eq!(sess.sid, claim.sub);
        assert_eq!(app.get_id(), claim.app);
    }

    #[test]
    #[cfg(feature = "integration-tests")]
    fn session_token_encode_shoudl_success() {
        use crate::security;
        
        dotenv::dotenv().unwrap();

        let app = new_app();
        let sess = new_session();
        let deadline = SystemTime::now() + Duration::from_secs(60);

        let before = SystemTime::now();
        let claim = Token::new(&sess, &app, deadline);
        let after = SystemTime::now();
        
        let token = security::encode_jwt(claim).unwrap();
        let claim = security::decode_jwt::<Token>(&token).unwrap();

        assert!(claim.iat >= before && claim.iat <= after);        
        assert_eq!(claim.exp, unix_timestamp(deadline));
        assert_eq!("tpauth.alvidir.com", claim.iss);
        assert_eq!(sess.sid, claim.sub);
        assert_eq!(app.get_id(), claim.app);
    }

    #[test]
    #[cfg(feature = "integration-tests")]
    fn session_token_expired_should_fail() {
        use crate::security;
        
        dotenv::dotenv().unwrap();

        let app = new_app();
        let sess = new_session();
        let deadline = SystemTime::now() - Duration::from_secs(60);

        let claim = Token::new(&sess, &app, deadline);
        let token = security::encode_jwt(claim).unwrap();
        assert!(security::decode_jwt::<Token>(&token).is_err());
    }
}