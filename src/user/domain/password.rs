use crate::{
    base64, on_error,
    user::error::{Error, Result},
};
use ::regex::Regex;
use argon2::{Algorithm, Argon2, Params, Version};
use rand::Rng;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const HASH_LEN: usize = 128;

/// Represents the hash and salt of a [Password].
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasswordHash {
    pub(crate) hash: String,
    pub(crate) salt: Salt,
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
        let salt = Salt::with_length(HASH_LEN)?;
        Self::with_salt(&password, &salt)
    }
}

impl AsRef<str> for PasswordHash {
    fn as_ref(&self) -> &str {
        &self.hash
    }
}

impl PasswordHash {
    const ARGON: Lazy<Argon2<'_>> =
        Lazy::new(|| Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default()));

    /// Builds a new password from the given value and salt
    fn with_salt(password: &Password, salt: &Salt) -> Result<Self> {
        let mut buffer = vec![0_u8; salt.len()];

        Self::ARGON
            .hash_password_into(password.as_ref(), salt.as_ref(), &mut buffer)
            .map(|_| Self {
                hash: base64::encode(&buffer),
                salt: salt.clone(),
            })
            .map_err(on_error!(Error, "salting and hashing password"))
    }

    /// Returns true if, and only if, the given password matches with self.
    pub fn matches(&self, other: &Password) -> Result<bool> {
        PasswordHash::with_salt(other, &self.salt).map(|subject| &subject == self)
    }

    /// Returns the salt of the hash.
    pub fn salt(&self) -> &Salt {
        &self.salt
    }
}

/// Represents a password.
#[derive(Debug, Clone)]
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

/// Represents the salt of a password.
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Salt(String);

impl AsRef<str> for Salt {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<[u8]> for Salt {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl TryFrom<String> for Salt {
    type Error = Error;

    /// Builds a [OTP] from the given string if, and only if, the string matches the
    /// otp's regex.
    fn try_from(password: String) -> Result<Self> {
        Self::REGEX
            .is_match(&password)
            .then_some(Self(password))
            .ok_or(Error::NotASalt)
    }
}

impl Salt {
    const PATTERN: &str = r"^[0-9]+$";
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(Self::PATTERN).unwrap());

    pub fn with_length(len: usize) -> Result<Self> {
        let mut buff = vec![0_u8; len];

        for index in 0..buff.len() {
            let mut rand = rand::thread_rng();
            let idx = rand.gen_range(0..Self::CHARSET.len());
            buff[index] = Self::CHARSET[idx]
        }

        String::from_utf8(buff)
            .map(|value| Salt(value))
            .map_err(Into::into)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
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
            .matches(&"abcABC124".to_string().try_into().unwrap())
            .unwrap());

        assert!(password
            .matches(&"abcABC123".to_string().try_into().unwrap())
            .unwrap());
    }
}
