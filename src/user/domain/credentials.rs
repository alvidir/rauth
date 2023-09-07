use super::{Email, PasswordHash};
use serde::{Deserialize, Serialize};

/// Represents the credentials of a [User].
#[derive(Debug, Default, Hash, Serialize, Deserialize)]
pub struct Credentials {
    pub email: Email,
    pub password: PasswordHash,
}

impl Credentials {
    pub fn new(email: Email, password: PasswordHash) -> Self {
        Credentials { email, password }
    }
}

#[cfg(test)]
mod tests {
    use super::Credentials;
    use crate::user::domain::{Email, Password};

    #[test]
    fn new_credentials() {
        let credentials = Credentials::new(
            Email::try_from("username@server.domain").unwrap(),
            Password::try_from("abcABC123&".to_string())
                .and_then(TryInto::try_into)
                .unwrap(),
        );

        assert_eq!(
            credentials.email,
            "username@server.domain".to_string().try_into().unwrap()
        );

        assert!(credentials
            .password
            .matches("abcABC123&".to_string().try_into().unwrap())
            .unwrap());
    }
}
