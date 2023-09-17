use super::{
    domain::{MfaMethod, Otp},
    error::{Error, Result},
};
use crate::user::domain::{Email, User};
use crate::{on_error, secret::domain::Secret};
use async_trait::async_trait;
use libreauth::oath::TOTPBuilder;
use libreauth::{hash::HashFunction::Sha256, oath::TOTP};
use std::{collections::HashMap, time::Duration};

#[async_trait]
pub trait MfaService {
    /// Runs the corresponding mfa method in order to validate the one time password.
    async fn verify(&self, user: &User, otp: Option<&Otp>) -> Result<()>;
    /// Runs the corresponding mfa method in order to activate it for the given user.
    async fn enable(&self, user: &User, otp: Option<&Otp>) -> Result<()>;
    /// Runs the corresponding mfa method in order to deactivate it for the given user.
    async fn disable(&self, user: &User, otp: Option<&Otp>) -> Result<()>;
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
            .key(self.as_ref().as_bytes())
            .hash_function(Sha256)
            .finalize()
            .map_err(on_error!(Error, "genereting time-based one time password"))
    }
}

/// Implements the [MfaService].
pub struct MultiFactor {
    pub otp_secret_len: usize,
    pub otp_length: usize,
    pub otp_timeout: Duration,
    pub methods: HashMap<MfaMethod, Box<dyn MfaService + Sync + Send>>,
    pub default: Box<dyn MfaService + Sync + Send>,
}

impl MultiFactor {
    fn method(&self, method: MfaMethod) -> &Box<dyn MfaService + Sync + Send> {
        self.methods.get(&method).unwrap_or(&self.default)
    }
}

#[async_trait]
impl MfaService for MultiFactor {
    async fn verify(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
        let Some(method) = user.preferences.multi_factor else {
            return Ok(());
        };

        self.method(method).verify(user, otp).await
    }

    async fn enable(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
        let Some(method) = user.preferences.multi_factor else {
            return Ok(());
        };

        self.method(method).enable(user, otp).await
    }

    async fn disable(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
        let Some(method) = user.preferences.multi_factor else {
            return Ok(());
        };

        self.method(method).disable(user, otp).await
    }
}
