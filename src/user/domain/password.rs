use crate::{
    crypto,
    user::error::{Error, Result},
};
use ::regex::Regex;

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
        crypto::randomize(&mut salt);

        crypto::salt(password.as_bytes(), &salt).map(|salted| Self {
            hash: crypto::encode_b64(&salted),
            salt: crypto::encode_b64(&salt),
        })
    }
}

impl Password {
    const PATTERN: &str = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$";

    pub const REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(Self::PATTERN).unwrap());

    pub fn new(hash: String, salt: String) -> Self {
        Self { hash, salt }
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn salt(&self) -> &str {
        &self.salt
    }

    pub fn matches(&self, other: &str) -> Result<bool> {
        let hash: Vec<u8> = crypto::decode_b64(self.hash())?;
        let salt = crypto::decode_b64(self.salt())?;

        crypto::salt(other.as_bytes(), &salt).map(|salted| salted == hash.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::Password;
    use crate::user::error::Result;

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
