use super::error::{Error, Result};
use crate::user::domain::Email;

/// Represents the identity of any user.
#[derive(Debug)]
pub enum Identity {
    Email(Email),
    Nick(String),
}

impl TryFrom<String> for Identity {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        match Email::try_from(value.as_str()) {
            Ok(email) => Ok(Identity::Email(email)),
            Err(error) if error.is_not_an_email() => Ok(Identity::Nick(value)),
            Err(error) => Err(error.into()),
        }
    }
}
