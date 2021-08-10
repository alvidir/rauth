pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::InMemorySessionRepository = {
        framework::InMemorySessionRepository::new()
    }; 
}   

#[cfg(not(test))]
pub fn get_repository() -> Box<&'static dyn domain::SessionRepository> {
    Box::new(&*REPO_PROVIDER)
}

#[cfg(test)]
pub fn get_repository() -> Box<dyn domain::SessionRepository> {
    Box::new(tests::Mock)
}

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use std::sync::{Arc, RwLock};
    use std::time::{SystemTime, Duration};
    use std::collections::HashMap;
    use crate::app::domain::App;
    use crate::user::tests::new_user;
    use crate::metadata::domain::InnerMetadata;
    use crate::directory::tests::new_directory;
    use crate::app::tests::new_app;
    use crate::security;
    use super::domain::{Session, SessionRepository, Token};

    lazy_static! {
        pub static ref TESTING_SESSIONS: RwLock<HashMap<String, Arc<RwLock<Session>>>> = {
            let repo = HashMap::new();
            RwLock::new(repo)
        };    
    }

    pub struct Mock;
    impl SessionRepository for Mock {
        fn find(&self, _cookie: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn find_by_email(&self, _email: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn insert(&self, mut session: Session) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {
            session.sid = "testing".to_string();

            let mut repo = TESTING_SESSIONS.write()?;
            let email = session.user.get_email().to_string();
            let mu = RwLock::new(session);
            let arc = Arc::new(mu);
            
            repo.insert(email.to_string(), arc);
            let sess = repo.get(&email).unwrap();
            Ok(Arc::clone(sess))
        }

        fn delete(&self, _session: &Session) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn delete_all_by_app(&self, _app: &App) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn find_all_by_app(&self, _app: &App) -> Result<Vec<Arc<RwLock<Session>>>, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn add_to_app_group(&self, _app: &App, _sess: &Session) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn delete_from_app_group(&self, _app: &App, _sess: &Session) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }
    }

    pub fn new_session() -> Session {
        Session{
            sid: "testing".to_string(),
            deadline: SystemTime::now(),
            user: new_user(),
            apps: HashMap::new(),
            meta: InnerMetadata::new(),
            _sandbox: HashMap::new(),
        }
    }

    #[test]
    fn session_new() {
        const TIMEOUT: Duration = Duration::from_secs(10);

        let user = new_user();
        let user_id = user.get_id();

        let before = SystemTime::now();
        let sess_arc = Session::new(user, TIMEOUT).unwrap();
        let after = SystemTime::now();
        let sess = sess_arc.read().unwrap();
        
        assert_eq!("testing", sess.sid);
        assert!(sess.deadline < after + TIMEOUT);
        assert!(sess.deadline > before + TIMEOUT);

        assert_eq!(sess.user.get_id(), user_id);
        assert_eq!(0, sess.apps.len());
        assert_eq!(0, sess._sandbox.len());
    }

    #[test]
    fn session_set_directory_ok() {
        let dir = new_directory();
        let app = new_app();

        let mut sess = new_session();
        let before = SystemTime::now();
        sess.set_directory(&app, dir).unwrap();
        let after = SystemTime::now();

        assert_eq!(1, sess.apps.len());
        assert!(sess.apps.get(&app.get_id()).is_some());
        assert!(sess.meta.touch_at >= before && sess.meta.touch_at <= after);
    }

    #[test]
    fn session_set_directory_ko() {
        let dir = new_directory();
        let app = new_app();

        let mut sess = new_session();
        sess.set_directory(&app, dir).unwrap();

        let dir = new_directory();
        assert!(sess.set_directory(&app, dir).is_err());
    }

    #[test]
    fn session_token_ok() {
        let app = new_app();
        let sess = new_session();
        let deadline = SystemTime::now() + Duration::from_secs(60);

        let before = SystemTime::now();
        let claim = Token::new(&sess, &app, deadline);
        let after = SystemTime::now();

        assert!(claim.iat >= before && claim.iat <= after);        
        assert_eq!(claim.exp, deadline);
        assert_eq!("oauth.alvidir.com", claim.iss);
        assert_eq!(sess.sid, claim.sub);
        assert_eq!(app.get_id(), claim.app);
    }

    // #[test]
    // fn session_token_encode() {
    //     let app = new_app();
    //     let sess = new_session();
    //     let deadline = SystemTime::now() + Duration::from_secs(60);

    //     let before = SystemTime::now();
    //     let claim = Token::new(&sess, &app, deadline);
    //     let after = SystemTime::now();
        
    //     let token = security::encode_jwt(claim).unwrap();
    //     let claim = security::decode_jwt::<Token>(&token).unwrap();

    //     assert!(claim.iat >= before && claim.iat <= after);        
    //     assert_eq!(claim.exp, deadline);
    //     assert_eq!("oauth.alvidir.com", claim.iss);
    //     assert_eq!(sess.sid, claim.sub);
    //     assert_eq!(app.get_id(), claim.app);
    // }

    // #[test]
    // fn session_token_ko() {
    //     let app = new_app();
    //     let sess = new_session();
    //     let deadline = SystemTime::now();

    //     let claim = Token::new(&sess, &app, deadline);
    //     let token = security::encode_jwt(claim).unwrap();
    //     assert!(security::decode_jwt::<Token>(&token).is_err());
    // }
}