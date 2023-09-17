use super::{
    domain::{MfaMethod, Otp},
    error::Result,
};
use crate::user::domain::User;
use async_trait::async_trait;
use std::collections::HashMap;

/// Represents all the features a mfa method/service must have.
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

#[cfg(test)]
pub(crate) mod test {
    use super::MfaService;
    use crate::mfa::error::{Error, Result};
    use crate::{mfa::domain::Otp, user::domain::User};
    use async_trait::async_trait;

    pub type VerifyFn = fn(&MfaServiceMock, user: &User, otp: Option<&Otp>) -> Result<()>;
    pub type EnableFn = fn(&MfaServiceMock, user: &User, otp: Option<&Otp>) -> Result<()>;
    pub type DisableFn = fn(&MfaServiceMock, user: &User, otp: Option<&Otp>) -> Result<()>;

    #[derive(Debug, Default)]
    pub struct MfaServiceMock {
        verify_fn: Option<VerifyFn>,
        enable_fn: Option<EnableFn>,
        disable_fn: Option<DisableFn>,
    }

    #[async_trait]
    impl MfaService for MfaServiceMock {
        async fn verify(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
            if let Some(verify_fn) = self.verify_fn {
                return verify_fn(self, user, otp);
            }

            Err(Error::Debug)
        }
        async fn enable(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
            if let Some(enable_fn) = self.enable_fn {
                return enable_fn(self, user, otp);
            }

            Err(Error::Debug)
        }
        async fn disable(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
            if let Some(disable_fn) = self.disable_fn {
                return disable_fn(self, user, otp);
            }

            Err(Error::Debug)
        }
    }
}