use std::error::Error;
use std::sync::{Arc, RwLock, RwLockWriteGuard, RwLockReadGuard};
use std::collections::{HashMap, HashSet};
use tonic::{Request, Response, Status};
use crate::security;
use crate::constants::{settings, errors};
use crate::app::domain::App;
use super::domain::{
    Session,
    SessionRepository,
    GroupByAppRepository
};

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("session");
}

// Proto generated server traits
use proto::session_service_server::SessionService;
pub use proto::session_service_server::SessionServiceServer;

// Proto message structs
use proto::{LoginRequest, LoginResponse};

pub struct SessionServiceImplementation;

#[tonic::async_trait]
impl SessionService for SessionServiceImplementation {
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<LoginResponse>, Status> {
        let msg_ref = request.into_inner();

        match super::application::session_login(&msg_ref.ident,
                                                &msg_ref.pwd,
                                                &msg_ref.totp,
                                                &msg_ref.app) {
                                                    
            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(token) => {
                Ok(Response::new(LoginResponse{
                    token: token,
                }))
            }
        }
    }

    async fn logout(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let metadata = request.metadata();
        if let None = metadata.get("token") {
            return Err(Status::failed_precondition("token required"));
        };

        let token = match metadata.get("token")
            .unwrap() // this line will not fail due to the previous check of None 
            .to_str() {
            Err(err) => return Err(Status::aborted(err.to_string())),
            Ok(token) => token,
        };

        if let Err(err) = super::application::session_logout(token){               
            return Err(Status::aborted(err.to_string()));
        }

        Ok(Response::new(()))
    }
}


pub struct InMemorySessionRepository {
    all_instances: RwLock<HashMap<String, Arc<RwLock<Session>>>>,
    sids_by_email: RwLock<HashMap<String, String>>,
    group_by_app: RwLock<HashMap<i32, Arc<RwLock<HashSet<String>>>>>,
}

impl InMemorySessionRepository {
    pub fn new() -> Self {
        InMemorySessionRepository {
            all_instances: {
                let repo = HashMap::new();
                RwLock::new(repo)
            },

            sids_by_email: {
                let repo = HashMap::new();
                RwLock::new(repo)
            },

            group_by_app: {
                let repo = HashMap::new();
                RwLock::new(repo)
            },
        }
    }

    fn get_readable_repo(&self) -> Result<RwLockReadGuard<HashMap<String, Arc<RwLock<Session>>>>, Box<dyn Error>> {
        match self.all_instances.read() {
            Ok(repo) => Ok(repo),
            Err(err) => {
                error!("read-only lock for all_instances from session's repo got poisoned: {}", err);
                Err(errors::POISONED.into())
            }
        }
    }

    fn get_writable_emails(&self) -> Result<RwLockWriteGuard<HashMap<String, String>>, Box<dyn Error>> {
        match self.sids_by_email.write() {
            Ok(repo) => Ok(repo),
            Err(err) => {
                error!("read-write lock for sids_by_email from session's repo got poisoned: {}", err);
                Err(errors::POISONED.into())
            }
        }
    }

    fn get_readable_emails(&self) -> Result<RwLockReadGuard<HashMap<String, String>>, Box<dyn Error>> {
        match self.sids_by_email.read() {
            Ok(repo) => Ok(repo),
            Err(err) => {
                error!("read-only lock for all_instances from session's repo got poisoned: {}", err);
                Err(errors::POISONED.into())
            }
        }
    }

    fn get_writable_repo(&self) -> Result<RwLockWriteGuard<HashMap<String, Arc<RwLock<Session>>>>, Box<dyn Error>> {
        match self.all_instances.write() {
            Ok(repo) => Ok(repo),
            Err(err) => {
                error!("read-write lock for all_instances from session's repo got poisoned: {}", err);
                Err(errors::POISONED.into())
            }
        }
    }

    fn get_readable_group(&self) -> Result<RwLockReadGuard<HashMap<i32, Arc<RwLock<HashSet<String>>>>>, Box<dyn Error>>{
        match self.group_by_app.read() {
            Ok(group) => Ok(group),
            Err(err) => {
                error!("read-only lock for group_by_app from session's repo got poisoned: {}", err);
                Err(errors::POISONED.into())
            }
        }
    }

    fn get_writable_group(&self) -> Result<RwLockWriteGuard<HashMap<i32, Arc<RwLock<HashSet<String>>>>>, Box<dyn Error>>{
        match self.group_by_app.write() {
            Ok(group) => Ok(group),
            Err(err) => {
                error!("read-write lock for group_by_app from session's repo got poisoned: {}", err);
                Err(errors::POISONED.into())
            }
        }
    }

    fn get_sid_by_email(&self, email: &str) -> Result<String, Box<dyn Error>> {
        let by_email = self.get_readable_emails()?;
        match by_email.get(email) {
            Some(sid) => Ok(sid.clone()),
            None => Err(errors::NOT_FOUND.into()),  
        }
    }

    fn get_session_by_sid(&self, sid: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {
        let repo = self.get_readable_repo()?;
        match repo.get(sid) {
            Some(sess) => Ok(Arc::clone(sess)),
            None => Err(errors::ALREADY_EXISTS.into()),
        }
    }

    fn insert_session_into_repo(&self, mut session: Session) -> Result<String, Box<dyn Error>> {
        let mut repo = self.get_writable_repo()?;

        loop { // make sure the token is unique
            let sid = security::get_random_string(settings::TOKEN_LEN);
            if repo.get(&sid).is_none() {
                session.sid = sid;
                break;
            }

            warn!("collition: generated sid already exists");
        }
        
        let token = session.sid.clone();
        let mu = RwLock::new(session);
        let arc = Arc::new(mu);

        repo.insert(token.to_string(), arc);
        Ok(token)
    }

    fn remove_session_by_sid(&self, sid: &str) -> Result<(), Box<dyn Error>>  {
        let mut repo = self.get_writable_repo()?;
        repo.remove(sid);
        Ok(())
    }

    fn insert_sid_by_email(&self, email: &str, sid: &str) -> Result<(), Box<dyn Error>> {
        let mut by_email = self.get_writable_emails()?;
        by_email.insert(email.to_string(), sid.to_string());
        Ok(())
    }

    fn remove_email(&self, email: &str) -> Result<(), Box<dyn Error>> {
        let mut by_email = self.get_writable_emails()?;
        by_email.remove(email);
        Ok(())
    }
}

impl SessionRepository for InMemorySessionRepository {
    fn find(&self, token: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {
        let repo = self.get_readable_repo()?;
        if let Some(sess) = repo.get(token) {
            return Ok(Arc::clone(sess));
        }

        Err(errors::NOT_FOUND.into())
    }

    fn find_by_email(&self, email: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {
        let sid = self.get_sid_by_email(email)?;
        
        let repo = self.get_readable_repo()?;
        if let Some(sess_arc) = repo.get(&sid) {
            Ok(Arc::clone(sess_arc))
        } else {
            Err(errors::NOT_FOUND.into())
        }
    }

    fn insert(&self, session: Session) -> Result<String, Box<dyn Error>> {
        let email = session.get_user().get_email().to_string();
        if let Ok(_) = self.get_sid_by_email(&email) {
            return Err(errors::ALREADY_EXISTS.into());
        }

        if let Ok(_) = self.get_session_by_sid(session.get_id()) {
            return Err(errors::ALREADY_EXISTS.into());
        }

        let token = self.insert_session_into_repo(session)?;
        self.insert_sid_by_email(&email, &token)?;
        Ok(token)
    }

    fn delete(&self, session: &Session) -> Result<(), Box<dyn Error>> {
        self.remove_session_by_sid(session.get_id())?;
        self.remove_email(session.get_user().get_email())?;
        Ok(())
    }
}

impl GroupByAppRepository for InMemorySessionRepository {
    fn find(&self, app: &App) -> Result<Arc<RwLock<HashSet<String>>>, Box<dyn Error>> {
        let group = self.get_readable_group()?;
        if let Some(group) = group.get(&app.get_id()) {
            return Ok(Arc::clone(group));
        }

        Err(errors::NOT_FOUND.into())
    }

    fn insert(&self, app: &App) -> Result<(), Box<dyn Error>> {
        let mut group = self.get_writable_group()?;
        if let Some(_) = group.get(&app.get_id()) {
            return Err(errors::ALREADY_EXISTS.into());
        }

        let sids = HashSet::new();
        let mu = RwLock::new(sids);
        let arc = Arc::new(mu);

        group.insert(app.get_id(), arc);
        Ok(())
    }

    fn delete(&self, app: &App) -> Result<(), Box<dyn Error>> {
        let mut group = self.get_writable_group()?;
        group.remove(&app.get_id());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use crate::constants::settings;
    use crate::user::domain::tests::new_user_custom;
    use crate::app::domain::tests::new_app_custom;
    use super::super::{
        get_repository as get_sess_repository,
        get_group_by_app,
        domain::Session,
    };

    #[test]
    fn session_insert_should_not_fail() {
        let user = new_user_custom(999, "session_insert_should_not_fail@testing.com");
        let timeout = Duration::from_secs(10);
        let sess = Session::new(user, timeout);
        
        let token = get_sess_repository().insert(sess).unwrap();
        let sess_arc = get_sess_repository().find(&token).unwrap();
        let sess = sess_arc.read().unwrap();

        assert_eq!(settings::TOKEN_LEN, sess.get_id().len());
    }

    #[test]
    fn session_insert_repeated_should_fail() {
        let user = new_user_custom(999, "session_insert_repeated_should_fail@testing.com");
        let timeout = Duration::from_secs(10);
        let sess = Session::new(user, timeout);
        
        assert!(get_sess_repository().insert(sess).is_ok());
        
        let user = new_user_custom(999, "session_insert_repeated_should_fail@testing.com");
        let timeout = Duration::from_secs(10);
        let sess = Session::new(user, timeout);
        assert!(get_sess_repository().insert(sess).is_err());
    }

    #[test]
    fn session_find_by_email_should_not_fail() {
        let user = new_user_custom(999, "session_find_by_email_should_not_fail@testing.com");
        let timeout = Duration::from_secs(10);
        let sess = Session::new(user, timeout);
        
        get_sess_repository().insert(sess).unwrap();
        assert!(get_sess_repository().find_by_email("session_find_by_email_should_not_fail@testing.com").is_ok());
    }

    #[test]
    fn session_delete_should_not_fail() {
        let user = new_user_custom(999, "session_delete_should_not_fail@testing.com");
        let timeout = Duration::from_secs(10);
        let sess = Session::new(user, timeout);
        
        let token = get_sess_repository().insert(sess).unwrap();
        let sess_arc = get_sess_repository().find(&token).unwrap();
        let sess = sess_arc.read().unwrap();

        assert!(get_sess_repository().delete(&sess).is_ok());
        assert!(get_sess_repository().find(&token).is_err());
    }

    #[test]
    fn group_by_app_insert_should_not_fail() {
        let app = new_app_custom(111, "http://group.by.app.insert.should.not.fail.com");
        assert!(get_group_by_app().insert(&app).is_ok());
        assert!(get_group_by_app().find(&app).is_ok());
    }

    #[test]
    fn group_by_app_delete_should_not_fail() {
        let app = new_app_custom(222, "http://group.by.app.delete.should.not.fail.com");
        assert!(get_group_by_app().insert(&app).is_ok());
        assert!(get_group_by_app().delete(&app).is_ok());
        assert!(get_group_by_app().find(&app).is_err());
    }
}