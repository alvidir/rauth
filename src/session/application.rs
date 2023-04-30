use crate::crypto;
use crate::regex;
use crate::result::{Error, Result};
use crate::secret::application::SecretRepository;
use crate::token::application::GenerateOptions;
use crate::token::application::TokenApplication;
use crate::token::application::TokenRepository;
use crate::token::application::VerifyOptions;
use crate::token::domain::TokenKind;
use crate::user::application::UserRepository;
use std::sync::Arc;

pub struct SessionApplication<'a, T: TokenRepository, U: UserRepository, E: SecretRepository> {
    pub user_repo: Arc<U>,
    pub secret_repo: Arc<E>,
    pub token_app: Arc<TokenApplication<'a, T>>,
    pub totp_secret_name: &'a str,
    pub pwd_sufix: &'a str,
}

impl<'a, T: TokenRepository, U: UserRepository, E: SecretRepository>
    SessionApplication<'a, T, U, E>
{
    pub async fn login(&self, ident: &str, pwd: &str, totp: &str) -> Result<String> {
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

        let pwd = crypto::obfuscate(pwd, self.pwd_sufix);
        if !user.match_password(&pwd) {
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

        self.token_app
            .generate(
                TokenKind::Session,
                &user.get_id().to_string(),
                None,
                GenerateOptions::default(),
            )
            .await
            .map(|token| token.signature().to_string())
    }

    pub async fn logout(&self, token: &str) -> Result<()> {
        info!("processing a \"logout\" request for token {} ", token);
        logout_strategy::<T>(&self.token_app, token).await
    }
}

pub(super) async fn logout_strategy<'b, R: TokenRepository>(
    token_app: &TokenApplication<'b, R>,
    token: &str,
) -> Result<()> {
    let token = token_app.decode(token).await?;

    token_app
        .verify(&token, VerifyOptions::new(TokenKind::Session))
        .await?;

    token_app.revoke(&token).await
}

#[cfg(test)]
pub mod tests {
    use super::{SessionApplication, TokenRepository};
    use crate::secret::application::tests::SecretRepositoryMock;
    use crate::secret::domain::tests::TEST_DEFAULT_SECRET_DATA;
    use crate::secret::domain::Secret;
    use crate::token::application::tests::{
        new_token, new_token_application, PRIVATE_KEY, PUBLIC_KEY,
    };
    use crate::token::domain::{Token, TokenKind};
    use crate::user::domain::tests::TEST_DEFAULT_PWD_SUFIX;
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
    use std::sync::Arc;

    type MockFnFind = Option<fn(this: &TokenRepositoryMock, key: &str) -> Result<String>>;
    type MockFnSave = Option<
        fn(this: &TokenRepositoryMock, key: &str, token: &str, expire: Option<u64>) -> Result<()>,
    >;
    type MockFnDelete = Option<fn(this: &TokenRepositoryMock, key: &str) -> Result<()>>;

    #[derive(Default, Clone)]
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

    pub fn new_session_application<'a, T: TokenRepository + Default>(
        token_repo: Option<T>,
    ) -> SessionApplication<'a, T, UserRepositoryMock, SecretRepositoryMock> {
        let user_repo = UserRepositoryMock::default();
        let secret_repo = SecretRepositoryMock::default();
        let token_app = new_token_application(token_repo);

        SessionApplication {
            user_repo: Arc::new(user_repo),
            secret_repo: Arc::new(secret_repo),
            token_app: Arc::new(token_app),
            totp_secret_name: ".dummy_totp_secret",
            pwd_sufix: TEST_DEFAULT_PWD_SUFIX,
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

        let mut app = new_session_application::<TokenRepositoryMock>(None);
        app.secret_repo = Arc::new(secret_repo);

        let token = app
            .login(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD, "")
            .await
            .map_err(|err| {
                println!(
                    "-\tlogin_by_email_should_not_fail has failed with error {}",
                    err
                )
            })
            .unwrap();
        let session: Token = crypto::decode_jwt(&PUBLIC_KEY, &token).unwrap();

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

        let mut app = new_session_application::<TokenRepositoryMock>(None);
        app.secret_repo = Arc::new(secret_repo);
        let token = app
            .login(TEST_DEFAULT_USER_NAME, TEST_DEFAULT_USER_PASSWORD, "")
            .await
            .map_err(|err| {
                println!(
                    "-\tlogin_by_username_should_not_fail has failed with error {}",
                    err
                )
            })
            .unwrap();
        let session: Token = crypto::decode_jwt(&PUBLIC_KEY, &token).unwrap();
        assert_eq!(session.sub, TEST_FIND_BY_NAME_ID.to_string());
    }

    #[tokio::test]
    async fn login_with_totp_should_not_fail() {
        let app = new_session_application::<TokenRepositoryMock>(None);
        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();
        let token = app
            .login(TEST_DEFAULT_USER_NAME, TEST_DEFAULT_USER_PASSWORD, &code)
            .await
            .map_err(|err| {
                println!(
                    "-\tlogin_with_totp_should_not_fail has failed with error {}",
                    err
                )
            })
            .unwrap();
        let session: Token = crypto::decode_jwt(&PUBLIC_KEY, &token).unwrap();
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

        let mut app = new_session_application::<TokenRepositoryMock>(None);
        app.user_repo = Arc::new(user_repo);

        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();

        app.login(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD, &code)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::WrongCredentials.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn login_wrong_password_should_fail() {
        let app = new_session_application::<TokenRepositoryMock>(None);
        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();
        app.login(TEST_DEFAULT_USER_NAME, "fake_password", &code)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::WrongCredentials.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn login_wrong_totp_should_fail() {
        let app = new_session_application::<TokenRepositoryMock>(None);

        app.login(
            TEST_DEFAULT_USER_NAME,
            TEST_DEFAULT_USER_PASSWORD,
            "fake_totp",
        )
        .await
        .map_err(|err| assert_eq!(err.to_string(), Error::Unauthorized.to_string()))
        .unwrap_err();
    }

    #[tokio::test]
    async fn logout_should_not_fail() {
        let token = crypto::sign_jwt(&PRIVATE_KEY, new_token(TokenKind::Session)).unwrap();
        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            ..Default::default()
        };

        let app = new_session_application::<TokenRepositoryMock>(Some(token_repo));
        app.logout(&token)
            .await
            .map_err(|err| println!("-\tlogout_should_not_fail has failed with error {}", err))
            .unwrap();
    }

    #[tokio::test]
    async fn logout_verification_token_kind_should_fail() {
        let token = new_token(TokenKind::Verification);
        let token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            ..Default::default()
        };

        let app = new_session_application::<TokenRepositoryMock>(Some(token_repo));
        app.logout(&token)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn logout_reset_token_kind_should_fail() {
        let token = new_token(TokenKind::Reset);
        let token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let token_repo = TokenRepositoryMock {
            token: token.clone(),
            ..Default::default()
        };

        let app = new_session_application::<TokenRepositoryMock>(Some(token_repo));
        app.logout(&token)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }
}
