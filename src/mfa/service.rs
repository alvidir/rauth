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
    async fn enable_method(
        &self,
        method: MfaMethod,
        user: &User,
        otp: Option<&Otp>,
    ) -> Result<Secret>;
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

/// Implements the [MfaService].
pub struct MultiFactor<S, M, C> {
    pub secret_len: usize,
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

    async fn enable_method(
        &self,
        method: MfaMethod,
        user: &User,
        otp: Option<&Otp>,
    ) -> Result<Secret> {
        todo!()
    }

    async fn disable_method(
        &self,
        method: MfaMethod,
        user: &User,
        otp: Option<&Otp>,
    ) -> Result<()> {
        todo!()
    }
}

impl<S, M, C> MultiFactor<S, M, C>
where
    S: SecretRepository + Sync + Send,
{
    async fn run_tp_app_method(&self, user: &User, totp: Option<&Otp>) -> Result<()> {
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

impl<S, M, C> MultiFactor<S, M, C>
where
    M: MailService + Sync + Send,
    C: Cache + Sync + Send,
{
    async fn check_otp(&self, user: &User, actual_otp: Otp, otp: Option<&Otp>) -> Result<()> {
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

    async fn emit_otp(&self, user: &User) -> Result<()> {
        let otp = Otp::with_length(self.secret_len)?;
        let otp = self
            .cache
            .save(&user.id.to_string(), &otp, Some(self.otp_timeout))
            .await
            .map(|_| otp)
            .map_err(Error::from)?;

        self.mail_srv
            .send_otp_email(&user.credentials.email, &otp)?;

        Err(Error::Required)
    }

    async fn run_email_method(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
        match self.cache.find::<Otp>(&user.id.to_string()).await {
            Ok(actual_otp) => self.check_otp(user, actual_otp, otp).await,
            Err(err) if err.is_not_found() => self.emit_otp(user).await,
            Err(err) => Err(err.into()),
        }
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
