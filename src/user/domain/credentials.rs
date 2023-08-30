use super::{Email, Password};
use serde::{Deserialize, Serialize};

/// Represents the credentials of a [User].
#[derive(Debug, Default, Hash, Serialize, Deserialize)]
pub struct Credentials {
    pub email: Email,
    pub password: Option<Password>,
}

impl From<Email> for Credentials {
    fn from(email: Email) -> Self {
        Self {
            email,
            ..Default::default()
        }
    }
}

impl From<(Email, Password)> for Credentials {
    fn from((email, password): (Email, Password)) -> Self {
        Credentials::from(email).with_password(password)
    }
}

impl Credentials {
    pub fn with_password(mut self, password: Password) -> Self {
        self.password = Some(password);
        self
    }

    pub fn set_password(&mut self, password: Option<Password>) {
        self.password = password;
    }
}

#[cfg(test)]
mod tests {
    use super::Credentials;
    use crate::user::domain::{Email, Password};

    #[test]
    fn credentials_from_single_str() {
        let credentials: Credentials = Email::try_from("username@server.domain").unwrap().into();
        assert_eq!(
            credentials.email,
            Email::new("username@server.domain".to_string())
        );
        assert_eq!(credentials.password, None);
    }

    #[test]
    fn credentials_from_tuple_of_str() {
        let credentials: Credentials = (
            Email::try_from("username@server.domain").unwrap(),
            Password::try_from("abcABC123&").unwrap(),
        )
            .into();
        assert_eq!(
            credentials.email,
            Email::new("username@server.domain".to_string())
        );
        assert_eq!(credentials.password, Some("abcABC123&".try_into().unwrap()));
    }
}
