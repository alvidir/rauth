use crate::time;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, SystemTime};

pub trait TokenDefinition {
    fn get_id(&self) -> String;
    fn get_kind(&self) -> &TokenKind;
}

pub struct SignedToken {
    pub(super) id: String,
    pub(super) signature: String,
}

impl SignedToken {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn signature(&self) -> &str {
        &self.signature
    }
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug, Clone, strum_macros::Display)]
pub enum TokenKind {
    Session = 0,
    Verification = 1,
    Reset = 2,
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone, PartialEq)]
pub struct Token {
    pub jti: String,     // JWT ID
    pub exp: usize,      // expiration time (as UTC timestamp) - required
    pub nbf: usize,      // not before time (as UTC timestamp) - non required
    pub iat: SystemTime, // issued at: creation time
    pub iss: String,     // issuer
    pub sub: String,     // subject
    pub knd: TokenKind,  // kind - required
}

impl Token {
    fn default_secret_value() -> Option<String> {
        None
    }

    pub fn new(kind: TokenKind, iss: &str, sub: &str, timeout: Duration) -> Self {
        let mut token = Token {
            jti: rand::thread_rng().gen::<u64>().to_string(), // noise
            exp: time::unix_timestamp(SystemTime::now() + timeout),
            nbf: time::unix_timestamp(SystemTime::now()),
            iat: SystemTime::now(),
            iss: iss.to_string(),
            sub: sub.to_string(),
            knd: kind,
        };

        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        token.jti = hasher.finish().to_string();

        token
    }
}

impl TokenDefinition for Token {
    fn get_id(&self) -> String {
        format!("{:?}::{}", self.knd, self.jti)
    }

    fn get_kind(&self) -> &TokenKind {
        &self.knd
    }
}

#[cfg(test)]
pub mod tests {
    use super::{Token, TokenKind};
    use crate::time::unix_timestamp;
    use crate::{crypto, time};
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
        assert!(claim.exp >= unix_timestamp(before + timeout));
        assert!(claim.exp <= unix_timestamp(after + timeout));
        assert_eq!(claim.knd, TokenKind::Session);
        assert_eq!(ISS, claim.iss);
        assert_eq!(SUB.to_string(), claim.sub);
    }

    #[test]
    fn token_encode_should_not_fail() {
        const ISS: &str = "test";
        const SUB: i32 = 999;
        let timeout = Duration::from_secs(TEST_DEFAULT_TOKEN_TIMEOUT);

        let before = SystemTime::now();
        let claim = Token::new(TokenKind::Session, ISS, &SUB.to_string(), timeout);

        let after = SystemTime::now();

        let secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let token = crypto::sign_jwt(&secret, claim).unwrap();

        let public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();
        let claim = crypto::decode_jwt::<Token>(&public, &token).unwrap();

        assert!(claim.iat >= before && claim.iat <= after);
        assert!(claim.exp >= unix_timestamp(before + timeout));
        assert!(claim.exp <= unix_timestamp(after + timeout));
        assert_eq!(ISS, claim.iss);
        assert_eq!(SUB.to_string(), claim.sub);
    }

    #[test]
    fn expired_token_verification_should_fail() {
        use crate::crypto;

        const ISS: &str = "test";
        const SUB: i32 = 999;

        let timeout = Duration::from_secs(TEST_DEFAULT_TOKEN_TIMEOUT);
        let mut claim = Token::new(TokenKind::Session, ISS, &SUB.to_string(), timeout);
        claim.exp = time::unix_timestamp(SystemTime::now() - Duration::from_secs(61));

        let secret = general_purpose::STANDARD.decode(JWT_SECRET).unwrap();
        let token = crypto::sign_jwt(&secret, claim).unwrap();
        let public = general_purpose::STANDARD.decode(JWT_PUBLIC).unwrap();

        assert!(crypto::decode_jwt::<Token>(&public, &token).is_err());
    }
}
