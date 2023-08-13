use crate::crypto;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::hash::Hash;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Hash, Serialize, Deserialize)]
pub struct Token {
    pub jti: String, // JWT ID
    #[serde(
        serialize_with = "as_unix_timestamp",
        deserialize_with = "from_unix_timestamp"
    )]
    pub exp: SystemTime, // expiration time - required
    #[serde(
        serialize_with = "as_unix_timestamp",
        deserialize_with = "from_unix_timestamp"
    )]
    pub nbf: SystemTime, // not before time - non required
    #[serde(
        serialize_with = "as_unix_timestamp",
        deserialize_with = "from_unix_timestamp"
    )]
    pub iat: SystemTime, // issued at: creation time
    pub iss: String, // issuer
    pub sub: String, // subject
    pub knd: TokenKind, // kind - required
}

fn as_unix_timestamp<S>(timestamp: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::Error;

    timestamp
        .duration_since(UNIX_EPOCH)
        .map_err(|err| Error::custom(err.to_string()))
        .and_then(|timestamp| serializer.serialize_u64(timestamp.as_secs()))
}

fn from_unix_timestamp<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
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

impl Token {
    pub fn new(kind: TokenKind, iss: &str, sub: &str, timeout: Duration) -> Self {
        let mut token = Token {
            jti: crypto::get_random_string(8), // noise
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

    // pub fn signed(&self, key: &str) -> Result<SignedToken> {
    //     todo!()
    // }
}

/// Represents a [Token] that has been signed.
pub struct SignedToken(String);

impl AsRef<str> for SignedToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
pub mod tests {
    use super::{Token, TokenKind};
    use crate::crypto;
    use base64::{engine::general_purpose, Engine as _};
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
        let claim = Token::new(TokenKind::Session, ISS, &SUB.to_string(), timeout);
        let after = SystemTime::now();

        assert!(claim.iat >= before && claim.iat <= after);
        assert!(claim.exp >= before + timeout);
        assert!(claim.exp <= after + timeout);
        assert!(matches!(claim.knd, TokenKind::Session));
        assert_eq!(ISS, claim.iss);
        assert_eq!(SUB.to_string(), claim.sub);
    }

    #[test]
    fn token_encode_should_not_fail() {
        const ISS: &str = "test";
        const SUB: i32 = 999;
        let timeout = Duration::from_secs(TEST_DEFAULT_TOKEN_TIMEOUT);
        let claim = Token::new(TokenKind::Session, ISS, &SUB.to_string(), timeout);

        let secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let token = crypto::sign_jwt(&secret, claim).unwrap();

        let public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();
        let _ = crypto::decode_jwt::<Token>(&public, &token).unwrap();
    }

    #[test]
    fn expired_token_verification_should_fail() {
        use crate::crypto;

        const ISS: &str = "test";
        const SUB: i32 = 999;

        let timeout = Duration::from_secs(TEST_DEFAULT_TOKEN_TIMEOUT);
        let mut claim = Token::new(TokenKind::Session, ISS, &SUB.to_string(), timeout);
        claim.exp = SystemTime::now() - Duration::from_secs(61);

        let secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let token = crypto::sign_jwt(&secret, claim).unwrap();
        let public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();

        assert!(crypto::decode_jwt::<Token>(&public, &token).is_err());
    }
}
