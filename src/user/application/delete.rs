use super::{MailService, UserApplication, UserRepository};
use crate::cache::Cache;
use crate::multi_factor::domain::Otp;
use crate::multi_factor::service::MultiFactorService;
use crate::on_error;
use crate::secret::service::SecretRepository;
use crate::token::domain::Token;
use crate::token::service::TokenService;
use crate::user::domain::{Password, UserID};
use crate::user::error::{Error, Result};
use std::str::FromStr;

impl<U, S, T, F, M, C> UserApplication<U, S, T, F, M, C>
where
    U: UserRepository,
    S: SecretRepository,
    T: TokenService,
    F: MultiFactorService,
    M: MailService,
    C: Cache,
{
    /// Given a valid session token and passwords, performs the deletion of the user.
    #[instrument(skip(self, password, otp))]
    pub async fn delete_with_token(
        &self,
        token: Token,
        password: Password,
        otp: Option<Otp>,
    ) -> Result<()> {
        let claims = self.token_srv.claims(token).await?;
        if !claims.payload().kind().is_session() {
            return Error::WrongToken.into();
        }

        let user_id = UserID::from_str(claims.payload().subject()).map_err(on_error!(
            uuid::Error as Error,
            "parsing token subject into user id"
        ))?;

        self.delete(user_id, password, otp).await?;
        self.token_srv.revoke(&claims).await.map_err(Into::into)
    }

    /// Given a valid user ID and passwords, performs the deletion of the corresponding user.
    #[instrument(skip(self, password, otp))]
    pub async fn delete(
        &self,
        user_id: UserID,
        password: Password,
        otp: Option<Otp>,
    ) -> Result<()> {
        let user = self.user_repo.find(user_id).await?;

        if !user.password_matches(&password)? {
            return Err(Error::WrongCredentials);
        }

        self.multi_factor_srv.verify(&user, otp.as_ref()).await?;

        self.secret_repo.delete_by_owner(&user).await?;
        self.user_repo.delete(&user).await
    }
}

#[cfg(test)]
mod test {
    use crate::{
        token::{
            domain::{Claims, Payload, Token, TokenKind},
            service::test::TokenServiceMock,
        },
        user::{
            application::test::{new_user_application, UserRepositoryMock},
            domain::{Credentials, Email, Password, PasswordHash, Preferences, Salt, User, UserID},
        },
    };
    use std::{str::FromStr, sync::Arc, time::Duration};

    #[tokio::test]
    async fn delete_must_not_fail() {
        let mut user_repo = UserRepositoryMock::default();
        user_repo.find_by_email_fn = Some(|email: &Email| {
            assert_eq!(email.as_ref(), "username@server.domain", "unexpected email");

            let password = Password::try_from("abcABC123&".to_string()).unwrap();
            let salt = Salt::with_length(32).unwrap();

            Ok(User {
                id: UserID::from_str("bca4ec1c-da63-4d73-bad5-a82fc9853828").unwrap(),
                credentials: Credentials {
                    email: email.clone(),
                    password: PasswordHash::with_salt(&password, &salt).unwrap(),
                },
                preferences: Preferences::default(),
            })
        });

        let mut token_srv = TokenServiceMock::default();
        token_srv.claims_fn = Some(|token: Token| {
            assert_eq!(token.as_ref(), "abc.abc.abc", "unexpected token");
            Ok(Claims {
                token,
                payload: Payload::new(TokenKind::Session, Duration::from_secs(60))
                    .with_subject("signup"),
            })
        });

        let mut user_app = new_user_application();
        user_app.user_repo = Arc::new(user_repo);
    }
}
