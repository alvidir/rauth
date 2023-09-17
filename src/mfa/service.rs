use super::{
    domain::{MfaMethod, Otp},
    error::Result,
};
use crate::user::domain::User;
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait MfaService {
    /// Runs the corresponding mfa method in order to validate the one time password.
    async fn verify(&self, user: &User, otp: Option<&Otp>) -> Result<()>;
    /// Runs the corresponding mfa method in order to activate it for the given user.
    async fn enable(&self, user: &User, otp: Option<&Otp>) -> Result<()>;
    /// Runs the corresponding mfa method in order to deactivate it for the given user.
    async fn disable(&self, user: &User, otp: Option<&Otp>) -> Result<()>;
}

/// Implements the [MfaService] as a mfa method router.
pub struct MultiFactor {
    pub methods: HashMap<MfaMethod, Box<dyn MfaService + Sync + Send>>,
    pub default: Box<dyn MfaService + Sync + Send>,
}

#[async_trait]
impl MfaService for MultiFactor {
    async fn verify(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
        let Some(method) = &user.preferences.multi_factor else {
            return Ok(());
        };

        self.method(method).verify(user, otp).await
    }

    async fn enable(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
        let Some(method) = &user.preferences.multi_factor else {
            return Ok(());
        };

        self.method(method).enable(user, otp).await
    }

    async fn disable(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
        let Some(method) = &user.preferences.multi_factor else {
            return Ok(());
        };

        self.method(method).disable(user, otp).await
    }
}

impl MultiFactor {
    fn method(&self, method: &MfaMethod) -> &Box<dyn MfaService + Sync + Send> {
        self.methods.get(method).unwrap_or(&self.default)
    }
}
