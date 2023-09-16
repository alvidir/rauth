use super::{EventService, MailService, UserApplication, UserRepository};
use crate::cache::Cache;
use crate::mfa::domain::Otp;
use crate::mfa::service::MfaService;
use crate::on_error;
use crate::secret::application::SecretRepository;
use crate::token::domain::Token;
use crate::token::service::TokenService;
use crate::user::domain::Password;
use crate::user::error::{Error, Result};
use std::num::ParseIntError;

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

        let user_id = claims.payload().sub.parse().map_err(on_error!(
            ParseIntError as Error,
            "parsing token subject to user id"
        ))?;

        self.delete(user_id, password, otp).await?;
        self.token_srv.revoke(&claims).await.map_err(Into::into)
    }

    /// Given a valid user ID and passwords, performs the deletion of the corresponding user.
    #[instrument(skip(self, password, otp))]
    pub async fn delete(&self, user_id: i32, password: Password, otp: Option<Otp>) -> Result<()> {
        let user = self.user_repo.find(user_id).await?;

        if !user.password_matches(&password)? {
            return Err(Error::WrongCredentials);
        }

        self.multi_factor_srv.verify(&user, otp.as_ref()).await?;

        self.secret_repo.delete_by_owner(&user).await?;
        self.user_repo.delete(&user).await?;

        self.event_srv.emit_user_deleted(&user).await
    }
}
