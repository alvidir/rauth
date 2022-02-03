use std::time::Duration;
use std::error::Error;
use std::sync::Arc;
use serde::{
    Serialize, 
    de::DeserializeOwned
};
use super::domain::SessionToken;
use crate::user::application::UserRepository;
use crate::secret::application::SecretRepository;
use crate::regex;
use crate::constants;
use crate::security;

pub trait SessionRepository {
    fn exists<T: Serialize + DeserializeOwned>(&self, key: u64) -> Result<(), Box<dyn Error>>;
    fn save<T: Serialize + DeserializeOwned>(&self, key: u64, token: &T) -> Result<(), Box<dyn Error>>;
    fn delete(&self, key: u64) -> Result<(), Box<dyn Error>>;
}

pub struct SessionApplication<S: SessionRepository, U: UserRepository, E: SecretRepository> {
    pub session_repo: Arc<S>,
    pub user_repo: Arc<U>,
    pub secret_repo: Arc<E>,
    pub timeout: u64,
}

impl<S: SessionRepository, U: UserRepository, E: SecretRepository> SessionApplication<S, U, E> {
    pub fn login(&self, ident: &str, pwd: &str, totp: &str, jwt_secret: &[u8]) -> Result<String, Box<dyn Error>> {
        info!("got a \"login\" request from email {} ", ident);        
        
        let user = if regex::match_regex(regex::EMAIL, ident).is_ok() {
            self.user_repo.find_by_email(ident)?
        } else {
            self.user_repo.find_by_name(ident)?
        };

        if !user.match_password(pwd) {
            return Err(constants::ERR_NOT_FOUND.into());
        }

        // if, and only if, the user has activated the totp
        if let Ok(secret) = self.secret_repo.find_by_user_and_name(user.get_id(), constants::TOTP_SECRET_NAME) {
            if !secret.is_deleted() {                
                if totp.len() == 0 {
                    return Err(constants::ERR_UNAUTHORIZED.into());
                }
    
                let data = secret.get_data();
                if !security::verify_totp(data, totp)? {
                    return Err(constants::ERR_UNAUTHORIZED.into());
                }
            }
        }

        let sess = SessionToken::new(constants::TOKEN_ISSUER, user.get_id(), Duration::from_secs(self.timeout));
        self.session_repo.save(sess.sid, &sess)?;

        let token = security::sign_jwt(jwt_secret, sess)?;
        Ok(token)
    }

    pub fn logout(&self, token: &str, jwt_public: &[u8]) -> Result<(), Box<dyn Error>> {
        info!("got a \"logout\" request for token {} ", token);  

        let claims: SessionToken = security::verify_jwt(jwt_public, &token)
            .map_err(|err| {
                warn!("{}: {}", constants::ERR_VERIFY_TOKEN, err);
                constants::ERR_VERIFY_TOKEN
            })?;

        self.session_repo.delete(claims.sid)
    }
}

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use std::time::{Duration, SystemTime};
    use std::sync::Arc;
    use serde::{
        Serialize, 
        de::DeserializeOwned
    };

    use crate::security;
    use crate::user::{
        application::tests::{UserRepositoryMock, TEST_FIND_BY_EMAIL_ID, TEST_FIND_BY_NAME_ID},
        domain::tests::{TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD, TEST_DEFAULT_USER_NAME}
    };

    use crate::time;
    use crate::secret::domain::Secret;
    use crate::secret::domain::tests::TEST_DEFAULT_SECRET_DATA;
    use crate::secret::application::tests::SecretRepositoryMock;
    use super::{SessionRepository, SessionApplication};
    use super::super::domain::{
        tests::new_session_token,
        SessionToken,
    };

    const JWT_SECRET: &[u8] = b"LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JR0hBZ0VBTUJNR0J5cUdTTTQ5QWdFR0NDcUdTTTQ5QXdFSEJHMHdhd0lCQVFRZy9JMGJTbVZxL1BBN2FhRHgKN1FFSGdoTGxCVS9NcWFWMUJab3ZhM2Y5aHJxaFJBTkNBQVJXZVcwd3MydmlnWi96SzRXcGk3Rm1mK0VPb3FybQpmUlIrZjF2azZ5dnBGd0gzZllkMlllNXl4b3ZsaTROK1ZNNlRXVFErTmVFc2ZmTWY2TkFBMloxbQotLS0tLUVORCBQUklWQVRFIEtFWS0tLS0tCg==";
    const JWT_PUBLIC: &[u8] = b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFVm5sdE1MTnI0b0dmOHl1RnFZdXhabi9oRHFLcQo1bjBVZm45YjVPc3I2UmNCOTMySGRtSHVjc2FMNVl1RGZsVE9rMWswUGpYaExIM3pIK2pRQU5tZFpnPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg==";
  
    pub struct SessionRepositoryMock {
        pub force_fail: bool,
    }

    impl SessionRepository for SessionRepositoryMock{
        fn exists<T: Serialize + DeserializeOwned>(&self, _: u64) -> Result<(), Box<dyn Error>> {
            if self.force_fail {
                return Err("forced failure".into());
            }

            Ok(())
        }

        fn save<T: Serialize + DeserializeOwned>(&self, _: u64, _: &T) -> Result<(), Box<dyn Error>> {
            if self.force_fail {
                return Err("forced failure".into());
            }

            Ok(())
        }

        fn delete(&self, _: u64) -> Result<(), Box<dyn Error>> {
            if self.force_fail {
                return Err("forced failure".into());
            }

            Ok(())
        }
    }

    pub fn new_session_application() -> SessionApplication<
            SessionRepositoryMock,
            UserRepositoryMock,
            SecretRepositoryMock> {
        let user_repo = UserRepositoryMock::new();
        let secret_repo = SecretRepositoryMock::new();
        let session_repo = SessionRepositoryMock{
            force_fail: false,
        };

        SessionApplication {
            user_repo: Arc::new(user_repo),
            secret_repo: Arc::new(secret_repo),
            session_repo: Arc::new(session_repo),
            timeout: 999,
        }
    }

    #[test]
    fn login_by_email_should_not_fail() {
        let user_repo = UserRepositoryMock::new();
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err("overrided".into())
        });

        let session_repo = SessionRepositoryMock{
            force_fail: false,
        };

        let app = SessionApplication {
            user_repo: Arc::new(user_repo),
            secret_repo: Arc::new(secret_repo),
            session_repo: Arc::new(session_repo),
            timeout: 999,
        };

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let token = app.login(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD, "", &jwt_secret).unwrap();
        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        let session: SessionToken = security::verify_jwt(&jwt_public, &token).unwrap();

        assert_eq!(session.sub, TEST_FIND_BY_EMAIL_ID);
    }

    #[test]
    fn login_by_username_should_not_fail() {
        let user_repo = UserRepositoryMock::new();
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err("overrided".into())
        });

        let session_repo = SessionRepositoryMock{
            force_fail: false,
        };

        let app = SessionApplication {
            user_repo: Arc::new(user_repo),
            secret_repo: Arc::new(secret_repo),
            session_repo: Arc::new(session_repo),
            timeout: 999,
        };
        
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let token = app.login(TEST_DEFAULT_USER_NAME, TEST_DEFAULT_USER_PASSWORD, "", &jwt_secret).unwrap();
        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        let session: SessionToken = security::verify_jwt(&jwt_public, &token).unwrap();
        
        assert_eq!(session.sub, TEST_FIND_BY_NAME_ID);
    }

    #[test]
    fn login_with_totp_should_not_fail() {
        let app = new_session_application();
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let code = security::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes()).unwrap().generate();
        let token = app.login(TEST_DEFAULT_USER_NAME, TEST_DEFAULT_USER_PASSWORD, &code, &jwt_secret).unwrap();
        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        let session: SessionToken = security::verify_jwt(&jwt_public, &token).unwrap();
        
        assert_eq!(session.sub, TEST_FIND_BY_NAME_ID);
    }

    #[test]
    fn login_wrong_password_should_fail() {
        let app = new_session_application();
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let code = security::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes()).unwrap().generate();
        
        assert!(app.login(TEST_DEFAULT_USER_NAME, "fake_password", &code, &jwt_secret).is_err());
    }

    #[test]
    fn login_wrong_totp_should_fail() {
        let app = new_session_application();
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        
        assert!(app.login(TEST_DEFAULT_USER_NAME, TEST_DEFAULT_USER_PASSWORD, "fake_totp", &jwt_secret).is_err());
    }

    #[test]
    fn login_empty_totp_should_fail() {
        let app = new_session_application();
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        
        assert!(app.login(TEST_DEFAULT_USER_NAME, TEST_DEFAULT_USER_PASSWORD, "", &jwt_secret).is_err());
    }

    #[test]
    fn logout_should_not_fail() {
        let app = new_session_application();
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let token = security::sign_jwt(&jwt_secret, new_session_token()).unwrap();
        
        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        assert!(app.logout(&token, &jwt_public).is_ok())
    }

    #[test]
    fn logout_wrong_token_should_fail() {    
        let user_repo = UserRepositoryMock::new();
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err("overrided".into())
        });

        let session_repo = SessionRepositoryMock{
            force_fail: true,
        };

        let app = SessionApplication {
            user_repo: Arc::new(user_repo),
            secret_repo: Arc::new(secret_repo),
            session_repo: Arc::new(session_repo),
            timeout: 999,
        };

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let token = security::sign_jwt(&jwt_secret, new_session_token()).unwrap();
        
        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        assert!(app.logout(&token, &jwt_public).is_err())
    }

    #[test]
    fn logout_expired_token_should_fail() {
        let app = new_session_application();
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let mut session_token = new_session_token();
        session_token.exp = time::unix_timestamp(SystemTime::now() - Duration::from_secs(1));
        
        let token = security::sign_jwt(&jwt_secret, session_token).unwrap();
        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        assert!(app.logout(&token, &jwt_public).is_err())
    }
}