use crate::session::application::SessionRepository;
use std::error::Error;
use std::sync::Arc;
use crate::metadata::domain::Metadata;
use crate::session::domain::VerificationToken;
use super::domain::User;

pub trait UserRepository {
    fn find(&self, id: i32) -> Result<User, Box<dyn Error>>;
    fn find_by_email(&self, email: &str) -> Result<User, Box<dyn Error>>;
    fn create(&self, user: &mut User) -> Result<(), Box<dyn Error>>;
    fn save(&self, user: &User) -> Result<(), Box<dyn Error>>;
    fn delete(&self, user: &User) -> Result<(), Box<dyn Error>>;
}

pub struct UserApplication<UR: UserRepository, SR: SessionRepository> {
    pub user_repo: Arc<UR>,
    pub sess_repo: Arc<SR>
}

impl<UR: UserRepository, SR: SessionRepository> UserApplication<UR, SR> {
    pub fn signup(&self, token: VerificationToken, email: &str, pwd: &str) -> Result<User, Box<dyn Error>> {
        info!("got a \"signup\" request from email {} ", email);

        let final_email = match token.sub {
            Some(verified_email) => verified_email,
            None => email.to_string(),
        };

        let final_pwd = match token.pwd {
            Some(verified_pwd) => verified_pwd,
            None => pwd.to_string(),
        };
   
        let meta = Metadata::new();
        let mut user = User::new(meta, &final_email, &final_pwd)?;
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