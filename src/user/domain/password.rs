use std::array::TryFromSliceError;

use crate::{
    crypto, on_error,
    user::error::{Error, Result},
};
use ::regex::Regex;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const HASH_LEN: usize = 128;

/// Represents the hash and salt of a [Password].
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasswordHash {
    pub(crate) hash: String,
    pub(crate) salt: String,
}

impl TryFrom<String> for PasswordHash {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Password::try_from(value).and_then(TryInto::try_into)
    }
}

impl TryFrom<Password> for PasswordHash {
    type Error = Error;

    fn try_from(password: Password) -> Result<Self> {
        let mut salt = [0_u8; HASH_LEN];
        crypto::randomize(&mut salt);

        Self::with_salt(password, &salt)
    }
}

impl AsRef<str> for PasswordHash {
    fn as_ref(&self) -> &str {
        &self.hash
    }
}

impl PasswordHash {
    /// Builds a new password from the given value and salt
    pub fn with_salt(password: Password, salt: &[u8]) -> Result<Self> {
        let salt: [u8; HASH_LEN] = salt.try_into().map_err(on_error!(
            TryFromSliceError as Error,
            "converting into sized array"
        ))?;

        crypto::salt(password.as_ref(), &salt)
            .map(|salted| Self {
                hash: crypto::encode_b64(&salted),
                salt: crypto::encode_b64(&salt),
            })
            .map_err(Into::into)
    }

    /// Returns true if, and only if, the given password matches with self.
    pub fn matches(&self, other: Password) -> Result<bool> {
        let salt = crypto::decode_b64::<Error>(self.salt())?;
        PasswordHash::with_salt(other, &salt).map(|subject| &subject == self)
    }

    /// Returns the salt of the hash.
    pub fn salt(&self) -> &str {
        &self.salt
    }
}

/// Represents a password.
pub struct Password(String);

impl AsRef<[u8]> for Password {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl TryFrom<String> for Password {
    type Error = Error;

    /// Builds a [Password] from the given string if, and only if, the string matches the
    /// password's regex.
    fn try_from(password: String) -> Result<Self> {
        Self::REGEX
            .is_match(&password)
            .then_some(Self(password))
            .ok_or(Error::NotAPassword)
    }
}

impl Password {
    const PATTERN: &str = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$";
    const REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(Self::PATTERN).unwrap());
}

#[cfg(test)]
mod tests {
    use super::{Password, PasswordHash};
    use crate::user::error::Result;

    #[test]
    fn password_from_str() {
        struct Test<'a> {
            name: &'a str,
            input: &'a str,
            must_fail: bool,
        }

        vec![
            Test {
                name: "valid password",
                input: "abcABC123&",
                must_fail: false,
            },
            Test {
                name: "password without special characters",
                input: "abcABC123",
                must_fail: true,
            },
            Test {
                name: "password without uppercase characters",
                input: "abcabc123&",
                must_fail: true,
            },
            Test {
                name: "password without lowercase characters",
                input: "ABCABC123&",
                must_fail: true,
            },
            Test {
                name: "password with less than 8 characters",
                input: "aB1&",
                must_fail: true,
            },
            Test {
                name: "none base64 password",
                input: "abcABC123&",
                must_fail: true,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let result: Result<Password> = test.input.to_string().try_into();
            assert_eq!(result.is_err(), test.must_fail, "{}", test.name);
        })
    }

    #[test]
    fn password_hash_matches() {
        let password: PasswordHash = "abcABC123".to_string().try_into().unwrap();

        assert!(!password
            .matches("abcABC124".to_string().try_into().unwrap())
            .unwrap());

        assert!(password
            .matches("abcABC123".to_string().try_into().unwrap())
            .unwrap());
    }
}
