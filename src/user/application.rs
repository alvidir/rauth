use std::error::Error;
use std::time::{SystemTime, Duration};
use std::sync::Arc;
use crate::secret::{
    application::SecretRepository,
    domain::Secret,
};
use crate::session::{
    application::{
        TokenRepository,
        util::{
            verify_token,
            generate_token,
            TokenDefinition
        }
    },
    domain::{Token, TokenKind}
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

pub struct UserApplication<U: UserRepository, E: SecretRepository, T: TokenRepository, M: Mailer> {
    pub user_repo: Arc<U>,
    pub secret_repo: Arc<E>,
    pub token_repo: Arc<T>,
    pub mailer: Arc<M>,
    pub timeout: u64,
}

impl<U: UserRepository, E: SecretRepository, T: TokenRepository, M: Mailer> UserApplication<U, E, T, M> {
    pub fn verify_signup_email(&self, email: &str, pwd: &str, jwt_secret: &[u8]) -> Result<(), Box<dyn Error>> {
        if self.user_repo.find_by_email(email).is_ok() {
            // returns Ok to not provide information about users
            return Ok(());
        }

        User::new(email, pwd)?;
        let token_to_store = Token::new_verification(
            constants::TOKEN_ISSUER,
            email,
            pwd,
            Duration::from_secs(self.timeout)
        );

        let mut token_to_send = Token::new_session(
            constants::TOKEN_ISSUER,
            &token_to_store.get_id(),
            Duration::from_secs(self.timeout)
        );
        
        let token_to_store = security::sign_jwt(jwt_secret, token_to_store)?;
        self.token_repo.save(&token_to_send.sub, &token_to_store, Some(self.timeout))?;
        
        token_to_send.knd = TokenKind::Verification;
        let token_to_send = security::sign_jwt(jwt_secret, token_to_send)?;

        self.mailer.send_verification_signup_email(email, &token_to_send)?;
        Ok(())
    }

    pub fn secure_signup(&self, token: &str, jwt_public: &[u8], jwt_secret: &[u8])  -> Result<String, Box<dyn Error>> {
        let claims: Token = security::verify_jwt(jwt_public, token)?;

        if claims.knd != TokenKind::Verification {
            warn!("{} checking token's kind with id {}: got {:?} want {:?}", constants::ERR_INVALID_TOKEN, claims.jti, claims.knd, TokenKind::Verification);
            return Err(constants::ERR_INVALID_TOKEN.into())
        }

        let token = self.token_repo.find(&claims.sub)
            .map_err(|_| constants::ERR_INVALID_TOKEN)?;

        let claims: Token = verify_token(self.token_repo.clone(), TokenKind::Verification, &token, jwt_public)?;
        let token_id = claims.get_id();
        let password = &claims.pwd.ok_or(constants::ERR_INVALID_TOKEN)?;
        let token = self.signup(&claims.sub, password, jwt_secret)?;
        
        self.token_repo.delete(&token_id)?;
        Ok(token)
    }

    pub fn signup(&self, email: &str, pwd: &str, jwt_secret: &[u8]) -> Result<String, Box<dyn Error>> {
        info!("got a \"signup\" request for email {} ", email);

        let mut user = User::new(email, &pwd)?;
        self.user_repo.create(&mut user)?;
        generate_token(
            self.token_repo.clone(),
            self.timeout,
            &user,
            jwt_secret
        )
    }

    pub fn secure_delete(&self, pwd: &str, totp: &str, token: &str, jwt_public: &[u8]) -> Result<(), Box<dyn Error>> {
        let claims: Token = verify_token(self.token_repo.clone(), TokenKind::Session, token, jwt_public)?;
        let user_id = claims.sub.parse()
            .map_err(|err| {
                warn!("{} parsing str to i32: {}", constants::ERR_INVALID_TOKEN, err);
                constants::ERR_INVALID_TOKEN
            })?;

        self.delete(user_id, pwd, totp)
    }

    pub fn delete(&self, user_id: i32, pwd: &str, totp: &str) -> Result<(), Box<dyn Error>> {
        info!("got a \"delete\" request for user id {} ", user_id);

        let user = self.user_repo.find(user_id)
            .map_err(|_| constants::ERR_WRONG_CREDENTIALS)?;

        if !user.match_password(pwd) {
            return Err(constants::ERR_WRONG_CREDENTIALS.into());
        }

        // if, and only if, the user has activated the totp
        if let Ok(secret) = self.secret_repo.find_by_user_and_name(user.id, constants::TOTP_SECRET_NAME) {
            if !secret.is_deleted() {    
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

    pub fn secure_enable_totp(&self, pwd: &str, totp: &str, token: &str, jwt_public: &[u8]) -> Result<Option<String>, Box<dyn Error>> {
        let claims: Token = verify_token(self.token_repo.clone(), TokenKind::Session, token, jwt_public)?;
        let user_id = claims.sub.parse()
            .map_err(|err| {
                warn!("{} parsing str to i32: {}", constants::ERR_INVALID_TOKEN, err);
                constants::ERR_INVALID_TOKEN
            })?;

        self.enable_totp(user_id, pwd, totp)
    }

    pub fn enable_totp(&self, user_id: i32, pwd: &str, totp: &str) -> Result<Option<String>, Box<dyn Error>> {
        info!("got an \"enable totp\" request for user id {} ", user_id);

        let user = self.user_repo.find(user_id)
            .map_err(|_| constants::ERR_WRONG_CREDENTIALS)?;

        if !user.match_password(pwd) {
            return Err(constants::ERR_WRONG_CREDENTIALS.into());
        }

        // if, and only if, the user has activated the totp
        if let Ok(secret) = &mut self.secret_repo.find_by_user_and_name(user.id, constants::TOTP_SECRET_NAME) {
            if !secret.is_deleted() {
                // the totp is already enabled
                return Err(constants::ERR_NOT_AVAILABLE.into())
            }

            let data = secret.get_data();
            if !security::verify_totp(data, totp)? {
                return Err(constants::ERR_UNAUTHORIZED.into());
            }

            secret.set_deleted_at(None);
            self.secret_repo.save(&secret)?;
            return Ok(None);
        }
        
        let token = security::get_random_string(constants::TOTP_SECRET_LEN);
        let mut secret = Secret::new(constants::TOTP_SECRET_NAME, token.as_bytes());
        secret.set_deleted_at(Some(SystemTime::now())); // unavailable till confirmed
        self.secret_repo.create(&mut secret)?;
        Ok(Some(token))
    }

    pub fn secure_disable_totp(&self, pwd: &str, totp: &str, token: &str, jwt_public: &[u8]) -> Result<(), Box<dyn Error>> {
        let claims: Token = verify_token(self.token_repo.clone(), TokenKind::Session, token, jwt_public)?;
        let user_id = claims.sub.parse()
            .map_err(|err| {
                warn!("{} parsing str to i32: {}", constants::ERR_INVALID_TOKEN, err);
                constants::ERR_INVALID_TOKEN
            })?;

        self.disable_totp(user_id, pwd, totp)
    }

    pub fn disable_totp(&self, user_id: i32, pwd: &str, totp: &str) -> Result<(), Box<dyn Error>> {
        info!("got an \"disable totp\" request for user id {} ", user_id);
        
        let user = self.user_repo.find(user_id)
            .map_err(|_| constants::ERR_WRONG_CREDENTIALS)?;

        if !user.match_password(pwd) {
            return Err(constants::ERR_WRONG_CREDENTIALS.into());
        }

        // if, and only if, the user has activated the totp
        if let Ok(secret) = &mut self.secret_repo.find_by_user_and_name(user.id, constants::TOTP_SECRET_NAME) {
            if secret.is_deleted() {
                // the totp is not enabled yet
                return Err(constants::ERR_NOT_AVAILABLE.into())
            }

            let data = secret.get_data();
            if !security::verify_totp(data, totp)? {
                return Err(constants::ERR_UNAUTHORIZED.into());
            }

            self.secret_repo.delete(&secret)?;
            return Ok(());
        }

        Err(constants::ERR_NOT_AVAILABLE.into())
    }

    pub fn verify_reset_email(&self, email: &str, jwt_secret: &[u8]) -> Result<(), Box<dyn Error>> {
        let user = match self.user_repo.find_by_email(email) {
            Err(_) => return Ok(()), // returns Ok to not provide information about users
            Ok(user) => user,
        };

        let token = Token::new_reset(
            constants::TOKEN_ISSUER,
            &user.get_id().to_string(),
            Duration::from_secs(self.timeout)
        );

        let key = token.get_id();
        let secure_token = security::sign_jwt(jwt_secret, token)?;
        self.token_repo.save(&key, &secure_token, Some(self.timeout))?;
        self.mailer.send_verification_reset_email(email, &secure_token)?;
        Ok(())
    }

    pub fn secure_reset(&self, new_pwd: &str, totp: &str, token: &str, jwt_public: &[u8])  -> Result<(), Box<dyn Error>> {
        let claims: Token = verify_token(self.token_repo.clone(), TokenKind::Reset, token, jwt_public)?;
        let user_id = claims.sub.parse()
            .map_err(|err| {
                warn!("{} parsing str to i32: {}", constants::ERR_INVALID_TOKEN, err);
                constants::ERR_INVALID_TOKEN
            })?;

        self.reset(user_id, new_pwd, totp)?;
        self.token_repo.delete(&claims.get_id())?;
        Ok(())
    }

    pub fn reset(&self, user_id: i32, new_pwd: &str, totp: &str) -> Result<(), Box<dyn Error>> {
        info!("got a \"reset password\" request for user_id {} ", user_id);        
        
        let mut user = self.user_repo.find(user_id)
            .map_err(|_| constants::ERR_WRONG_CREDENTIALS)?;

        if user.match_password(new_pwd) {
            return Err(constants::ERR_WRONG_CREDENTIALS.into());
        }

        // if, and only if, the user has activated the totp
        if let Ok(secret) = self.secret_repo.find_by_user_and_name(user.get_id(), constants::TOTP_SECRET_NAME) {
            if !secret.is_deleted() {                    
                let data = secret.get_data();
                if !security::verify_totp(data, totp)? {
                    return Err(constants::ERR_UNAUTHORIZED.into());
                }
            }
        }

        user.set_password(new_pwd)?;
        self.user_repo.save(&user)
    }
}

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use std::time::{Duration, SystemTime};
    use std::sync::Arc;
    use super::super::domain::{
        tests::new_user_custom,
        User,
    };
    use super::{UserRepository, UserApplication};
    use super::super::domain::tests::{TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD};
    use crate::secret::{
        application::tests::SecretRepositoryMock,
        domain::{
            Secret,
            tests::{TEST_DEFAULT_SECRET_DATA, new_secret},
        },
    };
    use crate::{security, constants};
    use crate::session::{
        application::{
            tests::TokenRepositoryMock,
            util::TokenDefinition,
        },
        domain::{Token, TokenKind},
    };
    use crate::smtp::{
        tests::MailerMock,
    };

    pub const TEST_CREATE_ID: i32 = 999;
    pub const TEST_FIND_BY_EMAIL_ID: i32 = 888;
    pub const TEST_FIND_BY_NAME_ID: i32 = 777;

    const JWT_SECRET: &[u8] = b"LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JR0hBZ0VBTUJNR0J5cUdTTTQ5QWdFR0NDcUdTTTQ5QXdFSEJHMHdhd0lCQVFRZy9JMGJTbVZxL1BBN2FhRHgKN1FFSGdoTGxCVS9NcWFWMUJab3ZhM2Y5aHJxaFJBTkNBQVJXZVcwd3MydmlnWi96SzRXcGk3Rm1mK0VPb3FybQpmUlIrZjF2azZ5dnBGd0gzZllkMlllNXl4b3ZsaTROK1ZNNlRXVFErTmVFc2ZmTWY2TkFBMloxbQotLS0tLUVORCBQUklWQVRFIEtFWS0tLS0tCg==";
    const JWT_PUBLIC: &[u8] = b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFVm5sdE1MTnI0b0dmOHl1RnFZdXhabi9oRHFLcQo1bjBVZm45YjVPc3I2UmNCOTMySGRtSHVjc2FMNVl1RGZsVE9rMWswUGpYaExIM3pIK2pRQU5tZFpnPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg==";

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
            TokenRepositoryMock,
            MailerMock> {
        let user_repo = UserRepositoryMock::new();
        let secret_repo = SecretRepositoryMock::new();
        let mailer_mock = MailerMock::new();
        let token_repo = TokenRepositoryMock::new();
        
        UserApplication {
            user_repo: Arc::new(user_repo),
            secret_repo: Arc::new(secret_repo),
            token_repo: Arc::new(token_repo),
            mailer: Arc::new(mailer_mock),
            timeout: 60,
        }
    }

    #[test]
    fn user_verify_should_not_fail() {
        let mut user_repo = UserRepositoryMock::new();
        user_repo.fn_find_by_email = Some(|_: &UserRepositoryMock, _: &str| -> Result<User, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut app = new_user_application(); 
        app.user_repo = Arc::new(user_repo);

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        app.verify_signup_email(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD, &jwt_secret).unwrap();
    }

    #[test]
    fn user_verify_already_exists_should_not_fail() {
        let app = new_user_application();
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        app.verify_signup_email(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD, &jwt_secret).unwrap();
    }

    #[test]
    fn user_verify_wrong_email_should_fail() {
        let mut user_repo = UserRepositoryMock::new();
        user_repo.fn_find_by_email = Some(|_: &UserRepositoryMock, _: &str| -> Result<User, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut app = new_user_application();
        app.user_repo = Arc::new(user_repo);

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        app.verify_signup_email("this is not an email", TEST_DEFAULT_USER_PASSWORD, &jwt_secret)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_INVALID_FORMAT))
            .unwrap_err();
    }

    #[test]
    fn user_secure_signup_should_not_fail() {
        let mut user_repo = UserRepositoryMock::new();
        user_repo.fn_find_by_email = Some(|_: &UserRepositoryMock, _: &str| -> Result<User, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let verif_token = Token::new_verification(
            "test",
            TEST_DEFAULT_USER_EMAIL,
            TEST_DEFAULT_USER_PASSWORD,
            Duration::from_secs(60),
        );

        let mut sess_token = Token::new_session(
            "test",
            &verif_token.get_id(),
            Duration::from_secs(60)
        );

        sess_token.knd = TokenKind::Verification;
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let secure_verif_token = security::sign_jwt(&jwt_secret, verif_token).unwrap();
        let secure_sess_token = security::sign_jwt(&jwt_secret, sess_token).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = secure_verif_token.clone();
        token_repo.fn_find = Some(|this: &TokenRepositoryMock, tid: &str| -> Result<String, Box<dyn Error>> {
            let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
            let claims: Token = security::verify_jwt(&jwt_public, &this.token)?;
            assert_eq!(claims.get_id(), tid);
            
            Ok(this.token.clone())
        });

        let mut app = new_user_application();
        app.user_repo = Arc::new(user_repo);
        app.token_repo = Arc::new(token_repo);

        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();        
        let token = app.secure_signup(&secure_sess_token, &jwt_public, &jwt_secret).unwrap();
        let claims: Token = security::verify_jwt(&jwt_public, &token).unwrap();
        
        assert_eq!(claims.sub, TEST_CREATE_ID.to_string());
    }

    #[test]
    fn user_secure_signup_verification_token_kind_should_fail() {
        let mut user_repo = UserRepositoryMock::new();
        user_repo.fn_find_by_email = Some(|_: &UserRepositoryMock, _: &str| -> Result<User, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut token = Token::new_session(
            "test",
            "test",
            Duration::from_secs(60)
        );

        token.knd = TokenKind::Verification;
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let token = security::sign_jwt(&jwt_secret, token).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = token.clone();
        token_repo.fn_find = Some(|this: &TokenRepositoryMock, _: &str| -> Result<String, Box<dyn Error>> {        
            Ok(this.token.clone())
        });

        let mut app = new_user_application();
        app.user_repo = Arc::new(user_repo);
        app.token_repo = Arc::new(token_repo);

        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();        
        app.secure_signup(&token, &jwt_public, &jwt_secret)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_INVALID_TOKEN))
            .unwrap_err();
    }

    #[test]
    fn user_secure_signup_reset_token_kind_should_fail() {
        let mut user_repo = UserRepositoryMock::new();
        user_repo.fn_find_by_email = Some(|_: &UserRepositoryMock, _: &str| -> Result<User, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut token = Token::new_session(
            "test",
            "test",
            Duration::from_secs(60)
        );

        token.knd = TokenKind::Reset;
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let token = security::sign_jwt(&jwt_secret, token).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = token.clone();
        token_repo.fn_find = Some(|this: &TokenRepositoryMock, _: &str| -> Result<String, Box<dyn Error>> {        
            Ok(this.token.clone())
        });

        let mut app = new_user_application();
        app.user_repo = Arc::new(user_repo);
        app.token_repo = Arc::new(token_repo);

        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();        
        app.secure_signup(&token, &jwt_public, &jwt_secret)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_INVALID_TOKEN))
            .unwrap_err();
    }

    #[test]
    fn user_signup_should_not_fail() {
        let mut user_repo = UserRepositoryMock::new();
        user_repo.fn_find_by_email = Some(|_: &UserRepositoryMock, _: &str| -> Result<User, Box<dyn Error>> {
            Err("overrided".into())
        });

       
        let mut app = new_user_application();
        app.user_repo = Arc::new(user_repo); 

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        let token = app.signup(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD, &jwt_secret).unwrap();
        let claims: Token = security::verify_jwt(&jwt_public, &token).unwrap();
        
        assert_eq!(claims.sub, TEST_CREATE_ID.to_string());
    }

    #[test]
    fn user_signup_wrong_email_should_fail() {
        let app = new_user_application();
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        app.signup("this is not an email", TEST_DEFAULT_USER_PASSWORD, &jwt_secret)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_INVALID_FORMAT))
            .unwrap_err();
    }

    #[test]
    fn user_signup_wrong_password_should_fail() {
        let app = new_user_application();
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        app.signup(TEST_DEFAULT_USER_EMAIL, "bad password", &jwt_secret)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_INVALID_FORMAT))
            .unwrap_err();
    }

    #[test]
    fn user_signup_already_exists_should_not_fail() {        
        let app = new_user_application();
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        app.signup(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD, &jwt_secret).unwrap();
    }

    #[test]
    fn user_secure_delete_should_not_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let token = Token::new_session("test", "0", Duration::from_secs(60));
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let secure_token = security::sign_jwt(&jwt_secret, token).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = secure_token.clone();
        token_repo.fn_find = Some(|this: &TokenRepositoryMock, _: &str| -> Result<String, Box<dyn Error>> {
            Ok(this.token.clone())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);
        app.token_repo = Arc::new(token_repo);

        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        app.secure_delete(TEST_DEFAULT_USER_PASSWORD, "", &secure_token, &jwt_public).unwrap();
    }

    #[test]
    fn user_secure_delete_verification_token_kind_should_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut token = Token::new_session("test", "0", Duration::from_secs(60));
        token.knd = TokenKind::Verification;
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let secure_token = security::sign_jwt(&jwt_secret, token).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = secure_token.clone();
        token_repo.fn_find = Some(|this: &TokenRepositoryMock, _: &str| -> Result<String, Box<dyn Error>> {
            Ok(this.token.clone())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);
        app.token_repo = Arc::new(token_repo);

        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        app.secure_delete(TEST_DEFAULT_USER_PASSWORD, "", &secure_token, &jwt_public)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_INVALID_TOKEN))
            .unwrap_err();
    }

    #[test]
    fn user_secure_delete_reset_token_kind_should_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut token = Token::new_session("test", "0", Duration::from_secs(60));
        token.knd = TokenKind::Reset;
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let secure_token = security::sign_jwt(&jwt_secret, token).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = secure_token.clone();
        token_repo.fn_find = Some(|this: &TokenRepositoryMock, _: &str| -> Result<String, Box<dyn Error>> {
            Ok(this.token.clone())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);
        app.token_repo = Arc::new(token_repo);

        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        app.secure_delete(TEST_DEFAULT_USER_PASSWORD, "", &secure_token, &jwt_public)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_INVALID_TOKEN))
            .unwrap_err();
    }

    #[test]
    fn user_delete_should_not_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);

        app.delete(0, TEST_DEFAULT_USER_PASSWORD, "").unwrap();
    }

    #[test]
    fn user_delete_totp_should_not_fail() {
        let app = new_user_application();
        let code = security::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes()).unwrap().generate();        
        app.delete(0, TEST_DEFAULT_USER_PASSWORD, &code).unwrap();
    }

    #[test]
    fn user_delete_not_found_should_fail() {
        let mut user_repo = UserRepositoryMock::new();
        user_repo.fn_find = Some(|_: &UserRepositoryMock, _: i32| -> Result<User, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut app = new_user_application();
        app.user_repo = Arc::new(user_repo);
        app.secret_repo = Arc::new(secret_repo);

        app.delete(0, TEST_DEFAULT_USER_PASSWORD, "")
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_WRONG_CREDENTIALS))
            .unwrap_err();
    }

    #[test]
    fn user_delete_wrong_password_should_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);


        app.delete(0, "bad password", "")
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_WRONG_CREDENTIALS))
            .unwrap_err();
    }

    #[test]
    fn user_delete_wrong_totp_should_fail() {
        let app = new_user_application();
        app.delete(0, TEST_DEFAULT_USER_PASSWORD, "bad totp")
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_UNAUTHORIZED))
            .unwrap_err();
    }

    #[test]
    fn user_secure_enable_totp_should_not_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let token = Token::new_session("test", "0", Duration::from_secs(60));
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let secure_token = security::sign_jwt(&jwt_secret, token).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = secure_token.clone();
        token_repo.fn_find = Some(|this: &TokenRepositoryMock, _: &str| -> Result<String, Box<dyn Error>> {
            Ok(this.token.clone())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);
        app.token_repo = Arc::new(token_repo);

        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        let totp = app.secure_enable_totp(TEST_DEFAULT_USER_PASSWORD, "", &secure_token, &jwt_public).unwrap();
        
        assert!(totp.is_some());
        assert_eq!(totp.unwrap().len(), constants::TOTP_SECRET_LEN);
    }

    #[test]
    fn user_secure_enable_totp_verification_token_kind_should_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut token = Token::new_session("test", "0", Duration::from_secs(60));
        token.knd = TokenKind::Verification;

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let secure_token = security::sign_jwt(&jwt_secret, token).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = secure_token.clone();
        token_repo.fn_find = Some(|this: &TokenRepositoryMock, _: &str| -> Result<String, Box<dyn Error>> {
            Ok(this.token.clone())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);
        app.token_repo = Arc::new(token_repo);

        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        app.secure_enable_totp(TEST_DEFAULT_USER_PASSWORD, "", &secure_token, &jwt_public)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_INVALID_TOKEN))
            .unwrap_err();
    }

    #[test]
    fn user_secure_enable_totp_reset_token_kind_should_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut token = Token::new_session("test", "0", Duration::from_secs(60));
        token.knd = TokenKind::Reset;

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let secure_token = security::sign_jwt(&jwt_secret, token).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = secure_token.clone();
        token_repo.fn_find = Some(|this: &TokenRepositoryMock, _: &str| -> Result<String, Box<dyn Error>> {
            Ok(this.token.clone())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);
        app.token_repo = Arc::new(token_repo);

        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        app.secure_enable_totp(TEST_DEFAULT_USER_PASSWORD, "", &secure_token, &jwt_public)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_INVALID_TOKEN))
            .unwrap_err();
    }

    #[test]
    fn user_enable_totp_should_not_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        secret_repo.fn_save = Some(|_: &SecretRepositoryMock, secret: &Secret| -> Result<(), Box<dyn Error>> {
            if !secret.is_deleted() {
                return Err("secret's deleted_at should not be None".into());
            }

            Ok(())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);

        let totp = app.enable_totp(0, TEST_DEFAULT_USER_PASSWORD, "").unwrap();
        assert!(totp.is_some());
        assert_eq!(totp.unwrap().len(), constants::TOTP_SECRET_LEN);
    }

    #[test]
    fn user_enable_totp_verify_should_not_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            let mut secret = new_secret();
            secret.set_deleted_at(Some(SystemTime::now()));
            Ok(secret)
        });

        secret_repo.fn_save = Some(|_: &SecretRepositoryMock, secret: &Secret| -> Result<(), Box<dyn Error>> {
            if secret.is_deleted() {
                return Err("secret's deleted_at should not be Some".into());
            }

            Ok(())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);

        let code = security::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes()).unwrap().generate();
        let totp = app.enable_totp(0, TEST_DEFAULT_USER_PASSWORD, &code).unwrap();
        assert_eq!(totp, None);
    }

    #[test]
    fn user_enable_totp_wrong_password_should_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            let mut secret = new_secret();
            secret.set_deleted_at(Some(SystemTime::now()));
            Ok(secret)
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);

        let code = security::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes()).unwrap().generate();
        app.enable_totp(0, "bad password", &code)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_WRONG_CREDENTIALS))
            .unwrap_err();
    }

    #[test]
    fn user_enable_totp_already_enabled_should_fail() {
        let app = new_user_application();
        let code = security::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes()).unwrap().generate();
        app.enable_totp(0, TEST_DEFAULT_USER_PASSWORD, &code)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_NOT_AVAILABLE))
            .unwrap_err();
    }

    #[test]
    fn user_secure_disable_totp_should_not_fail() {
        let token = Token::new_session("test", "0", Duration::from_secs(60));
        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let secure_token = security::sign_jwt(&jwt_secret, token).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = secure_token.clone();
        token_repo.fn_find = Some(|this: &TokenRepositoryMock, _: &str| -> Result<String, Box<dyn Error>> {
            Ok(this.token.clone())
        });

        let mut app = new_user_application();
        app.token_repo = Arc::new(token_repo);

        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        let code = security::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes()).unwrap().generate();
        app.secure_disable_totp(TEST_DEFAULT_USER_PASSWORD, &code, &secure_token, &jwt_public).unwrap();
    }

    #[test]
    fn user_secure_disable_totp_verification_token_kind_should_fail() {
        let mut token = Token::new_session("test", "0", Duration::from_secs(60));
        token.knd = TokenKind::Verification;

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let secure_token = security::sign_jwt(&jwt_secret, token).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = secure_token.clone();
        token_repo.fn_find = Some(|this: &TokenRepositoryMock, _: &str| -> Result<String, Box<dyn Error>> {
            Ok(this.token.clone())
        });

        let mut app = new_user_application();
        app.token_repo = Arc::new(token_repo);

        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        let code = security::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes()).unwrap().generate();
        app.secure_disable_totp(TEST_DEFAULT_USER_PASSWORD, &code, &secure_token, &jwt_public)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_INVALID_TOKEN))
            .unwrap_err();
    }


    #[test]
    fn user_secure_disable_totp_reset_token_kind_should_fail() {
        let mut token = Token::new_session("test", "0", Duration::from_secs(60));
        token.knd = TokenKind::Reset;

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let secure_token = security::sign_jwt(&jwt_secret, token).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = secure_token.clone();
        token_repo.fn_find = Some(|this: &TokenRepositoryMock, _: &str| -> Result<String, Box<dyn Error>> {
            Ok(this.token.clone())
        });

        let mut app = new_user_application();
        app.token_repo = Arc::new(token_repo);

        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        let code = security::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes()).unwrap().generate();
        app.secure_disable_totp(TEST_DEFAULT_USER_PASSWORD, &code, &secure_token, &jwt_public)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_INVALID_TOKEN))
            .unwrap_err();
    }

    #[test]
    fn user_disable_totp_should_not_fail() {
        let app = new_user_application();
        let code = security::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes()).unwrap().generate();
        app.disable_totp(0, TEST_DEFAULT_USER_PASSWORD, &code).unwrap();
    }

    #[test]
    fn user_disable_totp_wrong_password_should_fail() {
        let app = new_user_application();
        let code = security::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes()).unwrap().generate();
        app.disable_totp(0, "bad password", &code)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_WRONG_CREDENTIALS))
            .unwrap_err();
    }

    #[test]
    fn user_disable_totp_wrong_totp_should_fail() {
        let app = new_user_application();
        app.disable_totp(0, TEST_DEFAULT_USER_PASSWORD, "bad totp")
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_UNAUTHORIZED))
            .unwrap_err();
    }

    #[test]
    fn user_disable_totp_not_enabled_should_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);

        let code = security::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes()).unwrap().generate();
        app.disable_totp(0, TEST_DEFAULT_USER_PASSWORD, &code)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_NOT_AVAILABLE))
            .unwrap_err();
    }

    #[test]
    fn user_disable_totp_not_verified_should_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            let mut secret = new_secret();
            secret.set_deleted_at(Some(SystemTime::now()));
            Ok(secret)
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);

        let code = security::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes()).unwrap().generate();
        app.disable_totp(0, TEST_DEFAULT_USER_PASSWORD, &code)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_NOT_AVAILABLE))
            .unwrap_err();
    }

    #[test]
    fn user_secure_reset_should_not_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut token = Token::new_session("test", "0", Duration::from_secs(60));
        token.knd = TokenKind::Reset;

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let secure_token = security::sign_jwt(&jwt_secret, token).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = secure_token.clone();
        token_repo.fn_find = Some(|this: &TokenRepositoryMock, _: &str| -> Result<String, Box<dyn Error>> {
            Ok(this.token.clone())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);
        app.token_repo = Arc::new(token_repo);

        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        app.secure_reset("ABCDEF1234567891", "", &secure_token, &jwt_public).unwrap();
    }

    #[test]
    fn user_secure_reset_verification_token_kind_should_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut token = Token::new_session("test", "0", Duration::from_secs(60));
        token.knd = TokenKind::Verification;

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let secure_token = security::sign_jwt(&jwt_secret, token).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = secure_token.clone();
        token_repo.fn_find = Some(|this: &TokenRepositoryMock, _: &str| -> Result<String, Box<dyn Error>> {
            Ok(this.token.clone())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);
        app.token_repo = Arc::new(token_repo);

        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        app.secure_reset("another password", "", &secure_token, &jwt_public)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_INVALID_TOKEN))
            .unwrap_err();
    }

    #[test]
    fn user_secure_reset_session_token_kind_should_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut token = Token::new_session("test", "0", Duration::from_secs(60));
        token.knd = TokenKind::Session;

        let jwt_secret = base64::decode(JWT_SECRET).unwrap();
        let secure_token = security::sign_jwt(&jwt_secret, token).unwrap();

        let mut token_repo = TokenRepositoryMock::new();
        token_repo.token = secure_token.clone();
        token_repo.fn_find = Some(|this: &TokenRepositoryMock, _: &str| -> Result<String, Box<dyn Error>> {
            Ok(this.token.clone())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);
        app.token_repo = Arc::new(token_repo);

        let jwt_public = base64::decode(JWT_PUBLIC).unwrap();
        app.secure_reset("another password", "", &secure_token, &jwt_public)
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_INVALID_TOKEN))
            .unwrap_err();
    }

    #[test]
    fn user_reset_should_not_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);

        app.reset(0, "ABCDEF12345678901", "").unwrap();
    }

    #[test]
    fn user_reset_same_password_should_fail() {
        let mut secret_repo = SecretRepositoryMock::new();
        secret_repo.fn_find_by_user_and_name = Some(|_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret, Box<dyn Error>> {
            Err(constants::ERR_NOT_FOUND.into())
        });

        let mut app = new_user_application();
        app.secret_repo = Arc::new(secret_repo);

        app.reset(0, TEST_DEFAULT_USER_PASSWORD, "")
            .map_err(|err| assert_eq!(err.to_string(), constants::ERR_WRONG_CREDENTIALS))
            .unwrap_err();
    }
}