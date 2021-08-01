use std::error::Error;
use std::sync::{Arc, RwLock, RwLockWriteGuard, RwLockReadGuard};
use std::collections::{HashMap, HashSet};
use tonic::{Request, Response, Status};
use crate::user::framework::PostgresUserRepository;
use crate::user::domain::UserRepository;
use crate::app::framework::PostgresAppRepository;
use crate::directory::framework::MongoDirectoryRepository;
use crate::constants::TOKEN_LEN;
use crate::security;
use crate::constants::{ERR_NOT_FOUND, ERR_ALREADY_EXISTS, ERR_POISONED};
use super::domain::{Session, SessionRepository};
use super::application::GroupByAppRepository;

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("session");
}

// Proto generated server traits
use proto::session_service_server::SessionService;
pub use proto::session_service_server::SessionServiceServer;

// Proto message structs
use proto::{LoginRequest, LoginResponse};

pub struct SessionServiceImplementation {
    sess_repo: &'static InMemorySessionRepository,
    user_repo: &'static PostgresUserRepository,
    app_repo: &'static PostgresAppRepository,
    dir_repo: &'static MongoDirectoryRepository
}

impl SessionServiceImplementation {
    pub fn new(sess_repo: &'static InMemorySessionRepository,
               user_repo: &'static PostgresUserRepository,
               app_repo: &'static PostgresAppRepository,
               dir_repo: &'static MongoDirectoryRepository) -> Self {
        
        SessionServiceImplementation {
            sess_repo: sess_repo,
            user_repo: user_repo,
            app_repo: app_repo,
            dir_repo: dir_repo,
        }
    }
}

#[tonic::async_trait]
impl SessionService for SessionServiceImplementation {
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<LoginResponse>, Status> {
        let msg_ref = request.into_inner();

        match self.user_repo.find(&msg_ref.ident) {
            Err(_) => {
                // in order to give no clue about if the error was about the email or password
                // both cases must provide the same kind of error
                return Err(Status::not_found(ERR_NOT_FOUND))
            },

            Ok(user) => {
                if !user.match_password(&msg_ref.pwd) {
                    // same error as if the user was not found
                    return Err(Status::not_found(ERR_NOT_FOUND));
                }

                // if, and only if, the user has activated the 2fa
                if let Some(secret) = user.secret {
                    let data = secret.get_data();
                    if let Err(err) = security::verify_totp_password(data, &msg_ref.pwd) {
                        // in order to make the application know a valid TOTP is required
                        return Err(Status::unauthenticated(err.to_string()));
                    }
                }
            }
        };

        match super::application::session_login(&self.sess_repo,
                                                &self.user_repo,
                                                &self.app_repo,
                                                &self.dir_repo,
                                                &msg_ref.ident,
                                                &msg_ref.pwd,
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

        if let Err(err) = super::application::session_logout(&self.sess_repo,
                                                             &self.dir_repo,
                                                             token){               
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
            return Err(ERR_POISONED.into());
        }

        Ok(sids_rd.unwrap()) // this line will not panic due the previous check of Err
    }

    fn get_writable_sids(sids_arc: &Arc<RwLock<HashSet<String>>>) -> Result<RwLockWriteGuard<HashSet<String>>, Box<dyn Error>> {
        let sids_wr = sids_arc.write();
        if let Err(err) = sids_wr {
            error!("read-write lock for set of sessions IDs got poisoned: {}", err);
            return Err(ERR_POISONED.into());
        }

        Ok(sids_wr.unwrap()) // this line will not panic due the previous check of Err
    }

    fn get_readable_repo(&self) -> Result<RwLockReadGuard<HashMap<String, Arc<RwLock<Session>>>>, Box<dyn Error>> {
        let repo_rd = self.all_instances.read();
        if let Err(err) = &repo_rd {
            error!("read-only lock for all_instances from session's repo got poisoned: {}", err);
            return Err(ERR_POISONED.into());
        }

        Ok(repo_rd.unwrap()) // this line will not panic due the previous check of Err
    }

    fn get_writable_repo(&self) -> Result<RwLockWriteGuard<HashMap<String, Arc<RwLock<Session>>>>, Box<dyn Error>> {
        let repo_wr = self.all_instances.write();
        if let Err(err) = &repo_wr {
            error!("read-write lock for all_instances from session's repo got poisoned: {}", err);
            return Err(ERR_POISONED.into());
        }

        Ok(repo_wr.unwrap()) // this line will not panic due the previous check of Err
    }

    fn get_readable_group(&self) -> Result<RwLockReadGuard<HashMap<String, Arc<RwLock<HashSet<String>>>>>, Box<dyn Error>>{
        let group_rd = self.group_by_app.read();
        if let Err(err) = &group_rd {
            error!("read-only lock for group_by_app from session's repo got poisoned: {}", err);
            return Err(ERR_POISONED.into());
        }

        Ok(group_rd.unwrap()) // this line will not panic due the previous check of Err
    }

    fn get_writable_group(&self) -> Result<RwLockWriteGuard<HashMap<String, Arc<RwLock<HashSet<String>>>>>, Box<dyn Error>>{
        let group_wr = self.group_by_app.write();
        if let Err(err) = &group_wr {
            error!("read-write lock for group_by_app from session's repo got poisoned: {}", err);
            return Err(ERR_POISONED.into());
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

    fn destroy_group(&self, url: &str, force: bool) -> Result<(), Box<dyn Error>> {
        let mut group = self.get_writable_group()?;
        let empty = {
            if let Some(sids_arc) = group.get(url){
                let sids = InMemorySessionRepository::get_readable_sids(sids_arc)?;
                sids.len() == 0
            } else {
                false
            }
        };

        if empty || force {
            group.remove(url);
        }

        Ok(())
    }
}

impl SessionRepository for &InMemorySessionRepository {
    fn find(&self, token: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {
        let repo = self.get_readable_repo()?;
        if let Some(sess) = repo.get(token) {
            return Ok(Arc::clone(sess));
        }

        Err(ERR_NOT_FOUND.into())
    }

    fn find_by_email(&self, email: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {
        let repo = self.get_readable_repo()?;
        if let Some((_, sess)) = repo.iter().find(|(_, sess)| InMemorySessionRepository::session_has_email(sess, email)) {
            return Ok(Arc::clone(sess));
        }

        Err(ERR_NOT_FOUND.into())
    }

    fn save(&self, mut session: Session) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {
        let mut repo = self.get_writable_repo()?;
        if let Some(_) = repo.get(&session.sid) {
            return Err(ERR_ALREADY_EXISTS.into());
        }

        if let Some(_) = repo.iter().find(|(_, sess)| InMemorySessionRepository::session_has_email(sess, &session.user.email)) {
            return Err(ERR_ALREADY_EXISTS.into());
        }

        loop { // make sure the token is unique
            let sid = security::get_random_string(TOKEN_LEN);
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
            return Err(ERR_NOT_FOUND.into());
        }

        Ok(())
    }
}

impl GroupByAppRepository for &InMemorySessionRepository {
    fn get(&self, url: &str) -> Result<Arc<RwLock<HashSet<String>>>, Box<dyn Error>> {
        let group = self.get_readable_group()?;
        if let Some(sids) = group.get(url){
            return Ok(Arc::clone(sids));
        }
        
        Err(ERR_NOT_FOUND.into())
    }

    fn store(&self, url: &str, sid: &str) -> Result<(), Box<dyn Error>> {
        let create = {
            let group = self.get_readable_group()?;
            if let Some(sids_arc) = group.get(url){
                let mut sids = InMemorySessionRepository::get_writable_sids(sids_arc)?;
                if let None = sids.iter().position(|item| item == sid) {
                    sids.insert(sid.to_string());
                }

                false
            } else {
                true
            }
        };
        
        if create {
            self.create_group(url, sid)?;
        }

        Ok(())
    }

    fn remove(&self, url: &str, sid: &str) -> Result<(), Box<dyn Error>> {
        let destroy = { // read lock is released at the end of this block
            let group = self.get_readable_group()?;
            if let Some(sids_arc) = group.get(url){
                let mut sids = InMemorySessionRepository::get_writable_sids(sids_arc)?;
                sids.remove(sid);

                sids.len() == 0
            } else {
                false
            }
        };

        if destroy {
            self.destroy_group(url, false)?;
        }

        Ok(())
    }
}