use super::error::{Error, Result};
use crate::crypto;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::hash::Hash;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const PATTERN: &str = r"^(?:[\w-]*\.){2}[\w-]*$";
const REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(PATTERN).unwrap());

/// Represents the kind of a token.
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Session,
    Verification,
    Reset,
}

/// Represents the payload of a JWT, containing the claims.
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Payload {
    pub jti: String,
    #[serde(
        serialize_with = "as_unix_timestamp",
        deserialize_with = "from_unix_timestamp"
    )]
    pub exp: SystemTime,
    #[serde(
        serialize_with = "as_unix_timestamp",
        deserialize_with = "from_unix_timestamp"
    )]
    pub nbf: SystemTime,
    #[serde(
        serialize_with = "as_unix_timestamp",
        deserialize_with = "from_unix_timestamp"
    )]
    pub iat: SystemTime,
    pub iss: String,
    pub sub: String,
    pub knd: Kind,
}

fn as_unix_timestamp<S>(
    timestamp: &SystemTime,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::Error;

    timestamp
        .duration_since(UNIX_EPOCH)
        .map_err(|err| Error::custom(err.to_string()))
        .and_then(|timestamp| serializer.serialize_u64(timestamp.as_secs()))
}

fn from_unix_timestamp<'de, D>(deserializer: D) -> std::result::Result<SystemTime, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    u64::deserialize(deserializer)
        .map(Duration::from_secs)
        .and_then(|duration| {
            UNIX_EPOCH
                .checked_add(duration)
                .ok_or_else(|| Error::custom("cannot be represented as SystemTime".to_string()))
        })
}

impl Payload {
    pub fn new(kind: Kind, iss: &str, sub: &str, timeout: Duration) -> Self {
        let mut token = Payload {
            jti: Default::default(),
            exp: SystemTime::now() + timeout,
            nbf: SystemTime::now(),
            iat: SystemTime::now(),
            iss: iss.to_string(),
            sub: sub.to_string(),
            knd: kind,
        };

        token.jti = crypto::hash(&token).to_string();
        token
    }

    /// Returns the [Duration] from now for which the [Token] is valid.
    pub fn timeout(&self) -> Duration {
        self.exp
            .duration_since(SystemTime::now())
            .unwrap_or_default()
    }

    /// Returns the [Kind] field from self.
    pub fn kind(&self) -> Kind {
        self.knd
    }

    /// Returns the subject field (sub) from self.
    pub fn subject(&self) -> &str {
        &self.sub
    }
}

/// Represents a signed token.
#[derive(Debug)]
pub struct Token(String);

impl TryFrom<String> for Token {
    type Error = Error;

    fn try_from(token: String) -> Result<Self> {
        REGEX
            .is_match(&token)
            .then_some(Self(token))
            .ok_or(Error::NotAToken)
    }
}

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
pub mod tests {
    use super::{Kind, Payload};
    use std::time::{Duration, SystemTime};

    pub const TEST_DEFAULT_TOKEN_TIMEOUT: u64 = 60;
    const JWT_SECRET: &[u8] = b"LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JR0hBZ0VBTUJNR0J5cUdTTTQ5QWdFR0NDcUdTTTQ5QXdFSEJHMHdhd0lCQVFRZy9JMGJTbVZxL1BBN2FhRHgKN1FFSGdoTGxCVS9NcWFWMUJab3ZhM2Y5aHJxaFJBTkNBQVJXZVcwd3MydmlnWi96SzRXcGk3Rm1mK0VPb3FybQpmUlIrZjF2azZ5dnBGd0gzZllkMlllNXl4b3ZsaTROK1ZNNlRXVFErTmVFc2ZmTWY2TkFBMloxbQotLS0tLUVORCBQUklWQVRFIEtFWS0tLS0tCg==";
    const JWT_PUBLIC: &[u8] = b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFVm5sdE1MTnI0b0dmOHl1RnFZdXhabi9oRHFLcQo1bjBVZm45YjVPc3I2UmNCOTMySGRtSHVjc2FMNVl1RGZsVE9rMWswUGpYaExIM3pIK2pRQU5tZFpnPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg==";

    #[test]
    fn token_new_should_not_fail() {
        const ISS: &str = "test";
        const SUB: i32 = 999;

        let timeout = Duration::from_secs(TEST_DEFAULT_TOKEN_TIMEOUT);

        let before = SystemTime::now();
        let claim = Payload::new(Kind::Session, ISS, &SUB.to_string(), timeout);
        let after = SystemTime::now();

        assert!(claim.iat >= before && claim.iat <= after);
        assert!(claim.exp >= before + timeout);
        assert!(claim.exp <= after + timeout);
        assert!(matches!(claim.knd, Kind::Session));
        assert_eq!(ISS, claim.iss);
        assert_eq!(SUB.to_string(), claim.sub);
    }
}
