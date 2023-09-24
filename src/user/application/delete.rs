use super::{MailService, UserApplication, UserRepository};
use crate::cache::Cache;
use crate::multi_factor::domain::Otp;
use crate::multi_factor::service::MfaService;
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
    F: MfaService,
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
