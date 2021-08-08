pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::InMemorySessionRepository = {
        framework::InMemorySessionRepository::new()
    }; 
}   

pub fn get_repository() -> Box<&'static dyn domain::SessionRepository> {
    #[cfg(not(test))]
    return Box::new(&*REPO_PROVIDER);
    
    #[cfg(test)]
    return Box::new(&*tests::REPO_TEST);
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::sync::{Arc, RwLock};
    use std::time::{SystemTime, Duration};
    use std::collections::HashMap;
    use crate::metadata::tests::new_metadata;
    use crate::user::domain::{User, UserRepository};
    use crate::app::domain::App;
    use super::domain::{Session, SessionRepository};

    const PWD: &str = "ABCD1234";

    lazy_static! {
        pub static ref TESTING_SESSIONS: RwLock<HashMap<String, Arc<RwLock<Session>>>> = {
            let repo = HashMap::new();
            RwLock::new(repo)
        };    
    }

    pub struct Mock;
    lazy_static! {
        pub static ref REPO_TEST: Mock = Mock;
    } 

    impl UserRepository for Mock {
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
    
    impl SessionRepository for Mock {
        fn find(&self, _cookie: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn find_by_email(&self, _email: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn save(&self, mut session: Session) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {
            session.sid = "testing".to_string();

            let mut repo = TESTING_SESSIONS.write()?;
            let email = session.user.email.clone();
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

    #[test]
    fn domain_session_new_ok() {
        const EMAIL: &str = "dummy@example.com";
        const TIMEOUT: Duration = Duration::from_secs(10);

        let meta = new_metadata();
        let user = User::new(meta,
                             EMAIL,
                             PWD).unwrap();

        let before = SystemTime::now();
        let sess_arc = Session::new(user,
                                    TIMEOUT).unwrap();

        let after = SystemTime::now();
        let sess = sess_arc.read().unwrap();
        
        assert_eq!("testing", sess.sid);
        assert!(sess.deadline < after + TIMEOUT);
        assert!(sess.deadline > before + TIMEOUT);
    }
}