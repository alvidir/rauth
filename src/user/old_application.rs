use super::domain::{Credentials, Email, Otp, Password, PasswordHash, User};
use super::error::{Error, Result};
use crate::cache::Cache;
use crate::crypto;
use crate::on_error;
use crate::secret::application::SecretRepository;
use crate::secret::domain::SecretKind;
use crate::token::domain::Token;
use crate::token::{
    application::TokenApplication,
    domain::{Kind, Payload},
};
use async_trait::async_trait;
use std::num::ParseIntError;
use std::sync::Arc;

impl<'a, U, S, B, M, C> UserApplication<'a, U, S, B, M, C>
where
    U: UserRepository,
    S: SecretRepository,
    B: EventBus,
    M: Mailer,
    C: Cache,
{
    /// Given a valid session token and passwords, performs the deletion of the user.
    #[instrument(skip(self, token, password, otp))]
    pub async fn delete_with_token(
        &self,
        token: Token,
        password: Password,
        otp: Otp,
    ) -> Result<()> {
        let payload = self.token_srv.decode(token)?;
        if !payload.kind().is_session() {
            return Error::WrongToken.into();
        }

        // make sure the token is still valid
        self.token_srv.find(&payload.jti).await?;

        let user_id = payload
            .sub
            .parse()
            .map_err(on_error!("parsing token subject to user id"))?;

        self.delete(user_id, password, otp).await
    }

    /// Given a valid user ID and passwords, performs the deletion of the corresponding user.
    #[instrument(skip(self))]
    pub async fn delete(&self, user_id: i32, password: Password, otp: Otp) -> Result<()> {
        let user = self.user_repo.find(user_id).await?;

        if !user.password_matches(password)? {
            return Err(Error::WrongCredentials);
        }

        // if, and only if, the user has activated the totp
        let secret_lookup = self
            .secret_repo
            .find_by_owner_and_kind(user.id, SecretKind::Totp)
            .await
            .ok();

        if let Some(secret) = secret_lookup {
            if !crypto::verify_totp(secret.data(), otp)? {
                return Err(Error::Unauthorized);
            }
            self.secret_repo.delete(&secret).await?;
        }

        self.user_repo.delete(&user).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn enable_totp_with_token(
        &self,
        token: Token,
        password: &str,
        otp: &str,
    ) -> Result<Option<String>> {
        let claims: Payload = self.token_srv.payload_from(token.into())?;
        if !claims.knd.is_session() {
            return Err(Error::InvalidToken);
        }

        // make sure the token is still valid
        self.token_srv.find(&claims.jti).await?;

        let user_id = claims.sub.parse().map_err(|err: ParseIntError| {
            warn!(error = err.to_string(), "parsing str to i32");
            Error::InvalidToken
        })?;

        self.enable_totp(user_id, password, otp).await
    }

    #[instrument(skip(self))]
    pub async fn enable_totp(
        &self,
        user_id: i32,
        password: &str,
        otp: &str,
    ) -> Result<Option<String>> {
        let user = self
            .user_repo
            .find(user_id)
            .await
            .map_err(|_| Error::WrongCredentials)?;

        if PasswordHash::try_from(password).is_ok_and(|pwd| {
            user.credentials
                .password
                .as_ref()
                .map(|user_pwd| user_pwd == &pwd)
                .unwrap_or_default()
        }) {
            return Err(Error::WrongCredentials);
        }

        // if, and only if, the user has activated the totp
        // let mut secret_lookup = self
        //     .secret_repo
        //     .find_by_owner_and_kind(user.id, SecretKind::Totp)
        //     .await
        //     .ok();

        // if let Some(secret) = &mut secret_lookup {
        //     if !secret.is_deleted() {
        //         // the totp is already enabled
        //         return Err(Error::NotAvailable);
        //     }

        //     let data = secret.get_data();
        //     if !crypto::verify_totp(data, totp)? {
        //         return Err(Error::Unauthorized);
        //     }

        //     secret.set_deleted_at(None);
        //     self.secret_repo.save(secret).await?;
        //     return Ok(None);
        // }

        // let token = crypto::get_random_string(self.totp_secret_len);
        // let mut secret = Secret::new(SecretKind::Totp, token.as_bytes(), &user);
        // secret.set_deleted_at(Some(Utc::now().naive_utc())); // unavailable till confirmed
        // self.secret_repo.create(&mut secret).await?;
        // Ok(Some(token))
        todo!()
    }

    #[instrument(skip(self))]
    pub async fn disable_totp_with_token(
        &self,
        token: &str,
        password: &str,
        otp: &str,
    ) -> Result<()> {
        let claims: Payload = self.token_srv.payload_from(token.into())?;
        if !claims.knd.is_session() {
            return Err(Error::InvalidToken);
        }

        // make sure the token is still valid
        self.token_srv.find(&claims.jti).await?;

        let user_id = claims.sub.parse().map_err(|err: ParseIntError| {
            warn!(error = err.to_string(), "parsing str to i32",);
            Error::InvalidToken
        })?;

        self.disable_totp(user_id, password, otp).await
    }

    #[instrument(skip(self))]
    pub async fn disable_totp(&self, user_id: i32, password: &str, otp: &str) -> Result<()> {
        let user = self
            .user_repo
            .find(user_id)
            .await
            .map_err(|_| Error::WrongCredentials)?;

        if PasswordHash::try_from(password).is_ok_and(|pwd| {
            user.credentials
                .password
                .as_ref()
                .map(|user_pwd| user_pwd == &pwd)
                .unwrap_or_default()
        }) {
            return Err(Error::WrongCredentials);
        }

        // if, and only if, the user has activated the totp
        // let mut secret_lookup = self
        //     .secret_repo
        //     .find_by_user_and_kind(user.id, SecretKind::Totp)
        //     .await
        //     .ok();

        // if let Some(secret) = &mut secret_lookup {
        //     if secret.is_deleted() {
        //         // the totp is not enabled yet
        //         return Err(Error::NotAvailable);
        //     }

        //     let data = secret.get_data();
        //     if !crypto::verify_totp(data, totp)? {
        //         return Err(Error::Unauthorized);
        //     }

        //     self.secret_repo.delete(secret).await?;
        //     return Ok(());
        // }

        Err(Error::NotAvailable)
    }
}

#[cfg(test)]
pub mod tests {
    use super::super::{
        domain::User,
        error::{Error, Result},
    };
    use super::{EventBus, Mailer, UserApplication, UserRepository};
    use crate::cache::tests::InMemoryCache;
    use crate::crypto;
    use crate::secret::domain::SecretKind;
    use crate::secret::{application::tests::SecretRepositoryMock, domain::Secret};
    use crate::token::application::tests::{new_token_srvlication, PRIVATE_KEY, PUBLIC_KEY};
    use crate::token::domain::{Kind, Payload, Token};
    use crate::user::domain::Email;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Arc;
    use std::time::Duration;

    type MockFnFind = fn(this: &UserRepositoryMock, id: i32) -> Result<User>;
    type MockFnFindByEmail = fn(this: &UserRepositoryMock, email: &Email) -> Result<User>;
    type MockFnFindByName = fn(this: &UserRepositoryMock, name: &str) -> Result<User>;
    type MockFnCreate = fn(this: &UserRepositoryMock, user: &mut User) -> Result<()>;
    type MockFnSave = fn(this: &UserRepositoryMock, user: &User) -> Result<()>;
    type MockFnDelete = fn(this: &UserRepositoryMock, user: &User) -> Result<()>;

    #[derive(Default)]
    pub struct UserRepositoryMock {
        pub fn_find: Option<MockFnFind>,
        pub fn_find_by_email: Option<MockFnFindByEmail>,
        pub fn_find_by_name: Option<MockFnFindByName>,
        pub fn_create: Option<MockFnCreate>,
        pub fn_save: Option<MockFnSave>,
        pub fn_delete: Option<MockFnDelete>,
    }

    #[async_trait]
    impl UserRepository for UserRepositoryMock {
        async fn find(&self, id: i32) -> Result<User> {
            if let Some(f) = self.fn_find {
                return f(self, id);
            }

            Err(Error::Unknown)
        }

        async fn find_by_email(&self, email: &Email) -> Result<User> {
            if let Some(f) = self.fn_find_by_email {
                return f(self, email);
            }

            Err(Error::Unknown)
        }

        async fn find_by_name(&self, name: &str) -> Result<User> {
            if let Some(f) = self.fn_find_by_name {
                return f(self, name);
            }

            Err(Error::Unknown)
        }

        async fn create(&self, user: &mut User) -> Result<()> {
            if let Some(f) = self.fn_create {
                return f(self, user);
            }

            Err(Error::Unknown)
        }

        async fn save(&self, user: &User) -> Result<()> {
            if let Some(f) = self.fn_save {
                return f(self, user);
            }

            Err(Error::Unknown)
        }

        async fn delete(&self, user: &User) -> Result<()> {
            if let Some(f) = self.fn_delete {
                return f(self, user);
            }

            Err(Error::Unknown)
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
        fn send_credentials_verification_email(&self, _: &Email, _: &Token) -> Result<()> {
            if self.force_fail {
                return Err(Error::Unknown);
            }

            Ok(())
        }

        fn send_credentials_reset_email(&self, _: &Email, _: &Token) -> Result<()> {
            if self.force_fail {
                return Err(Error::Unknown);
            }

            Ok(())
        }
    }

    pub fn new_user_application() -> UserApplication<
        'static,
        UserRepositoryMock,
        SecretRepositoryMock,
        EventBusMock,
        MailerMock,
        InMemoryCache,
    > {
        let user_repo = UserRepositoryMock::default();
        let secret_repo = SecretRepositoryMock::default();
        let mailer_mock = MailerMock::default();
        let token_srv = new_token_srvlication();

        let event_bus = EventBusMock::default();
        UserApplication {
            user_repo: Arc::new(user_repo),
            secret_repo: Arc::new(secret_repo),
            cache: Arc::new(InMemoryCache),
            token_srv: Arc::new(token_srv),
            mailer: Arc::new(mailer_mock),
            event_bus: Arc::new(event_bus),
            totp_secret_len: 32_usize,
        }
    }

    // #[tokio::test]
    // async fn user_verify_should_not_fail() {
    //     let user_repo = UserRepositoryMock {
    //         fn_find_by_email: Some(|_: &UserRepositoryMock, _: &str| -> Result<User> {
    //             Err(Error::NotFound)
    //         }),
    //         ..Default::default()
    //     };

    //     let mut app = new_user_application();
    //     app.user_repo = Arc::new(user_repo);

    //     app.verify_credentials(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD)
    //         .await
    //         .unwrap();
    // }

    // #[tokio::test]
    // async fn user_verify_already_exists_should_not_fail() {
    //     let app = new_user_application();
    //     app.verify_credentials(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD)
    //         .await
    //         .unwrap();
    // }

    // #[tokio::test]
    // async fn user_verify_wrong_email_should_fail() {
    //     let user_repo = UserRepositoryMock {
    //         fn_find_by_email: Some(|_: &UserRepositoryMock, _: &str| -> Result<User> {
    //             Err(Error::NotFound)
    //         }),
    //         ..Default::default()
    //     };

    //     let mut app = new_user_application();
    //     app.user_repo = Arc::new(user_repo);

    //     app.verify_credentials("this is not an email", TEST_DEFAULT_USER_PASSWORD)
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::InvalidFormat.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_secure_signup_should_not_fail() {
    //     let user_repo = UserRepositoryMock {
    //         fn_find_by_email: Some(|_: &UserRepositoryMock, _: &str| -> Result<User> {
    //             Err(Error::NotFound)
    //         }),
    //         ..Default::default()
    //     };

    //     let token_to_keep = Token::new(
    //         TokenKind::Verification,
    //         "test",
    //         TEST_DEFAULT_USER_EMAIL,
    //         Duration::from_secs(60),
    //         // FIXME: update code to add the following statement:
    //         // Some(TEST_DEFAULT_USER_PASSWORD),
    //     );

    //     let token_to_send = Token::new(
    //         TokenKind::Verification,
    //         "test",
    //         &token_to_keep.jti.to_string(),
    //         Duration::from_secs(60),
    //     );

    //     let token_to_keep = crypto::sign_jwt(&PRIVATE_KEY, token_to_keep).unwrap();
    //     let token_to_send = crypto::sign_jwt(&PRIVATE_KEY, token_to_send).unwrap();
    //     let mut app = new_user_application();
    //     app.user_repo = Arc::new(user_repo);

    //     let token = app.signup_with_token(&token_to_send).await.unwrap();
    //     let claims: Token = crypto::decode_jwt(&PUBLIC_KEY, &token).unwrap();
    //     assert_eq!(claims.sub, TEST_CREATE_ID.to_string());
    // }

    // #[tokio::test]
    // async fn user_secure_signup_verification_token_kind_should_fail() {
    //     let user_repo = UserRepositoryMock {
    //         fn_find_by_email: Some(|_: &UserRepositoryMock, _: &str| -> Result<User> {
    //             Err(Error::NotFound)
    //         }),
    //         ..Default::default()
    //     };

    //     let token = Token::new(
    //         TokenKind::Verification,
    //         "test",
    //         "test",
    //         Duration::from_secs(60),
    //     );

    //     let token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
    //     let mut app = new_user_application();
    //     app.user_repo = Arc::new(user_repo);

    //     app.signup_with_token(&token)
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_secure_signup_reset_token_kind_should_fail() {
    //     let user_repo = UserRepositoryMock {
    //         fn_find_by_email: Some(|_: &UserRepositoryMock, _: &str| -> Result<User> {
    //             Err(Error::NotFound)
    //         }),
    //         ..Default::default()
    //     };

    //     let token = Token::new(TokenKind::Reset, "test", "test", Duration::from_secs(60));
    //     let token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
    //     let mut app = new_user_application();
    //     app.user_repo = Arc::new(user_repo);

    //     app.signup_with_token(&token)
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_signup_should_not_fail() {
    //     let user_repo = UserRepositoryMock {
    //         fn_find_by_email: Some(|_: &UserRepositoryMock, _: &str| -> Result<User> {
    //             Err(Error::Unknown)
    //         }),
    //         ..Default::default()
    //     };

    //     let mut app = new_user_application();
    //     app.user_repo = Arc::new(user_repo);

    //     let mut user = User::new(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD).unwrap();
    //     let token = app.signup(&mut user).await.unwrap();

    //     let claims: Token = crypto::decode_jwt(&PUBLIC_KEY, &token).unwrap();
    //     assert_eq!(claims.sub, TEST_CREATE_ID.to_string());
    // }

    // // #[tokio::test]
    // // async fn user_signup_already_exists_should_not_fail() {
    // //     // FIXME: mock UserRepo to return an instance on find.
    // //     let app = new_user_application();
    // //     app.signup_with_credentials(TEST_DEFAULT_USER_EMAIL, TEST_DEFAULT_USER_PASSWORD)
    // //         .await
    // //         .unwrap();
    // // }

    // #[tokio::test]
    // async fn user_secure_delete_should_not_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let token = Token::new(TokenKind::Session, "test", "0", Duration::from_secs(60));
    //     let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     app.delete_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, "")
    //         .await
    //         .unwrap();
    // }

    // #[tokio::test]
    // async fn user_secure_delete_verification_token_kind_should_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let token = Token::new(
    //         TokenKind::Verification,
    //         "test",
    //         "0",
    //         Duration::from_secs(60),
    //     );

    //     let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     app.delete_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, "")
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_secure_delete_reset_token_kind_should_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let token = Token::new(TokenKind::Reset, "test", "0", Duration::from_secs(60));
    //     let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     app.delete_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, "")
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_delete_should_not_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     app.delete(0, TEST_DEFAULT_USER_PASSWORD, "").await.unwrap();
    // }

    // #[tokio::test]
    // async fn user_delete_totp_should_not_fail() {
    //     let app = new_user_application();
    //     let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
    //         .unwrap()
    //         .generate();
    //     app.delete(0, TEST_DEFAULT_USER_PASSWORD, &code)
    //         .await
    //         .unwrap();
    // }

    // #[tokio::test]
    // async fn user_delete_not_found_should_fail() {
    //     let user_repo = UserRepositoryMock {
    //         fn_find: Some(|_: &UserRepositoryMock, _: i32| -> Result<User> {
    //             Err(Error::NotFound)
    //         }),
    //         ..Default::default()
    //     };

    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let mut app = new_user_application();
    //     app.user_repo = Arc::new(user_repo);
    //     app.secret_repo = Arc::new(secret_repo);

    //     app.delete(0, TEST_DEFAULT_USER_PASSWORD, "")
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::WrongCredentials.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_delete_wrong_password_should_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     app.delete(0, "bad password", "")
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::WrongCredentials.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_delete_wrong_totp_should_fail() {
    //     let app = new_user_application();
    //     app.delete(0, TEST_DEFAULT_USER_PASSWORD, "bad totp")
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::Unauthorized.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_secure_enable_totp_should_not_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let token = Token::new(TokenKind::Session, "test", "0", Duration::from_secs(60));
    //     let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     let totp = app
    //         .enable_totp_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, "")
    //         .await
    //         .unwrap();
    //     assert!(totp.is_some());
    //     assert_eq!(totp.unwrap().len(), app.totp_secret_len);
    // }

    // #[tokio::test]
    // async fn user_secure_enable_totp_verification_token_kind_should_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let token = Token::new(
    //         TokenKind::Verification,
    //         "test",
    //         "0",
    //         Duration::from_secs(60),
    //     );

    //     let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     app.enable_totp_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, "")
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_secure_enable_totp_reset_token_kind_should_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let token = Token::new(TokenKind::Reset, "test", "0", Duration::from_secs(60));
    //     let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     app.enable_totp_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, "")
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_enable_totp_should_not_fail() {
    //     let mut secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     secret_repo.fn_save = Some(|_: &SecretRepositoryMock, secret: &Secret| -> Result<()> {
    //         if !secret.is_deleted() {
    //             return Err(Error::Unknown);
    //         }

    //         Ok(())
    //     });

    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     let totp = app
    //         .enable_totp(0, TEST_DEFAULT_USER_PASSWORD, "")
    //         .await
    //         .unwrap();
    //     assert!(totp.is_some());
    //     assert_eq!(totp.unwrap().len(), app.totp_secret_len);
    // }

    // #[tokio::test]
    // async fn user_enable_totp_verify_should_not_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 let mut secret = new_secret();
    //                 secret.set_deleted_at(Some(Utc::now().naive_utc()));
    //                 Ok(secret)
    //             },
    //         ),
    //         fn_save: Some(|_: &SecretRepositoryMock, secret: &Secret| -> Result<()> {
    //             if secret.is_deleted() {
    //                 return Err(Error::Unknown);
    //             }

    //             Ok(())
    //         }),
    //         ..Default::default()
    //     };

    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
    //         .unwrap()
    //         .generate();
    //     let totp = app
    //         .enable_totp(0, TEST_DEFAULT_USER_PASSWORD, &code)
    //         .await
    //         .unwrap();
    //     assert_eq!(totp, None);
    // }

    // #[tokio::test]
    // async fn user_enable_totp_wrong_password_should_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 let mut secret = new_secret();
    //                 secret.set_deleted_at(Some(Utc::now().naive_utc()));
    //                 Ok(secret)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
    //         .unwrap()
    //         .generate();
    //     app.enable_totp(0, "bad password", &code)
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::WrongCredentials.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_enable_totp_already_enabled_should_fail() {
    //     let app = new_user_application();
    //     let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
    //         .unwrap()
    //         .generate();
    //     app.enable_totp(0, TEST_DEFAULT_USER_PASSWORD, &code)
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::NotAvailable.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_secure_disable_totp_should_not_fail() {
    //     let token = Token::new(TokenKind::Session, "test", "0", Duration::from_secs(60));

    //     let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
    //     let app = new_user_application();
    //     let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
    //         .unwrap()
    //         .generate();
    //     app.disable_totp_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, &code)
    //         .await
    //         .unwrap();
    // }

    // #[tokio::test]
    // async fn user_secure_disable_totp_verification_token_kind_should_fail() {
    //     let token = Token::new(
    //         TokenKind::Verification,
    //         "test",
    //         "0",
    //         Duration::from_secs(60),
    //     );

    //     let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
    //     let app = new_user_application();
    //     let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
    //         .unwrap()
    //         .generate();
    //     app.disable_totp_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, &code)
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_secure_disable_totp_reset_token_kind_should_fail() {
    //     let token = Token::new(TokenKind::Reset, "test", "0", Duration::from_secs(60));
    //     let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
    //     let app = new_user_application();
    //     let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
    //         .unwrap()
    //         .generate();
    //     app.disable_totp_with_token(&secure_token, TEST_DEFAULT_USER_PASSWORD, &code)
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_disable_totp_should_not_fail() {
    //     let app = new_user_application();
    //     let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
    //         .unwrap()
    //         .generate();
    //     app.disable_totp(0, TEST_DEFAULT_USER_PASSWORD, &code)
    //         .await
    //         .unwrap();
    // }

    // #[tokio::test]
    // async fn user_disable_totp_wrong_password_should_fail() {
    //     let app = new_user_application();
    //     let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
    //         .unwrap()
    //         .generate();
    //     app.disable_totp(0, "bad password", &code)
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::WrongCredentials.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_disable_totp_wrong_totp_should_fail() {
    //     let app = new_user_application();
    //     app.disable_totp(0, TEST_DEFAULT_USER_PASSWORD, "bad totp")
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::Unauthorized.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_disable_totp_not_enabled_should_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
    //         .unwrap()
    //         .generate();
    //     app.disable_totp(0, TEST_DEFAULT_USER_PASSWORD, &code)
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::NotAvailable.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_disable_totp_not_verified_should_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 let mut secret = new_secret();
    //                 secret.set_deleted_at(Some(Utc::now().naive_utc()));
    //                 Ok(secret)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     let code = crypto::generate_totp(TEST_DEFAULT_SECRET_DATA.as_bytes())
    //         .unwrap()
    //         .generate();
    //     app.disable_totp(0, TEST_DEFAULT_USER_PASSWORD, &code)
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::NotAvailable.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_secure_reset_should_not_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let token = Token::new(TokenKind::Reset, "test", "0", Duration::from_secs(60));
    //     let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     app.reset_with_token(&secure_token, "ABCDEF1234567891", "")
    //         .await
    //         .unwrap();
    // }

    // #[tokio::test]
    // async fn user_secure_reset_verification_token_kind_should_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let token = Token::new(
    //         TokenKind::Verification,
    //         "test",
    //         "0",
    //         Duration::from_secs(60),
    //     );

    //     let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     app.reset_with_token(&secure_token, "another password", "")
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_secure_reset_session_token_kind_should_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let token = Token::new(TokenKind::Session, "test", "0", Duration::from_secs(60));
    //     let secure_token = crypto::sign_jwt(&PRIVATE_KEY, token).unwrap();
    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     app.reset_with_token(&secure_token, "another password", "")
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::InvalidToken.to_string()))
    //         .unwrap_err();
    // }

    // #[tokio::test]
    // async fn user_reset_should_not_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     app.reset(0, "ABCDEF12345678901", "").await.unwrap();
    // }

    // #[tokio::test]
    // async fn user_reset_same_password_should_fail() {
    //     let secret_repo = SecretRepositoryMock {
    //         fn_find_by_user_and_kind: Some(
    //             |_: &SecretRepositoryMock, _: i32, _: SecretKind| -> Result<Secret> {
    //                 Err(Error::NotFound)
    //             },
    //         ),
    //         ..Default::default()
    //     };

    //     let mut app = new_user_application();
    //     app.secret_repo = Arc::new(secret_repo);

    //     app.reset(0, TEST_DEFAULT_USER_PASSWORD, "")
    //         .await
    //         .map_err(|err| assert_eq!(err.to_string(), Error::WrongCredentials.to_string()))
    //         .unwrap_err();
    // }
}
