use once_cell::sync::Lazy;
use rand::Rng;
use regex::Regex;

use super::error::{Error, Result};

/// Represents the multi factor authentication method to use.
#[derive(Debug, Clone, Copy, strum_macros::EnumString, strum_macros::Display)]
#[strum(serialize_all = "lowercase")]
pub enum MfaMethod {
    /// Uses a third-party application as totp provider.
    #[strum(serialize = "tp_app")]
    TpApp,
    /// Uses the email as otp provider.
    Email,
}

/// Represents a one time password.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Otp(String);

impl AsRef<str> for Otp {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<[u8]> for Otp {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl TryFrom<String> for Otp {
    type Error = Error;

    /// Builds a [OTP] from the given string if, and only if, the string matches the
    /// otp's regex.
    fn try_from(password: String) -> Result<Self> {
        Self::REGEX
            .is_match(&password)
            .then_some(Self(password))
            .ok_or(Error::NotAOneTimePassword)
    }
}

impl Otp {
    const PATTERN: &str = r"^[0-9]+$";
    const CHARSET: &[u8] = b"0123456789";
    const REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(Self::PATTERN).unwrap());

    pub fn with_length(len: usize) -> Result<Self> {
        let mut buff = vec![0_u8; len];

        for index in 0..buff.len() {
            let mut rand = rand::thread_rng();
            let idx = rand.gen_range(0..Self::CHARSET.len());
            buff[index] = Self::CHARSET[idx]
        }

        String::from_utf8(buff)
            .map(|value| Otp(value))
            .map_err(Into::into)
    }
}
