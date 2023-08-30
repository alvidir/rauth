use crate::{
    base64,
    crypto::{randomize_with, URL_SAFE},
    user::result::{Error, Result},
};
use ::regex::Regex;
use argon2::{Algorithm, Argon2, Params, Version};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// Represents a password.
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Password {
    hash: String,
    salt: String,
}

impl TryFrom<&str> for Password {
    type Error = Error;

    fn try_from(raw: &str) -> Result<Self> {
        raw.to_string().try_into()
    }
}

impl TryFrom<String> for Password {
    type Error = Error;

    /// Builds a [Password] from the given string if, and only if, the string matches the
    /// password's regex.
    fn try_from(password: String) -> Result<Self> {
        if !Self::REGEX.is_match(&password) {
            return Error::NotAPassword.into();
        }

        let mut salt = [0_u8; 128];
        randomize_with(URL_SAFE, &mut salt);

        let mut hash = [0_u8; 128];
        Self::ARGON
            .hash_password_into(password.as_bytes(), &salt, &mut hash)
            .map(|_| Self {
                hash: base64::encode(&hash),
                salt: base64::encode(&salt),
            })
            .map_err(|error| {
                error!(error = error.to_string(), "salting and hashing password");
                Error::Unknown
            })
    }
}

impl PartialEq<str> for Password {
    fn eq(&self, other: &str) -> bool {
        let Ok(hash) = base64::decode(self.hash()) else {
            return false;
        };

        let Ok(salt) = base64::decode(self.salt()) else {
            return false;
        };

        let mut subject = [0_u8; 128];
        Self::ARGON
            .hash_password_into(other.as_bytes(), &salt, &mut subject)
            .map(|_| subject == hash.as_slice())
            .unwrap_or_default()
    }
}

impl Password {
    const PATTERN: &str = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$";

    pub const REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(Self::PATTERN).unwrap());

    const ARGON: Lazy<Argon2<'_>> =
        Lazy::new(|| Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default()));

    pub fn new(hash: String, salt: String) -> Self {
        Self { hash, salt }
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn salt(&self) -> &str {
        &self.salt
    }
}

#[cfg(test)]
mod tests {
    use super::Password;
    use crate::user::result::Result;

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
}
