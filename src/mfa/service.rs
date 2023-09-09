use super::{
    domain::{MfaMethod, Otp},
    error::{Error, Result},
};
use crate::secret::application::SecretRepository;
use crate::secret::domain::SecretKind;
use crate::{
    cache::Cache,
    user::domain::{Email, User},
};
use crate::{on_error, secret::domain::Secret};
use async_trait::async_trait;
use libreauth::oath::TOTPBuilder;
use libreauth::{hash::HashFunction::Sha256, oath::TOTP};
use std::{sync::Arc, time::Duration};

/// Represents an executor of different strategies of multi-factor authentication.
#[async_trait]
pub trait MfaService {
    /// Runs the given mfa method in order to validate the one time password.
    async fn run_method(&self, method: MfaMethod, user: &User, otp: Option<&Otp>) -> Result<()>;
    /// Runs the given mfa method in order to activate it for the corresponding user.
    async fn enable_method(&self, method: MfaMethod, user: &User, otp: Option<&Otp>) -> Result<()>;
    /// Runs the given mfa method in order to deactivate it for the corresponding user.
    async fn disable_method(&self, method: MfaMethod, user: &User, otp: Option<&Otp>)
        -> Result<()>;
}

pub trait MailService {
    fn send_otp_email(&self, to: &Email, token: &Otp) -> Result<()>;
}

impl TryInto<TOTP> for Secret {
    type Error = Error;

    fn try_into(self) -> Result<TOTP> {
        TOTPBuilder::new()
            .key(self.data())
            .hash_function(Sha256)
            .finalize()
            .map_err(on_error!(Error, "genereting time-based one time password"))
    }
}

impl TryInto<TOTP> for Otp {
    type Error = Error;

    fn try_into(self) -> Result<TOTP> {
        TOTPBuilder::new()
            .key(self.as_ref())
            .hash_function(Sha256)
            .finalize()
            .map_err(on_error!(Error, "genereting time-based one time password"))
    }
}

/// Implements the [MfaService].
pub struct MultiFactor<S, M, C> {
    pub otp_secret_len: usize,
    pub otp_len: usize,
    pub otp_timeout: Duration,
    pub secret_repo: Arc<S>,
    pub mail_srv: Arc<M>,
    pub cache: Arc<C>,
}

#[async_trait]
impl<S, M, C> MfaService for MultiFactor<S, M, C>
where
    S: SecretRepository + Sync + Send,
    M: MailService + Sync + Send,
    C: Cache + Sync + Send,
{
    async fn run_method(&self, method: MfaMethod, user: &User, otp: Option<&Otp>) -> Result<()> {
        match method {
            MfaMethod::TpApp => self.run_tp_app_method(user, otp).await,
            MfaMethod::Email => self.run_email_method(user, otp).await,
        }
    }

    async fn enable_method(&self, method: MfaMethod, user: &User, otp: Option<&Otp>) -> Result<()> {
        match method {
            MfaMethod::TpApp => self.enable_tp_app_method(user, otp).await,
            MfaMethod::Email => self.enable_email_method(user, otp).await,
        }
    }

    async fn disable_method(
        &self,
        method: MfaMethod,
        user: &User,
        otp: Option<&Otp>,
    ) -> Result<()> {
        match method {
            MfaMethod::TpApp => self.disable_tp_app_method(user, otp).await,
            MfaMethod::Email => self.disable_email_method(user, otp).await,
        }
    }
}

impl<S, M, C> MultiFactor<S, M, C>
where
    C: Cache + Sync + Send,
{
    fn key(user: &User) -> String {
        [&user.id.to_string(), "mfa"].join("::")
    }

    async fn issue_otp(&self, user: &User, len: usize) -> Result<Otp> {
        let otp = Otp::with_length(len)?;
        let otp = self
            .cache
            .save(&Self::key(user), &otp, Some(self.otp_timeout))
            .await
            .map(|_| otp)
            .map_err(Error::from)?;

        Ok(otp)
    }
}

impl<S, M, C> MultiFactor<S, M, C>
where
    S: SecretRepository + Sync + Send,
    C: Cache + Sync + Send,
{
    async fn run_tp_app_method(&self, user: &User, totp: Option<&Otp>) -> Result<()> {
        let totp = totp.ok_or(Error::Required)?;

        let actual_totp: TOTP = self
            .secret_repo
            .find_by_owner_and_kind(user.id, SecretKind::Otp)
            .await
            .map_err(Into::into)
            .and_then(TryInto::try_into)?;

        if actual_totp.is_valid(totp.as_ref()) {
            return Ok(());
        }

        Err(Error::Invalid)
    }

    async fn verify_totp(&self, user: &User, secret: Otp, totp: Option<&Otp>) -> Result<()> {
        let totp = totp.ok_or(Error::Required)?;

        let actual_totp: TOTP = secret.try_into()?;
        if !actual_totp.is_valid(totp.as_ref()) {
            return Err(Error::Invalid);
        }

        let mut secret = Secret::new(SecretKind::Otp, user, totp.as_ref());
        self.secret_repo
            .create(&mut secret)
            .await
            .map_err(Into::into)
    }

    async fn enable_tp_app_method(&self, user: &User, totp: Option<&Otp>) -> Result<()> {
        match self.cache.find::<Otp>(&Self::key(user)).await {
            Ok(secret) => self.verify_totp(user, secret, totp).await,
            Err(err) if err.is_not_found() => {
                let otp = self.issue_otp(user, self.otp_secret_len).await?;
                Err(Error::Ack(AsRef::<str>::as_ref(&otp).to_string()))
            }
            Err(err) => Err(err.into()),
        }
    }

    async fn disable_tp_app_method(&self, user: &User, totp: Option<&Otp>) -> Result<()> {
        self.run_tp_app_method(user, totp).await?;

        let secret = self
            .secret_repo
            .find_by_owner_and_kind(user.id, SecretKind::Otp)
            .await?;

        self.secret_repo.delete(&secret).await.map_err(Into::into)
    }
}

impl<S, M, C> MultiFactor<S, M, C>
where
    M: MailService + Sync + Send,
    C: Cache + Sync + Send,
{
    fn otp_key(user: &User) -> String {
        [&user.id.to_string(), "otp"].join("::")
    }

    async fn otp_matches(&self, user: &User, actual_otp: Otp, otp: Option<&Otp>) -> Result<()> {
        let otp = otp.ok_or(Error::Required)?;
        if otp == &actual_otp {
            return self
                .cache
                .delete(&user.id.to_string())
                .await
                .map_err(Error::from);
        }

        Err(Error::Invalid)
    }

    async fn run_email_method(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
        match self.cache.find::<Otp>(&Self::otp_key(user)).await {
            Ok(actual_otp) => self.otp_matches(user, actual_otp, otp).await,
            Err(err) if err.is_not_found() => {
                let otp = self.issue_otp(user, self.otp_len).await?;
                self.mail_srv
                    .send_otp_email(&user.credentials.email, &otp)?;

                Err(Error::Required)
            }
            Err(err) => Err(err.into()),
        }
    }

    async fn enable_email_method(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
        self.run_email_method(user, otp).await
    }

    async fn disable_email_method(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
        self.run_email_method(user, otp).await
    }
}

// #[cfg(test)]
// pub mod tests {

//     #[test]
//     fn verify_totp_ok_should_not_fail() {
//         const SECRET: &[u8] = "hello world".as_bytes();

//         let code = generate_totp::<String>(SECRET).unwrap().generate();

//         assert_eq!(code.len(), 6);
//         assert!(totp_matches::<String>(SECRET, &code).is_ok());
//     }

//     #[test]
//     fn verify_totp_ko_should_not_fail() {
//         const SECRET: &[u8] = "hello world".as_bytes();
//         assert!(!totp_matches::<String>(SECRET, "tester").unwrap());
//     }
// }
