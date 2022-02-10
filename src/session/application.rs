use std::time::Duration;
use std::error::Error;
use std::sync::Arc;
use crate::security::WithOwnedId;
use super::domain::SessionToken;
use crate::user::application::UserRepository;
use crate::secret::application::SecretRepository;
use crate::regex;
use crate::constants;
use crate::security;

pub trait TokenRepository {
    fn find(&self, key: &str) -> Result<String, Box<dyn Error>>;
    fn save(&self, key: &str, token: &str, expire: Option<u64>) -> Result<(), Box<dyn Error>>;
    fn delete(&self, key: &str) -> Result<(), Box<dyn Error>>;
}

pub struct SessionApplication<T: TokenRepository, U: UserRepository, E: SecretRepository> {
    pub token_repo: Arc<T>,
    pub user_repo: Arc<U>,
    pub secret_repo: Arc<E>,
    pub timeout: u64,
}

impl<T: TokenRepository, U: UserRepository, E: SecretRepository> SessionApplication<T, U, E> {
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

        let sess = SessionToken::new(
            constants::TOKEN_ISSUER,
            &user.get_id().to_string(),
            Duration::from_secs(self.timeout)
        );

        let key = sess.get_id();
        let token = security::sign_jwt(jwt_secret, sess)
            .map_err(|err| {
                error!("{}: {}", constants::ERR_SIGN_TOKEN, err);
                constants::ERR_SIGN_TOKEN
            })?;

        self.token_repo.save(&key, &token, Some(self.timeout))?;
        Ok(token)
    }

    pub fn logout(&self, token: &str, jwt_public: &[u8]) -> Result<(), Box<dyn Error>> {
        info!("got a \"logout\" request for token {} ", token);  

        let claims: SessionToken = util::verify_token(self.token_repo.clone(), token, jwt_public)?;

        self.token_repo.delete(&claims.get_id())
            .map_err(|err| {
                error!("{} failed to remove token with id {}: {}", constants::ERR_UNKNOWN, claims.get_id(), err);
                constants::ERR_UNKNOWN
            })?;
        
        Ok(())
    }
}

pub mod util {
    use std::error::Error;
    use std::sync::Arc;
    use serde::{
        Serialize,
        de::DeserializeOwned
    };
    use super::TokenRepository;
    use crate::security::WithOwnedId;
    use crate::constants;
    use crate::security;

    pub fn verify_token<T: TokenRepository, S: Serialize + DeserializeOwned + PartialEq + Eq + WithOwnedId>(repo: Arc<T>, token: &str, jwt_public: &[u8]) -> Result<S, Box<dyn Error>> {
        let claims: S = security::verify_jwt(jwt_public, token)
            .map_err(|err| {
                warn!("{} verifying session token: {}", constants::ERR_VERIFY_TOKEN, err);
                constants::ERR_VERIFY_TOKEN
            })?;
    
        let key = claims.get_id(); 
        let present_data = repo.find(&key)
            .map_err(|err| {
                warn!("{} finding token with id {}: {}", constants::ERR_NOT_FOUND, &key, err);
                constants::ERR_NOT_FOUND
            })?;

        let present_token: S = security::verify_jwt(jwt_public, &present_data)
            .map_err(|err| {
                warn!("{} verifying found token with id {}: {}", constants::ERR_VERIFY_TOKEN, &key, err);
                constants::ERR_VERIFY_TOKEN
            })?;
    
        if present_token != claims {
            error!("{}: got and want token for id {} do not match", constants::ERR_NOT_FOUND, &key);
            return Err(constants::ERR_NOT_FOUND.into());
        }
    
        Ok(claims)
    }
}

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use crate::{security, time};
    use crate::user::{
        application::tests::{UserRepositoryMock, TEST_FIND_BY_EMAIL_ID, TEST_FIND_BY_NAME_ID},
        domain::tests::{TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD, TEST_DEFAULT_USER_NAME}
    };

    use crate::secret::domain::Secret;
    use crate::secret::domain::tests::TEST_DEFAULT_SECRET_DATA;
    use crate::secret::application::tests::SecretRepositoryMock;
    use super::{TokenRepository, SessionApplication};
    use super::super::domain::{
        tests::new_session_token,
        SessionToken,
    };

    const JWT_SECRET: &[u8] = b"LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JR0hBZ0VBTUJNR0J5cUdTTTQ5QWdFR0NDcUdTTTQ5QXdFSEJHMHdhd0lCQVFRZy9JMGJTbVZxL1BBN2FhRHgKN1FFSGdoTGxCVS9NcWFWMUJab3ZhM2Y5aHJxaFJBTkNBQVJXZVcwd3MydmlnWi96SzRXcGk3Rm1mK0VPb3FybQpmUlIrZjF2azZ5dnBGd0gzZllkMlllNXl4b3ZsaTROK1ZNNlRXVFErTmVFc2ZmTWY2TkFBMloxbQotLS0tLUVORCBQUklWQVRFIEtFWS0tLS0tCg==";
    const JWT_PUBLIC: &[u8] = b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFVm5sdE1MTnI0b0dmOHl1RnFZdXhabi9oRHFLcQo1bjBVZm45YjVPc3I2UmNCOTMySGRtSHVjc2FMNVl1RGZsVE9rMWswUGpYaExIM3pIK2pRQU5tZFpnPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg==";
  
    pub struct TokenRepositoryMock {
        pub fn_find: Option<fn (this: &TokenRepositoryMock, key: &str) -> Result<String, Box<dyn Error>>>,
        pub fn_save: Option<fn (this: &TokenRepositoryMock, key: &str, token: &str, expire: Option<u64>) -> Result<(), Box<dyn Error>>>,
        pub fn_delete: Option<fn (this: &TokenRepositoryMock, key: &str) -> Result<(), Box<dyn Error>>>,
        pub token: String,
    }

    impl TokenRepositoryMock {
        pub fn new() -> Self {
            TokenRepositoryMock{
                fn_find: None,
                fn_save: None,
                fn_delete: None,
                token: "".into(),
            }
        }
    }

    impl TokenRepository for TokenRepositoryMock{
        fn find(&self, key: &str) -> Result<String, Box<dyn Error>> {
            if let Some(fn_find) = self.fn_find {
                return fn_find(self, key);
            }

            Ok(self.token.clone())
        }

        fn save(&self, key: &str, token: &str, expire: Option<u64>) -> Result<(), Box<dyn Error>> {
            if let Some(fn_save) = self.fn_save {
                return fn_save(self, key, token, expire);
            }

            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), Box<dyn Error>> {
            if let Some(fn_delete) = self.fn_delete {
                return fn_delete(self, key);
            }

            Ok(())
        }
    }

    pub fn new_session_application() -> SessionApplication<
            TokenRepositoryMock,
            UserRepositoryMock,
            SecretRepositoryMock> {
        let user_repo = UserRepositoryMock::new();
        let secret_repo = SecretRepositoryMock::new();
        let token_repo = TokenRepositoryMock::new();

        SessionApplication {
            user_repo: Arc::new(user_repo),
            secret_repo: Arc::new(secret_repo),
            token_repo: Arc::new(token_repo),
            timeout: 999,
        }
    }

    #[test]
    fn login_by_email_should_not_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err("overrided".into())
        });

        let mut app = new_session_application();
        app.secret_repo = Arc::new(secret_repo);

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let token = app.login(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD, "", &jwt_secret).unwrap();
        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        let session: SessionToken = security::verify_jwt(&jwt_public, &token).unwrap();

        assert_eq!(session.sub, TEST_FIND_BY_EMAIL_ID.to_string());
    }

    #[test]
    fn login_by_username_should_not_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err("overrided".into())
        });


        let mut app = new_session_application();
        app.secret_repo = Arc::new(secret_repo);
        
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let token = app.login(TEST_DEFAULT_USER_NAME, TEST_DEFAULT_USER_PASSWORD, "", &jwt_secret).unwrap();
        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        let session: SessionToken = security::verify_jwt(&jwt_public, &token).unwrap();
        
        assert_eq!(session.sub, TEST_FIND_BY_NAME_ID.to_string());
    }

    #[test]
    fn login_with_totp_should_not_fail() {
        let app = new_session_application();
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let code = security::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes()).unwrap().generate();
        let token = app.login(TEST_DEFAULT_USER_NAME, TEST_DEFAULT_USER_PASSWORD, &code, &jwt_secret).unwrap();
        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        let session: SessionToken = security::verify_jwt(&jwt_public, &token).unwrap();
        
        assert_eq!(session.sub, TEST_FIND_BY_NAME_ID.to_string());
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
    fn logout_should_not_fail() {        
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let token = security::sign_jwt(&jwt_secret, new_session_token()).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = token.clone();

        let mut app = new_session_application();
        app.token_repo = Arc::new(token_repo);
        
        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        assert!(app.logout(&token, &jwt_public).is_ok())
    }

    #[test]
    fn logout_invalid_token_should_fail() {    
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err("overrided".into())
        });

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.fn_find = Some(|_: &TokenRepositoryMock, _: &str| -> Result<String, Box<dyn Error>> {
            Err("overrided".into())
        });

        let mut app = new_session_application();
        app.secret_repo = Arc::new(secret_repo);
        app.token_repo = Arc::new(token_repo);

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let token = security::sign_jwt(&jwt_secret, new_session_token()).unwrap();
        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        assert!(app.logout(&token, &jwt_public).is_err())
    }

    #[test]
    fn logout_wrong_token_should_fail() {    
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err("overrided".into())
        });

        let mut app = new_session_application();
        app.secret_repo = Arc::new(secret_repo);

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let token = security::sign_jwt(&jwt_secret, new_session_token()).unwrap()
            .replace('A', "a");
        
        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        assert!(app.logout(&token, &jwt_public).is_err())
    }

    #[test]
    fn logout_expired_token_should_fail() {
        let app = new_session_application();
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let mut session_token = new_session_token();
        session_token.exp = time::unix_timestamp(SystemTime::now() - Duration::from_secs(61));
        
        let token = security::sign_jwt(&jwt_secret, session_token).unwrap();
        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        assert!(app.logout(&token, &jwt_public).is_err())
    }
}