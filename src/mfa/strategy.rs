use super::{
    error::{Error, Result},
    service::{Mailer, MfaMethod},
};
use crate::{
    cache::Cache,
    crypto,
    secret::{application::SecretRepository, domain::SecretKind},
    user::domain::{Otp, User},
};
use async_trait::async_trait;
use std::sync::Arc;

pub struct MfaAppMethod<S> {
    secret_repo: Arc<S>,
}

#[async_trait]
impl<S> MfaMethod for MfaAppMethod<S>
where
    S: SecretRepository + Sync + Send,
{
    async fn execute(&self, user: &User, totp: Option<Otp>) -> Result<()> {
        let totp = totp.ok_or(Error::Required)?;

        let secret = self
            .secret_repo
            .find_by_owner_and_kind(user.id, SecretKind::Totp)
            .await?;

        if crypto::totp_matches(secret.data(), totp.as_ref())? {
            return Ok(());
        }

        Err(Error::Invalid)
    }
}

pub struct MfaEmailMethod<M, C> {
    mailer: Arc<M>,
    cache: Arc<C>,
}

#[async_trait]
impl<M, C> MfaMethod for MfaEmailMethod<M, C>
where
    M: Mailer + Sync + Send,
    C: Cache + Sync + Send,
{
    async fn execute(&self, user: &User, otp: Option<Otp>) -> Result<()> {
        todo!()
    }
}
