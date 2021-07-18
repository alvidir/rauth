use std::error::Error;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tonic::{Request, Response, Status};
use crate::user::framework::PostgresUserRepository;
use crate::user::domain::UserRepository;
use crate::app::framework::PostgresAppRepository;
use crate::metadata::framework::PostgresMetadataRepository;
use crate::constants::TOKEN_LEN;
use crate::security;
use super::domain::{Session, SessionRepository};

type Ropository = Mutex<HashMap<String, Arc<Mutex<Session>>>>;

lazy_static! {
    pub static ref SESSION_REPOSITORY: Ropository = {
        let repo = HashMap::new();
        Mutex::new(repo)
    };    
}

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
    meta_repo: &'static PostgresMetadataRepository,
}

impl SessionServiceImplementation {
    pub fn new(sess_repo: &'static InMemorySessionRepository,
               user_repo: &'static PostgresUserRepository,
               app_repo: &'static PostgresAppRepository,
               meta_repo: &'static PostgresMetadataRepository) -> Self {
        
        SessionServiceImplementation {
            sess_repo: sess_repo,
            user_repo: user_repo,
            app_repo: app_repo,
            meta_repo: meta_repo,
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

        match super::application::session_login(Box::new(self.sess_repo),
                                                Box::new(self.user_repo),
                                                Box::new(self.app_repo),
                                                Box::new(self.meta_repo),
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
    all_instances: &'static Ropository,
}

impl InMemorySessionRepository {
    pub fn new() -> Self {
        InMemorySessionRepository {
            all_instances: &SESSION_REPOSITORY,
        }
    }

    fn session_has_email(sess: &Arc<Mutex<Session>>, email: &str) -> bool {
        if let Ok(session) = sess.lock() {
            return session.user.email == email;
        }

        false
    }
}

impl SessionRepository for &InMemorySessionRepository {
    fn find(&self, token: &str) -> Result<Arc<Mutex<Session>>, Box<dyn Error>> {
        let repo = self.all_instances.lock()?;
        if let Some(sess) = repo.get(token) {
            return Ok(Arc::clone(sess));
        }

        Err("Not found".into())
    }

    fn find_by_email(&self, email: &str) -> Result<Arc<Mutex<Session>>, Box<dyn Error>> {
        let repo = self.all_instances.lock()?;
        if let Some((_, sess)) = repo.iter().find(|(_, sess)| InMemorySessionRepository::session_has_email(sess, email)) {
            return Ok(Arc::clone(sess));
        }

        Err("Not found".into())
    }

    fn save(&self, mut session: Session) -> Result<String, Box<dyn Error>> {
        let mut repo = self.all_instances.lock()?;
        if let Some(_) = repo.get(&session.token) {
            return Err("token already exists".into());
        }

        if let Some(_) = repo.iter().find(|(_, sess)| InMemorySessionRepository::session_has_email(sess, &session.user.email)) {
            return Err("email already exists".into());
        }

        loop { // make sure the token is unique
            let token = security::generate_token(TOKEN_LEN);
            if repo.get(&token).is_none() {
                session.token = token;
                break;
            }
        }
       
        let token = session.token.clone();
        let mu = Mutex::new(session);
        let arc = Arc::new(mu);

        repo.insert(token.to_string(), arc);
        Ok(token)
    }

    fn delete(&self, session: &Session) -> Result<(), Box<dyn Error>> {
        let mut repo = self.all_instances.lock()?;
        if let None = repo.remove(&session.token) {
            return Err("token does not exists".into());
        }

        Ok(())
    }
}
