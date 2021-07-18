pub mod framework;
pub mod application;
pub mod domain;

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, Duration};
    use std::collections::HashMap;
    use crate::metadata::domain::{Metadata, MetadataRepository};
    use crate::user::domain::{User, UserRepository};
    use super::domain::{Session, SessionRepository};

    lazy_static! {
        pub static ref TESTING_SESSIONS: Mutex<HashMap<String, Arc<Mutex<Session>>>> = {
            let repo = HashMap::new();
            Mutex::new(repo)
        };    
    }

    struct Mock {}

    impl UserRepository for &Mock {
        fn find(&self, _email: &str) -> Result<User, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn save(&self, user: &mut User) -> Result<(), Box<dyn Error>> {
            user.id = 999;
            Ok(())
        }

        fn delete(&self, _user: &User) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }
    }
    
    impl SessionRepository for &Mock {
        fn find(&self, _cookie: &str) -> Result<Arc<Mutex<Session>>, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn find_by_email(&self, _email: &str) -> Result<Arc<Mutex<Session>>, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn save(&self, mut session: Session) -> Result<String, Box<dyn Error>> {
            session.token = "testing".to_string();

            let mut repo = TESTING_SESSIONS.lock()?;
            let email = session.user.email.clone();
            let mu = Mutex::new(session);
            let arc = Arc::new(mu);
            repo.insert(email, arc);

            Ok("testing".into())
        }

        fn delete(&self, _session: &Session) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }
    }
    
    impl MetadataRepository for &Mock {
        fn find(&self, _id: i32) -> Result<Metadata, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn save(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>> {
            meta.id = 999;
            Ok(())
        }

        fn delete(&self, _meta: &Metadata) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }  
    }

    #[test]
    fn session_new_ok() {
        const EMAIL: &str = "dummy@example.com";
        const TIMEOUT: Duration = Duration::from_secs(10);
        let mock_impl = &Mock{};

        let meta = Metadata::now();
        let user = User::new(Box::new(mock_impl),
                             meta.clone(),
                             EMAIL).unwrap();

        let before = SystemTime::now();
        let token = Session::_new(Box::new(mock_impl),
                                 user,
                                 meta,
                                 TIMEOUT).unwrap();

        let after = SystemTime::now();

        let repo = TESTING_SESSIONS.lock().unwrap();
        let sess_mux = repo.get(EMAIL).unwrap();
        let sess = sess_mux.lock().unwrap();
        
        assert_eq!(token, sess.token);
        assert!(sess.deadline < after + TIMEOUT);
        assert!(sess.deadline > before + TIMEOUT);
    }
}