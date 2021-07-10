use std::error::Error;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::constants::TOKEN_LEN;
use super::domain::{Session, SessionRepository};

lazy_static! {
    static ref REPOSITORY: Mutex<HashMap<String, Arc<Mutex<Session>>>> = {
        let repo = HashMap::new();
        Mutex::new(repo)
    };    
}

impl REPOSITORY {
    fn session_has_email(sess: &Arc<Mutex<Session>>, email: &str) -> bool {
        if let Ok(session) = sess.lock() {
            return session.user.email == email;
        }

        false
    }
}

impl SessionRepository for REPOSITORY {
    fn find(cookie: &str) -> Result<Arc<Mutex<Session>>, Box<dyn Error>> {
        let repo = REPOSITORY.lock()?;
        if let Some(sess) = repo.get(cookie) {
            return Ok(Arc::clone(sess));
        }

        Err("Not found".into())
    }

    fn find_by_email(email: &str) -> Result<Arc<Mutex<Session>>, Box<dyn Error>> {
        let repo = REPOSITORY.lock()?;
        if let Some((_, sess)) = repo.iter().find(|(_, sess)| REPOSITORY::session_has_email(sess, email)) {
            return Ok(Arc::clone(sess));
        }

        Err("Not found".into())
    }

    fn save(session: Session) -> Result<(), Box<dyn Error>> {
        let mut repo = REPOSITORY.lock()?;
        if let Some(_) = repo.get(&session.token) {
            return Err("cookie already exists".into());
        }

        if let Some(_) = repo.iter().find(|(_, sess)| REPOSITORY::session_has_email(sess, &session.user.email)) {
            return Err("email already exists".into());
        }

        let mu = Mutex::new(session);
        let arc = Arc::new(mu);

        let token = Session::generate_token(TOKEN_LEN);
        repo.insert(token, arc);

        Ok(())
    }

    fn delete(session: &Session) -> Result<(), Box<dyn Error>> {
        let mut repo = REPOSITORY.lock()?;
        if let None = repo.remove(&session.token) {
            return Err("cookie does not exists".into());
        }

        Ok(())
    }
}
