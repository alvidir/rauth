use std::num::ParseIntError;

use super::{EventBus, MailService, UserApplication, UserRepository};
use crate::cache::Cache;
use crate::mfa::domain::{MfaMethod, Otp};
use crate::mfa::service::MfaService;
use crate::on_error;
use crate::secret::application::SecretRepository;
use crate::secret::domain::Secret;
use crate::token::domain::Token;
use crate::token::service::TokenService;
use crate::user::domain::Password;
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
    #[instrument(skip(self, password, otp))]
    pub async fn enable_mfa_with_token(
        &self,
        token: Token,
        method: MfaMethod,
        password: Password,
        otp: Option<Otp>,
    ) -> Result<Secret> {
        let claims = self.token_srv.claims(token).await?;
        if !claims.payload().kind().is_session() {
            return Err(Error::WrongToken);
        }

        let user_id = claims
            .payload()
            .subject()
            .parse()
            .map_err(on_error!(ParseIntError as Error, "parsing str to i32"))?;

        self.enable_mfa(user_id, method, password, otp).await
    }

    #[instrument(skip(self, password, otp))]
    pub async fn enable_mfa(
        &self,
        user_id: i32,
        method: MfaMethod,
        password: Password,
        otp: Option<Otp>,
    ) -> Result<Secret> {
        let user = self.user_repo.find(user_id).await?;

        if !user.password_matches(&password)? {
            return Err(Error::WrongCredentials);
        }

        self.multi_factor_srv
            .enable_method(method, &user, otp.as_ref())
            .await
            .map_err(Into::into)

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
    }

    #[instrument(skip(self, password, otp))]
    pub async fn disable_mfa_with_token(
        &self,
        token: Token,
        method: MfaMethod,
        password: Password,
        otp: Option<Otp>,
    ) -> Result<()> {
        let claims = self.token_srv.claims(token).await?;
        if !claims.payload().kind().is_session() {
            return Err(Error::WrongToken);
        }

        let user_id = claims
            .payload()
            .subject()
            .parse()
            .map_err(on_error!(ParseIntError as Error, "parsing str to i32"))?;

        self.disable_mfa(user_id, method, password, otp).await
    }

    #[instrument(skip(self, password, otp))]
    pub async fn disable_mfa(
        &self,
        user_id: i32,
        method: MfaMethod,
        password: Password,
        otp: Option<Otp>,
    ) -> Result<()> {
        let user = self.user_repo.find(user_id).await?;

        if !user.password_matches(&password)? {
            return Err(Error::WrongCredentials);
        }

        self.multi_factor(&user, otp.as_ref()).await?;

        self.multi_factor_srv
            .disable_method(method, &user, otp.as_ref())
            .await
            .map_err(Into::into)

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

        // Err(Error::NotAvailable)
    }
}
