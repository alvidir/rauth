use std::error::Error;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tonic::{Request, Response, Status};
use crate::user::framework::PostgresUserRepository;
use crate::user::domain::UserRepository;
use crate::app::framework::PostgresAppRepository;
use crate::directory::framework::MongoDirectoryRepository;
use crate::directory::domain::DirectoryRepository;
use crate::constants::TOKEN_LEN;
use crate::security;
use crate::constants::{ERR_NOT_FOUND, ERR_ALREADY_EXISTS};
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

        let user_search = self.user_repo.find(&msg_ref.ident);
        if let Err(err) = user_search {
            return Err(Status::not_found(err.to_string()));
        } 

        let user = user_search.unwrap();
        if user.secret.is_none() {
            return Err(Status::unauthenticated("user not verified"));
        }

        let secret = user.secret.unwrap();
        let data = secret.get_data();
        if let Err(err) = security::verify_totp_password(data, &msg_ref.pwd) {
            return Err(Status::unauthenticated(err.to_string()));
        }

        match super::application::session_login(&self.sess_repo,
                                                &self.user_repo,
                                                &self.app_repo,
                                                &self.dir_repo,
                                                &msg_ref.ident,
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
        let _msg_ref = request.into_inner();
        Err(Status::unimplemented(""))
    }
}


pub struct InMemorySessionRepository {
    all_instances: RwLock<HashMap<String, Arc<RwLock<Session>>>>,
    group_by_app: RwLock<HashMap<String, Arc<RwLock<Vec<String>>>>>,
    dir_repo: &'static MongoDirectoryRepository,
}

impl InMemorySessionRepository {
    pub fn new(dir_repo: &'static MongoDirectoryRepository) -> Self {
        InMemorySessionRepository {
            all_instances: {
                let repo = HashMap::new();
                RwLock::new(repo)
            },

            group_by_app: {
                let repo = HashMap::new();
                RwLock::new(repo)
            },

            dir_repo: dir_repo,
        }
    }

    fn session_has_email(sess: &Arc<RwLock<Session>>, email: &str) -> bool {
        if let Ok(session) = sess.read() {
            return session.user.email == email;
        }

        false
    }

    fn create_group(&self, url: &str, sid: &str) -> Result<(), Box<dyn Error>> {
        let sids = vec!(sid.to_string());
        let mu = RwLock::new(sids);
        let arc = Arc::new(mu);

        let mut group = self.group_by_app.write().unwrap();
        group.insert(url.to_string(), arc);
        Ok(())
    }

    fn destroy_group(&self, url: &str, force: bool) -> Result<(), Box<dyn Error>> {
        let mut group = self.group_by_app.write().unwrap();
        let empty = {
            if let Some(sids_arc) = group.get(url){
                let sids = sids_arc.read().unwrap();
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
        let repo = self.all_instances.read().unwrap();
        if let Some(sess) = repo.get(token) {
            return Ok(Arc::clone(sess));
        }

        Err(ERR_NOT_FOUND.into())
    }

    fn find_by_email(&self, email: &str) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {
        let repo = self.all_instances.read().unwrap();
        if let Some((_, sess)) = repo.iter().find(|(_, sess)| InMemorySessionRepository::session_has_email(sess, email)) {
            return Ok(Arc::clone(sess));
        }

        Err(ERR_NOT_FOUND.into())
    }

    fn save(&self, mut session: Session) -> Result<Arc<RwLock<Session>>, Box<dyn Error>> {
        let mut repo = self.all_instances.write().unwrap();
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
        let sess = repo.get(&token).unwrap();
        Ok(Arc::clone(sess))
    }

    fn delete(&self, session: &Session) -> Result<(), Box<dyn Error>> {
        let mut repo = self.all_instances.write().unwrap();
        if let Some(sess_arc) = repo.remove(&session.sid) {
            let mut sess = sess_arc.write().unwrap();
            let _ = sess.apps.iter_mut().map(|(url, dir)| {
                // foreach application the session was logged in
                if let Err(err) = self.dir_repo.save(dir) {
                    println!("got error while saving directory {} : {}", dir.id, err);
                } else if let Err(err) = self.remove(url, &session.sid) {
                    println!("got error while removing session from group : {}", err);
                }
            });
        }

        Err(ERR_NOT_FOUND.into())
    }
}

impl GroupByAppRepository for &InMemorySessionRepository {
    fn get(&self, url: &str) -> Result<Arc<RwLock<Vec<String>>>, Box<dyn Error>> {
        let group = self.group_by_app.read().unwrap();
        if let Some(sids) = group.get(url){
            return Ok(Arc::clone(sids));
        }
        
        Err(ERR_NOT_FOUND.into())
    }

    fn store(&self, url: &str, sid: &str) -> Result<(), Box<dyn Error>> {
        let create = {
            let group = self.group_by_app.read().unwrap();
            if let Some(sids_arc) = group.get(url){
                let mut sids = sids_arc.write().unwrap();
                if let None = sids.iter().position(|item| item == sid) {
                    sids.push(sid.to_string());
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
            let group = self.group_by_app.read().unwrap();
            if let Some(sids_arc) = group.get(url){
                let mut sids = sids_arc.write().unwrap();
                if let Some(index) = sids.iter().position(|item| item == sid) {
                    sids.remove(index);
                }

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