pub mod framework;
pub mod application;
pub mod domain;

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, Duration};
    use crate::metadata::domain::{Metadata, MetadataRepository};
    use crate::user::domain::{User, UserRepository};
    use super::domain::{Session, SessionRepository};

    struct Mock {
        sess: Option<Session>,
    }

    impl UserRepository for &Mock {
        fn find(&self, email: &str) -> Result<User, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn save(&self, user: &mut User) -> Result<(), Box<dyn Error>> {
            user.id = 999;
            Ok(())
        }

        fn delete(&self, user: &User) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }
    }
    
    impl SessionRepository for &Mock {
        fn find(&self, cookie: &str) -> Result<Arc<Mutex<Session>>, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn find_by_email(&self, email: &str) -> Result<Arc<Mutex<Session>>, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn save(&self, mut session: Session) -> Result<String, Box<dyn Error>> {
            session.token = "testing".to_string();
            //self.sess = Some(session);
            Ok("testing".into())
        }

        fn delete(&self, session: &Session) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }
    }
    
    impl MetadataRepository for &Mock {
        fn find(&self, id: i32) -> Result<Metadata, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn save(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>> {
            meta.id = 999;
            Ok(())
        }

        fn delete(&self, meta: &Metadata) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }  
    }

    // #[test]
    // fn session_new_ok() {
    //     const EMAIL: &str = "dummy@example.com";
    //     const timeout: Duration = Duration::from_secs(10);
    //     let mock_impl = &Mock{sess: None};

    //     let user = User::new(Box::new(mock_impl),
    //                          Box::new(mock_impl),
    //                          EMAIL).unwrap();

        
    //     let sess = Session::new(Box::new(mock_impl),
    //                             Box::new(mock_impl),
    //                             user, timeout).unwrap();

    //     if let Some(sess) = &mock_impl.sess {
    //         assert_eq!(sess.token, "testing");

    //     } else {
    //         assert!(false);
    //     }
    // }
}