use std::error::Error;
use std::time::{SystemTime, Duration};
use std::sync::Arc;
use crate::session::{
    application::SessionRepository,
    domain::{VerificationToken, SessionToken}
};
use crate::secret::{
    application::SecretRepository,
    domain::Secret,
};

use crate::constants;
use crate::security;
use crate::smtp::Mailer;
use super::domain::User;

pub trait UserRepository {
    fn find(&self, id: i32) -> Result<User, Box<dyn Error>>;
    fn find_by_email(&self, email: &str) -> Result<User, Box<dyn Error>>;
    fn find_by_name(&self, name: &str) -> Result<User, Box<dyn Error>>;
    fn create(&self, user: &mut User) -> Result<(), Box<dyn Error>>;
    fn save(&self, user: &User) -> Result<(), Box<dyn Error>>;
    fn delete(&self, user: &User) -> Result<(), Box<dyn Error>>;
}

pub struct UserApplication<U: UserRepository, E: SecretRepository, S: SessionRepository, M: Mailer> {
    pub user_repo: Arc<U>,
    pub secret_repo: Arc<E>,
    pub session_repo: Arc<S>,
    pub mailer: Arc<M>,
    pub timeout: u64,
}

impl<U: UserRepository, E: SecretRepository, S: SessionRepository, M: Mailer> UserApplication<U, E, S, M> {
    pub fn get_valid_session_token(&self, token: &str, jwt_public: &[u8]) -> Result<SessionToken, Box<dyn Error>> {
        let claims: SessionToken = security::verify_jwt(jwt_public, token)
            .map_err(|err| {
                warn!("{}: {}", constants::ERR_VERIFY_TOKEN, err);
                constants::ERR_VERIFY_TOKEN
            })?;
    
        self.session_repo.exists(&claims.sid.to_string())
            .map_err(|err| {
                warn!("{}: {}", constants::ERR_NOT_FOUND, err);
                constants::ERR_NOT_FOUND
            })?;
    
        Ok(claims)
    }

    pub fn verify_user(&self, email: &str, pwd: &str,  jwt_secret: &[u8]) -> Result<(), Box<dyn Error>> {
        info!("sending a verification email to {} ", email);
        
        if self.user_repo.find_by_email(email).is_ok() {
            // returns Ok to not provide information about users
            info!("user with email {} already exists", email);
            return Ok(());
        }

        User::new(email, pwd)?;

        let claims = VerificationToken::new(
            constants::TOKEN_ISSUER,
            email,
            pwd,
            Duration::from_secs(self.timeout)
        );
        
        let key = claims.tid.to_string();
        let token = security::sign_jwt(jwt_secret, claims)?;
        self.session_repo.save(&key, &token)?;
        self.mailer.send_verification_email(email, &token)?;
        Ok(())
    }

    pub fn secure_signup(&self, token: &str, jwt_public: &[u8])  -> Result<i32, Box<dyn Error>> {
        let claims: VerificationToken = security::verify_jwt(jwt_public, token)
            .map_err(|err| {
                warn!("{}: {}", constants::ERR_VERIFY_TOKEN, err);
                constants::ERR_VERIFY_TOKEN
            })?;

        self.session_repo.exists(&claims.tid.to_string())
            .map_err(|err| {
                warn!("{}: {}", constants::ERR_NOT_FOUND, err);
                constants::ERR_NOT_FOUND
            })?;

        self.signup(&claims.sub, &claims.pwd)
    }

    pub fn signup(&self, email: &str, pwd: &str) -> Result<i32, Box<dyn Error>> {
        info!("got a \"signup\" request for email {} ", email);

        let mut user = User::new(email, &pwd)?;
        self.user_repo.create(&mut user)?;
        
        Ok(user.id)
    }

    pub fn secure_delete(&self, pwd: &str, totp: &str, token: &str, jwt_public: &[u8]) -> Result<(), Box<dyn Error>> {
        let claims = self.get_valid_session_token(token, jwt_public)?;
        self.delete(claims.sub, pwd, totp)
    }

    pub fn delete(&self, user_id: i32, pwd: &str, totp: &str) -> Result<(), Box<dyn Error>> {
        info!("got a \"delete\" request for user id {} ", user_id);
        
        let user = self.user_repo.find(user_id)?;
        if !user.match_password(pwd) {
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

    pub fn secure_enable_totp(&self, pwd: &str, totp: &str, token: &str, jwt_public: &[u8]) -> Result<String, Box<dyn Error>> {
        let claims = self.get_valid_session_token(token, jwt_public)?;
        self.enable_totp(claims.sub, pwd, totp)
    }

    pub fn enable_totp(&self, user_id: i32, pwd: &str, totp: &str) -> Result<String, Box<dyn Error>> {
        info!("got an \"enable totp\" request for user id {} ", user_id);

        let user = self.user_repo.find(user_id)?;
        if !user.match_password(pwd) {
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

    pub fn secure_disable_totp(&self, pwd: &str, totp: &str, token: &str, jwt_public: &[u8]) -> Result<(), Box<dyn Error>> {
        let claims = self.get_valid_session_token(token, jwt_public)?;
        self.disable_totp(claims.sub, pwd, totp)
    }

    pub fn disable_totp(&self, user_id: i32, pwd: &str, totp: &str) -> Result<(), Box<dyn Error>> {
        info!("got an \"disable totp\" request for user id {} ", user_id);
        
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
    use super::Mailer;
    use super::super::domain::{
        tests::new_user_custom,
        User,
    };
    use super::{UserRepository, UserApplication};
    use super::super::domain::tests::{TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD};
    use crate::secret::application::tests::SecretRepositoryMock;
    use crate::session::application::tests::SessionRepositoryMock;

    pub const TEST_CREATE_ID: i32 = 999;
    pub const TEST_FIND_BY_EMAIL_ID: i32 = 888;
    pub const TEST_FIND_BY_NAME_ID: i32 = 777;

    const JWT_SECRET: &[u8] = b"LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JR0hBZ0VBTUJNR0J5cUdTTTQ5QWdFR0NDcUdTTTQ5QXdFSEJHMHdhd0lCQVFRZy9JMGJTbVZxL1BBN2FhRHgKN1FFSGdoTGxCVS9NcWFWMUJab3ZhM2Y5aHJxaFJBTkNBQVJXZVcwd3MydmlnWi96SzRXcGk3Rm1mK0VPb3FybQpmUlIrZjF2azZ5dnBGd0gzZllkMlllNXl4b3ZsaTROK1ZNNlRXVFErTmVFc2ZmTWY2TkFBMloxbQotLS0tLUVORCBQUklWQVRFIEtFWS0tLS0tCg==";
    //const JWT_PUBLIC: &[u8] = b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFVm5sdE1MTnI0b0dmOHl1RnFZdXhabi9oRHFLcQo1bjBVZm45YjVPc3I2UmNCOTMySGRtSHVjc2FMNVl1RGZsVE9rMWswUGpYaExIM3pIK2pRQU5tZFpnPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg==";

    pub struct MailerMock;

    impl Mailer for MailerMock {
        fn send_verification_email(&self, _: &str, _: &str) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
    }

    pub struct UserRepositoryMock {
        pub fn_find: Option<fn (this: &UserRepositoryMock, id: i32) -> Result<User, Box<dyn Error>>>,
        pub fn_find_by_email: Option<fn (this: &UserRepositoryMock, email: &str) -> Result<User, Box<dyn Error>>>,
        pub fn_find_by_name: Option<fn (this: &UserRepositoryMock, name: &str) -> Result<User, Box<dyn Error>>>,
        pub fn_create: Option<fn (this: &UserRepositoryMock, user: &mut User) -> Result<(), Box<dyn Error>>>,
        pub fn_save: Option<fn (this: &UserRepositoryMock, user: &User) -> Result<(), Box<dyn Error>>>,
        pub fn_delete: Option<fn (this: &UserRepositoryMock, user: &User) -> Result<(), Box<dyn Error>>>,
    }

    impl UserRepositoryMock {
        pub fn new() -> Self {
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

            Ok(new_user_custom(id, ""))
        }

        fn find_by_email(&self, email: &str) -> Result<User, Box<dyn Error>> {
            if let Some(f) = self.fn_find_by_email {
                return f(self, email);
            }

            Ok(new_user_custom(TEST_FIND_BY_EMAIL_ID, email))
        }

        fn find_by_name(&self, name: &str) -> Result<User, Box<dyn Error>> {
            if let Some(f) = self.fn_find_by_name {
                return f(self, name);
            }

            Ok(new_user_custom(TEST_FIND_BY_NAME_ID, name))
        }

        fn create(&self, user: &mut User) -> Result<(), Box<dyn Error>> {
            if let Some(f) = self.fn_create {
                return f(self, user);
            }

            user.id = TEST_CREATE_ID;
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

    pub fn new_user_application() -> UserApplication<
            UserRepositoryMock,
            SecretRepositoryMock,
            SessionRepositoryMock,
            MailerMock> {
        let user_repo = UserRepositoryMock::new();
        let secret_repo = SecretRepositoryMock::new();
        let mailer_mock = MailerMock{};
        let session_repo = SessionRepositoryMock{
            force_fail: false,
        };
        
        UserApplication {
            user_repo: Arc::new(user_repo),
            secret_repo: Arc::new(secret_repo),
            session_repo: Arc::new(session_repo),
            mailer: Arc::new(mailer_mock),
            timeout: 60,
        }
    }

    #[test]
    fn user_verify_should_not_fail() {
        let mut user_repo = UserRepositoryMock::new();
        user_repo.fn_find_by_email = Some(|_: &UserRepositoryMock, _: &str| -> Result<User, Box<dyn Error>> {
            Err("forced failure".into())
        });

        let secret_repo = SecretRepositoryMock::new();
        let mailer_mock = MailerMock{};
        let session_repo = SessionRepositoryMock{
            force_fail: false,
        };

        let app = UserApplication {
            user_repo: Arc::new(user_repo),
            secret_repo: Arc::new(secret_repo),
            session_repo: Arc::new(session_repo),
            mailer: Arc::new(mailer_mock),
            timeout: 60,
        };

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        app.verify_user(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD, &jwt_secret).unwrap();
    }

    #[test]
    fn user_verify_already_exists_should_not_fail() {

    }

    #[test]
    fn user_verify_wrong_email_should_fail() {

    }

    #[test]
    fn user_verify_wrong_password_should_fail() {

    }

    #[test]
    fn user_verify_cannot_send_email_should_fail() {

    }

    #[test]
    fn user_secure_signup_should_not_fail() {

    }

    #[test]
    fn user_secure_signup_expired_token_should_fail() {

    }

    #[test]
    fn user_secure_signup_wrong_token_should_fail() {

    }

    #[test]
    fn user_signup_should_not_fail() {
        let user_repo = UserRepositoryMock::new();
        let secret_repo = SecretRepositoryMock::new();
        let mailer_mock = MailerMock{};
        let session_repo = SessionRepositoryMock{
            force_fail: false,
        };
        
        let app = UserApplication{
            user_repo: Arc::new(user_repo),
            secret_repo: Arc::new(secret_repo),
            session_repo: Arc::new(session_repo),
            mailer: Arc::new(mailer_mock),
            timeout: 0,
        };

        let user_id = app.signup(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD).unwrap();
        assert_eq!(user_id, TEST_CREATE_ID);
    }

    #[test]
    fn user_signup_wrong_email_should_fail() {

    }

    #[test]
    fn user_signup_wrong_password_should_fail() {

    }

    #[test]
    fn user_signup_already_exists_should_fail() {

    }

    #[test]
    fn user_secure_delete_should_not_fail() {

    }

    #[test]
    fn user_secure_delete_expired_token_should_fail() {

    }

    #[test]
    fn user_secure_delete_wrong_token_should_fail() {

    }

    #[test]
    fn user_delete_should_not_fail() {

    }

    #[test]
    fn user_delete_totp_should_not_fail() {

    }

    #[test]
    fn user_delete_wrong_password_should_fail() {

    }

    #[test]
    fn user_delete_wrong_totp_should_fail() {

    }

    #[test]
    fn user_delete_not_found_should_fail() {

    }

    #[test]
    fn user_secure_enable_totp_should_not_fail() {

    }

    #[test]
    fn user_secure_enable_totp_expired_token_should_fail() {

    }

    #[test]
    fn user_secure_enable_totp_wrong_token_should_fail() {

    }

    #[test]
    fn user_enable_totp_should_not_fail() {

    }

    #[test]
    fn user_enable_totp_verify_should_not_fail() {

    }

    #[test]
    fn user_enable_totp_wrong_password_should_fail() {

    }

    #[test]
    fn user_enable_totp_already_enabled_should_fail() {

    }

    #[test]
    fn user_secure_disable_totp_should_not_fail() {

    }

    #[test]
    fn user_secure_disable_totp_expired_token_should_fail() {

    }

    #[test]
    fn user_secure_disable_totp_wrong_token_should_fail() {

    }

    #[test]
    fn user_disable_totp_should_not_fail() {

    }

    #[test]
    fn user_disable_totp_wrong_password_should_fail() {

    }

    #[test]
    fn user_disable_totp_wrong_totp_should_fail() {

    }

    #[test]
    fn user_disable_totp_not_enabled_should_fail() {

    }

    #[test]
    fn user_disable_totp_not_verified_should_fail() {

    }
}