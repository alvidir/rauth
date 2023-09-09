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
use std::sync::Arc;

/// Represents an executor of different strategies of multi-factor authentication.
#[async_trait]
pub trait MfaService {
    /// Executes the given [MfaMethod].
    async fn execute(&self, method: MfaMethod, user: &User, otp: Option<Otp>) -> Result<()>;
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
    pub secret_repo: Arc<S>,
    pub mailer: Arc<M>,
    pub cache: Arc<C>,
}

#[async_trait]
impl<S, M, C> MfaService for MultiFactor<S, M, C>
where
    S: SecretRepository + Sync + Send,
    M: MailService + Sync + Send,
    C: Cache + Sync + Send,
{
    async fn execute(&self, method: MfaMethod, user: &User, otp: Option<Otp>) -> Result<()> {
        match method {
            MfaMethod::TpApp => self.tp_app_totp_method(user, otp).await,
            MfaMethod::Email => self.email_otp_method(user, otp).await,
        }
    }
}

impl<S, M, C> MultiFactor<S, M, C>
where
    S: SecretRepository + Sync + Send,
    M: MailService + Sync + Send,
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
