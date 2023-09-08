use super::{
    domain::{MfaMethod, Otp},
    error::{Error, Result},
};
use crate::secret::application::SecretRepository;
use crate::secret::domain::SecretKind;
use crate::user::domain::User;
use crate::{cache::Cache, user::domain::Email};
use crate::{on_error, secret::domain::Secret};
use async_trait::async_trait;
use libreauth::oath::TOTPBuilder;
use libreauth::{hash::HashFunction::Sha256, oath::TOTP};
use std::sync::Arc;

#[async_trait]
pub trait MfaService {
    async fn run_method(&self, method: MfaMethod, user: &User, otp: Option<Otp>) -> Result<()>;
}

pub trait Mailer {
    fn send_otp_email(&self, to: &Email, token: &Otp) -> Result<()>;
}

impl TryInto<TOTP> for Secret {
    type Error = Error;

    fn try_into(self) -> Result<TOTP> {
        TOTPBuilder::new()
            .key(self.data())
            .hash_function(Sha256)
            .finalize()
            .map_err(on_error!("genereting time-based one time password"))
    }
}

/// Implements the [MfaService] trait with all the methods defined by [MfaMethod].
pub struct MfaServiceImpl<S, M, C> {
    pub secret_len: usize,
    pub secret_repo: Arc<S>,
    pub mailer: Arc<M>,
    pub cache: Arc<C>,
}

#[async_trait]
impl<S, M, C> MfaService for MfaServiceImpl<S, M, C>
where
    S: SecretRepository + Sync + Send,
    M: Mailer + Sync + Send,
    C: Cache + Sync + Send,
{
    async fn run_method(&self, method: MfaMethod, user: &User, otp: Option<Otp>) -> Result<()> {
        match method {
            MfaMethod::TpApp => self.tp_app_totp_method(user, otp).await,
            MfaMethod::Email => self.email_otp_method(user, otp).await,
        }
    }
}

impl<S, M, C> MfaServiceImpl<S, M, C>
where
    S: SecretRepository + Sync + Send,
    M: Mailer + Sync + Send,
    C: Cache + Sync + Send,
{
    async fn email_otp_method(&self, _user: &User, _otp: Option<Otp>) -> Result<()> {
        todo!()
    }

    async fn tp_app_totp_method(&self, user: &User, totp: Option<Otp>) -> Result<()> {
        let totp = totp.ok_or(Error::Required)?;

        let actual_totp: TOTP = self
            .secret_repo
            .find_by_owner_and_kind(user.id, SecretKind::Totp)
            .await
            .map_err(Into::into)
            .and_then(TryInto::try_into)?;

        if actual_totp.is_valid(totp.as_ref()) {
            return Ok(());
        }

        Err(Error::Invalid)
    }
}
