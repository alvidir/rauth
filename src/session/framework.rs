use std::error::Error;
use std::collections::HashMap;
use super::domain::{Session, SessionRepository};

struct Repository {
    all_instances: HashMap<String, Session>,
}

lazy_static! {
    static ref REPOSITORY: Repository = {
        Repository {
            all_instances: HashMap::new(),
        }
    };
}

impl SessionRepository for Repository {
    fn find(&self, cookie: &str) -> Result<Session, Box<dyn Error>> {
        Err("Unimplemented".into())
    }

    fn save(&self, session: &mut Session) -> Result<(), Box<dyn Error>> {
        Err("Unimplemented".into())
    }

    fn delete(&self, session: &Session) -> Result<(), Box<dyn Error>> {
        Err("Unimplemented".into())
    }
}
