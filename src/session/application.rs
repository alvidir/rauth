use std::error::Error;
use std::sync::Arc;
use super::domain::SessionToken;
use crate::user::application::UserRepository;
use crate::secret::application::SecretRepository;

pub trait SessionRepository {
    fn find(&mut self, token: &SessionToken) -> Result<SessionToken, Box<dyn Error>>;
    fn save(&mut self, token: &SessionToken) -> Result<(), Box<dyn Error>>;
    fn delete(&mut self, token: &SessionToken) -> Result<(), Box<dyn Error>>;
}

pub struct SessionApplication<S: SessionRepository, U: UserRepository, E: SecretRepository> {
    pub session_repo: Arc<S>,
    pub user_repo: Arc<U>,
    pub secret_repo: Arc<E>,
}

impl<S: SessionRepository, U: UserRepository, E: SecretRepository> SessionApplication<S, U, E> {
    pub fn login(&self, ident: &str, pwd: &str, totp: &str) -> Result<SessionToken, Box<dyn Error>> {
        info!("got a \"login\" request from email {} ", ident);        
        Err("".into())
    }

    pub fn logout(&self, token: &SessionToken) -> Result<(), Box<dyn Error>> {
        info!("got a \"logout\" request from user id {} ", token.sub);        
        Err("".into())
    }
}