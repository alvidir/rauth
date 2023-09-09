mod email;
pub use email::*;

mod password;
pub use password::*;

mod credentials;
pub use credentials::*;

use super::error::Result;
use crate::mfa::domain::MfaMethod;

/// Represents the preferences of a user.
#[derive(Debug, Default)]
pub struct Preferences {
    pub multi_factor: Option<MfaMethod>,
}

/// Represents a user.
#[derive(Debug, Default)]
pub struct User {
    pub id: i32,
    pub credentials: Credentials,
    pub preferences: Preferences,
}

impl From<Credentials> for User {
    fn from(credentials: Credentials) -> Self {
        Self {
            id: 0,
            credentials,
            ..Default::default()
        }
    }
}

impl User {
    /// Returns true if, and only if, the given password matches with the one from self.
    pub fn password_matches(&self, other: &Password) -> Result<bool> {
        self.credentials.password.matches(other)
    }
}
