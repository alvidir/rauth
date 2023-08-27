use crate::{
    crypto,
    result::{Error, Result},
};
use ::regex::Regex;
use argon2::{Algorithm, Argon2, Params, Version};
use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Represents an email with, or without, sufix.
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email(String);

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for Email {
    type Error = Error;

    /// Builds an [Email] from the given string if, and only if, the string matches the email's regex.
    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        value.to_string().try_into()
    }
}

impl TryFrom<String> for Email {
    type Error = Error;

    /// Builds an [Email] from the given string if, and only if, the string matches the email's regex.
    fn try_from(email: String) -> std::result::Result<Email, Error> {
        Self::REGEX
            .is_match(&email)
            .then_some(Self(email))
            .ok_or(Error::InvalidFormat)
    }
}

impl Email {
    const PATTERN: &str = r"^[a-zA-Z0-9+._-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,63}$";
    const DOMAIN_SEPARATOR: char = '@';
    const SUFIX_SEPARATOR: char = '+';

    pub const REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(Self::PATTERN).unwrap());

    /// Returns an email resulting from substracting the sufix from self, if any, otherwise [Option::None] is returned.
    pub fn actual_email(&self) -> Option<Self> {
        let email_parts: Vec<&str> = self
            .0
            .split(&[Self::SUFIX_SEPARATOR, Self::DOMAIN_SEPARATOR])
            .collect();

        (email_parts.len() == 3).then_some({
            Self([email_parts[0], email_parts[2]].join(&Self::DOMAIN_SEPARATOR.to_string()))
        })
    }

    /// Returns the username part from the email.
    pub fn username(&self) -> &str {
        self.0
            .split(&[Self::SUFIX_SEPARATOR, Self::DOMAIN_SEPARATOR])
            .into_iter()
            .next()
            .unwrap_or_default()
    }
}

/// Represents a password.
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Password {
    pub(super) hash: Vec<u8>,
    pub(super) salt: Vec<u8>,
}

impl TryFrom<&str> for Password {
    type Error = Error;

    fn try_from(raw: &str) -> std::result::Result<Self, Self::Error> {
        raw.to_string().try_into()
    }
}

impl TryFrom<String> for Password {
    type Error = Error;

    /// Builds a [Password] from the given string if, and only if, the string matches the
    /// password's regex.
    fn try_from(password: String) -> std::result::Result<Self, Self::Error> {
        if !Self::REGEX.is_match(&password) {
            return Err(Error::InvalidFormat);
        }

        let mut salt = [0_u8; 128];
        crypto::randomize(&mut salt);

        let mut hash = [0_u8; 128];
        Self::ARGON_CTX
            .hash_password_into(password.as_bytes(), &salt, &mut hash)
            .map(|_| Self {
                hash: hash.to_vec(),
                salt: salt.to_vec(),
            })
            .map_err(|_| Error::Unknown)
    }
}

impl PartialEq<str> for Password {
    fn eq(&self, other: &str) -> bool {
        Self::try_from(other.to_string())
            .map(|pwd| &pwd == self)
            .unwrap_or_default()
    }
}

impl Password {
    const PATTERN: &str = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$";

    pub const REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(Self::PATTERN).unwrap());

    const ARGON_CTX: Lazy<Argon2<'_>> =
        Lazy::new(|| Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default()));

    pub fn hash(&self) -> &[u8] {
        &self.hash
    }

    pub fn salt(&self) -> &[u8] {
        &self.salt
    }
}

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

/// Represents a signed up user
#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub credentials: Credentials,
}

impl From<Credentials> for User {
    fn from(credentials: Credentials) -> Self {
        Self { id: 0, credentials }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::result::Result;
    use crate::user::domain::{Credentials, Email, Password};

    #[test]
    fn email_from_str() {
        struct Test<'a> {
            name: &'a str,
            input: &'a str,
            output: Option<Email>,
            must_fail: bool,
        }

        vec![
            Test {
                name: "email without sufix",
                input: "username@server.domain",
                output: Some("username@server.domain".try_into().unwrap()),
                must_fail: false,
            },
            Test {
                name: "email with sufix",
                input: "username+sufix@server.domain",
                output: Some("username+sufix@server.domain".try_into().unwrap()),
                must_fail: false,
            },
            Test {
                name: "email with invalid characters",
                input: "username%@server.domain",
                output: None,
                must_fail: true,
            },
            Test {
                name: "email without usernamename",
                input: "@server.domain",
                output: None,
                must_fail: true,
            },
            Test {
                name: "email without servername",
                input: "username@.test",
                output: None,
                must_fail: true,
            },
            Test {
                name: "email without domain",
                input: "username@server",
                output: None,
                must_fail: true,
            },
            Test {
                name: "email with invalid domain",
                input: "username@server.d",
                output: None,
                must_fail: true,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let result: Result<Email> = test.input.try_into();
            assert_eq!(result.is_err(), test.must_fail, "{}", test.name);
            assert_eq!(result.ok(), test.output, "{}", test.name);
        })
    }

    #[test]
    fn actual_email_from_email() {
        struct Test<'a> {
            name: &'a str,
            input: Email,
            output: Option<Email>,
        }

        vec![
            Test {
                name: "email without sufix",
                input: Email("username@server.domain".to_string()),
                output: None,
            },
            Test {
                name: "email with sufix",
                input: Email("username+sufix@server.domain".to_string()),
                output: Some(Email("username@server.domain".to_string())),
            },
            Test {
                name: "email with empty sufix",
                input: Email("username+@server.domain".to_string()),
                output: Some(Email("username@server.domain".to_string())),
            },
        ]
        .into_iter()
        .for_each(|test| {
            assert_eq!(
                Email::try_from(test.input).unwrap().actual_email(),
                test.output,
                "{}",
                test.name
            );
        })
    }

    #[test]
    fn username_from_email() {
        assert_eq!(
            Email::try_from("username@server.domain")
                .unwrap()
                .username(),
            "username"
        )
    }

    #[test]
    fn password_from_str() {
        struct Test<'a> {
            name: &'a str,
            input: &'a str,
            output: Option<Password>,
            must_fail: bool,
        }

        vec![
            Test {
                name: "valid password",
                input: "abcABC123&",
                output: Some("abcABC123&".try_into().unwrap()),
                must_fail: false,
            },
            Test {
                name: "password without special characters",
                input: "abcABC123",
                output: None,
                must_fail: true,
            },
            Test {
                name: "password without uppercase characters",
                input: "abcabc123&",
                output: None,
                must_fail: true,
            },
            Test {
                name: "password without lowercase characters",
                input: "ABCABC123&",
                output: None,
                must_fail: true,
            },
            Test {
                name: "password with less than 8 characters",
                input: "aB1&",
                output: None,
                must_fail: true,
            },
            Test {
                name: "none base64 password",
                input: "abcABC123&",
                output: None,
                must_fail: true,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let result: Result<Password> = test.input.try_into();
            assert_eq!(result.is_err(), test.must_fail, "{}", test.name);
            assert_eq!(result.ok(), test.output, "{}", test.name);
        })
    }

    #[test]
    fn credentials_from_single_str() {
        let credentials: Credentials = Email::try_from("username@server.domain").unwrap().into();
        assert_eq!(
            credentials.email,
            Email("username@server.domain".to_string())
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
            Email("username@server.domain".to_string())
        );
        assert_eq!(credentials.password, Some("abcABC123&".try_into().unwrap()));
    }
}
