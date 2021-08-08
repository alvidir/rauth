use std::error::Error;
use std::sync::{Arc, RwLock, RwLockWriteGuard, RwLockReadGuard};
use std::collections::{HashMap, HashSet};
use tonic::{Request, Response, Status};
use crate::security;
use crate::constants::{settings, errors};
use crate::app::domain::App;
use super::domain::{Session, SessionRepository};
use super::application::get_writable_session;

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
    group_by_app: RwLock<HashMap<String, Arc<RwLock<HashSet<String>>>>>,
}

impl InMemorySessionRepository {
    pub fn new() -> Self {
        InMemorySessionRepository {
            all_instances: {
                let repo = HashMap::new();
                RwLock::new(repo)
            },

            group_by_app: {
                let repo = HashMap::new();
                RwLock::new(repo)
            },
        }
    }

    fn session_has_email(sess: &Arc<RwLock<Session>>, email: &str) -> bool {
        if let Ok(session) = sess.read() {
            return session.user.email == email;
        }

        false
    }

    fn get_readable_sids(sids_arc: &Arc<RwLock<HashSet<String>>>) -> Result<RwLockReadGuard<HashSet<String>>, Box<dyn Error>> {
        let sids_rd = sids_arc.read();
        if let Err(err) = sids_rd {
            error!("read-only lock for set of sessions IDs got poisoned: {}", err);
            return Err(errors::POISONED.into());
        }

        Ok(sids_rd.unwrap()) // this line will not panic due the previous check of Err
    }

    fn get_writable_sids(sids_arc: &Arc<RwLock<HashSet<String>>>) -> Result<RwLockWriteGuard<HashSet<String>>, Box<dyn Error>> {
        let sids_wr = sids_arc.write();
        if let Err(err) = sids_wr {
            error!("read-write lock for set of sessions IDs got poisoned: {}", err);
            return Err(errors::POISONED.into());
        }

        Ok(sids_wr.unwrap()) // this line will not panic due the previous check of Err
    }

    fn get_readable_repo(&self) -> Result<RwLockReadGuard<HashMap<String, Arc<RwLock<Session>>>>, Box<dyn Error>> {
        let repo_rd = self.all_instances.read();
        if let Err(err) = &repo_rd {
            error!("read-only lock for all_instances from session's repo got poisoned: {}", err);
            return Err(errors::POISONED.into());
        }

        Ok(repo_rd.unwrap()) // this line will not panic due the previous check of Err
    }

    fn get_writable_repo(&self) -> Result<RwLockWriteGuard<HashMap<String, Arc<RwLock<Session>>>>, Box<dyn Error>> {
        let repo_wr = self.all_instances.write();
        if let Err(err) = &repo_wr {
            error!("read-write lock for all_instances from session's repo got poisoned: {}", err);
            return Err(errors::POISONED.into());
        }

        Ok(repo_wr.unwrap()) // this line will not panic due the previous check of Err
    }

    fn get_readable_group(&self) -> Result<RwLockReadGuard<HashMap<String, Arc<RwLock<HashSet<String>>>>>, Box<dyn Error>>{
        let group_rd = self.group_by_app.read();
        if let Err(err) = &group_rd {
            error!("read-only lock for group_by_app from session's repo got poisoned: {}", err);
            return Err(errors::POISONED.into());
        }

        Ok(group_rd.unwrap()) // this line will not panic due the previous check of Err
    }

    fn get_writable_group(&self) -> Result<RwLockWriteGuard<HashMap<String, Arc<RwLock<HashSet<String>>>>>, Box<dyn Error>>{
        let group_wr = self.group_by_app.write();
        if let Err(err) = &group_wr {
            error!("read-write lock for group_by_app from session's repo got poisoned: {}", err);
            return Err(errors::POISONED.into());
        }

        Ok(group_wr.unwrap()) // this line will not panic due the previous check of Err
    }

    fn create_group(&self, url: &str, sid: &str) -> Result<(), Box<dyn Error>> {
        let mut sids = HashSet::new();
        sids.insert(sid.to_string());

        let mu = RwLock::new(sids);
        let arc = Arc::new(mu);

        let mut group = self.get_writable_group()?;
        group.insert(url.to_string(), arc);
        Ok(())
    }

    fn destroy_group(&self, url: &str) -> Result<(), Box<dyn Error>> {
        let mut group = self.get_writable_group()?;
        let size = {
            if let Some(sids_arc) = group.get(url){
                let sids = InMemorySessionRepository::get_readable_sids(sids_arc)?;
                sids.len()
            } else {
                0
            }
        };

        if size > 0 {
            warn!("cannot remove group {}, got size {} want 0", url, size);
        } else {
            group.remove(url);
        }

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
        let repo = self.get_readable_repo()?;
        if let Some((_, sess)) = repo.iter().find(|(_, sess)| InMemorySessionRepository::session_has_email(sess, email)) {
            return Ok(Arc::clone(sess));
        }

        Err(errors::NOT_FOUND.into())
    }

    fn insert(&self, mut session: Session) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {
        let mut repo = self.get_writable_repo()?;
        if let Some(_) = repo.get(&session.sid) {
            return Err(errors::ALREADY_EXISTS.into());
        }

        if let Some(_) = repo.iter().find(|(_, sess)| InMemorySessionRepository::session_has_email(sess, &session.user.email)) {
            return Err(errors::ALREADY_EXISTS.into());
        }

        loop { // make sure the token is unique
            let sid = security::get_random_string(settings::TOKEN_LEN);
            if repo.get(&sid).is_none() {
                session.sid = sid;
                break;
            }
        }
        
        let token = session.sid.clone();
        let mu = RwLock::new(session);
        let arc = Arc::new(mu);

        repo.insert(token.to_string(), arc);
        let sess = repo.get(&token).unwrap(); // this line will not panic due to the previous insert
        Ok(Arc::clone(sess))
    }

    fn delete(&self, session: &Session) -> Result<(), Box<dyn Error>> {
        let mut repo = self.get_writable_repo()?;
        if let None = repo.remove(&session.sid) {
            return Err(errors::NOT_FOUND.into());
        }

        Ok(())
    }

    fn delete_all_by_app(&self, app: &App) -> Result<(), Box<dyn Error>> {    
        { // write lock is released at the end of this block
            let group = self.get_readable_group()?;
            let sids_search = group.get(app.get_url());
            if let None = sids_search {
                return Err(errors::NOT_FOUND.into());
            }

            let sids_arc = sids_search.unwrap(); // this line will not panic due to the previous check of None
            let sids = InMemorySessionRepository::get_readable_sids(sids_arc)?;
            
            for sid in sids.iter() {
                let repo = self.get_writable_repo()?;
                if let Some(sess_arc) = repo.get(sid) {
                    let mut sess = get_writable_session(sess_arc)?;
                    sess.delete_directory(app);
                }
            }
        }

        // and empty group cannot exists, so it must be removed
        self.destroy_group(app.get_url())
    }

    fn find_all_by_app(&self, app: &App) -> Result<Vec<Arc<RwLock<Session>>>, Box<dyn Error>> {
        let sids_arc = {
            let group = self.get_readable_group()?;
            match group.get(app.get_url()){
                Some(sids) => Arc::clone(sids),
                None => {
                    return Err(errors::NOT_FOUND.into());
                }
            }
        };

        let sids = InMemorySessionRepository::get_readable_sids(&sids_arc)?;
        
        let mut all_sess = Vec::new();
        for sid in sids.iter() {
            let repo = self.get_readable_repo()?;
            if let Some(sess_arc) = repo.get(sid) {
                all_sess.push(Arc::clone(sess_arc));
            } else {
                error!("sid {} exists in group by {} but not found in repository", sid, app.get_url());
            }
        }

        Ok(all_sess)
    }

    fn add_to_app_group(&self, app: &App, sess: &Session) -> Result<(), Box<dyn Error>> {
        let create = {
            let group = self.get_readable_group()?;
            if let Some(sids_arc) = group.get(app.get_url()){
                let mut sids = InMemorySessionRepository::get_writable_sids(sids_arc)?;
                if let None = sids.iter().position(|item| *item == sess.sid) {
                    sids.insert(sess.sid.to_string());
                }

                false
            } else {
                // if no group for the given url has been found then it must be created,
                // being sid the first session_id to insert
                true
            }
        };
        
        if create { // if group == None then ...
            self.create_group(app.get_url(), &sess.sid)?;
        }

        Ok(())
    }

    fn delete_from_app_group(&self, app: &App, sess: &Session) -> Result<(), Box<dyn Error>> {
        let size = { // read lock is released at the end of this block
            let group = self.get_readable_group()?;
            if let Some(sids_arc) = group.get(app.get_url()){
                let mut sids = InMemorySessionRepository::get_writable_sids(sids_arc)?;
                sids.remove(&sess.sid);

                sids.len()
            } else {
                0
            }
        };
        
        if size == 0 {
            self.destroy_group(app.get_url())?;
        }

        Ok(())
    }
}