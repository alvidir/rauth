use std::error::Error;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tonic::{Request, Response, Status};
use crate::constants::TOKEN_LEN;
use super::domain::{Session, SessionRepository};

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("session");
}

// Proto generated server traits
use proto::session_service_server::SessionService;
pub use proto::session_service_server::SessionServiceServer;

// Proto message structs
use proto::{LoginRequest, LoginResponse};

#[derive(Default)]
pub struct SessionServiceImplementation {}

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
    all_instances: &'static Mutex<HashMap<String, Arc<Mutex<Session>>>>,
}

lazy_static! {
    pub static ref SESSION_REPOSITORY: Mutex<HashMap<String, Arc<Mutex<Session>>>> = {
        let repo = HashMap::new();
        Mutex::new(repo)
    };    
}

impl InMemorySessionRepository {
    fn new() -> Self {
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

impl SessionRepository for InMemorySessionRepository {
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

    fn save(&self, session: Session) -> Result<(), Box<dyn Error>> {
        let mut repo = self.all_instances.lock()?;
        if let Some(_) = repo.get(&session.token) {
            return Err("cookie already exists".into());
        }

        if let Some(_) = repo.iter().find(|(_, sess)| InMemorySessionRepository::session_has_email(sess, &session.user.email)) {
            return Err("email already exists".into());
        }

        let mu = Mutex::new(session);
        let arc = Arc::new(mu);

        let token = Session::generate_token(TOKEN_LEN);
        repo.insert(token, arc);

        Ok(())
    }

    fn delete(&self, session: &Session) -> Result<(), Box<dyn Error>> {
        let mut repo = self.all_instances.lock()?;
        if let None = repo.remove(&session.token) {
            return Err("cookie does not exists".into());
        }

        Ok(())
    }
}
