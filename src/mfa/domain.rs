use super::error::{Error, Result};
use rand::{distributions::Uniform, Rng};
use std::str::FromStr;

/// Represents the multi factor authentication method to use.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MfaMethod {
    /// Uses a third-party application as totp provider.
    TpApp,
    /// Uses the email as otp provider.
    Email,
    /// Uses any other method.
    Other(String),
}

impl ToString for MfaMethod {
    fn to_string(&self) -> String {
        match self {
            MfaMethod::TpApp => "tp_app".to_string(),
            MfaMethod::Email => "email".to_string(),
            MfaMethod::Other(other) => other.clone(),
        }
    }
}

impl FromStr for MfaMethod {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "tp_app" => Ok(Self::TpApp),
            "email" => Ok(Self::Email),
            other => Ok(Self::Other(other.to_string())),
        }
    }
}

/// Represents a one time password.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    fn try_from(otp: String) -> Result<Self> {
        if otp.is_empty() || otp.chars().any(|c| !c.is_numeric()) {
            return Err(Error::NotAOneTimePassword);
        }

        Ok(Self(otp))
    }
}

impl Otp {
    /// Builds a new [Otp] with the given length for any length greater than 0. Otherwise returns [Error::NotAOneTimePassword].
    pub fn with_length(len: usize) -> Result<Self> {
        rand::thread_rng()
            .sample_iter(Uniform::new_inclusive(0, 9))
            .take(len)
            .map(|digit| digit.to_string())
            .collect::<String>()
            .try_into()
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::mfa::{
        domain::{MfaMethod, Otp},
        error::Error,
    };

    #[test]
    fn display_mfa_method() {
        struct Test<'a> {
            name: &'a str,
            output: &'a str,
            input: MfaMethod,
        }

        vec![
            Test {
                name: "third party application method",
                output: "tp_app",
                input: MfaMethod::TpApp,
            },
            Test {
                name: "email method",
                output: "email",
                input: MfaMethod::Email,
            },
            Test {
                name: "an arbitrary name",
                output: "arbitrary method",
                input: MfaMethod::Other("arbitrary method".to_string()),
            },
        ]
        .into_iter()
        .for_each(|test| assert_eq!(test.input.to_string(), test.output, "{}", test.name))
    }

    #[test]
    fn mfa_method_from_str() {
        struct Test<'a> {
            name: &'a str,
            input: &'a str,
            output: MfaMethod,
        }

        vec![
            Test {
                name: "third party application method",
                input: "tp_app",
                output: MfaMethod::TpApp,
            },
            Test {
                name: "email method",
                input: "email",
                output: MfaMethod::Email,
            },
            Test {
                name: "an arbitrary name",
                input: "arbitrary method",
                output: MfaMethod::Other("arbitrary method".to_string()),
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = MfaMethod::from_str(test.input).unwrap();
            assert_eq!(got, test.output, "{}", test.name)
        })
    }

    #[test]
    fn otp_from_str() {
        struct Test<'a> {
            name: &'a str,
            input: &'a str,
            is_valid: bool,
        }

        vec![
            Test {
                name: "numeric otp",
                input: "1234",
                is_valid: true,
            },
            Test {
                name: "non numeric otp",
                input: "abc123&",
                is_valid: false,
            },
            Test {
                name: "empty otp",
                input: "",
                is_valid: false,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let result = Otp::try_from(test.input.to_string());
            if test.is_valid {
                let otp = result.unwrap();
                assert_eq!(otp.as_ref(), test.input, "{0}", test.name);
            } else {
                assert!(
                    matches!(result.err(), Some(Error::NotAOneTimePassword)),
                    "{}",
                    test.name
                );
            }
        })
    }

    #[test]
    fn otp_with_length() {
        struct Test<'a> {
            name: &'a str,
            len: usize,
            is_valid: bool,
        }

        vec![
            Test {
                name: "with length 10",
                len: 10,
                is_valid: true,
            },
            Test {
                name: "with length 100",
                len: 100,
                is_valid: true,
            },
            Test {
                name: "with no length",
                len: 0,
                is_valid: false,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let result = Otp::with_length(test.len);
            if test.is_valid {
                let otp = result.unwrap();
                assert_eq!(otp.as_ref().len(), test.len, "{}", test.name);

                assert!(
                    Otp::try_from(otp.as_ref().to_string()).is_ok(),
                    "{}",
                    test.name
                );
            } else {
                assert!(
                    matches!(result.err(), Some(Error::NotAOneTimePassword)),
                    "{}",
                    test.name
                );
            }
        })
    }
}
