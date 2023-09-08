use once_cell::sync::Lazy;
use regex::Regex;

use super::error::{Error, Result};

/// Represents the multi factor authentication method to use.
#[derive(Debug, Clone, Copy, strum_macros::EnumString, strum_macros::Display)]
#[strum(serialize_all = "lowercase")]
pub enum MfaMethod {
    /// Uses a third-party application as totp provider.
    TpApp,
    /// Uses the email as otp provider.
    Email,
}

/// Represents a one time password.
pub struct Otp(String);

impl AsRef<str> for Otp {
    fn as_ref(&self) -> &str {
        &self.0
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
    const PATTERN: &str = r"^[0-9A-Za-z]{6}$";
    const REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(Self::PATTERN).unwrap());
}
