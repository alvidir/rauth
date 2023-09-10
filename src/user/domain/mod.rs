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

#[cfg(test)]
mod test {
    use crate::user::domain::{Credentials, Email, Password, PasswordHash, Salt, User};

    #[test]
    fn user_password_matches() {
        let email = Email::try_from("username@server.domain").unwrap();
        let password = Password::try_from("abcABC123&".to_string()).unwrap();
        let salt = Salt::with_length(128).unwrap();
        let hash = PasswordHash::with_salt(&password, &salt).unwrap();
        let credentials = Credentials::new(email, hash);
        let user = User::from(credentials);

        assert!(
            user.password_matches(&password).unwrap(),
            "comparing password with its own hash"
        );

        let fake_password = Password::try_from("abcABC1234&".to_string()).unwrap();
        assert!(
            !user.password_matches(&fake_password).unwrap(),
            "comparing password with wrong hash"
        );
    }
}
