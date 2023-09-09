use std::num::ParseIntError;

use super::{EventBus, MailService, UserApplication, UserRepository};
use crate::cache::Cache;
use crate::mfa::domain::Otp;
use crate::mfa::service::MfaService;
use crate::on_error;
use crate::secret::application::SecretRepository;
use crate::token::domain::{Token, TokenKind};
use crate::token::service::TokenService;
use crate::user::domain::{Email, Password, PasswordHash};
use crate::user::error::{Error, Result};

impl<U, S, T, F, M, B, C> UserApplication<U, S, T, F, M, B, C>
where
    U: UserRepository,
    S: SecretRepository,
    T: TokenService,
    F: MfaService,
    M: MailService,
    B: EventBus,
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

        // reboke token as soon as possible
        self.token_srv.revoke(&claims).await.map_err(Error::from)?;

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
        self.multi_factor(&user, otp).await?;

        let new_password = PasswordHash::try_from(new_password)?;
        if user.credentials.password == new_password {
            return Err(Error::WrongCredentials);
        }

        user.credentials.password = new_password;
        self.user_repo.save(&user).await
    }
}
