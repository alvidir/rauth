use std::error::Error;
use std::sync::Arc;
use super::domain::SessionToken;
use crate::user::application::UserRepository;

pub trait SessionRepository {
    fn find(&mut self, token: &SessionToken) -> Result<SessionToken, Box<dyn Error>>;
    fn save(&mut self, token: &SessionToken) -> Result<(), Box<dyn Error>>;
    fn delete(&mut self, token: &SessionToken) -> Result<(), Box<dyn Error>>;
}

pub struct SessionApplication<SR: SessionRepository, UR: UserRepository> {
    pub sess_repo: Arc<SR>,
    pub user_repo: Arc<UR>,
}


impl<SR: SessionRepository, UR: UserRepository> SessionApplication<SR, UR> {
    pub fn login(&self, ident: &str, pwd: &str, totp: &str) -> Result<SessionToken, Box<dyn Error>> {
        info!("got a \"login\" request from email {} ", ident);        
        Err("".into())
    }

    pub fn logout(&self, token: &SessionToken) -> Result<(), Box<dyn Error>> {
        info!("got a \"logout\" request from user id {} ", token.sub);        
        Err("".into())
    }
}