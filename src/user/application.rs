use std::error::Error;
use std::sync::Arc;
use crate::metadata::domain::Metadata;
use crate::secret::application::SecretRepository;
use crate::session::application::SessionRepository;
use super::domain::User;

pub trait UserRepository {
    fn find(&self, id: i32) -> Result<User, Box<dyn Error>>;
    fn find_by_email(&self, email: &str) -> Result<User, Box<dyn Error>>;
    fn create(&self, user: &mut User) -> Result<(), Box<dyn Error>>;
    fn save(&self, user: &User) -> Result<(), Box<dyn Error>>;
    fn delete(&self, user: &User) -> Result<(), Box<dyn Error>>;
}

pub struct UserApplication<U: UserRepository, S: SessionRepository, E: SecretRepository> {
    pub user_repo: Arc<U>,
    pub session_repo: Arc<S>,
    pub secret_repo: Arc<E>,
}


impl<U: UserRepository, S: SessionRepository, E: SecretRepository> UserApplication<U, S, E> {
    pub fn signup(&self, email: &str, pwd: &str) -> Result<User, Box<dyn Error>> {
        info!("got a \"signup\" request from email {} ", email);
   
        let meta = Metadata::new();
        let mut user = User::new(meta, email, pwd)?;
        self.user_repo.create(&mut user)?;
        
        Ok(user)
    }

    pub fn delete(&self, user_id: i32, pwd: &str, totp: &str) -> Result<(), Box<dyn Error>> {
        info!("got a \"delete\" request from user id {} ", user_id);
        
        let user = self.user_repo.find(user_id)?;
        if !user.match_password(pwd) {
            return Err("not found".into());
        }

        self.user_repo.delete(&user)?;
        Ok(())
    }

    pub fn enable_totp(&self, user_id: i32, pwd: &str, totp: &str) -> Result<(), Box<dyn Error>> {
        info!("got an \"enable totp\" request from user id {} ", user_id);
        
        Err("unimplemented".into())
    }

    pub fn disable_totp(&self, user_id: i32, pwd: &str, totp: &str) -> Result<(), Box<dyn Error>> {
        info!("got an \"disable totp\" request from user id {} ", user_id);
        
        Err("unimplemented".into())
    }
}