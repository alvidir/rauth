use super::{MailService, UserApplication, UserRepository};
use crate::cache::Cache;
use crate::multi_factor::domain::{MultiFactorMethod, Otp};
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
    #[instrument(skip(self, password, otp))]
    pub async fn enable_multi_factor_with_token(
        &self,
        token: Token,
        method: MultiFactorMethod,
        password: Password,
        otp: Option<Otp>,
    ) -> Result<()> {
        let claims = self.token_srv.claims(token).await?;
        if !claims.payload().kind().is_session() {
            return Err(Error::WrongToken);
        }

        let user_id = UserID::from_str(claims.payload().subject()).map_err(on_error!(
            uuid::Error as Error,
            "parsing token subject into user id"
        ))?;

        self.enable_multi_factor(user_id, method, password, otp)
            .await
    }

    #[instrument(skip(self, password, otp))]
    pub async fn enable_multi_factor(
        &self,
        user_id: UserID,
        method: MultiFactorMethod,
        password: Password,
        otp: Option<Otp>,
    ) -> Result<()> {
        let mut user = self.user_repo.find(user_id).await?;

        if !user.password_matches(&password)? {
            return Err(Error::WrongCredentials);
        }

        user.preferences.multi_factor = Some(method);

        self.multi_factor_srv
            .enable(&user, otp.as_ref())
            .await
            .map_err(Error::from)?;

        self.user_repo.save(&user).await.map_err(Into::into)
    }

    #[instrument(skip(self, password, otp))]
    pub async fn disable_multi_factor_with_token(
        &self,
        token: Token,
        method: MultiFactorMethod,
        password: Password,
        otp: Option<Otp>,
    ) -> Result<()> {
        let claims = self.token_srv.claims(token).await?;
        if !claims.payload().kind().is_session() {
            return Err(Error::WrongToken);
        }

        let user_id = UserID::from_str(claims.payload().subject()).map_err(on_error!(
            uuid::Error as Error,
            "parsing token subject into user id"
        ))?;

        self.disable_multi_factor(user_id, method, password, otp)
            .await
    }

    #[instrument(skip(self, password, otp))]
    pub async fn disable_multi_factor(
        &self,
        user_id: UserID,
        method: MultiFactorMethod,
        password: Password,
        otp: Option<Otp>,
    ) -> Result<()> {
        let mut user = self.user_repo.find(user_id).await?;

        if !user.password_matches(&password)? {
            return Err(Error::WrongCredentials);
        }

        self.multi_factor_srv.verify(&user, otp.as_ref()).await?;

        self.multi_factor_srv
            .disable(&user, otp.as_ref())
            .await
            .map_err(Error::from)?;

        user.preferences.multi_factor = None;
        self.user_repo.save(&user).await.map_err(Into::into)
    }
}