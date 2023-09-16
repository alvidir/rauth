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
