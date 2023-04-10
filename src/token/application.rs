use super::domain::{Token, TokenDefinition, TokenKind};
use crate::crypto;
use crate::regex;
use crate::result::{Error, Result};
use crate::secret::application::SecretRepository;
use crate::user::application::UserRepository;
use crate::user::domain::User;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[async_trait]
pub trait TokenRepository {
    async fn find(&self, key: &str) -> Result<String>;
    async fn save(&self, key: &str, token: &str, expire: Option<u64>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}

pub struct TokenApplication<'a, T: TokenRepository, U: UserRepository, E: SecretRepository> {
    pub token_repo: Arc<T>,
    pub user_repo: Arc<U>,
    pub secret_repo: Arc<E>,
    pub timeout: u64,
    pub totp_secret_name: &'a str,
    pub token_issuer: &'a str,
}

impl<'a, T: TokenRepository, U: UserRepository, E: SecretRepository> TokenApplication<'a, T, U, E> {
    pub async fn generate(
        &self,
        user: &User,
        timeout: u64,
        issuer: &str,
        jwt_secret: &[u8],
    ) -> Result<String> {
        let sess = Token::new(
            issuer,
            &user.get_id().to_string(),
            Duration::from_secs(timeout),
            TokenKind::Session,
        );

        let key = sess.get_id();
        let token = crypto::sign_jwt(jwt_secret, sess)?;

        self.token_repo.save(&key, &token, Some(timeout)).await?;
        Ok(token)
    }

    pub async fn verify<S: Serialize + DeserializeOwned + TokenDefinition>(
        &self,
        kind: TokenKind,
        token: &str,
        jwt_public: &[u8],
    ) -> Result<S> {
        let claims: S = crypto::verify_jwt(jwt_public, token)?;

        if claims.get_kind() != kind {
            warn!(
                "{} checking token's kind with id {}, got {:?} want {:?}",
                Error::InvalidToken,
                claims.get_id(),
                claims.get_kind(),
                kind
            );
            return Err(Error::InvalidToken);
        }

        let key = claims.get_id();
        let present_data = self.token_repo.find(&key).await.map_err(|err| {
            warn!(
                "{} finding token with id {}: {}",
                Error::InvalidToken,
                &key,
                err
            );
            Error::InvalidToken
        })?;
        if present_data != token {
            error!(
                "{} comparing tokens with id {}: do not match",
                Error::InvalidToken,
                &key
            );
            return Err(Error::InvalidToken);
        }
        Ok(claims)
    }

    pub async fn login(
        &self,
        ident: &str,
        pwd: &str,
        totp: &str,
        jwt_secret: &[u8],
    ) -> Result<String> {
        info!(
            "processing a \"login\" request for user identified by {} ",
            ident
        );
        let user = {
            if regex::match_regex(regex::EMAIL, ident).is_ok() {
                self.user_repo.find_by_email(ident).await
            } else {
                self.user_repo.find_by_name(ident).await
            }
        }
        .map_err(|_| Error::WrongCredentials)?;

        if !user.match_password(pwd) {
            return Err(Error::WrongCredentials);
        }

        // if, and only if, the user has activated the totp
        if let Ok(secret) = self
            .secret_repo
            .find_by_user_and_name(user.get_id(), self.totp_secret_name)
            .await
        {
            if !secret.is_deleted() {
                let data = secret.get_data();
                if !crypto::verify_totp(data, totp)? {
                    return Err(Error::Unauthorized);
                }
            }
        }

        self.generate(&user, self.timeout, self.token_issuer, jwt_secret)
            .await
    }

    pub async fn logout(&self, token: &str, jwt_public: &[u8]) -> Result<()> {
        info!("processing a \"logout\" request for token {} ", token);

        let claims: Token = self.verify(TokenKind::Session, token, jwt_public).await?;

        self.token_repo
            .delete(&claims.get_id())
            .await
            .map_err(|err| {
                error!(
                    "{} removing token with id {}: {}",
                    Error::Unknown,
                    claims.get_id(),
                    err
                );
                Error::Unknown
            })?;
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::super::domain::{tests::new_session_token, Token, TokenKind};
    use super::{TokenApplication, TokenRepository};
    use crate::secret::application::tests::SecretRepositoryMock;
    use crate::secret::domain::tests::TEST_DEFAULT_SECRET_DATA;
    use crate::secret::domain::Secret;
    use crate::time;
    use crate::user::{
        application::tests::{UserRepositoryMock, TEST_FIND_BY_EMAIL_ID, TEST_FIND_BY_NAME_ID},
        domain::tests::{
            TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_NAME, TEST_DEFAULT_USER_PASSWORD,
        },
        domain::User,
    };
    use crate::{
        crypto,
        result::{Error, Result},
    };
    use async_trait::async_trait;
    use base64::{engine::general_purpose, Engine as _};
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    pub(super) const JWT_SECRET: &[u8] = b"LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JR0hBZ0VBTUJNR0J5cUdTTTQ5QWdFR0NDcUdTTTQ5QXdFSEJHMHdhd0lCQVFRZy9JMGJTbVZxL1BBN2FhRHgKN1FFSGdoTGxCVS9NcWFWMUJab3ZhM2Y5aHJxaFJBTkNBQVJXZVcwd3MydmlnWi96SzRXcGk3Rm1mK0VPb3FybQpmUlIrZjF2azZ5dnBGd0gzZllkMlllNXl4b3ZsaTROK1ZNNlRXVFErTmVFc2ZmTWY2TkFBMloxbQotLS0tLUVORCBQUklWQVRFIEtFWS0tLS0tCg==";
    pub(super) const JWT_PUBLIC: &[u8] = b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFVm5sdE1MTnI0b0dmOHl1RnFZdXhabi9oRHFLcQo1bjBVZm45YjVPc3I2UmNCOTMySGRtSHVjc2FMNVl1RGZsVE9rMWswUGpYaExIM3pIK2pRQU5tZFpnPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg==";

    type MockFnFind = Option<fn(this: &TokenRepositoryMock, key: &str) -> Result<String>>;
    type MockFnSave = Option<
        fn(this: &TokenRepositoryMock, key: &str, token: &str, expire: Option<u64>) -> Result<()>,
    >;
    type MockFnDelete = Option<fn(this: &TokenRepositoryMock, key: &str) -> Result<()>>;

    #[derive(Default)]
    pub struct TokenRepositoryMock {
        pub fn_find: MockFnFind,
        pub fn_save: MockFnSave,
        pub fn_delete: MockFnDelete,
        pub token: String,
    }

    #[async_trait]
    impl TokenRepository for TokenRepositoryMock {
        async fn find(&self, key: &str) -> Result<String> {
            if let Some(fn_find) = self.fn_find {
                return fn_find(self, key);
            }

            Ok(self.token.clone())
        }

        async fn save(&self, key: &str, token: &str, expire: Option<u64>) -> Result<()> {
            if let Some(fn_save) = self.fn_save {
                return fn_save(self, key, token, expire);
            }

            Ok(())
        }

        async fn delete(&self, key: &str) -> Result<()> {
            if let Some(fn_delete) = self.fn_delete {
                return fn_delete(self, key);
            }

            Ok(())
        }
    }

    pub fn new_token_application<'a>(
    ) -> TokenApplication<'a, TokenRepositoryMock, UserRepositoryMock, SecretRepositoryMock> {
        let user_repo = UserRepositoryMock::default();
        let secret_repo = SecretRepositoryMock::default();
        let token_repo = TokenRepositoryMock::default();

        TokenApplication {
            user_repo: Arc::new(user_repo),
            secret_repo: Arc::new(secret_repo),
            token_repo: Arc::new(token_repo),
            timeout: 999,
            totp_secret_name: ".dummy_totp_secret",
            token_issuer: "dummy",
        }
    }

    #[tokio::test]
    async fn login_by_email_should_not_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_name: Some(
                |_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let mut app = new_token_application();
        app.secret_repo = Arc::new(secret_repo);

        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let token = app
            .login(
                TEST_DEFAULT_USER_EMAIL,
                TEST_DEFAULT_USER_PASSWORD,
                "",
                &jwt_secret,
            )
            .await
            .map_err(|err| {
                println!(
                    "-\tlogin_by_email_should_not_fail has failed with error {}",
                    err
                )
            })
            .unwrap();
        let jwt_public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();
        let session: Token = crypto::verify_jwt(&jwt_public, &token).unwrap();

        assert_eq!(session.sub, TEST_FIND_BY_EMAIL_ID.to_string());
    }

    #[tokio::test]
    async fn login_by_username_should_not_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_name: Some(
                |_: &SecretRepositoryMock, _: i32, _: &str| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let mut app = new_token_application();
        app.secret_repo = Arc::new(secret_repo);
        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let token = app
            .login(
                TEST_DEFAULT_USER_NAME,
                TEST_DEFAULT_USER_PASSWORD,
                "",
                &jwt_secret,
            )
            .await
            .map_err(|err| {
                println!(
                    "-\tlogin_by_username_should_not_fail has failed with error {}",
                    err
                )
            })
            .unwrap();
        let jwt_public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();
        let session: Token = crypto::verify_jwt(&jwt_public, &token).unwrap();
        assert_eq!(session.sub, TEST_FIND_BY_NAME_ID.to_string());
    }

    #[tokio::test]
    async fn login_with_totp_should_not_fail() {
        let app = new_token_application();
        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();
        let token = app
            .login(
                TEST_DEFAULT_USER_NAME,
                TEST_DEFAULT_USER_PASSWORD,
                &code,
                &jwt_secret,
            )
            .await
            .map_err(|err| {
                println!(
                    "-\tlogin_with_totp_should_not_fail has failed with error {}",
                    err
                )
            })
            .unwrap();
        let jwt_public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();
        let session: Token = crypto::verify_jwt(&jwt_public, &token).unwrap();
        assert_eq!(session.sub, TEST_FIND_BY_NAME_ID.to_string());
    }

    #[tokio::test]
    async fn login_user_not_found_should_fail() {
        let user_repo = UserRepositoryMock {
            fn_find_by_email: Some(|_: &UserRepositoryMock, _: &str| -> Result<User> {
                Err(Error::WrongCredentials)
            }),
            ..Default::default()
        };

        let mut app = new_token_application();
        app.user_repo = Arc::new(user_repo);

        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();

        app.login(
            TEST_DEFAULT_USER_EMAIL,
            TEST_DEFAULT_USER_PASSWORD,
            &code,
            &jwt_secret,
        )
        .await
        .map_err(|err| assert_eq!(err.to_string(), Error::WrongCredentials.to_string()))
        .unwrap_err();
    }

    #[tokio::test]
    async fn login_wrong_password_should_fail() {
        let app = new_token_application();
        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();
        app.login(TEST_DEFAULT_USER_NAME, "fake_password", &code, &jwt_secret)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::WrongCredentials.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn login_wrong_totp_should_fail() {
        let app = new_token_application();
        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();

        app.login(
            TEST_DEFAULT_USER_NAME,
            TEST_DEFAULT_USER_PASSWORD,
            "fake_totp",
            &jwt_secret,
        )
        .await
        .map_err(|err| assert_eq!(err.to_string(), Error::Unauthorized.to_string()))
        .unwrap_err();
    }

    #[tokio::test]
    async fn logout_should_not_fail() {
        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let token = crypto::sign_jwt(&jwt_secret, new_session_token()).unwrap();

        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            ..Default::default()
        };

        let mut app = new_token_application();
        app.token_repo = Arc::new(token_repo);
        let jwt_public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();
        app.logout(&token, &jwt_public)
            .await
            .map_err(|err| println!("-\tlogout_should_not_fail has failed with error {}", err))
            .unwrap();
    }

    #[tokio::test]
    async fn logout_verification_token_kind_should_fail() {
        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let mut token = new_session_token();
        token.knd = TokenKind::Verification;

        let token = crypto::sign_jwt(&jwt_secret, token).unwrap();

        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            ..Default::default()
        };

        let mut app = new_token_application();
        app.token_repo = Arc::new(token_repo);
        let jwt_public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();
        app.logout(&token, &jwt_public)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn logout_reset_token_kind_should_fail() {
        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let mut token = new_session_token();
        token.knd = TokenKind::Reset;

        let token = crypto::sign_jwt(&jwt_secret, token).unwrap();

        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            ..Default::default()
        };

        let mut app = new_token_application();
        app.token_repo = Arc::new(token_repo);
        let jwt_public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();
        app.logout(&token, &jwt_public)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn verify_token_should_not_fail() {
        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let mut token = new_session_token();
        token.knd = TokenKind::Verification;

        let token = crypto::sign_jwt(&jwt_secret, token).unwrap();

        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            ..Default::default()
        };

        let mut app = new_token_application();

        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let token = crypto::sign_jwt(&jwt_secret, new_session_token()).unwrap();
        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };
        let public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();
        app.verify::<Token>(TokenKind::Session, &token, &public)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn verif_token_expired_should_fail() {
        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let mut token = new_session_token();
        token.knd = TokenKind::Verification;

        let token = crypto::sign_jwt(&jwt_secret, token).unwrap();

        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            ..Default::default()
        };

        let mut app = new_token_application();

        let mut claim = new_session_token();
        claim.exp = time::unix_timestamp(SystemTime::now() - Duration::from_secs(61));
        let secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let token = crypto::sign_jwt(&secret, claim).unwrap();
        let public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();
        let token_repo = TokenRepositoryMock::default();
        app.verify::<Token>(TokenKind::Session, &token, &public)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn verify_token_invalid_should_fail() {
        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let mut token = new_session_token();
        token.knd = TokenKind::Verification;

        let token = crypto::sign_jwt(&jwt_secret, token).unwrap();

        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            ..Default::default()
        };

        let mut app = new_token_application();

        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let token = crypto::sign_jwt(&jwt_secret, new_session_token())
            .unwrap()
            .replace('A', "a");
        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };
        let public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();
        app.verify::<Token>(TokenKind::Session, &token, &public)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn verify_token_wrong_kind_should_fail() {
        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let mut token = new_session_token();
        token.knd = TokenKind::Verification;

        let token = crypto::sign_jwt(&jwt_secret, token).unwrap();

        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            ..Default::default()
        };

        let mut app = new_token_application();

        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let token = crypto::sign_jwt(&jwt_secret, new_session_token()).unwrap();
        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };
        let public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();
        app.verify::<Token>(TokenKind::Verification, &token, &public)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn verify_token_not_present_should_fail() {
        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let mut token = new_session_token();
        token.knd = TokenKind::Verification;

        let token = crypto::sign_jwt(&jwt_secret, token).unwrap();

        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            ..Default::default()
        };

        let mut app = new_token_application();

        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let token = crypto::sign_jwt(&jwt_secret, new_session_token()).unwrap();
        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            fn_find: Some(|_: &TokenRepositoryMock, _: &str| -> Result<String> {
                Err(Error::NotFound)
            }),
            ..Default::default()
        };
        let public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();
        app.verify::<Token>(TokenKind::Verification, &token, &public)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn verify_token_mismatch_should_fail() {
        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let mut token = new_session_token();
        token.knd = TokenKind::Verification;

        let token = crypto::sign_jwt(&jwt_secret, token).unwrap();

        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            ..Default::default()
        };

        let mut app = new_token_application();

        let jwt_secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let token = crypto::sign_jwt(&jwt_secret, new_session_token()).unwrap();
        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            fn_find: Some(|_: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok("hello world".to_string())
            }),
            ..Default::default()
        };
        let public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();
        app.verify::<Token>(TokenKind::Verification, &token, &public)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }
}
