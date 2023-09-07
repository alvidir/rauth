use super::{
    domain::Mfa,
    error::{Error, Result},
    strategy::{MfaAppMethod, MfaEmailMethod},
};
use crate::cache::Cache;
use crate::crypto;
use crate::secret::application::SecretRepository;
use crate::secret::domain::SecretKind;
use crate::user::domain::Email;
use crate::user::domain::Otp;
use crate::user::domain::User;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait MfaMethod {
    async fn execute(&self, user: &User, totp: Option<Otp>) -> Result<()>;
}

pub trait Mailer {
    fn send_otp_email(&self, to: &Email, token: &Otp) -> Result<()>;
}

pub struct MfaService<S, M, C> {
    pub secret_repo: Arc<S>,
    pub mailer: Arc<M>,
    pub cache: Arc<C>,
}

impl<S, M, C> MfaService<S, M, C>
where
    S: SecretRepository + Sync + Send,
    M: Mailer + Sync + Send,
    C: Cache + Sync + Send,
{
    pub fn mfa_method(&self, mfa: Mfa) -> Box<&dyn MfaMethod> {
        match mfa {
            Mfa::App => Box::new(&MfaAppMethod {
                secret_repo: self.secret_repo.clone(),
            }),
            Mfa::Email => Box::new(&MfaEmailMethod {
                mailer: self.mailer.clone(),
                cache: self.cache.clone(),
            }),
        }
    }
}
