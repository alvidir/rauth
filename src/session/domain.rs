use super::error::{Error, Result};
use crate::user::domain::Email;

/// Represents the identity of any user.
pub enum Identity {
    Email(Email),
    Nick(String),
}

impl TryFrom<String> for Identity {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Ok(match Email::try_from(value) {
            Ok(email) => Identity::Email(email),
            Err(error) if error.is_not_an_email() => Identity::Nick(value),
            other => other.map_err(Error::from),
        })
    }
}
