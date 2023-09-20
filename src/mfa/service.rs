use super::{
    domain::{MfaMethod, Otp},
    error::{Error, Result},
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
#[derive(Default)]
pub struct MultiFactor {
    pub methods: HashMap<MfaMethod, Box<dyn MfaService + Sync + Send>>,
}

#[async_trait]
impl MfaService for MultiFactor {
    async fn verify(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
        let Some(method) = &user.preferences.multi_factor else {
            return Ok(());
        };

        self.methods
            .get(method)
            .ok_or(Error::NotFound)?
            .verify(user, otp)
            .await
    }

    async fn enable(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
        let Some(method) = &user.preferences.multi_factor else {
            return Ok(());
        };

        self.methods
            .get(method)
            .ok_or(Error::NotFound)?
            .enable(user, otp)
            .await
    }

    async fn disable(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
        let Some(method) = &user.preferences.multi_factor else {
            return Ok(());
        };

        self.methods
            .get(method)
            .ok_or(Error::NotFound)?
            .disable(user, otp)
            .await
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::MfaService;
    use crate::mfa::error::{Error, Result};
    use crate::{mfa::domain::Otp, user::domain::User};
    use async_trait::async_trait;

    pub type VerifyFn = fn(user: &User, otp: Option<&Otp>) -> Result<()>;
    pub type EnableFn = fn(user: &User, otp: Option<&Otp>) -> Result<()>;
    pub type DisableFn = fn(user: &User, otp: Option<&Otp>) -> Result<()>;

    #[derive(Debug, Default)]
    pub struct MfaServiceMock {
        pub verify_fn: Option<VerifyFn>,
        pub enable_fn: Option<EnableFn>,
        pub disable_fn: Option<DisableFn>,
    }

    #[async_trait]
    impl MfaService for MfaServiceMock {
        async fn verify(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
            if let Some(verify_fn) = self.verify_fn {
                return verify_fn(user, otp);
            }

            Err(Error::Debug)
        }
        async fn enable(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
            if let Some(enable_fn) = self.enable_fn {
                return enable_fn(user, otp);
            }

            Err(Error::Debug)
        }
        async fn disable(&self, user: &User, otp: Option<&Otp>) -> Result<()> {
            if let Some(disable_fn) = self.disable_fn {
                return disable_fn(user, otp);
            }

            Err(Error::Debug)
        }
    }
}
