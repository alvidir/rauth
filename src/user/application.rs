use std::time::SystemTime;
use std::error::Error;
use std::sync::Arc;
use crate::metadata::domain::Metadata;
use crate::secret::{
    application::SecretRepository,
    domain::Secret,
};

use crate::constants;
use crate::security;
use super::domain::User;

pub trait UserRepository {
    fn find(&self, id: i32) -> Result<User, Box<dyn Error>>;
    fn find_by_email(&self, email: &str) -> Result<User, Box<dyn Error>>;
    fn find_by_name(&self, name: &str) -> Result<User, Box<dyn Error>>;
    fn create(&self, user: &mut User) -> Result<(), Box<dyn Error>>;
    fn save(&self, user: &User) -> Result<(), Box<dyn Error>>;
    fn delete(&self, user: &User) -> Result<(), Box<dyn Error>>;
}

pub struct UserApplication<U: UserRepository, E: SecretRepository> {
    pub user_repo: Arc<U>,
    pub secret_repo: Arc<E>,
}


impl<U: UserRepository, E: SecretRepository> UserApplication<U, E> {
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
        let shadowed_pwd = security::shadow(pwd, constants::PWD_SUFIX);
        if !user.match_password(&shadowed_pwd) {
            return Err(constants::ERR_NOT_FOUND.into());
        }

        // if, and only if, the user has activated the totp
        if let Ok(secret) = self.secret_repo.find_by_user_and_name(user.id, constants::TOTP_SECRET_NAME) {
            if !secret.is_deleted() {
                if totp.len() == 0 {
                    return Err(constants::ERR_UNAUTHORIZED.into());
                }
    
                let data = secret.get_data();
                if !security::verify_totp(data, totp)? {
                    return Err(constants::ERR_UNAUTHORIZED.into());
                }
    
                self.secret_repo.delete(&secret)?;
            }
        }

        self.user_repo.delete(&user)?;
        Ok(())
    }

    pub fn enable_totp(&self, user_id: i32, pwd: &str, totp: &str) -> Result<String, Box<dyn Error>> {
        info!("got an \"enable totp\" request from user id {} ", user_id);

        let user = self.user_repo.find(user_id)?;
        let shadowed_pwd = security::shadow(pwd, constants::PWD_SUFIX);
        if !user.match_password(&shadowed_pwd) {
            return Err(constants::ERR_NOT_FOUND.into());
        }

        // if, and only if, the user has activated the totp
        if let Ok(secret) = &mut self.secret_repo.find_by_user_and_name(user.id, constants::TOTP_SECRET_NAME) {
            if !secret.is_deleted() {
                // the totp is already enabled
                return Err(constants::ERR_UNAUTHORIZED.into())
            }

            let data = secret.get_data();
            if !security::verify_totp(data, totp)? {
                return Err(constants::ERR_UNAUTHORIZED.into());
            }

            secret.set_deleted_at(None);
            self.secret_repo.save(&secret)?;
        }
        
        let token = security::get_random_string(constants::TOTP_SECRET_LEN);
        let mut secret = Secret::new(constants::TOTP_SECRET_NAME, token.as_bytes());
        secret.set_deleted_at(Some(SystemTime::now())); // unavailable till confirmed
        self.secret_repo.create(&mut secret)?;
        Ok(token)
    }

    pub fn disable_totp(&self, user_id: i32, pwd: &str, totp: &str) -> Result<(), Box<dyn Error>> {
        info!("got an \"disable totp\" request from user id {} ", user_id);
        
        let user = self.user_repo.find(user_id)?;
        let shadowed_pwd = security::shadow(pwd, constants::PWD_SUFIX);
        if !user.match_password(&shadowed_pwd) {
            return Err(constants::ERR_NOT_FOUND.into());
        }

        // if, and only if, the user has activated the totp
        if let Ok(secret) = &mut self.secret_repo.find_by_user_and_name(user.id, constants::TOTP_SECRET_NAME) {
            if secret.is_deleted() {
                // the totp is not enabled yet
                return Err(constants::ERR_UNAUTHORIZED.into())
            }

            let data = secret.get_data();
            if !security::verify_totp(data, totp)? {
                return Err(constants::ERR_UNAUTHORIZED.into());
            }

            self.secret_repo.delete(&secret)?;
            return Ok(());
        }

        Err(constants::ERR_NOT_FOUND.into())
    }
}

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use std::sync::Arc;
    use super::super::domain::{
        tests::new_user,
        User
    };
    use super::{UserRepository, UserApplication};
    use crate::secret::application::tests::SecretRepositoryMock;

    pub struct UserRepositoryMock {
        fn_find: Option<fn (this: &UserRepositoryMock, id: i32) -> Result<User, Box<dyn Error>>>,
        fn_find_by_email: Option<fn (this: &UserRepositoryMock, email: &str) -> Result<User, Box<dyn Error>>>,
        fn_find_by_name: Option<fn (this: &UserRepositoryMock, name: &str) -> Result<User, Box<dyn Error>>>,
        fn_create: Option<fn (this: &UserRepositoryMock, user: &mut User) -> Result<(), Box<dyn Error>>>,
        fn_save: Option<fn (this: &UserRepositoryMock, user: &User) -> Result<(), Box<dyn Error>>>,
        fn_delete: Option<fn (this: &UserRepositoryMock, user: &User) -> Result<(), Box<dyn Error>>>,
    }

    impl UserRepositoryMock {
        fn new() -> Self {
            UserRepositoryMock {
                fn_find: None,
                fn_find_by_email: None,
                fn_find_by_name: None,
                fn_create: None,
                fn_save: None,
                fn_delete: None,
            }
        }
    }

    impl UserRepository for UserRepositoryMock {
        fn find(&self, id: i32) -> Result<User, Box<dyn Error>> {
            if let Some(f) = self.fn_find {
                return f(self, id);
            }

            Ok(new_user())
        }

        fn find_by_email(&self, email: &str) -> Result<User, Box<dyn Error>> {
            if let Some(f) = self.fn_find_by_email {
                return f(self, email);
            }

            Ok(new_user())
        }

        fn find_by_name(&self, name: &str) -> Result<User, Box<dyn Error>> {
            if let Some(f) = self.fn_find_by_name {
                return f(self, name);
            }

            Ok(new_user())
        }

        fn create(&self, user: &mut User) -> Result<(), Box<dyn Error>> {
            if let Some(f) = self.fn_create {
                return f(self, user);
            }

            user.id = 999;
            Ok(())
        }

        fn save(&self, user: &User) -> Result<(), Box<dyn Error>> {
            if let Some(f) = self.fn_save {
                return f(self, user);
            }

            Ok(())
        }

        fn delete(&self, user: &User) -> Result<(), Box<dyn Error>> {
            if let Some(f) = self.fn_delete {
                return f(self, user);
            }

            Ok(())
        }
    }

    #[test]
    fn user_signup_should_not_fail() {
        let user_repo = UserRepositoryMock::new();
        let secret_repo = SecretRepositoryMock::new();
        
        let app = UserApplication{
            user_repo: Arc::new(user_repo),
            secret_repo: Arc::new(secret_repo),
        };

        const PWD: &str = "ABCDEF1234567890";
        const EMAIL: &str = "dummy@test.com";
        let user = app.signup(EMAIL, PWD).unwrap();

        assert_eq!(user.id, 999);
    }
}