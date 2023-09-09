use super::error::{Error, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const PATTERN: &str = r"^(?:[\w-]*\.){2}[\w-]*$";
const REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(PATTERN).unwrap());

/// Represents the kind of a token.
#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenKind {
    Session,
    Verification,
    Reset,
}

impl TokenKind {
    pub fn is_session(&self) -> bool {
        matches!(self, TokenKind::Session)
    }

    pub fn is_verification(&self) -> bool {
        matches!(self, TokenKind::Verification)
    }

    pub fn is_reset(&self) -> bool {
        matches!(self, TokenKind::Reset)
    }
}

/// Represents the payload of a JWT, containing the claims.
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Payload {
    pub jti: String,
    pub iss: String,
    pub sub: String,
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
    pub knd: TokenKind,
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
    pub fn new(token_kind: TokenKind, timeout: Duration) -> Self {
        Payload {
            jti: Default::default(),
            iss: Default::default(),
            sub: Default::default(),
            exp: SystemTime::now() + timeout,
            nbf: SystemTime::now(),
            iat: SystemTime::now(),
            knd: token_kind,
        }
    }

    pub fn with_subject(mut self, subject: &str) -> Self {
        self.sub = subject.to_string();
        self
    }

    pub fn with_issuer(mut self, issuer: &str) -> Self {
        self.iss = issuer.to_string();
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.jti = id;
        self
    }

    /// Returns the [Duration] from now for which the [Token] is valid.
    pub fn timeout(&self) -> Duration {
        self.exp
            .duration_since(SystemTime::now())
            .unwrap_or_default()
    }

    /// Returns the [Kind] field from self.
    pub fn kind(&self) -> TokenKind {
        self.knd.clone()
    }

    /// Returns the subject field (sub) from self.
    pub fn subject(&self) -> &str {
        &self.sub
    }

    /// Returns the result of hashing self.
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        Hash::hash(self, &mut hasher);
        hasher.finish()
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

impl From<Claims> for Token {
    fn from(value: Claims) -> Self {
        value.token
    }
}

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Represents a token and its corresponding payload, containing the claims.
#[derive(Debug)]
pub struct Claims {
    pub(super) token: Token,
    pub(super) payload: Payload,
}

impl Claims {
    /// Returns the token with the corresponding claims.
    pub fn token(&self) -> &Token {
        &self.token
    }

    /// Returns the payload, containing the claims.
    pub fn payload(&self) -> &Payload {
        &self.payload
    }
}

#[cfg(test)]
pub mod tests {
    use super::{Payload, TokenKind};
    use std::time::{Duration, SystemTime};

    pub const TEST_DEFAULT_TOKEN_TIMEOUT: u64 = 60;
    const JWT_SECRET: &[u8] = b"LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JR0hBZ0VBTUJNR0J5cUdTTTQ5QWdFR0NDcUdTTTQ5QXdFSEJHMHdhd0lCQVFRZy9JMGJTbVZxL1BBN2FhRHgKN1FFSGdoTGxCVS9NcWFWMUJab3ZhM2Y5aHJxaFJBTkNBQVJXZVcwd3MydmlnWi96SzRXcGk3Rm1mK0VPb3FybQpmUlIrZjF2azZ5dnBGd0gzZllkMlllNXl4b3ZsaTROK1ZNNlRXVFErTmVFc2ZmTWY2TkFBMloxbQotLS0tLUVORCBQUklWQVRFIEtFWS0tLS0tCg==";
    const JWT_PUBLIC: &[u8] = b"LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFVm5sdE1MTnI0b0dmOHl1RnFZdXhabi9oRHFLcQo1bjBVZm45YjVPc3I2UmNCOTMySGRtSHVjc2FMNVl1RGZsVE9rMWswUGpYaExIM3pIK2pRQU5tZFpnPT0KLS0tLS1FTkQgUFVCTElDIEtFWS0tLS0tCg==";

    #[test]
    fn token_new_should_not_fail() {
        let timeout = Duration::from_secs(TEST_DEFAULT_TOKEN_TIMEOUT);

        let before = SystemTime::now();
        let payload = Payload::new(TokenKind::Session, timeout);
        let after = SystemTime::now();

        assert!(payload.iat >= before && payload.iat <= after);
        assert!(payload.exp >= before + timeout);
        assert!(payload.exp <= after + timeout);
        assert!(matches!(payload.knd, TokenKind::Session));
    }
}
