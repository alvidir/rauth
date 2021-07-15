use std::error::Error;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tonic::{Request, Response, Status};
use crate::constants::TOKEN_LEN;
use crate::token;
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
    session_repo: &'static InMemorySessionRepository,
}

impl SessionServiceImplementation {
    pub fn new(session_repo: &'static InMemorySessionRepository) -> Self {
        SessionServiceImplementation {
            session_repo: session_repo,
        }
    }
}

#[tonic::async_trait]
impl SessionService for SessionServiceImplementation {
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<LoginResponse>, Status> {
        let msg_ref = request.into_inner();
        Err(Status::unimplemented(""))
    }

    async fn logout(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();
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
    fn find(&self, cookie: &str) -> Result<Arc<Mutex<Session>>, Box<dyn Error>> {
        let repo = self.all_instances.lock()?;
        if let Some(sess) = repo.get(cookie) {
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
            return Err("cookie already exists".into());
        }

        if let Some(_) = repo.iter().find(|(_, sess)| InMemorySessionRepository::session_has_email(sess, &session.user.email)) {
            return Err("email already exists".into());
        }

        let token = token::new(TOKEN_LEN);
        session.token = token.clone();

        let mu = Mutex::new(session);
        let arc = Arc::new(mu);

        repo.insert(token.clone(), arc);
        Ok(token)
    }

    fn delete(&self, session: &Session) -> Result<(), Box<dyn Error>> {
        let mut repo = self.all_instances.lock()?;
        if let None = repo.remove(&session.token) {
            return Err("cookie does not exists".into());
        }

        Ok(())
    }
}
