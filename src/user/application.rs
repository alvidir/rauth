use super::domain::User;
use crate::crypto;
use crate::result::{Error, Result};
use crate::secret::domain::SecretKind;
use crate::secret::{application::SecretRepository, domain::Secret};
use crate::token::application::{GenerateOptions, VerifyOptions};
use crate::token::domain::TokenDefinition;
use crate::token::{
    application::{TokenApplication, TokenRepository},
    domain::{Token, TokenKind},
};
use async_trait::async_trait;
use chrono::Utc;
use std::num::ParseIntError;
use std::sync::Arc;

#[async_trait]
pub trait UserRepository {
    async fn find(&self, id: i32) -> Result<User>;
    async fn find_by_email(&self, email: &str) -> Result<User>;
    async fn find_by_name(&self, name: &str) -> Result<User>;
    async fn create(&self, user: &mut User) -> Result<()>;
    async fn save(&self, user: &User) -> Result<()>;
    async fn delete(&self, user: &User) -> Result<()>;
}

#[async_trait]
pub trait EventBus {
    async fn emit_user_created(&self, user: &User) -> Result<()>;
}

pub trait Mailer {
    fn send_verification_signup_email(&self, to: &str, token: &str) -> Result<()>;
    fn send_verification_reset_email(&self, to: &str, token: &str) -> Result<()>;
}

pub struct UserApplication<
    'a,
    U: UserRepository,
    E: SecretRepository,
    T: TokenRepository,
    B: EventBus,
    M: Mailer,
> {
    pub user_repo: Arc<U>,
    pub secret_repo: Arc<E>,
    pub token_app: Arc<TokenApplication<'a, T>>,
    pub mailer: Arc<M>,
    pub event_bus: Arc<B>,
    pub totp_secret_len: usize,
    pub pwd_sufix: &'a str,
}

impl<'a, U: UserRepository, E: SecretRepository, T: TokenRepository, B: EventBus, M: Mailer>
    UserApplication<'a, U, E, T, B, M>
{
    #[instrument(skip(self))]
    pub async fn verify_signup_email(&self, email: &str, pwd: &str) -> Result<()> {
        if self.user_repo.find_by_email(email).await.is_ok() {
            // returns Ok to not provide information about users
            return Ok(());
        }

        let pwd = crypto::obfuscate(pwd, self.pwd_sufix);
        User::new(email, &pwd)?;
        let token_to_keep = self
            .token_app
            .generate(
                TokenKind::Verification,
                email,
                Some(&pwd),
                GenerateOptions::default(),
            )
            .await?;

        let token_to_send = self
            .token_app
            .generate(
                TokenKind::Verification,
                token_to_keep.id(),
                None,
                GenerateOptions { store: false },
            )
            .await?;

        self.mailer
            .send_verification_signup_email(email, token_to_send.signature())?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn signup_with_token(&self, token: &str) -> Result<String> {
        let claims: Token = self.token_app.decode(token).await?;
        self.token_app
            .verify(
                &claims,
                VerifyOptions {
                    must_exists: false,
                    kind: Some(TokenKind::Verification),
                },
            )
            .await?;

        let claims: Token = self.token_app.retrieve(&claims.sub).await?;
        self.token_app
            .verify(&claims, VerifyOptions::new(TokenKind::Verification))
            .await?;

        let password = &claims.get_secret().ok_or(Error::InvalidToken)?;
        let token = self.signup(&claims.sub, password).await?;
        self.token_app.revoke(&claims).await?;

        Ok(token)
    }

    #[instrument(skip(self))]
    pub async fn signup(&self, email: &str, pwd: &str) -> Result<String> {
        let mut user = User::new(email, pwd)?;
        self.user_repo.create(&mut user).await?;
        self.event_bus.emit_user_created(&user).await?;
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

    #[instrument(skip(self))]
    pub async fn delete_with_token(&self, token: &str, pwd: &str, totp: &str) -> Result<()> {
        let claims: Token = self.token_app.decode(token).await?;
        self.token_app
            .verify(&claims, VerifyOptions::new(TokenKind::Session))
            .await?;

        let user_id = claims.sub.parse().map_err(|err: ParseIntError| {
            warn!(error = err.to_string(), "parsing str to i32");
            Error::InvalidToken
        })?;

        self.delete(user_id, pwd, totp).await
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, user_id: i32, pwd: &str, totp: &str) -> Result<()> {
        let user = self
            .user_repo
            .find(user_id)
            .await
            .map_err(|_| Error::WrongCredentials)?;

        let pwd = crypto::obfuscate(pwd, self.pwd_sufix);
        if !user.match_password(&pwd) {
            return Err(Error::WrongCredentials);
        }

        // if, and only if, the user has activated the totp
        let secret_lookup = self
            .secret_repo
            .find_by_user_and_kind(user.id, SecretKind::Totp)
            .await
            .ok();

        if let Some(secret) = secret_lookup {
            if !secret.is_deleted() {
                let data = secret.get_data();
                if !crypto::verify_totp(data, totp)? {
                    return Err(Error::Unauthorized);
                }
                self.secret_repo.delete(&secret).await?;
            }
        }

        self.user_repo.delete(&user).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn enable_totp_with_token(
        &self,
        token: &str,
        pwd: &str,
        totp: &str,
    ) -> Result<Option<String>> {
        let claims: Token = self.token_app.decode(token).await?;
        self.token_app
            .verify(&claims, VerifyOptions::new(TokenKind::Session))
            .await?;

        let user_id = claims.sub.parse().map_err(|err: ParseIntError| {
            warn!(error = err.to_string(), "parsing str to i32");
            Error::InvalidToken
        })?;

        self.enable_totp(user_id, pwd, totp).await
    }

    #[instrument(skip(self))]
    pub async fn enable_totp(&self, user_id: i32, pwd: &str, totp: &str) -> Result<Option<String>> {
        let user = self
            .user_repo
            .find(user_id)
            .await
            .map_err(|_| Error::WrongCredentials)?;

        let pwd = crypto::obfuscate(pwd, self.pwd_sufix);
        if !user.match_password(&pwd) {
            return Err(Error::WrongCredentials);
        }

        // if, and only if, the user has activated the totp
        let mut secret_lookup = self
            .secret_repo
            .find_by_user_and_kind(user.id, SecretKind::Totp)
            .await
            .ok();

        if let Some(secret) = &mut secret_lookup {
            if !secret.is_deleted() {
                // the totp is already enabled
                return Err(Error::NotAvailable);
            }

            let data = secret.get_data();
            if !crypto::verify_totp(data, totp)? {
                return Err(Error::Unauthorized);
            }

            secret.set_deleted_at(None);
            self.secret_repo.save(secret).await?;
            return Ok(None);
        }

        let token = crypto::get_random_string(self.totp_secret_len);
        let mut secret = Secret::new(SecretKind::Totp, token.as_bytes(), &user);
        secret.set_deleted_at(Some(Utc::now().naive_utc())); // unavailable till confirmed
        self.secret_repo.create(&mut secret).await?;
        Ok(Some(token))
    }

    #[instrument(skip(self))]
    pub async fn disable_totp_with_token(&self, token: &str, pwd: &str, totp: &str) -> Result<()> {
        let claims: Token = self.token_app.decode(token).await?;
        self.token_app
            .verify(&claims, VerifyOptions::new(TokenKind::Session))
            .await?;

        let user_id = claims.sub.parse().map_err(|err: ParseIntError| {
            warn!(error = err.to_string(), "parsing str to i32",);
            Error::InvalidToken
        })?;

        self.disable_totp(user_id, pwd, totp).await
    }

    #[instrument(skip(self))]
    pub async fn disable_totp(&self, user_id: i32, pwd: &str, totp: &str) -> Result<()> {
        let user = self
            .user_repo
            .find(user_id)
            .await
            .map_err(|_| Error::WrongCredentials)?;

        let pwd = crypto::obfuscate(pwd, self.pwd_sufix);
        if !user.match_password(&pwd) {
            return Err(Error::WrongCredentials);
        }

        // if, and only if, the user has activated the totp
        let mut secret_lookup = self
            .secret_repo
            .find_by_user_and_kind(user.id, SecretKind::Totp)
            .await
            .ok();

        if let Some(secret) = &mut secret_lookup {
            if secret.is_deleted() {
                // the totp is not enabled yet
                return Err(Error::NotAvailable);
            }

            let data = secret.get_data();
            if !crypto::verify_totp(data, totp)? {
                return Err(Error::Unauthorized);
            }

            self.secret_repo.delete(secret).await?;
            return Ok(());
        }

        Err(Error::NotAvailable)
    }

    #[instrument(skip(self))]
    pub async fn verify_reset_email(&self, email: &str) -> Result<()> {
        let user = match self.user_repo.find_by_email(email).await {
            Err(_) => return Ok(()), // returns Ok to not provide information about users
            Ok(user) => user,
        };

        let token = self
            .token_app
            .generate(
                TokenKind::Reset,
                &user.get_id().to_string(),
                None,
                GenerateOptions::default(),
            )
            .await?;

        self.mailer
            .send_verification_reset_email(email, token.signature())?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn reset_with_token(&self, token: &str, new_pwd: &str, totp: &str) -> Result<()> {
        let claims: Token = self.token_app.decode(token).await?;
        self.token_app
            .verify(&claims, VerifyOptions::new(TokenKind::Reset))
            .await?;

        let user_id = claims.sub.parse().map_err(|err: ParseIntError| {
            warn!(error = err.to_string(), "parsing str to i32",);
            Error::InvalidToken
        })?;

        self.reset(user_id, new_pwd, totp).await?;
        self.token_app.revoke(&claims).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn reset(&self, user_id: i32, new_pwd: &str, totp: &str) -> Result<()> {
        let mut user = self
            .user_repo
            .find(user_id)
            .await
            .map_err(|_| Error::WrongCredentials)?;

        let new_pwd = crypto::obfuscate(new_pwd, self.pwd_sufix);
        if user.match_password(&new_pwd) {
            return Err(Error::WrongCredentials);
        }

        // if, and only if, the user has activated the totp
        if let Ok(secret) = self
            .secret_repo
            .find_by_user_and_kind(user.get_id(), SecretKind::Totp)
            .await
        {
            if !secret.is_deleted() {
                let data = secret.get_data();
                if !crypto::verify_totp(data, totp)? {
                    return Err(Error::Unauthorized);
                }
            }
        }

        user.set_password(&new_pwd)?;
        self.user_repo.save(&user).await
    }
}

#[cfg(test)]
pub mod tests {
    use super::super::domain::tests::{TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD};
    use super::super::domain::{tests::new_user_custom, User};
    use super::{EventBus, Mailer, UserApplication, UserRepository};
    use crate::secret::domain::SecretKind;
    use crate::secret::{
        application::tests::SecretRepositoryMock,
        domain::{
            tests::{new_secret, TEST_DEFAULT_SECRET_DATA},
            Secret,
        },
    };
    use crate::token::application::tests::{new_token_application, PRIVATE_KEY, PUBLIC_KEY};
    use crate::token::{
        application::tests::TokenRepositoryMock,
        domain::{Token, TokenDefinition, TokenKind},
    };
    use crate::user::domain::tests::TEST_DEFAULT_PWD_SUFIX;
    use crate::{
        crypto,
        result::{Error, Result},
    };
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Arc;
    use std::time::Duration;

    pub const TEST_CREATE_ID: i32 = 999;
    pub const TEST_FIND_BY_EMAIL_ID: i32 = 888;
    pub const TEST_FIND_BY_NAME_ID: i32 = 777;

    type MockFnFind = Option<fn(this: &UserRepositoryMock, id: i32) -> Result<User>>;
    type MockFnFindByEmail = Option<fn(this: &UserRepositoryMock, email: &str) -> Result<User>>;
    type MockFnFindByName = Option<fn(this: &UserRepositoryMock, name: &str) -> Result<User>>;
    type MockFnCreate = Option<fn(this: &UserRepositoryMock, user: &mut User) -> Result<()>>;
    type MockFnSave = Option<fn(this: &UserRepositoryMock, user: &User) -> Result<()>>;
    type MockFnDelete = Option<fn(this: &UserRepositoryMock, user: &User) -> Result<()>>;

    #[derive(Default)]
    pub struct UserRepositoryMock {
        pub fn_find: MockFnFind,
        pub fn_find_by_email: MockFnFindByEmail,
        pub fn_find_by_name: MockFnFindByName,
        pub fn_create: MockFnCreate,
        pub fn_save: MockFnSave,
        pub fn_delete: MockFnDelete,
    }

    #[async_trait]
    impl UserRepository for UserRepositoryMock {
        async fn find(&self, id: i32) -> Result<User> {
            if let Some(f) = self.fn_find {
                return f(self, id);
            }

            Ok(new_user_custom(id, ""))
        }

        async fn find_by_email(&self, email: &str) -> Result<User> {
            if let Some(f) = self.fn_find_by_email {
                return f(self, email);
            }

            Ok(new_user_custom(TEST_FIND_BY_EMAIL_ID, email))
        }

        async fn find_by_name(&self, name: &str) -> Result<User> {
            if let Some(f) = self.fn_find_by_name {
                return f(self, name);
            }

            Ok(new_user_custom(TEST_FIND_BY_NAME_ID, name))
        }

        async fn create(&self, user: &mut User) -> Result<()> {
            if let Some(f) = self.fn_create {
                return f(self, user);
            }

            user.id = TEST_CREATE_ID;
            Ok(())
        }

        async fn save(&self, user: &User) -> Result<()> {
            if let Some(f) = self.fn_save {
                return f(self, user);
            }

            Ok(())
        }

        async fn delete(&self, user: &User) -> Result<()> {
            if let Some(f) = self.fn_delete {
                return f(self, user);
            }

            Ok(())
        }
    }

    type MockFnEmitUserCreated = Option<fn(this: &EventBusMock, user: &User) -> Result<()>>;

    #[derive(Default)]
    pub struct EventBusMock {
        pub fn_emit_user_created: MockFnEmitUserCreated,
    }

    #[async_trait]
    impl EventBus for EventBusMock {
        async fn emit_user_created(&self, user: &User) -> Result<()> {
            if let Some(f) = self.fn_emit_user_created {
                return f(self, user);
            }

            Ok(())
        }
    }

    #[derive(Default)]
    pub struct MailerMock {
        pub force_fail: bool,
    }

    impl Mailer for MailerMock {
        fn send_verification_signup_email(&self, _: &str, _: &str) -> Result<()> {
            if self.force_fail {
                return Err(Error::Unknown);
            }

            Ok(())
        }

        fn send_verification_reset_email(&self, _: &str, _: &str) -> Result<()> {
            if self.force_fail {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    pub fn new_user_application(
        token_repo: Option<&TokenRepositoryMock>,
    ) -> UserApplication<
        'static,
        UserRepositoryMock,
        SecretRepositoryMock,
        TokenRepositoryMock,
        EventBusMock,
        MailerMock,
    > {
        let user_repo = UserRepositoryMock::default();
        let secret_repo = SecretRepositoryMock::default();
        let mailer_mock = MailerMock::default();
        let token_app = new_token_application(token_repo.cloned());

        let event_bus = EventBusMock::default();
        UserApplication {
            user_repo: Arc::new(user_repo),
            secret_repo: Arc::new(secret_repo),
            token_app: Arc::new(token_app),
            mailer: Arc::new(mailer_mock),
            event_bus: Arc::new(event_bus),
            totp_secret_len: 32_usize,
            pwd_sufix: TEST_DEFAULT_PWD_SUFIX,
        }
    }

    #[tokio::test]
    async fn user_verify_should_not_fail() {
        let user_repo = UserRepositoryMock {
            fn_find_by_email: Some(|_: &UserRepositoryMock, _: &str| -> Result<User> {
                Err(Error::NotFound)
            }),
            ..Default::default()
        };

        let mut app = new_user_application(None);
        app.user_repo = Arc::new(user_repo);

        app.verify_signup_email(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn user_verify_already_exists_should_not_fail() {
        let app = new_user_application(None);
        app.verify_signup_email(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn user_verify_wrong_email_should_fail() {
        let user_repo = UserRepositoryMock {
            fn_find_by_email: Some(|_: &UserRepositoryMock, _: &str| -> Result<User> {
                Err(Error::NotFound)
            }),
            ..Default::default()
        };

        let mut app = new_user_application(None);
        app.user_repo = Arc::new(user_repo);

        app.verify_signup_email("this is not an email", TEST_DEFAULT_USER_PASSWORD)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidFormat.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_secure_signup_should_not_fail() {
        let user_repo = UserRepositoryMock {
            fn_find_by_email: Some(|_: &UserRepositoryMock, _: &str| -> Result<User> {
                Err(Error::NotFound)
            }),
            ..Default::default()
        };

        let token_to_keep = Token::new(
            "test",
            TEST_DEFAULT_USER_EMAIL,
            Duration::from_secs(60),
            TokenKind::Verification,
            Some(TEST_DEFAULT_USER_PASSWORD),
        );

        let token_to_send = Token::new(
            "test",
            &token_to_keep.get_id(),
            Duration::from_secs(60),
            TokenKind::Verification,
            None,
        );

        let token_to_keep = crypto::sign_jwt(&PRIVATE_KEY, token_to_keep).unwrap();
        let token_to_send = crypto::sign_jwt(&PRIVATE_KEY, token_to_send).unwrap();

        let token_repo = TokenRepositoryMock {
            token: token_to_keep.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, tid: &str| -> Result<String> {
                let claims: Token = crypto::decode_jwt(&PUBLIC_KEY, &this.token)?;
                assert_eq!(claims.get_id(), tid);

                Ok(this.token.clone())
            }),
            ..Default::default()
        };
        let mut app = new_user_application(Some(&token_repo));
        app.user_repo = Arc::new(user_repo);

        let token = app.signup_with_token(&token_to_send).await.unwrap();
        let claims: Token = crypto::decode_jwt(&PUBLIC_KEY, &token).unwrap();
        assert_eq!(claims.sub, TEST_CREATE_ID.to_string());
    }

    #[tokio::test]
    async fn user_secure_signup_verification_token_kind_should_fail() {
        let user_repo = UserRepositoryMock {
            fn_find_by_email: Some(|_: &UserRepositoryMock, _: &str| -> Result<User> {
                Err(Error::NotFound)
            }),
            ..Default::default()
        };

        let token = Token::new(
            "test",
            "test",
            Duration::from_secs(60),
            TokenKind::Verification,
            None,
        );

        let token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let mut app = new_user_application(None);
        app.user_repo = Arc::new(user_repo);

        app.signup_with_token(&token)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_secure_signup_reset_token_kind_should_fail() {
        let user_repo = UserRepositoryMock {
            fn_find_by_email: Some(|_: &UserRepositoryMock, _: &str| -> Result<User> {
                Err(Error::NotFound)
            }),
            ..Default::default()
        };

        let token = Token::new(
            "test",
            "test",
            Duration::from_secs(60),
            TokenKind::Reset,
            None,
        );

        let token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let mut app = new_user_application(None);
        app.user_repo = Arc::new(user_repo);

        app.signup_with_token(&token)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_signup_should_not_fail() {
        let user_repo = UserRepositoryMock {
            fn_find_by_email: Some(|_: &UserRepositoryMock, _: &str| -> Result<User> {
                Err(Error::Unknown)
            }),
            ..Default::default()
        };

        let mut app = new_user_application(None);
        app.user_repo = Arc::new(user_repo);

        let token = app
            .signup(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD)
            .await
            .unwrap();
        let claims: Token = crypto::decode_jwt(&PUBLIC_KEY, &token).unwrap();
        assert_eq!(claims.sub, TEST_CREATE_ID.to_string());
    }

    #[tokio::test]
    async fn user_signup_wrong_email_should_fail() {
        let app = new_user_application(None);
        app.signup("this is not an email", TEST_DEFAULT_USER_PASSWORD)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidFormat.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_signup_wrong_password_should_fail() {
        let app = new_user_application(None);
        app.signup(TEST_DEFAULT_USER_EMAIL, "bad password")
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidFormat.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_signup_already_exists_should_not_fail() {
        let app = new_user_application(None);
        app.signup(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn user_secure_delete_should_not_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let token = Token::new(
            "test",
            "0",
            Duration::from_secs(60),
            TokenKind::Session,
            None,
        );

        let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let token_repo = TokenRepositoryMock {
            token: secure_token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };

        let mut app = new_user_application(Some(&token_repo));
        app.secret_repo = Arc::new(secret_repo);

        app.delete_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, "")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn user_secure_delete_verification_token_kind_should_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let token = Token::new(
            "test",
            "0",
            Duration::from_secs(60),
            TokenKind::Verification,
            None,
        );

        let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let token_repo = TokenRepositoryMock {
            token: secure_token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };

        let mut app = new_user_application(Some(&token_repo));
        app.secret_repo = Arc::new(secret_repo);

        app.delete_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, "")
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_secure_delete_reset_token_kind_should_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let token = Token::new("test", "0", Duration::from_secs(60), TokenKind::Reset, None);

        let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let token_repo = TokenRepositoryMock {
            token: secure_token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };

        let mut app = new_user_application(Some(&token_repo));
        app.secret_repo = Arc::new(secret_repo);

        app.delete_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, "")
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_delete_should_not_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let mut app = new_user_application(None);
        app.secret_repo = Arc::new(secret_repo);

        app.delete(0, TEST_DEFAULT_USER_PASSWORD, "").await.unwrap();
    }

    #[tokio::test]
    async fn user_delete_totp_should_not_fail() {
        let app = new_user_application(None);
        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();
        app.delete(0, TEST_DEFAULT_USER_PASSWORD, &code)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn user_delete_not_found_should_fail() {
        let user_repo = UserRepositoryMock {
            fn_find: Some(|_: &UserRepositoryMock, _: i32| -> Result<User> {
                Err(Error::NotFound)
            }),
            ..Default::default()
        };

        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let mut app = new_user_application(None);
        app.user_repo = Arc::new(user_repo);
        app.secret_repo = Arc::new(secret_repo);

        app.delete(0, TEST_DEFAULT_USER_PASSWORD, "")
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::WrongCredentials.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_delete_wrong_password_should_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let mut app = new_user_application(None);
        app.secret_repo = Arc::new(secret_repo);

        app.delete(0, "bad password", "")
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::WrongCredentials.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_delete_wrong_totp_should_fail() {
        let app = new_user_application(None);
        app.delete(0, TEST_DEFAULT_USER_PASSWORD, "bad totp")
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::Unauthorized.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_secure_enable_totp_should_not_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let token = Token::new(
            "test",
            "0",
            Duration::from_secs(60),
            TokenKind::Session,
            None,
        );

        let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let token_repo = TokenRepositoryMock {
            token: secure_token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };

        let mut app = new_user_application(Some(&token_repo));
        app.secret_repo = Arc::new(secret_repo);

        let totp = app
            .enable_totp_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, "")
            .await
            .unwrap();
        assert!(totp.is_some());
        assert_eq!(totp.unwrap().len(), app.totp_secret_len);
    }

    #[tokio::test]
    async fn user_secure_enable_totp_verification_token_kind_should_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let token = Token::new(
            "test",
            "0",
            Duration::from_secs(60),
            TokenKind::Verification,
            None,
        );

        let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let token_repo = TokenRepositoryMock {
            token: secure_token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };

        let mut app = new_user_application(Some(&token_repo));
        app.secret_repo = Arc::new(secret_repo);

        app.enable_totp_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, "")
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_secure_enable_totp_reset_token_kind_should_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let token = Token::new("test", "0", Duration::from_secs(60), TokenKind::Reset, None);
        let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let token_repo = TokenRepositoryMock {
            token: secure_token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };

        let mut app = new_user_application(Some(&token_repo));
        app.secret_repo = Arc::new(secret_repo);

        app.enable_totp_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, "")
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_enable_totp_should_not_fail() {
        let mut secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        secret_repo.fn_save = Some(|_: &SecretRepositoryMock, secret: &Secret| -> Result<()> {
            if !secret.is_deleted() {
                return Err(Error::Unknown);
            }

            Ok(())
        });

        let mut app = new_user_application(None);
        app.secret_repo = Arc::new(secret_repo);

        let totp = app
            .enable_totp(0, TEST_DEFAULT_USER_PASSWORD, "")
            .await
            .unwrap();
        assert!(totp.is_some());
        assert_eq!(totp.unwrap().len(), app.totp_secret_len);
    }

    #[tokio::test]
    async fn user_enable_totp_verify_should_not_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    let mut secret = new_secret();
                    secret.set_deleted_at(Some(Utc::now().naive_utc()));
                    Ok(secret)
                },
            ),
            fn_save: Some(|_: &SecretRepositoryMock, secret: &Secret| -> Result<()> {
                if secret.is_deleted() {
                    return Err(Error::Unknown);
                }

                Ok(())
            }),
            ..Default::default()
        };

        let mut app = new_user_application(None);
        app.secret_repo = Arc::new(secret_repo);

        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();
        let totp = app
            .enable_totp(0, TEST_DEFAULT_USER_PASSWORD, &code)
            .await
            .unwrap();
        assert_eq!(totp, None);
    }

    #[tokio::test]
    async fn user_enable_totp_wrong_password_should_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    let mut secret = new_secret();
                    secret.set_deleted_at(Some(Utc::now().naive_utc()));
                    Ok(secret)
                },
            ),
            ..Default::default()
        };

        let mut app = new_user_application(None);
        app.secret_repo = Arc::new(secret_repo);

        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();
        app.enable_totp(0, "bad password", &code)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::WrongCredentials.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_enable_totp_already_enabled_should_fail() {
        let app = new_user_application(None);
        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();
        app.enable_totp(0, TEST_DEFAULT_USER_PASSWORD, &code)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::NotAvailable.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_secure_disable_totp_should_not_fail() {
        let token = Token::new(
            "test",
            "0",
            Duration::from_secs(60),
            TokenKind::Session,
            None,
        );

        let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let token_repo = TokenRepositoryMock {
            token: secure_token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };

        let app = new_user_application(Some(&token_repo));
        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();
        app.disable_totp_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, &code)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn user_secure_disable_totp_verification_token_kind_should_fail() {
        let token = Token::new(
            "test",
            "0",
            Duration::from_secs(60),
            TokenKind::Verification,
            None,
        );

        let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let token_repo = TokenRepositoryMock {
            token: secure_token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };

        let app = new_user_application(Some(&token_repo));
        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();
        app.disable_totp_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, &code)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_secure_disable_totp_reset_token_kind_should_fail() {
        let token = Token::new("test", "0", Duration::from_secs(60), TokenKind::Reset, None);
        let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let token_repo = TokenRepositoryMock {
            token: secure_token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };

        let app = new_user_application(Some(&token_repo));
        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();
        app.disable_totp_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, &code)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_disable_totp_should_not_fail() {
        let app = new_user_application(None);
        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();
        app.disable_totp(0, TEST_DEFAULT_USER_PASSWORD, &code)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn user_disable_totp_wrong_password_should_fail() {
        let app = new_user_application(None);
        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();
        app.disable_totp(0, "bad password", &code)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::WrongCredentials.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_disable_totp_wrong_totp_should_fail() {
        let app = new_user_application(None);
        app.disable_totp(0, TEST_DEFAULT_USER_PASSWORD, "bad totp")
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::Unauthorized.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_disable_totp_not_enabled_should_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let mut app = new_user_application(None);
        app.secret_repo = Arc::new(secret_repo);

        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();
        app.disable_totp(0, TEST_DEFAULT_USER_PASSWORD, &code)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::NotAvailable.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_disable_totp_not_verified_should_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    let mut secret = new_secret();
                    secret.set_deleted_at(Some(Utc::now().naive_utc()));
                    Ok(secret)
                },
            ),
            ..Default::default()
        };

        let mut app = new_user_application(None);
        app.secret_repo = Arc::new(secret_repo);

        let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
            .unwrap()
            .generate();
        app.disable_totp(0, TEST_DEFAULT_USER_PASSWORD, &code)
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::NotAvailable.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_secure_reset_should_not_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let token = Token::new("test", "0", Duration::from_secs(60), TokenKind::Reset, None);
        let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let token_repo = TokenRepositoryMock {
            token: secure_token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };

        let mut app = new_user_application(Some(&token_repo));
        app.secret_repo = Arc::new(secret_repo);

        app.reset_with_token(&secure_token, "ABCDEF1234567891", "")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn user_secure_reset_verification_token_kind_should_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let token = Token::new(
            "test",
            "0",
            Duration::from_secs(60),
            TokenKind::Verification,
            None,
        );

        let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let token_repo = TokenRepositoryMock {
            token: secure_token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };

        let mut app = new_user_application(Some(&token_repo));
        app.secret_repo = Arc::new(secret_repo);

        app.reset_with_token(&secure_token, "another password", "")
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_secure_reset_session_token_kind_should_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let token = Token::new(
            "test",
            "0",
            Duration::from_secs(60),
            TokenKind::Session,
            None,
        );

        let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
        let token_repo = TokenRepositoryMock {
            token: secure_token.clone(),
            fn_find: Some(|this: &TokenRepositoryMock, _: &str| -> Result<String> {
                Ok(this.token.clone())
            }),
            ..Default::default()
        };

        let mut app = new_user_application(Some(&token_repo));
        app.secret_repo = Arc::new(secret_repo);

        app.reset_with_token(&secure_token, "another password", "")
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
            .unwrap_err();
    }

    #[tokio::test]
    async fn user_reset_should_not_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let mut app = new_user_application(None);
        app.secret_repo = Arc::new(secret_repo);

        app.reset(0, "ABCDEF12345678901", "").await.unwrap();
    }

    #[tokio::test]
    async fn user_reset_same_password_should_fail() {
        let secret_repo = SecretRepositoryMock {
            fn_find_by_user_and_kind: Some(
                |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
                    Err(Error::NotFound)
                },
            ),
            ..Default::default()
        };

        let mut app = new_user_application(None);
        app.secret_repo = Arc::new(secret_repo);

        app.reset(0, TEST_DEFAULT_USER_PASSWORD, "")
            .await
            .map_err(|err| assert_eq!(err.to_string(), Error::WrongCredentials.to_string()))
            .unwrap_err();
    }
}
