use std::num::ParseIntError;

use super::{EventService, MailService, UserApplication, UserRepository};
use crate::cache::Cache;
use crate::mfa::domain::Otp;
use crate::mfa::service::MfaService;
use crate::on_error;
use crate::secret::application::SecretRepository;
use crate::token::domain::{Token, TokenKind};
use crate::token::service::TokenService;
use crate::user::domain::{Email, Password, PasswordHash, Salt};
use crate::user::error::{Error, Result};

impl<U, S, T, F, M, B, C> UserApplication<U, S, T, F, M, B, C>
where
    U: UserRepository,
    S: SecretRepository,
    T: TokenService,
    F: MfaService,
    M: MailService,
    B: EventService,
    C: Cache,
{
    /// Sends an email with the token to be passed as parameter to the reset_credentials_with_token method.
    #[instrument(skip(self))]
    pub async fn verify_credentials_reset(&self, email: Email) -> Result<()> {
        let user = self.user_repo.find_by_email(&email).await?;

        let claims = self
            .token_srv
            .issue(TokenKind::Reset, &user.id.to_string())
            .await?;

        self.mail_srv
            .send_credentials_reset_email(&email, claims.token())?;

        Ok(())
    }

    #[instrument(skip(self, new_password, otp))]
    pub async fn reset_credentials_with_token(
        &self,
        token: Token,
        new_password: Password,
        otp: Option<Otp>,
    ) -> Result<()> {
        let claims = self.token_srv.claims(token).await?;
        if !claims.payload().kind().is_reset() {
            return Err(Error::WrongToken);
        }

        let user_id = claims.payload().subject().parse().map_err(on_error!(
            ParseIntError as Error,
            "parsing token subject into user id"
        ))?;

        self.reset_credentials(user_id, new_password, otp).await
    }

    #[instrument(skip(self, new_password, otp))]
    pub async fn reset_credentials(
        &self,
        user_id: i32,
        new_password: Password,
        otp: Option<Otp>,
    ) -> Result<()> {
        let mut user = self.user_repo.find(user_id).await?;
        self.multi_factor_srv.verify(&user, otp.as_ref()).await?;

        if user.password_matches(&new_password)? {
            // is the same password, nothing have to be done.
            return Ok(());
        }

        let salt = Salt::with_length(self.hash_length)?;
        user.credentials.password = PasswordHash::with_salt(&new_password, &salt)?;
        self.user_repo.save(&user).await
    }
}

#[cfg(test)]
mod test {
    use crate::{
        mfa::{domain::Otp, service::test::MfaServiceMock},
        token::{
            domain::{Claims, Payload, Token, TokenKind},
            service::test::TokenServiceMock,
        },
        user::{
            application::test::{new_user_application, MailServiceMock, UserRepositoryMock},
            domain::{Credentials, Email, Password, PasswordHash, Preferences, Salt, User},
            error::Error,
        },
    };
    use std::{sync::Arc, time::Duration};

    #[tokio::test]
    async fn verify_credentials_reset_must_not_fail() {
        let mut user_repo = UserRepositoryMock::default();
        user_repo.find_by_email_fn = Some(|email: &Email| {
            assert_eq!(email.as_ref(), "username@server.domain", "unexpected email");

            let password = Password::try_from("abcABC123&".to_string()).unwrap();
            let salt = Salt::with_length(32).unwrap();

            Ok(User {
                id: 999,
                credentials: Credentials {
                    email: email.clone(),
                    password: PasswordHash::with_salt(&password, &salt).unwrap(),
                },
                preferences: Preferences::default(),
            })
        });

        let mut token_srv = TokenServiceMock::default();
        token_srv.issue_fn = Some(|kind: TokenKind, sub: &str| {
            assert_eq!(kind, TokenKind::Reset, "unexpected token kind");
            assert_eq!(sub, "999", "unexpected token subject");

            Ok(Claims {
                token: "abc.abc.abc".to_string().try_into().unwrap(),
                payload: Payload::new(kind, Duration::from_secs(60)).with_subject(sub),
            })
        });

        let mut mail_srv = MailServiceMock::default();
        mail_srv.send_credentials_reset_email_fn = Some(|email: &Email, token: &Token| {
            assert_eq!(email.as_ref(), "username@server.domain", "unexpected email");
            assert_eq!(token.as_ref(), "abc.abc.abc", "unexpected token");

            Ok(())
        });

        let mut user_app = new_user_application();
        user_app.user_repo = Arc::new(user_repo);
        user_app.token_srv = Arc::new(token_srv);
        user_app.mail_srv = Arc::new(mail_srv);

        let email = Email::try_from("username@server.domain").unwrap();
        user_app.verify_credentials_reset(email).await.unwrap();
    }

    #[tokio::test]
    async fn verify_credentials_reset_when_user_does_not_exists() {
        let mut user_repo = UserRepositoryMock::default();
        user_repo.find_by_email_fn = Some(|_: &Email| Err(Error::NotFound));

        let mut mail_srv = MailServiceMock::default();
        mail_srv.send_credentials_reset_email_fn = Some(|_: &Email, _: &Token| {
            assert!(false, "unexpected execution");
            Err(Error::Debug)
        });

        let mut user_app = new_user_application();
        user_app.user_repo = Arc::new(user_repo);
        user_app.mail_srv = Arc::new(mail_srv);

        let email = Email::try_from("username@server.domain").unwrap();
        let result = user_app.verify_credentials_reset(email).await;

        assert!(
            matches!(result, Err(Error::NotFound)),
            "got result = {:?}, want error = {}",
            result,
            Error::NotFound
        )
    }

    #[tokio::test]
    async fn reset_credentials_must_not_fail() {
        let mut user_repo = UserRepositoryMock::default();
        user_repo.find_fn = Some(|user_id: i32| {
            assert_eq!(user_id, 999, "unexpected user id");

            let password = Password::try_from("abcABC123&".to_string()).unwrap();
            let salt = Salt::with_length(32).unwrap();

            Ok(User {
                id: 999,
                credentials: Credentials {
                    email: "username@server.domain".try_into().unwrap(),
                    password: PasswordHash::with_salt(&password, &salt).unwrap(),
                },
                preferences: Preferences::default(),
            })
        });

        user_repo.save_fn = Some(|user: &User| {
            assert_eq!(user.id, 999, "unexpected user id");
            Ok(())
        });

        let mut multi_factor_srv = MfaServiceMock::default();
        multi_factor_srv.verify_fn = Some(|user: &User, otp: Option<&Otp>| {
            assert_eq!(user.id, 999, "unexpected user id");
            assert_eq!(otp, None, "unexpected otp");
            Ok(())
        });

        let mut user_app = new_user_application();
        user_app.hash_length = 32;
        user_app.multi_factor_srv = Arc::new(multi_factor_srv);
        user_app.user_repo = Arc::new(user_repo);

        let new_password = Password::try_from("abcABC1234&".to_string()).unwrap();

        user_app
            .reset_credentials(999, new_password, None)
            .await
            .unwrap();
    }

    // #[tokio::test]
    // async fn reset_credentials_with_token_must_not_fail() {
    //     let mut token_srv = TokenServiceMock::default();
    //     token_srv.claims_fn = Some(|token: Token| {
    //         assert_eq!(token.as_ref(), "abc.abc.abc", "unexpected token");

    //         Ok(Claims {
    //             token,
    //             payload: Payload::new(TokenKind::Reset, Duration::from_secs(60))
    //                 .with_subject("reset"),
    //         })
    //     });

    //     let mut user_app = new_user_application();
    //     user_app.token_srv = Arc::new(token_srv);

    //     let token: Token = "abc.abc.abc".to_string().try_into().unwrap();
    //     let password = Password::try_from("abcABC123&".to_string()).unwrap();

    //     user_app
    //         .reset_credentials_with_token(token, password, None)
    //         .await
    //         .unwrap();
    // }
}
