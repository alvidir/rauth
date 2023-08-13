use std::marker::PhantomData;

use crate::metadata::domain::Metadata;
use crate::{
    regex,
    result::{Error, Result},
};
use serde::{Deserialize, Serialize};

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

    fn try_from(email: &str) -> Result<Self> {
        regex::match_regex(Self::REGEX, email).map_err(|err| {
            warn!(error = err.to_string(), "validating email format",);
            Error::InvalidFormat
        })?;

        Ok(Self(email.to_string()))
    }
}

impl Email {
    const REGEX: &str = r"^[a-zA-Z0-9+._-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,63}$";
    const DOMAIN_SEPARATOR: char = '@';
    const SUFIX_SEPARATOR: char = '+';

    /// Returns the email without the sufix, if any.
    pub fn actual_email(&self) -> Self {
        let email_parts: Vec<&str> = self
            .0
            .split(&[Self::SUFIX_SEPARATOR, Self::DOMAIN_SEPARATOR])
            .collect();

        if email_parts.len() != 3 {
            // the email has no sufix
            return self.clone();
        }

        Self([email_parts[0], email_parts[2]].join(&Self::DOMAIN_SEPARATOR.to_string()))
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

/// Raw marker. Determines the password has not been digested.
#[derive(Debug, Hash, PartialEq, Eq)]
struct Raw;

/// Opaque marker. Determines the password has been digested.
#[derive(Debug, Hash, PartialEq, Eq)]
struct Opaque;

/// Represents a password.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Password<State = Raw> {
    value: String,
    state: PhantomData<State>,
}

impl AsRef<str> for Password<Opaque> {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

impl From<&str> for Password<Opaque> {
    /// Converts an string containing the hash of a password into an instance of [Password].
    fn from(password: &str) -> Self {
        Self {
            value: password.to_string(),
            state: PhantomData,
        }
    }
}

impl TryFrom<&str> for Password<Raw> {
    type Error = Error;

    /// Converts an string containing a raw password into an instance of [Password].
    fn try_from(password: &str) -> Result<Self> {
        regex::match_regex(Self::REGEX, &password).map_err(|err| {
            warn!(error = err.to_string(), "validating raw password format",);
            Error::InvalidFormat
        })?;

        Ok(Self {
            value: password.to_string(),
            state: PhantomData,
        })
    }
}

impl Password<Raw> {
    pub const REGEX: &str = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$";

    /// Build a new instance of raw password baypassing the preconditions.
    #[cfg(test)]
    pub fn new(value: String) -> Self {
        Self {
            value,
            state: PhantomData,
        }
    }

    /// Given a sufix digests the password and returns it as an opaque password.
    pub fn opaque(mut self, sufix: &str) -> Password<Opaque> {
        let digest = sha256::digest(self.value);
        let sufixed = format!("{}{}", digest, sufix);
        let digest = sha256::digest(sufixed.as_bytes());

        Password::<Opaque> {
            value: digest,
            state: PhantomData,
        }
    }
}

/// Represents the credentials of a [User].
#[derive(Debug, Default, Hash, Serialize, Deserialize)]
pub struct Credentials {
    pub(super) email: Email,
    pub(super) password: Option<Password>,
}

impl TryFrom<&str> for Credentials {
    type Error = Error;

    fn try_from(email: &str) -> Result<Self> {
        email.try_into().map(|email| Self::new(email))
    }
}

impl TryFrom<(&str, &str)> for Credentials {
    type Error = Error;

    fn try_from((email, password): (&str, &str)) -> Result<Self> {
        let password = password.try_into()?;
        Self::try_from(email).map(|credentials| credentials.with_password(password))
    }
}

impl Credentials {
    pub fn new(email: Email) -> Self {
        Self {
            email,
            ..Default::default()
        }
    }

    pub fn with_password(mut self, password: Password) -> Self {
        self.password = Some(password);
        self
    }
}

/// Represents a signed up user
#[derive(Debug)]
pub struct User {
    pub(super) id: i32,
    pub(super) credentials: Credentials,
    pub(super) meta: Metadata,
}

impl From<Credentials> for User {
    fn from(credentials: Credentials) -> Self {
        Self {
            id: 0,
            credentials,
            meta: Metadata::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::user::domain::{Credentials, Email, Password};
    use crate::{base64, result::Result};
    use ::base64::Engine;

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
                output: Some(Email("username@server.domain".to_string())),
                must_fail: false,
            },
            Test {
                name: "email with sufix",
                input: "username+sufix@server.domain",
                output: Some(Email("username+sufix@server.domain".to_string())),
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
            assert_eq!(result.is_err(), test.must_fail);
            assert_eq!(result.ok(), test.output);
        })
    }

    #[test]
    fn actual_email_from_email() {
        struct Test<'a> {
            name: &'a str,
            input: Email,
            output: Email,
        }

        vec![
            Test {
                name: "email without sufix",
                input: Email("username@server.domain".to_string()),
                output: Email("username@server.domain".to_string()),
            },
            Test {
                name: "email with sufix",
                input: Email("username+sufix@server.domain".to_string()),
                output: Email("username@server.domain".to_string()),
            },
        ]
        .into_iter()
        .for_each(|test| {
            assert_eq!(Email::try_from(test.input).unwrap(), test.output);
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
            {
                let encoded_pwd = base64::B64_CUSTOM_ENGINE.encode("abcABC123&");
                Test {
                    name: "valid password",
                    input: &encoded_pwd,
                    output: Some(Password::new(encoded_pwd)),
                    must_fail: false,
                }
            },
            Test {
                name: "password without special characters",
                input: &base64::B64_CUSTOM_ENGINE.encode("abcABC123"),
                output: None,
                must_fail: true,
            },
            Test {
                name: "password without uppercase characters",
                input: &base64::B64_CUSTOM_ENGINE.encode("abcabc123&"),
                output: None,
                must_fail: true,
            },
            Test {
                name: "password without lowercase characters",
                input: &base64::B64_CUSTOM_ENGINE.encode("ABCABC123&"),
                output: None,
                must_fail: true,
            },
            Test {
                name: "password with less than 8 characters",
                input: &base64::B64_CUSTOM_ENGINE.encode("aB1&"),
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
            assert_eq!(result.is_err(), test.must_fail);
            assert_eq!(result.ok(), test.output);
        })
    }

    #[test]
    fn credentials_from_single_str() {
        let credentials: Credentials = "username@server.domain".try_into().unwrap();
        assert_eq!(
            credentials.email,
            Email("username@server.domain".to_string())
        );
        assert_eq!(credentials.password, None);
    }

    #[test]
    fn credentials_from_tuple_of_str() {
        let credentials: Credentials = ("username@server.domain", "abcABC123&").try_into().unwrap();
        assert_eq!(
            credentials.email,
            Email("username@server.domain".to_string())
        );
        assert_eq!(
            credentials.password,
            Some(Password::new("abcABC123&".to_string()))
        );
    }
}
